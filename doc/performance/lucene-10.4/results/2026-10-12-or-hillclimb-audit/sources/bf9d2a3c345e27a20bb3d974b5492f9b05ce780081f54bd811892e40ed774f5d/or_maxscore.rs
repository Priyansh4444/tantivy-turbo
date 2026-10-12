use crate::query::score_combiner::{SmallScoreSumUpperBound, SumCombiner};
use crate::query::term_query::{ScoredTermBatch, TermScorer};
use crate::query::weight::for_each_pruning_scorer;
use crate::query::{BufferedUnionScorer, Scorer};
use crate::{DocId, DocSet, Score, TERMINATED};

/// Hot metadata for one clause, sorted by pruning preference. The ordinal still
/// addresses the incoming-order scorer and contribution arrays.
struct ClauseState {
    ordinal: usize,
    global_max: Score,
    cost: u32,
    doc: DocId,
    local_max: Score,
}

impl ClauseState {
    #[inline]
    fn assert_coherent(&self, scorers: &[TermScorer]) {
        debug_assert_eq!(self.doc, scorers[self.ordinal].doc());
    }

    // Actual cursor mutations own the mirror update. Shallow block selection
    // leaves the decoded doc stale in both objects until one of these runs.
    #[inline]
    fn seek(&mut self, scorers: &mut [TermScorer], target: DocId) {
        self.assert_coherent(scorers);
        self.doc = scorers[self.ordinal].seek(target);
        self.assert_coherent(scorers);
    }

    fn fill_scored_batch(
        &mut self,
        scorers: &mut [TermScorer],
        floor: DocId,
        up_to: DocId,
        batch: &mut ScoredTermBatch,
    ) -> usize {
        self.assert_coherent(scorers);
        let len = scorers[self.ordinal].fill_scored_batch(floor, up_to, batch);
        self.doc = scorers[self.ordinal].doc();
        self.assert_coherent(scorers);
        len
    }
}

const OR_WINDOW_DOCS: usize = 128;

/// Retained replay candidates are separate from ClauseState's first-unread
/// cursors. Only an empty window may gather under a new region certificate.
struct ExactLeafWindow {
    incoming_leaves: Vec<Score>,
    essential_known: [f64; OR_WINDOW_DOCS],
    matches: [u64; 2],
    base_doc: DocId,
    num_terms: usize,
}

impl ExactLeafWindow {
    fn new(num_terms: usize) -> Self {
        Self {
            incoming_leaves: vec![0.0; OR_WINDOW_DOCS * num_terms],
            essential_known: [0.0; OR_WINDOW_DOCS],
            matches: [0; 2],
            base_doc: 0,
            num_terms,
        }
    }

    fn is_empty(&self) -> bool {
        self.matches == [0; 2]
    }

    /// Own the complete min/end/fill/scatter phase. Numerical region bounds
    /// stay fixed even when these fills load later decoder blocks.
    fn gather_window(
        &mut self,
        essentials: &mut [ClauseState],
        scorers: &mut [TermScorer],
        hi: DocId,
        batch: &mut ScoredTermBatch,
    ) -> bool {
        debug_assert!(self.is_empty(), "pending rows must drain before gathering");
        debug_assert!(essentials.len() >= 2);
        loop {
            self.base_doc = essentials.iter().map(|clause| clause.doc).min().unwrap();
            if self.base_doc >= hi {
                return false;
            }
            let end = hi.min(self.base_doc.saturating_add(OR_WINDOW_DOCS as DocId));
            debug_assert!(end > self.base_doc);
            #[cfg(test)]
            let mut filled = 0;
            // Preference order preserves the scalar pruning-known additions.
            for clause in essentials.iter_mut() {
                while clause.doc < end {
                    let previous_doc = clause.doc;
                    let len = clause.fill_scored_batch(scorers, self.base_doc, end, batch);
                    debug_assert!(clause.doc > previous_doc, "batch cursor must progress");
                    debug_assert!(len > 0 || clause.doc >= end);
                    #[cfg(test)]
                    if len > 0 {
                        filled += 1;
                    }
                    for index in 0..len {
                        let doc = batch.docs[index];
                        debug_assert!(self.base_doc <= doc && doc < end);
                        let offset = (doc - self.base_doc) as usize;
                        let word = offset / 64;
                        let bit = 1u64 << (offset % 64);
                        let row_start = offset * self.num_terms;
                        if self.matches[word] & bit == 0 {
                            // Dirty storage becomes visible only on first touch.
                            self.incoming_leaves[row_start..row_start + self.num_terms].fill(0.0);
                            self.essential_known[offset] = 0.0;
                            self.matches[word] |= bit;
                        }
                        let leaf = batch.scores[index];
                        self.incoming_leaves[row_start + clause.ordinal] = leaf;
                        self.essential_known[offset] += f64::from(leaf);
                    }
                }
            }
            debug_assert!(essentials.iter().all(|clause| clause.doc >= end));
            #[cfg(test)]
            MULTI_WINDOWS.with(|windows| {
                if let Some(windows) = windows.borrow_mut().as_mut() {
                    windows.push(WindowEvidence {
                        base: self.base_doc,
                        end,
                        hi,
                        matches: self.matches,
                        essentials: essentials.iter().map(|clause| clause.ordinal).collect(),
                        filled,
                        unread: essentials.iter().map(|clause| clause.doc).collect(),
                        matrix_len: self.incoming_leaves.len(),
                        matrix_capacity: self.incoming_leaves.capacity(),
                    });
                }
            });
            if !self.is_empty() {
                return true;
            }
            // A reconciled shallow cursor may emit nothing. Jump directly to
            // the new minimum, rather than walking empty physical addresses.
        }
    }

    fn pop_candidate(&mut self) -> Option<(DocId, f64, &mut [Score])> {
        let word = if self.matches[0] != 0 { 0 } else { 1 };
        if self.matches[word] == 0 {
            return None;
        }
        let bit = self.matches[word].trailing_zeros() as usize;
        self.matches[word] &= self.matches[word] - 1;
        let offset = word * 64 + bit;
        let row_start = offset * self.num_terms;
        Some((
            self.base_doc + offset as DocId,
            self.essential_known[offset],
            &mut self.incoming_leaves[row_start..row_start + self.num_terms],
        ))
    }
}

#[cfg(test)]
#[derive(Debug)]
struct WindowEvidence {
    base: DocId,
    end: DocId,
    hi: DocId,
    matches: [u64; 2],
    essentials: Vec<usize>,
    filled: usize,
    unread: Vec<DocId>,
    matrix_len: usize,
    matrix_capacity: usize,
}

#[cfg(test)]
type RegionChoice = (DocId, usize, usize, u64);

#[cfg(test)]
thread_local! {
    // Test-only region evidence; no observer or counter exists in release.
    static GLOBAL_CHOICES: std::cell::RefCell<Option<Vec<RegionChoice>>> =
        const { std::cell::RefCell::new(None) };
    static BATCH_LENGTHS: std::cell::RefCell<Option<Vec<usize>>> =
        const { std::cell::RefCell::new(None) };
    static MULTI_WINDOWS: std::cell::RefCell<Option<Vec<WindowEvidence>>> =
        const { std::cell::RefCell::new(None) };
}

/// Merge only clauses whose local score bounds can exceed the optional prefix.
/// Scorers stay in incoming order: the preference order is for pruning only,
/// while published scores replay the original f32 leaves in f64.
pub(crate) fn or_maxscore(
    mut scorers: Vec<TermScorer>,
    max_doc: DocId,
    mut threshold: Score,
    callback: &mut dyn FnMut(DocId, Score) -> Score,
) {
    debug_assert!((3..=32).contains(&scorers.len()));
    // Reject exceptional/signed score domains before shallow seeks change any
    // cursor. In particular, a negative weight's zero maximum does not certify
    // nonnegative leaves. The fallback retains canonical SumCombiner arithmetic.
    let eligible = !threshold.is_nan()
        && scorers.iter().all(|scorer| {
            let maximum = scorer.max_score();
            scorer.bm25_weight().has_safe_score_bounds() && maximum.is_finite() && maximum > 0.0
        });
    if !eligible {
        let mut union = BufferedUnionScorer::build(scorers, SumCombiner::default, max_doc);
        for_each_pruning_scorer(&mut union, threshold, callback);
        return;
    }

    let num_terms = scorers.len();
    // Numeric admission above certifies finite nonnegative f32 leaves. Every
    // prefix/probe sum contains at most the admitted number of original terms.
    let upper_bound =
        SmallScoreSumUpperBound::new(num_terms).expect("admitted 3..=32-term score sum");
    let mut clauses: Vec<ClauseState> = scorers
        .iter()
        .enumerate()
        .map(|(ordinal, scorer)| ClauseState {
            ordinal,
            global_max: scorer.max_score(),
            cost: scorer.size_hint(),
            doc: scorer.doc(),
            local_max: 0.0,
        })
        .collect();
    // Low-value dense clauses make useful optional terms. Freeze this order for
    // the query instead of restoring a document order after every candidate.
    clauses.sort_unstable_by(|left, right| {
        let priority =
            |clause: &ClauseState| f64::from(clause.global_max) / f64::from(clause.cost.max(1));
        priority(left)
            .total_cmp(&priority(right))
            .then_with(|| left.ordinal.cmp(&right.ordinal))
    });
    // prefix[k] includes precisely clauses[..k]; candidate tests use j+1 so the
    // next optional clause is included. No bound is formed by subtraction.
    let mut prefix = vec![0.0f64; num_terms + 1];
    let mut contributions = vec![0.0; num_terms];
    let mut batch = ScoredTermBatch::new();
    let mut multi_window: Option<ExactLeafWindow> = None;
    let mut lo = clauses
        .iter()
        .map(|clause| clause.doc)
        .min()
        .unwrap_or(TERMINATED);

    while lo < max_doc {
        debug_assert!(clauses
            .iter()
            .all(|clause| clause.doc == scorers[clause.ordinal].doc()));
        let mut globally_optional = 0;
        let mut global_sum = 0.0f64;
        for clause in &clauses {
            let next_sum = global_sum + f64::from(clause.global_max);
            if upper_bound.score(next_sum) > threshold {
                break;
            }
            global_sum = next_sum;
            globally_optional += 1;
        }
        if globally_optional == num_terms {
            return;
        }
        #[cfg(test)]
        let certified = globally_optional;
        #[cfg(test)]
        let mut observed_remaining_cost = 0;
        if globally_optional > 0 {
            let remaining_cost = clauses[globally_optional..]
                .iter()
                .filter(|clause| clause.doc != TERMINATED)
                .map(|clause| u64::from(clause.cost))
                .max()
                .unwrap_or(0);
            #[cfg(test)]
            {
                observed_remaining_cost = remaining_cost;
            }
            // A looser global certificate is useful when it leaves a much
            // sparser candidate driver. For comparable-density clauses, keep
            // tight local bounds so another dense term need not be essential.
            let sparse_remainder = clauses[..globally_optional]
                .iter()
                .all(|clause| u64::from(clause.cost) >= 2 * remaining_cost);
            if !sparse_remainder {
                globally_optional = 0;
            }
        }
        #[cfg(test)]
        GLOBAL_CHOICES.with(|choices| {
            if let Some(choices) = choices.borrow_mut().as_mut() {
                choices.push((lo, certified, globally_optional, observed_remaining_cost));
            }
        });
        let mut hi = max_doc;
        for (rank, clause) in clauses.iter_mut().enumerate() {
            clause.local_max = 0.0;
            // These maxima certify the entire physical range. Dense optional
            // terms need neither shallow selection nor a region boundary at
            // each of their blocks when only sparse essentials drive candidates.
            if rank < globally_optional {
                clause.local_max = clause.global_max;
                continue;
            }
            if clause.doc == TERMINATED {
                continue;
            }
            clause.assert_coherent(&scorers);
            scorers[clause.ordinal].seek_block(lo.max(clause.doc));
            clause.assert_coherent(&scorers);
            let last_doc = scorers[clause.ordinal].last_doc_in_block();
            // Shallow selection can leave doc() in an old decoded block. A
            // selected tail has no finite end, so reconcile it at the floor;
            // remaining_docs alone does not prove there is a doc after lo.
            if last_doc == TERMINATED && clause.doc < lo {
                clause.seek(&mut scorers, lo);
            }
            if clause.doc == TERMINATED {
                continue;
            }
            // Keep the previously selected last_doc after tail reconciliation.
            // The cap makes both a tail and TERMINATED overflow harmless and
            // uses the physical address range, including deleted documents.
            hi = hi.min(last_doc.min(max_doc - 1) + 1);
            let bound = scorers[clause.ordinal].block_max_score();
            clause.assert_coherent(&scorers);
            clause.local_max = if bound.is_finite() && bound >= 0.0 {
                bound
            } else {
                clause.global_max
            };
        }
        debug_assert!(hi > lo);
        for (rank, clause) in clauses.iter_mut().enumerate() {
            // A real loaded position beyond the interval cannot contribute.
            // A stale position below lo does not justify a zero bound. Cleanup
            // and prefix construction share the same sequential record pass.
            if clause.doc >= hi {
                clause.local_max = 0.0;
            }
            prefix[rank + 1] = prefix[rank] + f64::from(clause.local_max);
        }
        let mut first_essential = 0;
        while first_essential < num_terms
            && upper_bound.score(prefix[first_essential + 1]) <= threshold
        {
            first_essential += 1;
        }
        if first_essential == num_terms {
            debug_assert!(clauses
                .iter()
                .all(|clause| clause.doc == scorers[clause.ordinal].doc()));
            lo = hi;
            continue;
        }

        for clause in &mut clauses[first_essential..] {
            if clause.doc < lo {
                // If shallow selection moved, every old decoded doc is below
                // lo. seek(lo) therefore cannot take an old-doc shortcut.
                clause.seek(&mut scorers, lo);
            }
        }
        // The buffered phase retains the current certified partition. Cursor
        // docs always mean first unread; batch docs own the replay candidates.
        let single_essential = num_terms - first_essential == 1;
        let mut batch_len = 0;
        let mut batch_index = 0;
        loop {
            let (doc, mut known, contributions) = if single_essential {
                if batch_index == batch_len {
                    batch_len = clauses[first_essential].fill_scored_batch(
                        &mut scorers,
                        lo,
                        hi,
                        &mut batch,
                    );
                    batch_index = 0;
                    #[cfg(test)]
                    BATCH_LENGTHS.with(|lengths| {
                        if let Some(lengths) = lengths.borrow_mut().as_mut() {
                            lengths.push(batch_len);
                        }
                    });
                    if batch_len == 0 {
                        break;
                    }
                }
                let doc = batch.docs[batch_index];
                if doc >= hi {
                    break;
                }
                contributions.fill(0.0);
                let leaf = batch.scores[batch_index];
                contributions[clauses[first_essential].ordinal] = leaf;
                (doc, f64::from(leaf), contributions.as_mut_slice())
            } else {
                let window = multi_window.get_or_insert_with(|| ExactLeafWindow::new(num_terms));
                if window.is_empty()
                    && !window.gather_window(
                        &mut clauses[first_essential..],
                        &mut scorers,
                        hi,
                        &mut batch,
                    )
                {
                    break;
                }
                // Pending rows govern completion, even if every real cursor
                // is already beyond hi or TERMINATED after the gather.
                window.pop_candidate().unwrap()
            };
            let mut competitive = true;
            for rank in (0..first_essential).rev() {
                if upper_bound.score(known + prefix[rank + 1]) <= threshold {
                    competitive = false;
                    break;
                }
                let clause = &mut clauses[rank];
                if clause.doc < doc {
                    clause.seek(&mut scorers, doc);
                }
                if clause.doc == doc {
                    clause.assert_coherent(&scorers);
                    let leaf = scorers[clause.ordinal].score();
                    clause.assert_coherent(&scorers);
                    contributions[clause.ordinal] = leaf;
                    known += f64::from(leaf);
                }
            }
            if competitive {
                let score = contributions
                    .iter()
                    .map(|&leaf| f64::from(leaf))
                    .sum::<f64>() as Score;
                if score > threshold {
                    let next_threshold = callback(doc, score);
                    debug_assert!(next_threshold >= threshold);
                    threshold = next_threshold;
                }
            }
            if single_essential {
                batch_index += 1;
            }
        }
        debug_assert!(clauses
            .iter()
            .all(|clause| clause.doc == scorers[clause.ordinal].doc()));
        debug_assert!(multi_window.as_ref().is_none_or(ExactLeafWindow::is_empty));
        // Discard every local certificate before entering another region.
        // Optional clauses remain alive and may become essential there.
        lo = hi;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::Bm25Weight;

    fn fixture(postings: &[(DocId, u32)], norms: &[u32], boost: Score) -> TermScorer {
        let average =
            norms.iter().map(|&norm| u64::from(norm)).sum::<u64>() as Score / norms.len() as Score;
        let weight = Bm25Weight::new_without_explain(1.0, average).boost_by(boost);
        TermScorer::create_for_test(postings, norms, weight)
    }

    // Record every callback, not just the final heap. The independent buffered
    // union visits every matching doc and applies the canonical SumCombiner.
    fn trace(
        threshold: Score,
        limit: usize,
        run: impl FnOnce(&mut dyn FnMut(DocId, Score) -> Score),
    ) -> Vec<(DocId, u32)> {
        let mut accepted = Vec::new();
        let mut best = Vec::new();
        run(&mut |doc, score| {
            accepted.push((doc, score.to_bits()));
            if limit == 0 {
                return threshold;
            }
            best.push((score, doc));
            best.sort_unstable_by(|left, right| {
                right
                    .0
                    .total_cmp(&left.0)
                    .then_with(|| left.1.cmp(&right.1))
            });
            best.truncate(limit);
            if best.len() == limit {
                threshold.max(best.last().unwrap().0)
            } else {
                threshold
            }
        });
        assert!(accepted.windows(2).all(|pair| pair[0].0 < pair[1].0));
        accepted
    }

    fn check(
        scorers: Vec<TermScorer>,
        max_doc: DocId,
        threshold: Score,
        limit: usize,
    ) -> Vec<(DocId, u32)> {
        let expected = trace(threshold, limit, |callback| {
            let mut union =
                BufferedUnionScorer::build(scorers.clone(), SumCombiner::default, max_doc);
            for_each_pruning_scorer(&mut union, threshold, callback);
        });
        let actual = trace(threshold, limit, |callback| {
            or_maxscore(scorers, max_doc, threshold, callback);
        });
        assert_eq!(actual, expected, "threshold={threshold:?} limit={limit}");
        actual
    }

    fn observed_check(
        scorers: Vec<TermScorer>,
        max_doc: DocId,
        threshold: Score,
        limit: usize,
    ) -> (Vec<(DocId, u32)>, Vec<WindowEvidence>) {
        MULTI_WINDOWS.with(|windows| *windows.borrow_mut() = Some(Vec::new()));
        let accepted = check(scorers, max_doc, threshold, limit);
        let windows = MULTI_WINDOWS.with(|windows| windows.borrow_mut().take().unwrap());
        for window in &windows {
            assert!(window.essentials.len() >= 2);
            assert!(window.base < window.end && window.end <= window.hi);
            assert!(window.end - window.base <= 128);
            assert!(window.unread.iter().all(|&doc| doc >= window.end));
            assert_eq!(window.matrix_len % 128, 0);
            assert!(window.matrix_len <= 128 * 32);
            assert!(window.matrix_capacity >= window.matrix_len);
        }
        (accepted, windows)
    }

    #[test]
    fn multi_bitmap_replays_pending_rows_after_every_actual_cursor_exhausts() {
        let norms = vec![1; 128];
        let postings = [(0, 1), (7, 2), (63, 3), (64, 4), (127, 5)];
        let scorers = vec![
            fixture(&postings, &norms, 0.3),
            fixture(&postings, &norms, 0.7),
            fixture(&postings, &norms, 0.5),
        ];
        let (accepted, windows) = observed_check(scorers, 128, 0.0, 0);
        assert_eq!(
            accepted.iter().map(|&(doc, _)| doc).collect::<Vec<_>>(),
            [0, 7, 63, 64, 127]
        );
        assert_eq!(windows.len(), 1);
        let window = &windows[0];
        assert_eq!(window.filled, 3);
        assert_eq!(
            window.matches,
            [(1 << 0) | (1 << 7) | (1 << 63), (1 << 0) | (1 << 63)]
        );
        assert!(window.unread.iter().all(|&doc| doc == TERMINATED));
        let scratch = ExactLeafWindow::new(32);
        let foundation = 32 * std::mem::size_of::<ClauseState>()
            + 33 * std::mem::size_of::<f64>()
            + 32 * std::mem::size_of::<Score>()
            + std::mem::size_of::<ScoredTermBatch>();
        let payload = foundation
            + scratch.incoming_leaves.len() * std::mem::size_of::<Score>()
            + std::mem::size_of_val(&scratch.essential_known)
            + std::mem::size_of_val(&scratch.matches);
        eprintln!(
            "OR_MULTI_LAYOUT window={} matrix_len={} matrix_capacity={} foundation={} payload={}",
            std::mem::size_of::<ExactLeafWindow>(),
            scratch.incoming_leaves.len(),
            scratch.incoming_leaves.capacity(),
            foundation,
            payload
        );
        assert_eq!(scratch.incoming_leaves.len(), 128 * 32);
        // Report the host layout without fixing a target ABI in this oracle.
        assert_eq!(std::mem::size_of_val(&scratch.essential_known), 1024);
        assert_eq!(std::mem::size_of_val(&scratch.matches), 16);
    }

    #[test]
    fn multi_bitmap_full_callback_oracle_covers_n3_n32_disjoint_overlap_and_tails() {
        let norms: Vec<u32> = (0..1024).map(|doc| 1 + doc % 11).collect();
        for count in [3, 32] {
            for length in [127, 128, 129, 255, 256, 257] {
                for disjoint in [false, true] {
                    let scorers: Vec<_> = (0..count)
                        .map(|ordinal| {
                            let postings: Vec<_> = (0..length)
                                .filter(|&doc| !disjoint || doc as usize % count == ordinal)
                                .map(|doc| (doc, 1 + (doc + ordinal as u32) % 17))
                                .collect();
                            fixture(&postings, &norms, 0.5 + ordinal as Score / 16.0)
                        })
                        .collect();
                    let (accepted, windows) = observed_check(scorers.clone(), 1024, 0.0, 0);
                    assert_eq!(accepted.len(), length as usize);
                    assert!(windows.iter().any(|window| window.filled >= 2));
                    for limit in [1, 10] {
                        observed_check(scorers.clone(), 1024, 0.0, limit);
                    }
                }
            }
        }
    }

    #[test]
    fn multi_bitmap_first_touch_clears_old_essential_and_optional_leaves() {
        let norms = vec![1; 384];
        let weak: Vec<_> = (0..384).map(|doc| (doc, 1)).collect();
        let first = [
            (0, 1),
            (7, 1),
            (63, 1),
            (64, 1),
            (127, 1),
            (128, 1),
            (256, 1),
        ];
        let second = [
            (0, 1),
            (7, 1),
            (63, 1),
            (64, 1),
            (127, 1),
            (128, 1),
            (135, 1),
            (191, 1),
            (192, 1),
            (255, 1),
            (256, 1),
        ];
        let scorers = vec![
            fixture(&first, &norms, 1.0),
            fixture(&weak, &norms, 2.0f32.powi(-10)),
            fixture(&second, &norms, 2.0),
        ];
        let (accepted, windows) = observed_check(scorers, 384, 0.1, 0);
        assert!(windows
            .iter()
            .any(|window| window.base == 0 && window.filled == 2));
        assert!(windows
            .iter()
            .any(|window| window.base == 128 && window.filled == 2));
        assert_eq!(
            accepted.iter().find(|&&(doc, _)| doc == 7).unwrap().1,
            ((f64::from(1.0f32) + f64::from(2.0f32.powi(-10)) + f64::from(2.0f32)) as Score)
                .to_bits()
        );
        let expected = (f64::from(2.0f32.powi(-10)) + f64::from(2.0f32)) as Score;
        for doc in [135, 191, 192, 255] {
            assert_eq!(
                accepted
                    .iter()
                    .find(|&&(actual_doc, _)| actual_doc == doc)
                    .unwrap()
                    .1,
                expected.to_bits()
            );
        }
        assert!(windows.iter().all(|window| window.essentials == [0, 2]));

        // Both essential columns overwrite offset 7 in the next window. Only
        // first-touch clearing can remove the old optional contribution there.
        // The hits at 0/128 pin the actual window bases, so 7/135 reuse one row.
        let norms = vec![1; 256];
        let essentials = [(0, 1), (7, 1), (128, 1), (135, 1)];
        let optional = [(7, 1)];
        let (accepted, windows) = observed_check(
            vec![
                fixture(&essentials, &norms, 1.0),
                fixture(&optional, &norms, 2.0f32.powi(-10)),
                fixture(&essentials, &norms, 2.0),
            ],
            256,
            0.1,
            0,
        );
        assert_eq!(windows.len(), 2, "{windows:?}");
        for (window, base) in windows.iter().zip([0, 128]) {
            assert_eq!(window.base, base);
            assert_eq!(window.end, base + 128);
            assert_eq!(window.hi, 256);
            assert_eq!(window.essentials, [0, 2]);
            assert_eq!(window.filled, 2);
            assert_eq!(window.matches, [(1 << 0) | (1 << 7), 0]);
        }
        // Literal legacy leaves are 1, 2^-10, 2. Doc 135 must sum 1 + 2;
        // retaining the optional leaf from doc 7 would instead publish 3.0009766.
        assert_eq!(
            accepted,
            [
                (0, 0x4040_0000),
                (7, 0x4040_1000),
                (128, 0x4040_0000),
                (135, 0x4040_0000)
            ]
        );
    }

    #[test]
    fn multi_bitmap_reuses_dirty_rows_when_fixed_suffix_grows_then_shrinks() {
        let norms = vec![1; 800];
        let a: Vec<_> = (0..384)
            .map(|doc| (doc, if (128..256).contains(&doc) { 1000 } else { 1 }))
            .collect();
        let b: Vec<_> = (0..384)
            .map(|doc| (doc, if (128..256).contains(&doc) { 1 } else { 1000 }))
            .collect();
        let rare = [(0, 1), (128, 1), (256, 1), (700, 1)];
        let scorers = vec![
            fixture(&a, &norms, 0.1),
            fixture(&b, &norms, 0.1),
            fixture(&rare, &norms, 1.0),
        ];
        let (_, windows) = observed_check(scorers.clone(), 800, 0.15, 0);
        let masks: Vec<_> = windows
            .iter()
            .map(|window| (window.base, window.essentials.as_slice()))
            .collect();
        assert!(masks.contains(&(0, &[1, 2][..])), "{windows:?}");
        assert!(masks.contains(&(128, &[0, 1, 2][..])), "{windows:?}");
        assert!(masks.contains(&(256, &[1, 2][..])), "{windows:?}");
        for limit in [1, 10] {
            check(scorers.clone(), 800, 0.15, limit);
        }
    }

    #[test]
    fn multi_bitmap_jumps_huge_physical_gaps_and_clips_region_ends() {
        let norms = vec![1; 1_000_001];
        let a = [
            (0, 1),
            (63, 1),
            (64, 1),
            (127, 1),
            (500_000, 1),
            (1_000_000, 1),
        ];
        let b = [(0, 1), (7, 1), (128, 1), (500_001, 1), (999_999, 1)];
        let c = [(7, 1), (129, 1), (500_064, 1), (1_000_000, 1)];
        let (accepted, windows) = observed_check(
            vec![
                fixture(&a, &norms, 1.0),
                fixture(&b, &norms, 2.0),
                fixture(&c, &norms, 0.5),
            ],
            1_000_001,
            0.0,
            0,
        );
        assert_eq!(accepted.len(), 12);
        assert!(windows.iter().any(|window| window.base == 500_000));
        assert!(windows.iter().any(|window| window.end == 1_000_001));
        assert!(
            windows.len() <= 6,
            "must jump directly over empty addresses: {windows:?}"
        );
    }

    #[test]
    fn multi_bitmap_midpoints_replay_incoming_order_in_legacy_and_native_profiles() {
        let norms = vec![1; 257];
        let postings: Vec<_> = (0..257).map(|doc| (doc, 1)).collect();
        for native in [false, true] {
            let leaves = [1.0, 2.0f32.powi(-24), 2.0f32.powi(-53), 2.0f32.powi(-53)];
            let scorers: Vec<_> = leaves
                .iter()
                .map(|&boost| {
                    if native {
                        TermScorer::create_for_test(
                            &postings,
                            &norms,
                            Bm25Weight::for_native_block_bounds(1.0).boost_by(boost),
                        )
                    } else {
                        fixture(&postings, &norms, boost)
                    }
                })
                .collect();
            for order in [[0, 1, 2, 3], [2, 3, 1, 0], [1, 0, 3, 2]] {
                let ordered: Vec<_> = order.iter().map(|&i| scorers[i].clone()).collect();
                let expected = ordered
                    .iter()
                    .map(|scorer| f64::from(scorer.clone().score()))
                    .sum::<f64>() as Score;
                let (accepted, windows) = observed_check(ordered.clone(), 257, 0.0, 0);
                assert_eq!(accepted.len(), 257);
                assert!(accepted.iter().all(|&(_, bits)| bits == expected.to_bits()));
                assert!(windows.iter().any(|window| window.filled == 4));
                for threshold in [expected.next_down(), expected, expected.next_up()] {
                    for limit in [0, 1, 10] {
                        check(ordered.clone(), 257, threshold, limit);
                    }
                }
            }
        }

        // Near the low-TF sum, two substantial terms must remain essential.
        // Each full block and the tail contain a high-TF hit whose individual
        // strong leaf exceeds that threshold, keeping both local bounds live.
        // TF is strictly positive throughout; native uses DEFAULT parameters.
        let boosts = [
            0.5,
            0.5,
            2.0f32.powi(-24),
            2.0f32.powi(-53),
            2.0f32.powi(-53),
        ];
        let postings: Vec<_> = (0..257)
            .map(|doc| {
                (
                    doc,
                    if [127, 255, 256].contains(&doc) {
                        1000
                    } else {
                        1
                    },
                )
            })
            .collect();
        for native in [false, true] {
            let scorers: Vec<_> = boosts
                .iter()
                .map(|&boost| {
                    if native {
                        TermScorer::create_for_test(
                            &postings,
                            &norms,
                            Bm25Weight::for_native_block_bounds(1.0).boost_by(boost),
                        )
                    } else {
                        fixture(&postings, &norms, boost)
                    }
                })
                .collect();
            if !native {
                for (scorer, &leaf) in scorers.iter().zip(&boosts) {
                    assert_eq!(scorer.clone().score().to_bits(), leaf.to_bits());
                }
            }
            for order in [[0, 1, 2, 3, 4], [3, 4, 2, 0, 1], [2, 0, 1, 4, 3]] {
                let ordered: Vec<_> = order.iter().map(|&i| scorers[i].clone()).collect();
                // Derive native expectations from its actual scalar leaves;
                // both profiles publish one f32 cast after the incoming fold.
                let expected = ordered
                    .iter()
                    .map(|scorer| f64::from(scorer.clone().score()))
                    .sum::<f64>() as Score;
                let strong_ordinals: Vec<_> = order
                    .iter()
                    .enumerate()
                    .filter_map(|(ordinal, &original)| (original < 2).then_some(ordinal))
                    .collect();
                for &ordinal in &strong_ordinals {
                    for doc in [127, 255, 256] {
                        let mut scorer = ordered[ordinal].clone();
                        assert_eq!(scorer.seek(doc), doc);
                        assert!(scorer.score() > expected.next_up());
                    }
                }
                for threshold in [expected.next_down(), expected, expected.next_up()] {
                    for limit in [0, 1, 10] {
                        let (accepted, windows) =
                            observed_check(ordered.clone(), 257, threshold, limit);
                        assert!(
                            windows.iter().any(|window| window.base == 0
                                && window.filled >= 2
                                && strong_ordinals
                                    .iter()
                                    .all(|ordinal| window.essentials.contains(ordinal))),
                            "native={native} order={order:?} threshold={threshold:?} \
                             limit={limit} windows={windows:?}"
                        );
                        assert!(accepted.iter().any(|&(doc, _)| doc == 127));
                        let first_low = accepted.iter().find(|&&(doc, _)| doc == 0);
                        if threshold < expected {
                            assert_eq!(first_low.unwrap().1, expected.to_bits());
                        } else {
                            assert!(first_low.is_none());
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn multi_bitmap_current_threshold_and_simulated_collector_deletion_rejection() {
        let norms = vec![1; 128];
        let postings = [(0, 1), (7, 3), (63, 5), (64, 8), (127, 20)];
        let scorers = vec![
            fixture(&postings, &norms, 0.3),
            fixture(&postings, &norms, 0.5),
            fixture(&postings, &norms, 0.001),
        ];
        for limit in [0, 1, 10] {
            let (_, windows) = observed_check(scorers.clone(), 128, 0.1, limit);
            assert!(windows.iter().any(|window| window.filled >= 2));
        }
        let run = |bitmap: bool| {
            let mut accepted = Vec::new();
            let mut current = 0.1;
            let mut callback = |doc, score: Score| {
                accepted.push((doc, score.to_bits()));
                if doc != 0 && doc != 63 {
                    current = current.max(score);
                }
                current
            };
            if bitmap {
                or_maxscore(scorers.clone(), 128, 0.1, &mut callback);
            } else {
                let mut union =
                    BufferedUnionScorer::build(scorers.clone(), SumCombiner::default, 128);
                for_each_pruning_scorer(&mut union, 0.1, &mut callback);
            }
            accepted
        };
        assert_eq!(run(true), run(false));
        assert_eq!(
            run(true).iter().map(|&(doc, _)| doc).collect::<Vec<_>>(),
            [0, 7, 63, 64, 127]
        );
    }

    #[test]
    fn multi_bitmap_actual_indexed_deletes_keep_physical_pending_candidates() -> crate::Result<()> {
        use crate::collector::TopDocs;
        use crate::indexer::NoMergePolicy;
        use crate::query::{BooleanQuery, EnableScoring, Query, TermQuery};
        use crate::schema::{IndexRecordOption, Schema, TEXT};
        use crate::{DocAddress, Index, Term};

        let mut schema = Schema::builder();
        let text = schema.add_text_field("text", TEXT);
        let marker = schema.add_text_field("marker", TEXT);
        let index = Index::create_in_ram(schema.build());
        let mut writer = index.writer_for_tests()?;
        writer.set_merge_policy(Box::new(NoMergePolicy));
        for doc in 0..160 {
            let frequency = if [0, 63, 127].contains(&doc) { 30 } else { 1 };
            let content = std::iter::repeat_n("alpha beta gamma", frequency)
                .chain(std::iter::repeat_n("filler", 90 - frequency * 3))
                .collect::<Vec<_>>()
                .join(" ");
            writer.add_document(doc!(text => content, marker => if [0, 63].contains(&doc) { "delete" } else { "keep" }))?;
        }
        writer.commit()?;
        writer.delete_term(Term::from_field_text(marker, "delete"));
        writer.commit()?;
        let reader = index.reader()?;
        let searcher = reader.searcher();
        assert_eq!(searcher.segment_readers().len(), 1);
        let segment = searcher.segment_reader(0);
        assert_eq!(segment.max_doc(), 160);
        assert_eq!(segment.num_docs(), 158);
        let queries: Vec<_> = ["alpha", "beta", "gamma"]
            .iter()
            .map(|term| {
                TermQuery::new(
                    Term::from_field_text(text, term),
                    IndexRecordOption::WithFreqs,
                )
            })
            .collect();
        let scorers: Vec<_> = queries
            .iter()
            .map(|query| {
                query
                    .specialized_weight(EnableScoring::enabled_from_searcher(&searcher))?
                    .term_scorer_for_test(segment, 1.0)?
                    .ok_or_else(|| crate::TantivyError::InvalidArgument("missing test term".into()))
            })
            .collect::<crate::Result<_>>()?;
        let run = |bitmap: bool| {
            let mut accepted = Vec::new();
            let mut current = -Score::INFINITY;
            let mut callback = |doc, score: Score| {
                accepted.push((doc, score.to_bits()));
                // This is the real index's deletion bitset, with the same
                // threshold rejection ownership as the TopDocs collector.
                if segment.alive_bitset().unwrap().is_alive(doc) {
                    current = current.max(score);
                }
                current
            };
            if bitmap {
                or_maxscore(
                    scorers.clone(),
                    segment.max_doc(),
                    -Score::INFINITY,
                    &mut callback,
                );
            } else {
                let mut union = BufferedUnionScorer::build(
                    scorers.clone(),
                    SumCombiner::default,
                    segment.max_doc(),
                );
                for_each_pruning_scorer(&mut union, -Score::INFINITY, &mut callback);
            }
            accepted
        };
        MULTI_WINDOWS.with(|windows| *windows.borrow_mut() = Some(Vec::new()));
        let actual = run(true);
        let windows = MULTI_WINDOWS.with(|windows| windows.borrow_mut().take().unwrap());
        assert_eq!(actual, run(false));
        assert!(actual.iter().any(|&(doc, _)| doc == 0));
        assert!(actual.iter().any(|&(doc, _)| doc == 63));
        assert!(actual.iter().any(|&(doc, _)| doc == 127));
        assert!(windows.iter().any(|window| window.filled == 3
            && window.unread.iter().all(|&doc| doc >= window.hi)
            && window.matches.iter().any(|&word| word != 0)));
        let query = BooleanQuery::union(
            queries
                .into_iter()
                .map(|query| Box::new(query) as Box<dyn Query>)
                .collect(),
        );
        MULTI_WINDOWS.with(|windows| *windows.borrow_mut() = Some(Vec::new()));
        let top = searcher.search(&query, &TopDocs::with_limit(1).order_by_score())?;
        let windows = MULTI_WINDOWS.with(|windows| windows.borrow_mut().take().unwrap());
        assert!(windows.iter().any(|window| window.filled == 3));
        assert_eq!(top[0].1, DocAddress::new(0, 127));
        let expected_bits = actual.iter().find(|&&(doc, _)| doc == 127).unwrap().1;
        assert_eq!(top[0].0.to_bits(), expected_bits);
        Ok(())
    }

    #[test]
    fn single_driver_batches_replay_every_dynamic_threshold_exactly() {
        let norms = vec![1; 4096];
        let weak: Vec<_> = (0..4096).map(|doc| (doc, 1)).collect();
        let driver: Vec<_> = (0..257)
            .map(|i| {
                (
                    i * 4,
                    match i {
                        33 => 10,
                        240 => 30,
                        _ => 1,
                    },
                )
            })
            .collect();
        let scorers = vec![
            fixture(&weak, &norms, 0.001),
            fixture(&driver, &norms, 1.0),
            fixture(&weak, &norms, 0.002),
        ];
        BATCH_LENGTHS.with(|lengths| *lengths.borrow_mut() = Some(Vec::new()));
        let accepted = check(scorers.clone(), 4096, 0.1, 1);
        let lengths = BATCH_LENGTHS.with(|lengths| lengths.borrow_mut().take().unwrap());
        assert!(
            lengths.contains(&128),
            "must exercise a full buffered block"
        );
        assert_eq!(
            accepted.iter().map(|&(doc, _)| doc).collect::<Vec<_>>(),
            [0, 132, 960]
        );
        check(scorers.clone(), 4096, 0.1, 0);
        check(scorers.clone(), 4096, 0.1, 10);
        // Model collector-owned deletions: rejected early winners must leave
        // the callback threshold unchanged for the next buffered candidate.
        let with_deletions = |buffered: bool| {
            let mut accepted = Vec::new();
            let mut current = 0.1;
            let mut callback = |doc, score: Score| {
                accepted.push((doc, score.to_bits()));
                if doc != 0 && doc != 132 {
                    current = score;
                }
                current
            };
            if buffered {
                or_maxscore(scorers.clone(), 4096, 0.1, &mut callback);
            } else {
                let mut union =
                    BufferedUnionScorer::build(scorers.clone(), SumCombiner::default, 4096);
                for_each_pruning_scorer(&mut union, 0.1, &mut callback);
            }
            accepted
        };
        let deletion_trace = with_deletions(true);
        assert_eq!(deletion_trace, with_deletions(false));
        assert_eq!(
            deletion_trace
                .iter()
                .map(|&(doc, _)| doc)
                .collect::<Vec<_>>(),
            [0, 4, 132, 960]
        );
        eprintln!(
            "OR_BATCH_LAYOUT batch={} max_array_payload={}",
            std::mem::size_of::<ScoredTermBatch>(),
            36 * 32 + 8 + std::mem::size_of::<ScoredTermBatch>()
        );
    }

    #[test]
    fn exact_traces_at_full_block_and_tail_boundaries() {
        let norms: Vec<u32> = (0..1024).map(|i| 1 + (i % 13)).collect();
        for length in [1, 127, 128, 129, 255, 256, 257] {
            let dense: Vec<_> = (0..length).map(|i| (i * 2, 1 + i % 5)).collect();
            let gapped: Vec<_> = (0..length).map(|i| (i * 3 + 1, 1 + i % 7)).collect();
            let rare = [(0, 1), (128, 20), (256, 1), (511, 50), (900, 1)];
            let scorers = vec![
                fixture(&dense, &norms, 0.01),
                fixture(&gapped, &norms, 0.02),
                fixture(&rare, &norms, 1.0),
            ];
            // High fixed thresholds leave full blocks and tails unvisited.
            // Growing Top-K thresholds cover both within/across-region changes.
            for threshold in [-Score::INFINITY, 0.0, 0.6, 3.0, Score::INFINITY] {
                for limit in [0, 1, 10, 100] {
                    check(scorers.clone(), 1024, threshold, limit);
                }
            }
        }
    }

    #[test]
    fn deferred_optional_clause_becomes_essential_after_shallow_full_blocks() {
        let norms = vec![1; 800];
        let dense: Vec<_> = (0..600)
            .map(|doc| (doc, if doc < 256 { 1 } else { 1000 }))
            .collect();
        let weak: Vec<_> = (0..600).map(|doc| (doc, 1)).collect();
        let rare = [(0, 1), (700, 1)];
        let scorers = vec![
            fixture(&dense, &norms, 0.1),
            fixture(&rare, &norms, 1.0),
            fixture(&weak, &norms, 0.01),
        ];
        // The first two full dense blocks cannot win without the rare term.
        // Their decoder is then stale when the high-TF third block becomes
        // essential; seeking must load it before score()/advance().
        let accepted = check(scorers.clone(), 800, 0.15, 0);
        assert_eq!(accepted.first().unwrap().0, 0);
        assert_eq!(accepted[1].0, 256);
        assert_eq!(accepted.last().unwrap().0, 700);
        assert_eq!(accepted.len(), 346);
        // A high threshold skips the optional clause all the way into a tail.
        check(scorers, 800, 1.5, 0);
    }

    #[test]
    fn globally_optional_dense_terms_cover_sparse_essential_gaps_and_tails() {
        let norms = vec![1; 4096];
        let sparse = [(0, 1), (128, 1), (256, 1), (4095, 1)];
        for length in [127, 128, 129, 255, 256, 257, 4096] {
            let dense: Vec<_> = (0..length).map(|doc| (doc, 1)).collect();
            let scorers = vec![
                fixture(&dense, &norms, 0.01),
                fixture(&sparse, &norms, 1.0),
                fixture(&dense, &norms, 0.02),
            ];
            // Both dense global maxima sum below this threshold. Their
            // full-block ends do not constrain the sparse term's tail region;
            // optional seeks still cross full blocks and establish exhaustion.
            let accepted = check(scorers.clone(), 4096, 0.1, 0);
            assert_eq!(
                accepted.iter().map(|&(doc, _)| doc).collect::<Vec<_>>(),
                [0, 128, 256, 4095]
            );
            check(scorers, 4096, 0.1, 1);
        }
    }

    #[test]
    fn threshold_growth_promotes_local_optionals_to_global_certificates() {
        let norms = vec![1; 1024];
        let dense: Vec<_> = (0..1024).map(|doc| (doc, 1)).collect();
        let sparse: Vec<_> = (0..340)
            .map(|i| (i * 3, if i < 100 { 1 } else { 20 }))
            .collect();
        let scorers = vec![
            fixture(&dense, &norms, 0.01),
            fixture(&sparse, &norms, 1.0),
            fixture(&dense, &norms, 0.02),
        ];
        // Local dense bounds sum to .03, below the initial threshold, while
        // their global maxima sum to .066, above it. The first accepted hit
        // raises the Top-1 threshold, enabling global certificates at the next
        // region. A later high-TF essential hit must still be returned exactly.
        let accepted = check(scorers.clone(), 1024, 0.05, 1);
        assert_eq!(
            accepted.iter().map(|&(doc, _)| doc).collect::<Vec<_>>(),
            [0, 300]
        );
        check(scorers, 1024, 0.05, 10);
    }

    #[test]
    fn similar_density_terms_keep_local_certificates_until_both_are_optional() {
        let norms = vec![1; 1024];
        let dense: Vec<_> = (0..768).map(|doc| (doc, 1)).collect();
        let other_dense: Vec<_> = (0..700)
            .map(|doc| (doc, if doc < 256 { 1 } else { 20 }))
            .collect();
        let rare = [(0, 1), (300, 20), (900, 1)];
        let scorers = vec![
            fixture(&dense, &norms, 0.01),
            fixture(&other_dense, &norms, 0.02),
            fixture(&rare, &norms, 1.0),
        ];
        // At .04 only the first dense term fits the global prefix, leaving a
        // comparable-density driver: retain local certificates. At .1 both
        // dense terms fit and the remainder is sparse, enabling widening.
        // Growing thresholds cross that boundary without changing exact traces.
        for threshold in [0.04, 0.1] {
            for limit in [0, 1, 10] {
                check(scorers.clone(), 1024, threshold, limit);
            }
        }
    }

    #[test]
    fn exhausted_high_cost_remainder_changes_live_density_choice() {
        let norms = vec![1; 4096];
        let weak: Vec<_> = (0..300).map(|i| (1 + 10 * i, 1)).collect();
        let high_cost: Vec<_> = (0..200).map(|doc| (doc, 1)).collect();
        let rare = [(0, 1), (3500, 1)];
        let scorers = vec![
            fixture(&weak, &norms, 0.01),
            fixture(&high_cost, &norms, 0.02),
            fixture(&rare, &norms, 1.0),
        ];
        // The first global maximum fits .04, but the second does not. The
        // 300-posting optional term is too small to widen while the 200-posting
        // remainder is live. After actual tail reconciliation exhausts that
        // remainder, the same global prefix can widen against the 2-posting one.
        GLOBAL_CHOICES.with(|choices| *choices.borrow_mut() = Some(Vec::new()));
        let accepted = check(scorers.clone(), 4096, 0.04, 0);
        let choices = GLOBAL_CHOICES.with(|choices| choices.borrow_mut().take().unwrap());
        assert_eq!(
            accepted.iter().map(|&(doc, _)| doc).collect::<Vec<_>>(),
            [0, 3500]
        );
        assert!(choices.iter().all(|&(_, certified, _, _)| certified == 1));
        let rejected = choices
            .iter()
            .position(|&(_, _, selected, cost)| selected == 0 && cost == 200)
            .unwrap();
        let widened = choices
            .iter()
            .position(|&(_, _, selected, cost)| selected == 1 && cost == 2)
            .unwrap();
        assert!(rejected < widened, "region choices: {choices:?}");
        // Record the actual host layout while exercising the oracle fixture;
        // this reports scratch accounting rather than asserting a chosen ABI.
        let record_bytes = std::mem::size_of::<ClauseState>();
        let payload =
            32 * record_bytes + 33 * std::mem::size_of::<f64>() + 32 * std::mem::size_of::<Score>();
        eprintln!("ClauseState bytes={record_bytes}; max-32 array payload bytes={payload}");
        check(scorers, 4096, 0.04, 1);
    }

    #[test]
    fn exact_midpoint_leaves_preserve_incoming_order_and_strict_ties() {
        let norms = vec![1; 260];
        let postings: Vec<_> = (0..260).map(|doc| (doc, 1)).collect();
        let leaves = [1.0, 2.0f32.powi(-24), 2.0f32.powi(-53), 2.0f32.powi(-53)];
        let scorers: Vec<_> = leaves
            .iter()
            .map(|&leaf| fixture(&postings, &norms, leaf))
            .collect();
        for (scorer, &leaf) in scorers.iter().zip(&leaves) {
            assert_eq!(scorer.clone().score().to_bits(), leaf.to_bits());
        }
        for order in [[0, 1, 2, 3], [2, 3, 1, 0], [1, 0, 3, 2]] {
            let mut sum = 0.0f64;
            for i in order {
                sum += f64::from(leaves[i]);
            }
            let expected = sum as Score;
            let ordered: Vec<_> = order.iter().map(|&i| scorers[i].clone()).collect();
            for threshold in [expected.next_down(), expected, expected.next_up()] {
                for limit in [0, 1, 10] {
                    let accepted = check(ordered.clone(), 260, threshold, limit);
                    if threshold >= expected {
                        assert!(accepted.is_empty());
                    } else {
                        assert_eq!(accepted.len(), if limit == 0 { 260 } else { limit });
                        assert!(accepted.iter().all(|&(_, bits)| bits == expected.to_bits()));
                    }
                }
            }
        }
    }

    #[test]
    fn duplicates_disparate_boosts_exhaustion_and_sparse_regions() {
        let norms: Vec<u32> = (0..4096).map(|i| 1 + i % 257).collect();
        let dense: Vec<_> = (0..1024).map(|i| (i, 1 + i % 97)).collect();
        let sparse = [(0, 1), (1023, 2), (2048, 100), (4095, 1)];
        let a = fixture(&dense, &norms, 1.0);
        let b = fixture(&sparse, &norms, 0.01);
        let c = fixture(&sparse[..1], &norms, 10000.0);
        for count in [3, 4, 20, 32] {
            let mut scorers = vec![a.clone(), b.clone(), c.clone()];
            scorers.extend(std::iter::repeat_n(a.clone(), count - 3));
            for threshold in [0.0, 1.0, 1000.0] {
                for limit in [0, 1, 10] {
                    check(scorers.clone(), 4096, threshold, limit);
                    check(
                        scorers.iter().rev().cloned().collect(),
                        4096,
                        threshold,
                        limit,
                    );
                }
            }
        }
        let mut exhausted = c;
        while exhausted.advance() != TERMINATED {}
        check(vec![exhausted, a, b], 4096, 0.0, 10);
    }

    #[test]
    fn exact_growing_threshold_traces_for_mixed_dense_and_sparse_terms() {
        // Independent deterministic corpora exercise many local partitions and
        // advancing positions without relying on the MaxScore implementation.
        for seed in 0..24u64 {
            let norms: Vec<_> = (0..1024).map(|doc| 1 + ((doc * 17) % 113)).collect();
            let mut state = seed + 1;
            let mut scorers = Vec::new();
            for ordinal in 0..(3 + seed as usize % 6) {
                let mut postings = vec![(0, 1)];
                for doc in 1..1024 {
                    state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                    if (state >> 32) % (2 + ordinal as u64) == 0 {
                        postings.push((doc, 1 + ((state >> 48) % 64) as u32));
                    }
                }
                let boost = 2.0f32.powi(ordinal as i32 - 5);
                scorers.push(fixture(&postings, &norms, boost));
            }
            for threshold in [0.0, 0.25, 2.0] {
                for limit in [1, 10, 100] {
                    check(scorers.clone(), 1024, threshold, limit);
                }
            }
        }
    }

    #[test]
    fn unsafe_domains_use_exhaustive_canonical_sum_before_any_seek() {
        MULTI_WINDOWS.with(|windows| *windows.borrow_mut() = Some(Vec::new()));
        let norms = vec![1; 300];
        let postings: Vec<_> = (0..300).map(|doc| (doc, 1 + doc % 7)).collect();
        let positive = fixture(&postings, &norms, 1.0);
        for boost in [0.0, -1.0, Score::INFINITY, Score::NAN] {
            let exceptional = fixture(&postings, &norms, boost);
            check(
                vec![positive.clone(), exceptional, positive.clone()],
                300,
                -Score::INFINITY,
                0,
            );
        }
        for average in [-1.0, 0.0, Score::NAN] {
            let weight = Bm25Weight::new_without_explain(1.0, average);
            assert!(!weight.has_safe_score_bounds());
            let unsafe_scorer = TermScorer::create_for_test(&postings, &norms, weight);
            check(
                vec![positive.clone(), unsafe_scorer, positive.clone()],
                300,
                -Score::INFINITY,
                0,
            );
        }
        assert!(check(vec![positive; 3], 300, Score::NAN, 0).is_empty());
        assert!(MULTI_WINDOWS
            .with(|windows| windows.borrow_mut().take().unwrap())
            .is_empty());
    }
}
