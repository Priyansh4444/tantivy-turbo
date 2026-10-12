# Batched exact leaves, then bounded OR windows

Design only. No production edits, builds, tests, profiles, timings or commits were performed. I read grounding.md in full, the complete production body of or_maxscore.rs, TermScorer's production implementation, SegmentPostings ownership/DocSet implementation, BlockSegmentPostings block loading/selection/accessors, the BM25 scalar expression, BooleanWeight routing, the score Top-K collector, and the port's TermScorer batching and MaxScoreBulkScorer production implementation.

## Caller first: one driver batch is the first isolated attempt

Keep BooleanWeight::for_each_pruning and its 3..32 admission exactly as they stand. Keep safe-domain fallback before mutation, current global-density policy, preference order, local region boundary, conservative prefix construction and partition. Inside a certified region, if the essential suffix contains exactly one live clause, replace its repeated doc/score/advance calls with a block-local scored batch. Replay that batch in increasing doc order through the existing optional-probe loop, exact original-order contributions and dynamic callback threshold.

The call site is approximately:

```rust
if live_essential_count == 1 {
    let driver_rank = /* unique live essential */;
    while clauses[driver_rank].doc < hi {
        let len = clauses[driver_rank].fill_scored_batch(
            &mut scorers, lo, hi, &mut batch,
        );
        if len == 0 { break; }
        for i in 0..len {
            let doc = batch.docs[i];
            contributions.fill(0.0);
            contributions[batch.ordinal] = batch.scores[i];
            let known = f64::from(batch.scores[i]);
            // Existing reverse optional probing, with current threshold.
            // Existing incoming-order f64 fold and one final f32 cast.
            // Existing callback only when exact score > current threshold.
        }
    }
} else {
    // Existing general essential union loop, unchanged.
}
```

Do not accidentally classify an exhausted suffix as a second driver. Conversely, choosing one non-exhausted essential is insufficient when another suffix clause still has a physical doc below hi. A tight first attempt can require `num_terms - first_essential == 1`, avoiding a separate live-count selection mechanism entirely. Broader live-single selection can be measured later.

The driver cursor advances ahead of callback replay, but its exact leaves are retained in the batch. Optional cursors still seek in ascending candidate order. The driver's ClauseState.doc always mirrors the actual first unread doc; it is deliberately not the current replay candidate. While buffered candidates remain, do not use that mirror to rebuild a region or repartition. This is a small state machine with an explicit buffered phase, not a second approximate doc mirror.

This first unit changes four existing modules, adds no new public API, and leaves the generic OR path as a direct control. Two-term routing stays unchanged. It isolates the batching/vector arithmetic mechanism from new heap/window and partition policies. It cannot alone prove a win for all ORs, particularly the 198 two-term queries.

## Source facts and causal hypothesis

Our per-driver hit currently executes the essential min scan, scans essentials again to gather leaves, calls TermScorer::score through its decoded cursor and norm lookup, folds incoming contributions, then scans and advances matching essentials. The port's single-essential path obtains a block's docs/scores at once and replays candidates. Its TermScorer gathers norm inverses first and then executes a pure arithmetic loop; its multiple-essential engine gathers into a bitmap window.

Our decoded docs/frequencies already exist in BlockSegmentPostings. No new codec bits are required. SegmentPostings.cur is private and is the authority for `doc`, `term_freq`, seek, advance and position offsets. A batch method on SegmentPostings must own that state change; exposing cur or mutating only BlockSegmentPostings would categorically create incoherent cursors.

Native scalar score is literally `weight - weight / (1.0 + frequency * norm)` with f32 inputs/results and frequency cast from u32. Legacy score is literally `weight * (frequency / (frequency + norm))`. The batch kernel must use these expressions, not the port's weight/cache conventions, alternate algebra, reciprocal approximation, FMA or a frequency-clamping trick. Match the scoring convention once outside the arithmetic loop.

The exact latest trace says region prefix construction is hot (8.89% sampled store and 2.12% loop increment), and candidate essential/scoring work is also visible. Those first samples are not contribution clearing. A two-pass kernel or single-driver batching does not remove region prefix reconstruction, so it has no demonstrated ability to halve total runtime. Its potential comes from collapsing cursor operations, moving weight-mode branching out of a loop, contiguous norm/frequency arithmetic and independent divisions that LLVM might vectorize. Retain assembly/profile evidence after measurement; do not describe vectorization as observed before inspecting generated code.

The structural second design below attacks multi-essential min/scans and leaf bookkeeping too. It has more potential to close a 2.3x gap, with a larger fixed scratch cost. Neither mechanism is a language-based promise.

## Private data and module map

Use the codec's COMPRESSION_BLOCK_SIZE (currently 128), rather than a new magic batch width.

```rust
// term_query/term_scorer.rs or a private sibling if this materially shortens it.
struct ScoredTermBatch {
    docs: [DocId; COMPRESSION_BLOCK_SIZE],
    freqs: [u32; COMPRESSION_BLOCK_SIZE],
    scores: [Score; COMPRESSION_BLOCK_SIZE],
    norms: [Score; COMPRESSION_BLOCK_SIZE],
    len: usize,
    ordinal: usize, // OR layer may instead keep this separately.
}

// postings/segment_postings.rs, crate-private.
fn read_postings_batch(
    &mut self,
    floor: DocId,
    up_to: DocId, // exclusive; floor <= up_to <= TERMINATED
    docs: &mut [DocId; COMPRESSION_BLOCK_SIZE],
    freqs: &mut [u32; COMPRESSION_BLOCK_SIZE],
) -> usize;

// query/bm25.rs, crate-private; arrays may be slices with equal-length checks.
fn score_batch(
    &self,
    fieldnorms: &FieldNormReader,
    docs: &[DocId],
    freqs: &[u32],
    norm_scratch: &mut [Score],
    scores: &mut [Score],
);

// term_query/term_scorer.rs, crate-private.
fn fill_scored_batch(
    &mut self, floor: DocId, up_to: DocId, batch: &mut ScoredTermBatch,
) -> usize;

// boolean_query/or_maxscore.rs, private ClauseState mutation owner.
fn fill_scored_batch(
    &mut self, scorers: &mut [TermScorer], floor: DocId, up_to: DocId,
    batch: &mut ScoredTermBatch,
) -> usize;
```

Avoid carrying ordinal inside the term scorer's batch if that couples the postings API to query order: `ScoredTermBatch` can omit ordinal and the OR caller keeps the incoming ordinal locally. The batch has no heap allocations; allocate it once per admitted query, not once per region. Existing arrays remain allocated once. There is no pooling layer in this first unit.

`score_batch` imports the already used FieldNormReader and takes docs directly, so it can gather cached norm components without allocating a separate u8 norm-ID array. An alternative is a norm-ID buffer and a BM25-only kernel, but that adds 128 bytes plus another pass and API parameter without an actual caller benefit here.

## SegmentPostings: actual cursor ownership

`read_postings_batch` first reconciles any shallow-selected unloaded block. If `!block_cursor.block_is_loaded()` or current decoded doc is below floor, use `cur = block_cursor.seek(floor)` directly. Do not call the ordinary SegmentPostings::seek shortcut on an unloaded selection and assume equality means the decoded doc is current. The private batch method explicitly handles loaded-vs-shallow state.

The caller guarantees floor is at least its current logical region floor and never below any already-consumed doc; up_to is the certified half-open region end. Current or_maxscore selects a shallow block at `max(lo, clause.doc)`. A stale old doc below lo cannot short-circuit reconciliation. After copying one block suffix, return with an exact decoded first-unread doc.

Pseudocode:

```text
if decoded doc is TERMINATED and block is loaded: return 0
if block is not loaded OR decoded doc < floor:
    cur = block_cursor.seek(floor)  // loads selected block, establishes cur
if doc() >= up_to OR doc() == TERMINATED: return 0
start = cur
end = first decoded real index >= up_to, or block_len
copy docs[start..end] and freqs[start..end] to caller buffers
len = end - start
if end == COMPRESSION_BLOCK_SIZE:
    cur = 0
    block_cursor.advance()         // loads next block; authoritative doc
else:
    cur = end                      // partial-block boundary or padded tail sentinel
return len
```

A full block's exact end advances to the next block and loads it, matching scalar advance from slot127. A tail with block_len<128 ends on the existing TERMINATED padding at `cur=block_len`, without reading beyond real docs. If up_to cuts a block, cur stays on its first doc>=up_to. Empty/exhausted lists remain terminated. Never include TERMINATED in the norm-gather batch. Full/burst/gapped blocks are all the same decoded-array contract.

For Basic/SkipFreq, frequencies should use the same values as scalar `term_freq()`. Current BlockSegmentPostings initializes its frequency decoder to ones and only replaces it for ReadFreq; ensure the array-copy helper copies a length valid for this representation, or explicitly fill the output frequency prefix with 1 when not ReadFreq. This branch is outside the per-hit arithmetic loop. Do not assume `freqs().len()` equals real docs for a no-frequency decoder without inspecting/asserting the invariant.

Position semantics remain correct: current_position_read_offset uses current cur and the selected block position_offset; its cache compares block offset and cur. A batched advance across blocks changes that offset exactly as normal advance. No positions are read by this OR path, but a cursor-ownership fixture should test subsequent positions on an unread doc to prove no hidden offset regression.

TermScorer delegates to this helper, calls exact BM25 batch scoring, and leaves batch.len populated. ClauseState calls this one method, then sets `self.doc = scorer.doc()` once and checks mirror coherence before/after. All batch cursor movement is centralized in this method; no raw block_cursor mutation is added to the OR engine.

## Exact two-pass arithmetic

First loop: for each doc, get the actual fieldnorm ID from FieldNormReader and gather the literal cached component into norms[i]. Never clamp valid doc IDs or norm lookups; valid batches already require doc<physical max_doc. Avoid copying the port's corrupt-input clamp since changing validation behavior is not part of this performance unit.

Second loop, convention selected once:

```text
Native:
    frequency = freqs[i] as f32
    scores[i] = weight - weight / (1.0f32 + frequency * norms[i])
Legacy:
    frequency = freqs[i] as f32
    scores[i] = weight * (frequency / (frequency + norms[i]))
```

The default/configured cached normalization computation is unchanged. BM25 eligibility is not widened: the current safe-domain guard still controls admission; exceptional/signed/NaN domains fall back before mutation. Even if the batch kernel can compute those domains, the OR pruning engine must not admit them anew.

Published Boolean scores retain incoming leaf ordinal. The first attempt stores driver_scores[i] and executes optional scoring just in time, inserts every matching leaf into the same incoming contributions vector, sums it in f64 and casts once. It does not need an n×batch leaf matrix because only one clause is prefetched. This is the compact candidate replay strategy: one leaf column retained, other leaves computed for the current candidate.

Known-score pruning arithmetic remains the existing preference-order f64 sum and ScoreSumUpperBound(original_n). It is not used as the published score. No subtraction from total maximum creates bounds.

## Shallow selection, tails, callback thresholds and deletions

All existing region caps remain. A batch cannot extend beyond hi because its leaf contributions were certified with the currently selected local bounds. Globally optional dense terms still omit their end caps only under the existing density policy. Tail reconciliation happens as today; keep the previously selected last_doc when calculating hi.

Fetching a batch ahead does not justify returning early when the driver's current decoded doc passes hi: every retained candidate below hi must first be replayed or safely rejected. Once len rows are exhausted, the first unread cursor is ready for the next batch/region.

The callback threshold is read fresh for each replay candidate. Thus a previous hit/deletion callback changes subsequent optional-pruning decisions exactly as in the scalar driver. Retain the region's certified partition until the batch is exhausted; a rising threshold makes old bounds/partition conservative, merely less efficient. Do not add threshold-crossing repartition inside this first unit. Later adaptive partition changes must explicitly distinguish unread cursor docs from already buffered docs.

Deletion filtering remains in the Top-K collector callback. The batching API does not drop deleted docs: doing so would change observable callback traces for custom callers. Deleted IDs remain part of physical max_doc and local ranges. The collector rejects a deleted candidate and returns its existing threshold; replay proceeds with that value. No `reader.num_docs()` substitution.

Strict `score > threshold`, ascending callback order and exact f32 bits preserve ties and doc ordering. A batch must not call the callback for all candidates using a stale threshold captured at entry.

## Scratch and work budget for attempt one

At n=32 the existing compact metadata/prefix/contributions arrays contain 1,160 bytes. A batch contains four128-element 4-byte arrays: docs512 + freqs512 + scores512 + cached norms512 =2,048 bytes. Combined array payload: 3,208 bytes. Vec headers remain the existing three72 bytes; batch scalar fields/alignment add roughly16 bytes (record actual sizeof). The first unit adds no allocations beyond those already present. Stack placement may increase frame size by~2KiB; if that harms stack behavior, box once/query, accepting one allocation explicitly rather than inventing a pool.

Per128 driver hits: one suffix copy, one norm gather, one arithmetic loop, one authoritative cursor update. Per candidate: optional seeks/pruning, the original contribution clear/fold and callback remain. Region prefix and block-bound work remains. Copies and scoring advance at most one decoded block per fill. Dynamic threshold growth can cause wasted prefetched division work; scalar driver also scores every current essential candidate, but prefetch loses the opportunity for any newly added repartition to avoid those leaves. Since the current engine does not repartition within a region, this first unit adds no such pruning loss today.

Likely losses: tiny tail batches; sparse regions where a block is clipped to one hit; optional probes dominating runtime; single-essential regions rare; weight-mode loop duplication bloating the inlined BooleanWeight function; copying arrays outweighing eliminated cursor operations. Dense matched driver blocks are the best prospective cases. Profile/receipt-driven selection is mandatory; do not ship an unmeasured special path.

## Structurally distinct second attempt: exact-leaf bitmap window

This is not part of the first diff. It shares the batched term API but replaces multiple-essential candidate merging with a bounded physical window (initial W=128), a bitmap and an original-order f32 leaf matrix. It removes the O(essential_count) minimum scan and matching/advance scans for every candidate.

Choose `window_lo = min essential doc`, `window_hi = min(region_hi, saturating(window_lo + W))`. Batch each essential clause only below window_hi, scatter f32 leaves into `leaves[doc-window_lo][incoming_ordinal]` and set the matching bit. Then enumerate bitmap candidates in ascending doc order. Optional clauses can initially stay scalar per candidate for an isolated essential-window unit. Final scores fold that row in original ordinal f64, cast once and use the current callback threshold.

For a fuller clause-major engine, process optional clauses in reverse pruning order across sorted candidate rows at batch-entry threshold. Compact survivors in an index list; gather matched doc/freq pairs into the existing batch, exact-score them together, then scatter f32 leaves. Keep conservative pruning `known + inclusive_remaining_prefix`; known may be accumulated in pruning order only for this bound, not publication. All candidate rows are exact-scored before callbacks. During the final ascending replay, compare each exact score to the current threshold after every callback. A threshold increase may create avoidable work, but cannot discard a genuinely competitive later hit or call the callback below its current threshold. This differs from scalar internal work while preserving the externally visible callback trace.

No claim that an arbitrary sorted f64 sum is the exact published sum. The port's mutable wrapper-order accumulation is rejected. Incoming leaves must survive until replay, including optional leaves; otherwise exact sums cannot be reconstructed without rewinding consumed cursors.

With W128,n32: leaf matrix16,384B, match bitmap16B, known score column1,024B, candidate offsets at most512B, batch2,048B, metadata/prefix/contributions1,160B ->21,144B array payload (some existing contributions may be deleted in this attempt). One query-time allocation for the matrix with capacity n×W, zero reused in place. No allocation per region/window. Row-major leaves keep exact replay contiguous; the batch outputs arithmetic scores contiguously before scatter. Clearing21KiB for a one-candidate window is an unacceptable default. A sparse single-driver branch must bypass matrix work, and a touched-row reset strategy should clear only rows previously used.

An ordinal-major matrix improves scatter stores but makes candidate replay strided; measure separately rather than choosing from intuition. A 4096-wide matrix32×4096×4 =512KiB is rejected as a compactness regression. W128/256 is a real bounded tradeoff against the port's80.5KiB scratch, not an assertion that the port's exact same work fits ours.

Do not widen `window_hi` beyond the existing certified region. The port has richer impact levels and can validly certify broader ranges. Our same-format local block maximum only covers its selected block. Replacing it with a global maximum for a wide window changes the pruning mechanism and may repeat the prior401/719/791/691/909/817 failures; it is a separate attempt if needed, never hidden in the batch implementation.

Multiple essentials consume their window postings ahead. Their ClauseState.doc mirrors the actual first unread doc, while the matrix owns all buffered leaves. The outer region/partition code runs only after window replay. Tail and unloaded reconciliation follow the same batch API. This ownership model prevents cursor rewind and stale-score reads.

## Alternatives rejected or separated

- Generic3..32 engine for two clauses: separate routing trial; current two-term arithmetic/algorithm is a major negative control and its2.7x gap cannot be solved by this unit's unchanged route.
- Port running f64 accumulation: loses the original ordinal score contract; rejected.
- Huge row matrices/window pooling: extra memory/lifetime architecture before proof of a win; rejected initially.
- Exposing SegmentPostings.cur or scoring by mutating BlockSegmentPostings alone: incoherent doc/term_freq/position state; rejected categorically.
- Native saturation-input algebra in place of literal score: can differ at f32 boundaries; rejected.
- More region caching or local repartition in the same first attempt: confounds batch effect; separate candidates.
- Batched optional probing alone: still scalar seek cost and greater eager scoring; only worth a follow-on when profiles show optional scoring dominance.
- Prefiltering deleted docs in batch: changes generic callback behavior; keep collector-owned deletes.

## Independent meaningful verification

1. Cursor ownership oracle: serialize independent literal posting lists with lengths1,127,128,129,255,256,257, gaps and bursts; drive scalar seek/score/advance and batch reads with arbitrary half-open floors/ends. Compare complete emitted docs/raw leaf bits and exact first-unread doc/frequency after each call. Mix subsequent scalar seek/advance; test physical max_doc and TERMINATED boundaries. Include Basic/SkipFreq and dense encoding, not merely normal WithFreqs.
2. Shallow stale-state fixture: select ahead through complete blocks, then batch with a floor at that region and a deferred optional-to-essential clause. Include a selected tail where no actual doc>=floor remains. Independent buffered union must match the whole callback raw-bit trace.
3. Native/legacy arithmetic oracle: every normID0..255, frequencies1,7,128,16_777_217,u32::MAX, default and configured native weights, legacy weights, boosts and float-neighbor thresholds. Compare batch leaf bits to existing scalar score before Boolean replay. Include valid configured b1/norm0 behavior and signed/exceptional fallback domains.
4. Batch-entry versus callback threshold: one driver with128+ hits, early Top1 winner raising the threshold, later weak candidates and later stronger winner. Exact whole trace against BufferedUnionScorer, not final heap alone. Include deleted early winner in an actual index to ensure it does not raise threshold.
5. Original-order midpoint adversary: incoming leaves chosen around f32 rounding midpoints with differing magnitudes and an order unlike pruning order; compare raw published bits at score.next_down()/score/score.next_up(), limits0/1/10. This catches matrix/port running-sum shortcuts.
6. First-unread boundary fixture: driver batch exactly ends at region hi or128th slot; keep optional term stale below floor; subsequent region must see all unread hits exactly once. Test zero-length clipped batch to prove progress/no infinite loop.
7. For the later bitmap engine, independently compare candidate sets and full callbacks for sparse/disjoint essential terms, overlapping dense terms, partial windows, huge doc gaps, n3/n32, and thresholds changed repeatedly within one window. Include touched-row resets across windows to catch leftover leaves.

Retain all current10 raw-bit trace fixtures, full1,428+ tests,1,227 query exact default and two configured gates. Main balanced measurements include all1,221 cases, all301OR/1033+ and existing two-term/nonOR controls, AB/BA and candidate/candidate/baseline/baseline spread. Inspect all per-query regressions including named previous policy failures. A pilot gain alone is not acceptance. Fresh actual-port comparison is the final ruler; unchanged indexes and same harness/build/compiler bind the result. The expected single-essential unit can at best advance the3+ target; declare the allOR target pending while the two-term route remains unchanged.
