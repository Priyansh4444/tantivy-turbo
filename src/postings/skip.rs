use std::num::NonZeroU32;

use crate::directory::OwnedBytes;
use crate::postings::compression::{
    compressed_block_size, compressed_freq_block_size, dense_block_size, COMPRESSION_BLOCK_SIZE,
};
use crate::query::Bm25Weight;
use crate::schema::IndexRecordOption;
use crate::{DocId, Score, TERMINATED};

// doc num bits uses the following encoding:
// given 0b a b cdefgh
//         |1|2|3|  4  |
// - 1: unused
// - 2: is delta-1 encoded. 0 if not, 1, if yes
// - 3: unused
// - 4: a 5 bit number in 0..32, the actual bitwidth. Bitpacking could in theory say this is 32
//   (requiring a 6th bit), but the biggest doc_id we can want to encode is TERMINATED-1, which can
//   be represented on 31b without delta encoding.
fn encode_bitwidth(bitwidth: u8, delta_1: bool) -> u8 {
    assert!(
        bitwidth < 32,
        "bitwidth needs to be less than 32, but got {}",
        bitwidth
    );
    bitwidth | ((delta_1 as u8) << 6)
}

fn decode_bitwidth(raw_bitwidth: u8) -> (u8, bool) {
    let delta_1 = ((raw_bitwidth >> 6) & 1) != 0;
    let bitwidth = raw_bitwidth & 0x1f;
    (bitwidth, delta_1)
}

/// Marker bit for dense bitset blocks (see `Lucene104PostingsWriter`'s
/// unary-coding path).
///
/// When set on the doc-bitwidth byte, the low 7 bits hold `num_longs` (the
/// number of u64s, `1..=MAX_DENSE_LONGS`) instead of a FOR bitwidth. The
/// postings block that follows is `num_longs * 8` bytes of little-endian
/// bitset, decoded relative to the previous block's last doc (see
/// `bitset_base_doc`).
pub(crate) const DENSE_FLAG: u8 = 0x80;
/// Maximum bitset longs we ever write (mirrors Lucene's
/// `BLOCK_SIZE / 2` cap of 64 longs = 512 bytes for 128 docs).
pub(crate) const MAX_DENSE_LONGS: u8 = 64;

fn encode_dense_marker(num_longs: u8) -> u8 {
    assert!(
        num_longs >= 1 && num_longs <= MAX_DENSE_LONGS,
        "dense num_longs out of range: {num_longs}"
    );
    DENSE_FLAG | num_longs
}

fn decode_dense_marker(raw: u8) -> Option<u8> {
    let num_longs = raw & !DENSE_FLAG;
    (raw & DENSE_FLAG != 0 && (1..=MAX_DENSE_LONGS).contains(&num_longs)).then_some(num_longs)
}

#[inline]
fn encode_block_wand_max_tf(max_tf: u32) -> u8 {
    max_tf.min(u8::MAX as u32) as u8
}

#[inline]
fn decode_block_wand_max_tf(max_tf_code: u8) -> u32 {
    if max_tf_code == u8::MAX {
        u32::MAX
    } else {
        max_tf_code as u32
    }
}

#[inline]
fn read_u32(data: &[u8]) -> u32 {
    u32::from_le_bytes(data[..4].try_into().unwrap())
}

#[inline]
fn write_u32(val: u32, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&val.to_le_bytes());
}

pub struct SkipSerializer {
    buffer: Vec<u8>,
}

impl SkipSerializer {
    pub fn new() -> SkipSerializer {
        SkipSerializer { buffer: Vec::new() }
    }

    pub fn write_doc(&mut self, last_doc: DocId, doc_num_bits: u8) {
        write_u32(last_doc, &mut self.buffer);
        self.buffer.push(encode_bitwidth(doc_num_bits, true));
    }

    pub fn write_doc_dense(&mut self, last_doc: DocId, num_longs: u8) {
        write_u32(last_doc, &mut self.buffer);
        self.buffer.push(encode_dense_marker(num_longs));
    }

    pub fn write_term_freq(&mut self, tf_num_bits: u8) {
        self.buffer.push(tf_num_bits);
    }

    pub fn write_total_term_freq(&mut self, tf_sum: u32) {
        write_u32(tf_sum, &mut self.buffer);
    }

    pub fn write_blockwand_max(&mut self, fieldnorm_id: u8, term_freq: u32) {
        let block_wand_tf = encode_block_wand_max_tf(term_freq);
        self.buffer
            .extend_from_slice(&[fieldnorm_id, block_wand_tf]);
    }

    pub fn data(&self) -> &[u8] {
        &self.buffer[..]
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

#[derive(Clone)]
pub(crate) struct SkipReader {
    last_doc_in_block: DocId,
    pub(crate) last_doc_in_previous_block: DocId,
    owned_read: OwnedBytes,
    skip_info: IndexRecordOption,
    byte_offset: usize,
    remaining_docs: u32, // number of docs remaining, including the
    // documents in the current block.
    block_info: BlockInfo,

    position_offset: u64,
}

#[derive(Clone, Eq, PartialEq, Copy, Debug)]
pub(crate) enum BlockInfo {
    BitPacked {
        doc_num_bits: u8,
        strict_delta_encoded: bool,
        tf_num_bits: u8,
        tf_sum: u32,
        block_wand_fieldnorm_id: u8,
        block_wand_term_freq: u32,
    },
    Dense {
        num_longs: u8,
        tf_num_bits: u8,
        tf_sum: u32,
        block_wand_fieldnorm_id: u8,
        block_wand_term_freq: u32,
    },
    VInt {
        num_docs: u32,
    },
}

/// A complete block's stored DEFAULT-input maximizer. The frequency is a
/// ceiling for this selected posting, never a block-wide maximum frequency.
#[derive(Clone, Copy)]
pub(crate) struct SelectedInputPair {
    pub(crate) norm: u8,
    pub(crate) selected_tf_ceiling: NonZeroU32,
}

/// Immutable view of the existing fixed-width skip records. A query may inspect
/// arbitrary complete blocks without changing its live postings/positions owner.
#[derive(Clone)]
pub(crate) struct SkipDirectory {
    data: OwnedBytes,
    blocks: usize,
    stride: usize,
    has_tail: bool,
}

impl SkipDirectory {
    pub(crate) fn new(data: OwnedBytes, doc_freq: u32, option: IndexRecordOption) -> Self {
        let stride = match option {
            IndexRecordOption::Basic => 5,
            IndexRecordOption::WithFreqs => 8,
            IndexRecordOption::WithFreqsAndPositions => 12,
        };
        Self {
            data,
            blocks: doc_freq as usize / COMPRESSION_BLOCK_SIZE,
            stride,
            has_tail: !(doc_freq as usize).is_multiple_of(COMPRESSION_BLOCK_SIZE),
        }
    }

    pub(crate) fn full_blocks(&self) -> usize {
        self.blocks
    }

    pub(crate) fn len(&self) -> usize {
        self.blocks + usize::from(self.has_tail)
    }

    pub(crate) fn end(&self, block: usize) -> DocId {
        if block >= self.blocks {
            TERMINATED
        } else {
            read_u32(&self.data[block * self.stride..])
        }
    }

    pub(crate) fn block_at(&self, doc: DocId) -> usize {
        let mut left = 0;
        let mut right = self.blocks;
        while left < right {
            let mid = left + (right - left) / 2;
            if self.end(mid) < doc {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        left
    }

    /// `block_at(doc)`, galloping forward from the hint `from`. Any hint is
    /// correct; one at or below the answer is fast.
    pub(crate) fn block_at_from(&self, doc: DocId, from: usize) -> usize {
        let mut left = from.min(self.blocks);
        if left > 0 && self.end(left - 1) >= doc {
            return self.block_at(doc);
        }
        // Every block before `left` ends before `doc`.
        let mut step = 1;
        let mut right = loop {
            if left >= self.blocks || self.end(left) >= doc {
                return left;
            }
            let probe = left + step;
            if probe >= self.blocks {
                break self.blocks;
            }
            if self.end(probe) >= doc {
                break probe;
            }
            left = probe + 1;
            step *= 2;
        };
        // end(left) < doc, and right is a block ending at or after doc (or
        // the count): the answer lies in (left, right].
        while left < right {
            let mid = left + (right - left) / 2;
            if self.end(mid) < doc {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        left
    }

    pub(crate) fn pair(&self, block: usize) -> Option<SelectedInputPair> {
        if block >= self.blocks || self.stride == 5 {
            return None;
        }
        let offset = (block + 1) * self.stride - 2;
        NonZeroU32::new(decode_block_wand_max_tf(self.data[offset + 1])).map(
            |selected_tf_ceiling| SelectedInputPair {
                norm: self.data[offset],
                selected_tf_ceiling,
            },
        )
    }
}

impl Default for BlockInfo {
    fn default() -> Self {
        BlockInfo::VInt { num_docs: 0u32 }
    }
}

impl SkipReader {
    pub(crate) fn record_option(&self) -> IndexRecordOption {
        self.skip_info
    }

    pub(crate) fn selected_input_pair(&self) -> Option<SelectedInputPair> {
        match self.block_info {
            BlockInfo::BitPacked {
                block_wand_fieldnorm_id,
                block_wand_term_freq,
                ..
            }
            | BlockInfo::Dense {
                block_wand_fieldnorm_id,
                block_wand_term_freq,
                ..
            } => {
                NonZeroU32::new(block_wand_term_freq).map(|selected_tf_ceiling| SelectedInputPair {
                    norm: block_wand_fieldnorm_id,
                    selected_tf_ceiling,
                })
            }
            BlockInfo::VInt { .. } => None,
        }
    }

    pub fn new(data: OwnedBytes, doc_freq: u32, skip_info: IndexRecordOption) -> SkipReader {
        let mut skip_reader = SkipReader {
            last_doc_in_block: if doc_freq >= COMPRESSION_BLOCK_SIZE as u32 {
                0
            } else {
                TERMINATED
            },
            last_doc_in_previous_block: 0u32,
            owned_read: data,
            skip_info,
            block_info: BlockInfo::VInt { num_docs: doc_freq },
            byte_offset: 0,
            remaining_docs: doc_freq,
            position_offset: 0u64,
        };
        if doc_freq >= COMPRESSION_BLOCK_SIZE as u32 {
            skip_reader.read_block_info();
        }
        skip_reader
    }

    #[inline(always)]
    pub fn has_remaining_docs(&self) -> bool {
        self.remaining_docs != 0
    }

    pub fn reset(&mut self, data: OwnedBytes, doc_freq: u32) {
        self.last_doc_in_block = if doc_freq >= COMPRESSION_BLOCK_SIZE as u32 {
            0
        } else {
            TERMINATED
        };
        self.last_doc_in_previous_block = 0u32;
        self.owned_read = data;
        self.block_info = BlockInfo::VInt { num_docs: doc_freq };
        self.byte_offset = 0;
        self.remaining_docs = doc_freq;
        self.position_offset = 0u64;
        if doc_freq >= COMPRESSION_BLOCK_SIZE as u32 {
            self.read_block_info();
        }
    }

    // Returns the block max score for this block if available.
    //
    // The block max score is available for all full bitpacked block,
    // but no available for the last VInt encoded incomplete block.
    pub fn block_max_score(&self, bm25_weight: &Bm25Weight) -> Option<Score> {
        match self.block_info {
            BlockInfo::BitPacked {
                block_wand_fieldnorm_id,
                block_wand_term_freq,
                ..
            }
            | BlockInfo::Dense {
                block_wand_fieldnorm_id,
                block_wand_term_freq,
                ..
            } if block_wand_term_freq != 0
                && (block_wand_term_freq != u32::MAX
                    || bm25_weight.supports_frequency_ceiling()) =>
            {
                Some(bm25_weight.score(block_wand_fieldnorm_id, block_wand_term_freq))
            }
            // Zero means absent metadata. Saturated frequencies decode to
            // u32::MAX. Native subtraction scoring is monotone in frequency,
            // so this ceiling is safe. The legacy rounded ratio need not be
            // monotone; retain its exact or global fallback.
            BlockInfo::BitPacked { .. } | BlockInfo::Dense { .. } => None,
            BlockInfo::VInt { .. } => None,
        }
    }

    pub(crate) fn last_doc_in_block(&self) -> DocId {
        self.last_doc_in_block
    }

    /// Number of docs from the start of the current block to the end of the postings
    /// (i.e. the current block plus every block after it).
    pub(crate) fn remaining_docs(&self) -> u32 {
        self.remaining_docs
    }

    pub fn position_offset(&self) -> u64 {
        self.position_offset
    }

    #[inline]
    pub fn byte_offset(&self) -> usize {
        self.byte_offset
    }

    fn read_block_info(&mut self) {
        let bytes = self.owned_read.as_slice();
        let advance_len: usize;
        self.last_doc_in_block = read_u32(bytes);
        if let Some(num_longs) = decode_dense_marker(bytes[4]) {
            match self.skip_info {
                IndexRecordOption::Basic => {
                    advance_len = 5;
                    self.block_info = BlockInfo::Dense {
                        num_longs,
                        tf_num_bits: 0,
                        tf_sum: 0,
                        block_wand_fieldnorm_id: 0,
                        block_wand_term_freq: 0,
                    };
                }
                IndexRecordOption::WithFreqs => {
                    let tf_num_bits = bytes[5];
                    let block_wand_fieldnorm_id = bytes[6];
                    let block_wand_term_freq = decode_block_wand_max_tf(bytes[7]);
                    advance_len = 8;
                    self.block_info = BlockInfo::Dense {
                        num_longs,
                        tf_num_bits,
                        tf_sum: 0,
                        block_wand_fieldnorm_id,
                        block_wand_term_freq,
                    };
                }
                IndexRecordOption::WithFreqsAndPositions => {
                    let tf_num_bits = bytes[5];
                    let tf_sum = read_u32(&bytes[6..10]);
                    let block_wand_fieldnorm_id = bytes[10];
                    let block_wand_term_freq = decode_block_wand_max_tf(bytes[11]);
                    advance_len = 12;
                    self.block_info = BlockInfo::Dense {
                        num_longs,
                        tf_num_bits,
                        tf_sum,
                        block_wand_fieldnorm_id,
                        block_wand_term_freq,
                    };
                }
            }
            self.owned_read.advance(advance_len);
            return;
        }
        let (doc_num_bits, strict_delta_encoded) = decode_bitwidth(bytes[4]);
        match self.skip_info {
            IndexRecordOption::Basic => {
                advance_len = 5;
                self.block_info = BlockInfo::BitPacked {
                    doc_num_bits,
                    strict_delta_encoded,
                    tf_num_bits: 0,
                    tf_sum: 0,
                    block_wand_fieldnorm_id: 0,
                    block_wand_term_freq: 0,
                };
            }
            IndexRecordOption::WithFreqs => {
                let tf_num_bits = bytes[5];
                let block_wand_fieldnorm_id = bytes[6];
                let block_wand_term_freq = decode_block_wand_max_tf(bytes[7]);
                advance_len = 8;
                self.block_info = BlockInfo::BitPacked {
                    doc_num_bits,
                    strict_delta_encoded,
                    tf_num_bits,
                    tf_sum: 0,
                    block_wand_fieldnorm_id,
                    block_wand_term_freq,
                };
            }
            IndexRecordOption::WithFreqsAndPositions => {
                let tf_num_bits = bytes[5];
                let tf_sum = read_u32(&bytes[6..10]);
                let block_wand_fieldnorm_id = bytes[10];
                let block_wand_term_freq = decode_block_wand_max_tf(bytes[11]);
                advance_len = 12;
                self.block_info = BlockInfo::BitPacked {
                    doc_num_bits,
                    strict_delta_encoded,
                    tf_num_bits,
                    tf_sum,
                    block_wand_fieldnorm_id,
                    block_wand_term_freq,
                };
            }
        }
        self.owned_read.advance(advance_len);
    }

    pub fn block_info(&self) -> BlockInfo {
        self.block_info
    }

    /// Advance the skip reader to the block that may contain the target.
    ///
    /// If the target is larger than all documents, the skip_reader
    /// then advance to the last Variable In block.
    pub fn seek(&mut self, target: DocId) -> bool {
        if self.last_doc_in_block() >= target {
            return false;
        }
        // The current block is a full one ending before the target. Account
        // for it as `advance` does, then pass over every following full block
        // that also ends before the target reading only its last doc and
        // sizes; only the block finally reached is parsed.
        self.account_current_block();
        let stride = match self.skip_info {
            IndexRecordOption::Basic => 5,
            IndexRecordOption::WithFreqs => 8,
            IndexRecordOption::WithFreqsAndPositions => 12,
        };
        loop {
            if self.remaining_docs < COMPRESSION_BLOCK_SIZE as u32 {
                self.last_doc_in_block = TERMINATED;
                self.block_info = BlockInfo::VInt {
                    num_docs: self.remaining_docs,
                };
                return true;
            }
            let bytes = self.owned_read.as_slice();
            let last_doc = read_u32(bytes);
            if last_doc >= target {
                self.read_block_info();
                return true;
            }
            let tf_header = if self.skip_info == IndexRecordOption::Basic {
                0
            } else {
                bytes[5]
            };
            let doc_bytes = match decode_dense_marker(bytes[4]) {
                Some(num_longs) => dense_block_size(num_longs),
                None => compressed_block_size(decode_bitwidth(bytes[4]).0),
            };
            if self.skip_info == IndexRecordOption::WithFreqsAndPositions {
                self.position_offset += u64::from(read_u32(&bytes[6..10]));
            }
            self.byte_offset += doc_bytes + compressed_freq_block_size(tf_header);
            self.remaining_docs -= COMPRESSION_BLOCK_SIZE as u32;
            self.last_doc_in_block = last_doc;
            self.last_doc_in_previous_block = last_doc;
            self.owned_read.advance(stride);
        }
    }

    /// The bookkeeping half of `advance` for a parsed full block.
    fn account_current_block(&mut self) {
        let (doc_bytes, tf_num_bits, tf_sum) = match self.block_info {
            BlockInfo::BitPacked {
                doc_num_bits,
                tf_num_bits,
                tf_sum,
                ..
            } => (compressed_block_size(doc_num_bits), tf_num_bits, tf_sum),
            BlockInfo::Dense {
                num_longs,
                tf_num_bits,
                tf_sum,
                ..
            } => (dense_block_size(num_longs), tf_num_bits, tf_sum),
            BlockInfo::VInt { .. } => unreachable!("a tail block ends at TERMINATED"),
        };
        self.remaining_docs -= COMPRESSION_BLOCK_SIZE as u32;
        self.byte_offset += doc_bytes + compressed_freq_block_size(tf_num_bits);
        self.position_offset += tf_sum as u64;
        self.last_doc_in_previous_block = self.last_doc_in_block;
    }

    pub fn advance(&mut self) {
        match self.block_info {
            BlockInfo::BitPacked {
                doc_num_bits,
                tf_num_bits,
                tf_sum,
                ..
            } => {
                self.remaining_docs -= COMPRESSION_BLOCK_SIZE as u32;
                self.byte_offset +=
                    compressed_block_size(doc_num_bits) + compressed_freq_block_size(tf_num_bits);
                self.position_offset += tf_sum as u64;
            }
            BlockInfo::Dense {
                num_longs,
                tf_num_bits,
                tf_sum,
                ..
            } => {
                self.remaining_docs -= COMPRESSION_BLOCK_SIZE as u32;
                self.byte_offset +=
                    dense_block_size(num_longs) + compressed_freq_block_size(tf_num_bits);
                self.position_offset += tf_sum as u64;
            }
            BlockInfo::VInt { num_docs } => {
                debug_assert_eq!(num_docs, self.remaining_docs);
                self.remaining_docs = 0;
                self.byte_offset = usize::MAX;
            }
        }
        self.last_doc_in_previous_block = self.last_doc_in_block;
        if self.remaining_docs >= COMPRESSION_BLOCK_SIZE as u32 {
            self.read_block_info();
        } else {
            self.last_doc_in_block = TERMINATED;
            self.block_info = BlockInfo::VInt {
                num_docs: self.remaining_docs,
            };
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{
        decode_bitwidth, encode_bitwidth, BlockInfo, IndexRecordOption, SkipReader, SkipSerializer,
    };
    use crate::directory::OwnedBytes;
    use crate::postings::compression::COMPRESSION_BLOCK_SIZE;

    #[test]
    fn seek_matches_repeated_advance() {
        let mut state = 0x2545_f491_4f6c_dd1du64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for option in [
            IndexRecordOption::Basic,
            IndexRecordOption::WithFreqs,
            IndexRecordOption::WithFreqsAndPositions,
        ] {
            for round in 0..60 {
                let blocks = 1 + (next() % 40) as usize;
                let mut serializer = SkipSerializer::new();
                let mut last = 0u32;
                let mut ends = Vec::new();
                for _ in 0..blocks {
                    last += 128 + (next() % 3000) as u32;
                    ends.push(last);
                    if next() % 2 == 0 {
                        serializer.write_doc_dense(last, 1 + (next() % 20) as u8);
                    } else {
                        serializer.write_doc(last, (next() % 32) as u8);
                    }
                    if option != IndexRecordOption::Basic {
                        serializer.write_term_freq((next() % 33) as u8);
                        if option == IndexRecordOption::WithFreqsAndPositions {
                            serializer.write_total_term_freq((next() % 5000) as u32);
                        }
                        serializer.write_blockwand_max((next() % 256) as u8, (next() % 300) as u32);
                    }
                }
                let tail = (round % 3) as u32 * 50;
                let doc_freq = (blocks * COMPRESSION_BLOCK_SIZE) as u32 + tail;
                let data = OwnedBytes::new(serializer.data().to_vec());
                let fresh = || SkipReader::new(data.clone(), doc_freq, option);
                for _ in 0..30 {
                    let mut seeking = fresh();
                    let mut stepping = fresh();
                    let mut target = 0u32;
                    for _ in 0..4 {
                        target += (next() % u64::from(last / 2 + 2)) as u32;
                        let moved = seeking.seek(target);
                        let mut stepped = false;
                        while stepping.last_doc_in_block() < target {
                            stepping.advance();
                            stepped = true;
                        }
                        assert_eq!(moved, stepped);
                        assert_eq!(seeking.last_doc_in_block, stepping.last_doc_in_block);
                        assert_eq!(
                            seeking.last_doc_in_previous_block,
                            stepping.last_doc_in_previous_block
                        );
                        assert_eq!(seeking.byte_offset, stepping.byte_offset);
                        assert_eq!(seeking.position_offset, stepping.position_offset);
                        assert_eq!(seeking.remaining_docs, stepping.remaining_docs);
                        assert_eq!(seeking.block_info, stepping.block_info);
                        assert_eq!(seeking.owned_read.as_slice(), stepping.owned_read.as_slice());
                    }
                }
            }
        }
    }

    #[test]
    fn block_at_from_matches_block_at_for_every_hint() {
        let ends: Vec<u32> = (0..37u32).map(|block| 3 + block * 7 + block * block).collect();
        let mut data = Vec::new();
        for end in &ends {
            data.extend_from_slice(&end.to_le_bytes());
            data.push(0);
        }
        let directory = super::SkipDirectory::new(
            OwnedBytes::new(data),
            ends.len() as u32 * COMPRESSION_BLOCK_SIZE as u32 + 5,
            IndexRecordOption::Basic,
        );
        assert!(directory.block_at(0) == 0 && directory.block_at(ends[36] + 1) == 37);
        for doc in 0..ends[36] + 3 {
            let expected = directory.block_at(doc);
            for hint in 0..40 {
                assert_eq!(directory.block_at_from(doc, hint), expected, "{doc} {hint}");
            }
        }
    }

    #[test]
    fn test_encode_block_wand_max_tf() {
        for tf in 0..255 {
            assert_eq!(super::encode_block_wand_max_tf(tf), tf as u8);
        }
        for &tf in &[255, 256, 1_000_000, u32::MAX] {
            assert_eq!(super::encode_block_wand_max_tf(tf), 255);
        }
    }

    #[test]
    fn test_decode_block_wand_max_tf() {
        for tf in 0..255 {
            assert_eq!(super::decode_block_wand_max_tf(tf), tf as u32);
        }
        assert_eq!(super::decode_block_wand_max_tf(255), u32::MAX);
    }

    #[test]
    fn test_saturated_frequency_has_no_stored_score_bound() {
        // Raising the frequency to the saturation sentinel can lower the
        // computed f32 factor by an ulp, despite the mathematical monotonicity.
        let original_freq = 4_294_967_040u32 as f32;
        let saturated_freq = u32::MAX as f32;
        let norm = 512.0f32;
        assert!(original_freq / (original_freq + norm) > saturated_freq / (saturated_freq + norm));
        let weight = crate::query::Bm25Weight::for_one_term(128, 1024, 10.0);
        for dense in [false, true] {
            let mut serializer = SkipSerializer::new();
            if dense {
                serializer.write_doc_dense(200, 4);
            } else {
                serializer.write_doc(200, 2);
            }
            serializer.write_term_freq(32);
            serializer.write_blockwand_max(255, 4_294_967_040);
            let reader = SkipReader::new(
                OwnedBytes::new(serializer.data().to_vec()),
                COMPRESSION_BLOCK_SIZE as u32,
                IndexRecordOption::WithFreqs,
            );
            assert_eq!(reader.block_max_score(&weight), None);
        }
    }

    #[test]
    fn test_skip_with_freq() {
        let buf = {
            let mut skip_serializer = SkipSerializer::new();
            skip_serializer.write_doc(1u32, 2u8);
            skip_serializer.write_term_freq(3u8);
            skip_serializer.write_blockwand_max(13u8, 3u32);
            skip_serializer.write_doc(5u32, 5u8);
            skip_serializer.write_term_freq(2u8);
            skip_serializer.write_blockwand_max(8u8, 2u32);
            skip_serializer.data().to_owned()
        };
        let doc_freq = 3u32 + (COMPRESSION_BLOCK_SIZE * 2) as u32;
        let mut skip_reader =
            SkipReader::new(OwnedBytes::new(buf), doc_freq, IndexRecordOption::WithFreqs);
        assert_eq!(skip_reader.last_doc_in_block(), 1u32);
        assert_eq!(
            skip_reader.block_info,
            BlockInfo::BitPacked {
                doc_num_bits: 2u8,
                strict_delta_encoded: true,
                tf_num_bits: 3u8,
                tf_sum: 0,
                block_wand_fieldnorm_id: 13,
                block_wand_term_freq: 3
            }
        );
        skip_reader.advance();
        assert_eq!(skip_reader.last_doc_in_block(), 5u32);
        assert_eq!(
            skip_reader.block_info(),
            BlockInfo::BitPacked {
                doc_num_bits: 5u8,
                strict_delta_encoded: true,
                tf_num_bits: 2u8,
                tf_sum: 0,
                block_wand_fieldnorm_id: 8,
                block_wand_term_freq: 2
            }
        );
        skip_reader.advance();
        assert_eq!(skip_reader.block_info(), BlockInfo::VInt { num_docs: 3u32 });
        skip_reader.advance();
        assert_eq!(skip_reader.block_info(), BlockInfo::VInt { num_docs: 0u32 });
        skip_reader.advance();
        assert_eq!(skip_reader.block_info(), BlockInfo::VInt { num_docs: 0u32 });
    }

    #[test]
    fn test_skip_no_freq() {
        let buf = {
            let mut skip_serializer = SkipSerializer::new();
            skip_serializer.write_doc(1u32, 2u8);
            skip_serializer.write_doc(5u32, 5u8);
            skip_serializer.data().to_owned()
        };
        let doc_freq = 3u32 + (COMPRESSION_BLOCK_SIZE * 2) as u32;
        let mut skip_reader =
            SkipReader::new(OwnedBytes::new(buf), doc_freq, IndexRecordOption::Basic);
        assert_eq!(skip_reader.last_doc_in_block(), 1u32);
        assert_eq!(
            skip_reader.block_info(),
            BlockInfo::BitPacked {
                doc_num_bits: 2u8,
                strict_delta_encoded: true,
                tf_num_bits: 0,
                tf_sum: 0u32,
                block_wand_fieldnorm_id: 0,
                block_wand_term_freq: 0
            }
        );
        skip_reader.advance();
        assert_eq!(skip_reader.last_doc_in_block(), 5u32);
        assert_eq!(
            skip_reader.block_info(),
            BlockInfo::BitPacked {
                doc_num_bits: 5u8,
                strict_delta_encoded: true,
                tf_num_bits: 0,
                tf_sum: 0u32,
                block_wand_fieldnorm_id: 0,
                block_wand_term_freq: 0
            }
        );
        skip_reader.advance();
        assert_eq!(skip_reader.block_info(), BlockInfo::VInt { num_docs: 3u32 });
        skip_reader.advance();
        assert_eq!(skip_reader.block_info(), BlockInfo::VInt { num_docs: 0u32 });
        skip_reader.advance();
        assert_eq!(skip_reader.block_info(), BlockInfo::VInt { num_docs: 0u32 });
    }

    #[test]
    fn test_skip_multiple_of_block_size() {
        let buf = {
            let mut skip_serializer = SkipSerializer::new();
            skip_serializer.write_doc(1u32, 2u8);
            skip_serializer.data().to_owned()
        };
        let doc_freq = COMPRESSION_BLOCK_SIZE as u32;
        let mut skip_reader =
            SkipReader::new(OwnedBytes::new(buf), doc_freq, IndexRecordOption::Basic);
        assert_eq!(skip_reader.last_doc_in_block(), 1u32);
        assert_eq!(
            skip_reader.block_info(),
            BlockInfo::BitPacked {
                doc_num_bits: 2u8,
                strict_delta_encoded: true,
                tf_num_bits: 0,
                tf_sum: 0u32,
                block_wand_fieldnorm_id: 0,
                block_wand_term_freq: 0
            }
        );
        skip_reader.advance();
        assert_eq!(skip_reader.block_info(), BlockInfo::VInt { num_docs: 0u32 });
    }

    #[test]
    fn test_encode_decode_bitwidth() {
        for bitwidth in 0..32 {
            for delta_1 in [false, true] {
                assert_eq!(
                    (bitwidth, delta_1),
                    decode_bitwidth(encode_bitwidth(bitwidth, delta_1))
                );
            }
        }
        assert_eq!(0b01000010, encode_bitwidth(0b10, true));
        assert_eq!(0b00000010, encode_bitwidth(0b10, false));
    }

    #[test]
    fn test_skip_dense_basic() {
        use crate::postings::compression::dense_block_size;
        let buf = {
            let mut skip_serializer = SkipSerializer::new();
            skip_serializer.write_doc_dense(200u32, 4u8);
            skip_serializer.write_doc(500u32, 6u8);
            skip_serializer.data().to_owned()
        };
        let doc_freq = (COMPRESSION_BLOCK_SIZE * 2) as u32;
        let mut skip_reader =
            SkipReader::new(OwnedBytes::new(buf), doc_freq, IndexRecordOption::Basic);
        assert_eq!(skip_reader.last_doc_in_block(), 200u32);
        assert_eq!(
            skip_reader.block_info(),
            BlockInfo::Dense {
                num_longs: 4u8,
                tf_num_bits: 0,
                tf_sum: 0u32,
                block_wand_fieldnorm_id: 0,
                block_wand_term_freq: 0,
            }
        );
        assert_eq!(skip_reader.byte_offset(), 0);
        skip_reader.advance();
        assert_eq!(skip_reader.last_doc_in_block(), 500u32);
        assert_eq!(skip_reader.byte_offset(), dense_block_size(4));
    }

    #[test]
    fn test_skip_dense_with_freq() {
        let buf = {
            let mut skip_serializer = SkipSerializer::new();
            skip_serializer.write_doc_dense(200u32, 3u8);
            skip_serializer.write_term_freq(4u8);
            skip_serializer.write_blockwand_max(9u8, 7u32);
            skip_serializer.data().to_owned()
        };
        let doc_freq = (COMPRESSION_BLOCK_SIZE) as u32 + 5;
        let skip_reader =
            SkipReader::new(OwnedBytes::new(buf), doc_freq, IndexRecordOption::WithFreqs);
        assert_eq!(
            skip_reader.block_info(),
            BlockInfo::Dense {
                num_longs: 3u8,
                tf_num_bits: 4u8,
                tf_sum: 0u32,
                block_wand_fieldnorm_id: 9,
                block_wand_term_freq: 7,
            }
        );
    }
}
