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
        #[cfg(test)]
        let before = self.doc;
        self.doc = scorers[self.ordinal].seek(target);
        self.assert_coherent(scorers);
        #[cfg(test)]
        CURSOR_PROGRESS.with(|progress| {
            if let Some(progress) = progress.borrow_mut().as_mut() {
                let (tf, bits) = if self.doc == TERMINATED {
                    (0, 0)
                } else {
                    (
                        scorers[self.ordinal].term_freq(),
                        scorers[self.ordinal].score().to_bits(),
                    )
                };
                progress.push(CursorProgress::Seek {
                    ordinal: self.ordinal,
                    before,
                    target,
                    doc: self.doc,
                    tf,
                    bits,
                });
            }
        });
    }

    fn fill_scored_batch(
        &mut self,
        scorers: &mut [TermScorer],
        floor: DocId,
        up_to: DocId,
        batch: &mut ScoredTermBatch,
    ) -> usize {
        self.assert_coherent(scorers);
        #[cfg(test)]
        let before = self.doc;
        let len = scorers[self.ordinal].fill_scored_batch(floor, up_to, batch);
        self.doc = scorers[self.ordinal].doc();
        self.assert_coherent(scorers);
        #[cfg(test)]
        CURSOR_PROGRESS.with(|progress| {
            if let Some(progress) = progress.borrow_mut().as_mut() {
                progress.push(CursorProgress::Batch {
                    ordinal: self.ordinal,
                    before,
                    floor,
                    up_to,
                    docs: batch.docs[..len].to_vec(),
                    bits: batch.scores[..len]
                        .iter()
                        .map(|score| score.to_bits())
                        .collect(),
                    doc: self.doc,
                    tf: if self.doc == TERMINATED {
                        0
                    } else {
                        scorers[self.ordinal].term_freq()
                    },
                });
            }
        });
        len
    }

    #[inline]
    fn advance(&mut self, scorers: &mut [TermScorer]) {
        self.assert_coherent(scorers);
        #[cfg(test)]
        let before = self.doc;
        self.doc = scorers[self.ordinal].advance();
        self.assert_coherent(scorers);
        #[cfg(test)]
        CURSOR_PROGRESS.with(|progress| {
            if let Some(progress) = progress.borrow_mut().as_mut() {
                let (tf, bits) = if self.doc == TERMINATED {
                    (0, 0)
                } else {
                    (
                        scorers[self.ordinal].term_freq(),
                        scorers[self.ordinal].score().to_bits(),
                    )
                };
                progress.push(CursorProgress::Advance {
                    ordinal: self.ordinal,
                    before,
                    doc: self.doc,
                    tf,
                    bits,
                });
            }
        });
    }
}

#[cfg(test)]
#[derive(Debug)]
struct WidenedRegion {
    lo: DocId,
    tight_hi: DocId,
    hi: DocId,
    tight_first_essential: usize,
    first_essential: usize,
    ordinals: Vec<usize>,
    docs: Vec<DocId>,
    bounds: Vec<Score>,
    selected_ends: Vec<DocId>,
    selected_bounds: Vec<Score>,
}

#[cfg(test)]
#[derive(Debug)]
enum CursorProgress {
    Seek {
        ordinal: usize,
        before: DocId,
        target: DocId,
        doc: DocId,
        tf: u32,
        bits: u32,
    },
    Advance {
        ordinal: usize,
        before: DocId,
        doc: DocId,
        tf: u32,
        bits: u32,
    },
    Batch {
        ordinal: usize,
        before: DocId,
        floor: DocId,
        up_to: DocId,
        docs: Vec<DocId>,
        bits: Vec<u32>,
        doc: DocId,
        tf: u32,
    },
}

#[cfg(test)]
type RegionChoice = (DocId, usize, usize, u64);

#[cfg(test)]
type GlobalPrefixState = (DocId, u32, usize, u64, usize);

#[cfg(test)]
thread_local! {
    // Test-only region evidence; no observer or counter exists in release.
    static GLOBAL_CHOICES: std::cell::RefCell<Option<Vec<RegionChoice>>> =
        const { std::cell::RefCell::new(None) };
    static GLOBAL_PREFIX_STATES: std::cell::RefCell<Option<Vec<GlobalPrefixState>>> =
        const { std::cell::RefCell::new(None) };
    static BATCH_LENGTHS: std::cell::RefCell<Option<Vec<usize>>> =
        const { std::cell::RefCell::new(None) };
    static WIDENED_REGIONS: std::cell::RefCell<Option<Vec<WidenedRegion>>> =
        const { std::cell::RefCell::new(None) };
    static TIGHT_SKIPS: std::cell::RefCell<Option<Vec<(DocId, DocId)>>> =
        const { std::cell::RefCell::new(None) };
    static CURSOR_PROGRESS: std::cell::RefCell<Option<Vec<CursorProgress>>> =
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
    // Cold preparation scratch; ClauseState stays compact in the replay loops.
    let mut selected_ends = [max_doc; 32];
    let mut lo = clauses
        .iter()
        .map(|clause| clause.doc)
        .min()
        .unwrap_or(TERMINATED);

    // Global maxima and preference order are immutable, and callback thresholds
    // only rise. Retain the exact sequential sum of the certified rank prefix;
    // the density policy below still chooses its effective count per region.
    let mut certified_global = 0;
    let mut certified_global_sum = 0.0f64;
    #[cfg(test)]
    let mut global_appends = 0;

    while lo < max_doc {
        debug_assert!(clauses
            .iter()
            .all(|clause| clause.doc == scorers[clause.ordinal].doc()));
        while certified_global < num_terms {
            let next_sum = certified_global_sum + f64::from(clauses[certified_global].global_max);
            if upper_bound.score(next_sum) > threshold {
                break;
            }
            certified_global_sum = next_sum;
            certified_global += 1;
            #[cfg(test)]
            {
                global_appends += 1;
            }
        }
        #[cfg(test)]
        {
            // Independent old from-zero computation also uses the generic
            // bound helper, rather than relying on the cached sum or count.
            let naive_bound = crate::query::score_combiner::ScoreSumUpperBound::new(num_terms);
            let mut naive_count = 0;
            let mut naive_sum = 0.0f64;
            for clause in &clauses {
                let next_sum = naive_sum + f64::from(clause.global_max);
                if naive_bound.score(next_sum) > threshold {
                    break;
                }
                naive_sum = next_sum;
                naive_count += 1;
            }
            assert_eq!(certified_global, naive_count);
            assert_eq!(certified_global_sum.to_bits(), naive_sum.to_bits());
            assert_eq!(global_appends, certified_global);
            assert!(global_appends <= num_terms);
            GLOBAL_PREFIX_STATES.with(|states| {
                if let Some(states) = states.borrow_mut().as_mut() {
                    states.push((
                        lo,
                        threshold.to_bits(),
                        certified_global,
                        certified_global_sum.to_bits(),
                        global_appends,
                    ));
                }
            });
        }
        if certified_global == num_terms {
            return;
        }
        let mut globally_optional = certified_global;
        #[cfg(test)]
        let certified = certified_global;
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
        selected_ends[..num_terms].fill(max_doc);
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
            selected_ends[rank] = last_doc.min(max_doc - 1) + 1;
            hi = hi.min(selected_ends[rank]);
            let bound = scorers[clause.ordinal].block_max_score();
            clause.assert_coherent(&scorers);
            clause.local_max = if bound.is_finite() && bound >= 0.0 {
                bound
            } else {
                clause.global_max
            };
        }
        debug_assert!(hi > lo);
        for (rank, clause) in clauses.iter().enumerate() {
            // Tentative cleanup is virtual: a doc beyond this tight endpoint
            // may contribute after widening. Keep its selected bound intact.
            let effective = if clause.doc >= hi {
                0.0
            } else {
                clause.local_max
            };
            prefix[rank + 1] = prefix[rank] + f64::from(effective);
        }
        let mut first_essential = 0;
        while first_essential < num_terms
            && upper_bound.score(prefix[first_essential + 1]) <= threshold
        {
            first_essential += 1;
        }
        if first_essential == num_terms {
            #[cfg(test)]
            TIGHT_SKIPS.with(|skips| {
                if let Some(skips) = skips.borrow_mut().as_mut() {
                    skips.push((lo, hi));
                }
            });
            debug_assert!(clauses
                .iter()
                .all(|clause| clause.doc == scorers[clause.ordinal].doc()));
            lo = hi;
            continue;
        }

        let tight_hi = hi;
        #[cfg(test)]
        let tight_first_essential = first_essential;
        // Propose once from the old essential suffix. Crossed old optionals
        // receive range certificates before the final partition is chosen.
        hi = selected_ends[first_essential..num_terms]
            .iter()
            .copied()
            .min()
            .unwrap_or(max_doc);
        debug_assert!(tight_hi <= hi && hi <= max_doc);
        if hi > tight_hi {
            #[cfg(test)]
            let selected_bounds = clauses.iter().map(|clause| clause.local_max).collect();
            for (rank, clause) in clauses.iter_mut().enumerate() {
                clause.local_max = if clause.doc >= hi {
                    0.0
                } else if selected_ends[rank] < hi {
                    // Preserve the original selected certificate too. Lookahead
                    // reads only cloned metadata and never enters the real
                    // selected-block cache or the retained-candidate owner.
                    clause
                        .local_max
                        .max(scorers[clause.ordinal].range_max_score(lo, hi))
                } else {
                    clause.local_max
                };
                prefix[rank + 1] = prefix[rank] + f64::from(clause.local_max);
            }
            first_essential = 0;
            while first_essential < num_terms
                && upper_bound.score(prefix[first_essential + 1]) <= threshold
            {
                first_essential += 1;
            }
            #[cfg(test)]
            WIDENED_REGIONS.with(|regions| {
                if let Some(regions) = regions.borrow_mut().as_mut() {
                    regions.push(WidenedRegion {
                        lo,
                        tight_hi,
                        hi,
                        tight_first_essential,
                        first_essential,
                        ordinals: clauses.iter().map(|clause| clause.ordinal).collect(),
                        docs: clauses.iter().map(|clause| clause.doc).collect(),
                        bounds: clauses.iter().map(|clause| clause.local_max).collect(),
                        selected_ends: selected_ends[..num_terms].to_vec(),
                        selected_bounds,
                    });
                }
            });
            if first_essential == num_terms {
                lo = hi;
                continue;
            }
        } else {
            // The tentative prefix already used these same zeros; retain its
            // partition when the endpoint does not expand.
            for clause in &mut clauses {
                if clause.doc >= hi {
                    clause.local_max = 0.0;
                }
            }
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
            let doc = if single_essential {
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
                batch.docs[batch_index]
            } else {
                clauses[first_essential..]
                    .iter()
                    .map(|clause| clause.doc)
                    .min()
                    .unwrap_or(TERMINATED)
            };
            if doc >= hi {
                break;
            }
            contributions.fill(0.0);
            let mut known = 0.0f64;
            if single_essential {
                let leaf = batch.scores[batch_index];
                contributions[clauses[first_essential].ordinal] = leaf;
                known = f64::from(leaf);
            } else {
                for clause in &clauses[first_essential..] {
                    if clause.doc == doc {
                        clause.assert_coherent(&scorers);
                        let leaf = scorers[clause.ordinal].score();
                        clause.assert_coherent(&scorers);
                        contributions[clause.ordinal] = leaf;
                        known += f64::from(leaf);
                    }
                }
            }
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
            } else {
                for clause in &mut clauses[first_essential..] {
                    if clause.doc == doc {
                        clause.advance(&mut scorers);
                    }
                }
            }
        }
        debug_assert!(clauses
            .iter()
            .all(|clause| clause.doc == scorers[clause.ordinal].doc()));
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

    type ScalarLeaf = (DocId, u32, u32);

    struct WideningEvidence {
        regions: Vec<WidenedRegion>,
        progress: Vec<CursorProgress>,
        skips: Vec<(DocId, DocId)>,
    }

    fn scalar_leaves(scorers: &[TermScorer]) -> Vec<Vec<ScalarLeaf>> {
        scorers
            .iter()
            .cloned()
            .map(|mut scorer| {
                let mut leaves = Vec::new();
                while scorer.doc() != TERMINATED {
                    leaves.push((scorer.doc(), scorer.term_freq(), scorer.score().to_bits()));
                    scorer.advance();
                }
                leaves
            })
            .collect()
    }

    fn begin_widening_evidence() {
        WIDENED_REGIONS.with(|regions| *regions.borrow_mut() = Some(Vec::new()));
        CURSOR_PROGRESS.with(|progress| *progress.borrow_mut() = Some(Vec::new()));
        TIGHT_SKIPS.with(|skips| *skips.borrow_mut() = Some(Vec::new()));
    }

    fn finish_widening_evidence(leaves: &[Vec<ScalarLeaf>]) -> WideningEvidence {
        let evidence = WideningEvidence {
            regions: WIDENED_REGIONS.with(|regions| regions.borrow_mut().take().unwrap()),
            progress: CURSOR_PROGRESS.with(|progress| progress.borrow_mut().take().unwrap()),
            skips: TIGHT_SKIPS.with(|skips| skips.borrow_mut().take().unwrap()),
        };
        for region in &evidence.regions {
            assert!(region.lo < region.tight_hi && region.tight_hi < region.hi);
            assert!(region.tight_first_essential < leaves.len());
            assert_eq!(
                region.hi,
                *region.selected_ends[region.tight_first_essential..]
                    .iter()
                    .min()
                    .unwrap()
            );
            if region.first_essential + 1 == leaves.len() {
                // The last rank was an old essential too. A widened sole driver
                // cannot cross its own selected block end; refills cross regions.
                assert!(region.hi <= region.selected_ends[leaves.len() - 1]);
            }
            for (rank, &ordinal) in region.ordinals.iter().enumerate() {
                let floor = region.lo.max(region.docs[rank]);
                for &(doc, _, bits) in &leaves[ordinal] {
                    if floor <= doc && doc < region.hi {
                        let leaf = Score::from_bits(bits);
                        assert!(
                            leaf <= region.bounds[rank],
                            "uncovered literal leaf: region={region:?} rank={rank} doc={doc} \
                             leaf={leaf}"
                        );
                    }
                }
                if region.docs[rank] >= region.hi {
                    assert_eq!(region.bounds[rank].to_bits(), 0.0f32.to_bits());
                }
            }
        }
        let at = |ordinal: usize, floor: DocId| -> ScalarLeaf {
            leaves[ordinal]
                .iter()
                .copied()
                .find(|&(doc, _, _)| doc >= floor)
                .unwrap_or((TERMINATED, 0, 0))
        };
        for event in &evidence.progress {
            match event {
                CursorProgress::Seek {
                    ordinal,
                    before,
                    target,
                    doc,
                    tf,
                    bits,
                } => {
                    assert_eq!(
                        (*doc, *tf, *bits),
                        at(*ordinal, (*before).max(*target)),
                        "{event:?}"
                    );
                }
                CursorProgress::Advance {
                    ordinal,
                    before,
                    doc,
                    tf,
                    bits,
                } => {
                    assert_ne!(*before, TERMINATED);
                    assert_eq!((*doc, *tf, *bits), at(*ordinal, before + 1), "{event:?}");
                }
                CursorProgress::Batch {
                    ordinal,
                    before,
                    floor,
                    up_to,
                    docs,
                    bits,
                    doc,
                    tf,
                } => {
                    let expected = &leaves[*ordinal];
                    let start =
                        expected.partition_point(|&(doc, _, _)| doc < (*before).max(*floor));
                    assert_eq!(docs.len(), bits.len());
                    assert!(docs.len() <= 128);
                    assert!(docs.iter().all(|&doc| *floor <= doc && doc < *up_to));
                    for (offset, (&doc, &bits)) in docs.iter().zip(bits).enumerate() {
                        let (expected_doc, _, expected_bits) = expected[start + offset];
                        assert_eq!((doc, bits), (expected_doc, expected_bits), "{event:?}");
                    }
                    let (next_doc, next_tf, _) = expected
                        .get(start + docs.len())
                        .copied()
                        .unwrap_or((TERMINATED, 0, 0));
                    assert_eq!((*doc, *tf), (next_doc, next_tf), "{event:?}");
                    if docs.is_empty() {
                        assert!(*doc >= *up_to);
                    }
                }
            }
        }
        evidence
    }

    fn check_widening(
        scorers: Vec<TermScorer>,
        max_doc: DocId,
        threshold: Score,
        limit: usize,
    ) -> (Vec<(DocId, u32)>, WideningEvidence) {
        let leaves = scalar_leaves(&scorers);
        begin_widening_evidence();
        let accepted = check(scorers, max_doc, threshold, limit);
        (accepted, finish_widening_evidence(&leaves))
    }

    #[test]
    fn wider_region_restores_old_zero_and_promotes_future_high_tf_block() {
        let norms = vec![1; 1200];
        let dense: Vec<_> = (0..1024).map(|doc| (doc, 1)).collect();
        let future: Vec<_> = (300..900)
            .map(|doc| (doc, if doc == 428 { 1000 } else { 1 }))
            .collect();
        let driver: Vec<_> = (0..130).map(|i| (i * 4, 1)).collect();
        let scorers = vec![
            fixture(&dense, &norms, 0.01),
            fixture(&future, &norms, 0.02),
            fixture(&driver, &norms, 0.1),
        ];
        for reverse in [false, true] {
            let mut ordered = scorers.clone();
            if reverse {
                ordered.reverse();
            }
            let future_ordinal = 1;
            let (accepted, evidence) = check_widening(ordered, 1200, 0.015, 1);
            assert!(accepted.iter().any(|&(doc, _)| doc == 428));
            let region = evidence
                .regions
                .iter()
                .find(|region| region.lo == 0)
                .unwrap();
            let rank = region
                .ordinals
                .iter()
                .position(|&ordinal| ordinal == future_ordinal)
                .unwrap();
            assert!(region.tight_hi <= 300 && 300 < region.hi);
            assert_eq!(region.docs[rank], 300, "old cleanup must have been zero");
            assert!(region.bounds[rank] > 0.0);
            assert!(
                rank >= region.first_essential,
                "crossed optional must be promoted"
            );
            assert!(region.selected_ends[rank] < region.hi);
            let future_leaf = scalar_leaves(&scorers)[1]
                .iter()
                .find(|&&(doc, _, _)| doc == 428)
                .unwrap()
                .2;
            assert!(Score::from_bits(future_leaf) > region.selected_bounds[rank]);
            assert_eq!(
                region.bounds[rank].to_bits(),
                scorers[1].max_score().to_bits()
            );
            assert!(rank < region.tight_first_essential);
            assert!(
                evidence.progress.iter().any(|event| matches!(event,
                    CursorProgress::Advance { ordinal, before: 427, doc: 428, tf: 1000, .. }
                    if *ordinal == future_ordinal
                )),
                "promoted scalar driver must cross its full-block boundary"
            );
        }
    }

    #[test]
    fn wider_single_driver_retains_full_partial_batches_and_exact_end_heads() {
        let norms = vec![1; 4096];
        let dense: Vec<_> = (0..1024).map(|doc| (doc, 1)).collect();
        for length in [127u32, 128, 129, 255, 256, 257, 768] {
            // At small lengths increase the dense driver's physical gap while
            // retaining comparable cost so the global density test stays false.
            let weak_len = if length < 512 { length + 1 } else { 1024 };
            let weak: Vec<_> = dense[..weak_len as usize].to_vec();
            let driver: Vec<_> = (0..length)
                .map(|i| {
                    (
                        if i == 128 { 382 } else { i * 3 },
                        // A partial final block must still exceed the Top1
                        // threshold raised by the earlier TF20 leaf. Otherwise
                        // correctly skipping that tail provides no batch witness.
                        if i + 1 == length && length % 128 != 0 {
                            30
                        } else if i == 128 {
                            20
                        } else {
                            1
                        },
                    )
                })
                .collect();
            let scorers = vec![
                fixture(&weak, &norms, 0.001),
                fixture(&driver, &norms, 1.0),
                fixture(&weak, &norms, 0.002),
            ];
            let (accepted, evidence) = check_widening(scorers, 4096, 0.01, 1);
            assert!(!evidence.regions.is_empty(), "length={length}");
            assert!(evidence
                .regions
                .iter()
                .all(|region| region.first_essential == 2));
            if length >= 129 {
                assert!(accepted.iter().any(|&(doc, _)| doc == 382));
                assert!(evidence.progress.iter().any(|event| matches!(event,
                    CursorProgress::Batch { docs, doc: 382, up_to: 382, .. } if docs.len() == 128
                )), "exact exclusive end must remain first unread: length={length}");
            }
            assert!(
                evidence.progress.iter().any(|event| matches!(event,
                    CursorProgress::Batch { docs, .. } if !docs.is_empty() && docs.len() < 128
                )) || length == 128
                    || length == 256
                    || length == 768,
                "required competitive partial batch: length={length}"
            );
        }
    }

    #[test]
    fn wider_certificates_cover_n3_n32_overlap_disjoint_and_tail_leafs() {
        let norms: Vec<_> = (0..2048).map(|doc| 1 + doc % 7).collect();
        for count in [3, 32] {
            for disjoint in [false, true] {
                let weak: Vec<_> = (0..512)
                    .map(|i| (if disjoint { 2 * i + 1 } else { i }, 1 + i % 3))
                    .collect();
                let driver: Vec<_> = (0..384)
                    .map(|i| (i * if disjoint { 4 } else { 3 }, 1 + i % 11))
                    .collect();
                let mut scorers: Vec<_> = (0..count - 1)
                    .map(|ordinal| fixture(&weak, &norms, 0.00001 * (ordinal + 1) as Score))
                    .collect();
                scorers.push(fixture(&driver, &norms, 1.0));
                for reverse in [false, true] {
                    let mut ordered = scorers.clone();
                    if reverse {
                        ordered.reverse();
                    }
                    for limit in [0, 1, 10] {
                        let (_, evidence) = check_widening(ordered.clone(), 2048, 0.05, limit);
                        assert!(
                            !evidence.regions.is_empty(),
                            "count={count} disjoint={disjoint} limit={limit}"
                        );
                    }
                }
            }
        }
        eprintln!(
            "OR_WIDEN_LAYOUT cap_scratch={} ClauseState={}",
            std::mem::size_of::<[DocId; 32]>(),
            std::mem::size_of::<ClauseState>()
        );
    }

    #[test]
    fn wider_stale_full_selection_tail_exhaustion_and_initial_skips_are_exact() {
        let norms = vec![1; 1800];
        let weak: Vec<_> = (0..768).map(|doc| (doc, 1)).collect();
        let driver: Vec<_> = (0..700)
            .map(|i| (i * 2, if i == 150 || i == 699 { 50 } else { 1 }))
            .collect();
        let mut exhausted = fixture(&weak, &norms, 0.002);
        while exhausted.advance() != TERMINATED {}
        let scorers = vec![
            fixture(&weak, &norms, 0.003),
            exhausted,
            fixture(&driver, &norms, 1.0),
        ];
        let (accepted, evidence) = check_widening(scorers.clone(), 1800, 1.5, 0);
        assert_eq!(
            accepted.iter().map(|&(doc, _)| doc).collect::<Vec<_>>(),
            [300, 1398]
        );
        assert!(!evidence.skips.is_empty());
        assert!(evidence.skips.iter().all(|&(lo, hi)| lo < hi));
        assert!(!evidence.regions.is_empty());
        assert!(
            evidence.progress.iter().any(|event| matches!(
                event,
                CursorProgress::Seek {
                    ordinal: 2,
                    before: 0,
                    target: 255,
                    doc: 256,
                    ..
                }
            )),
            "new essential must reconcile the shallow-selected full block"
        );
        for threshold in [0.0, 0.5, 1.5, Score::INFINITY] {
            check_widening(scorers.clone(), 1800, threshold, 1);
        }
    }

    #[test]
    fn wider_midpoint_publication_keeps_literal_incoming_order_and_strict_neighbors() {
        let norms = vec![1; 1200];
        let weak: Vec<_> = (0..512).map(|doc| (doc, 1)).collect();
        // Preserve doc0's exact midpoint leaf, but require a wider first
        // certificate even when the initial threshold rejects doc0 itself.
        // The later stronger leaf lies in the same selected full driver block.
        let driver: Vec<_> = (0..300)
            .map(|i| (i * 3, if i == 127 { 2 } else { 1 }))
            .collect();
        let leaves = [1.0, 2.0f32.powi(-24), 2.0f32.powi(-53), 2.0f32.powi(-53)];
        let scorers: Vec<_> = leaves
            .iter()
            .enumerate()
            .map(|(ordinal, &leaf)| {
                fixture(if ordinal == 0 { &driver } else { &weak }, &norms, leaf)
            })
            .collect();
        for order in [[0, 1, 2, 3], [2, 3, 1, 0], [1, 0, 3, 2]] {
            let ordered: Vec<_> = order
                .iter()
                .map(|&ordinal| scorers[ordinal].clone())
                .collect();
            let score = order
                .iter()
                .map(|&ordinal| f64::from(leaves[ordinal]))
                .sum::<f64>() as Score;
            for threshold in [score.next_down(), score, score.next_up()] {
                for limit in [0, 1, 10] {
                    let (accepted, evidence) =
                        check_widening(ordered.clone(), 1200, threshold, limit);
                    assert!(!evidence.regions.is_empty());
                    assert!(
                        accepted.iter().any(|&(doc, _)| doc == 381),
                        "later competitive full-block leaf: order={order:?} \
                         threshold={threshold:?} limit={limit}"
                    );
                    if threshold < score {
                        assert_eq!(accepted[0], (0, score.to_bits()));
                    } else {
                        assert!(!accepted.iter().any(|&(doc, _)| doc == 0));
                    }
                }
            }
        }
    }

    fn indexed_widening_fixture(
        with_deletes: bool,
    ) -> crate::Result<(crate::Index, crate::schema::Field)> {
        use crate::schema::{Schema, INDEXED, TEXT};
        let mut schema = Schema::builder();
        let text = schema.add_text_field("text", TEXT);
        let id = schema.add_u64_field("id", INDEXED);
        let index = crate::Index::create_in_ram(schema.build());
        let mut writer = index.writer_for_tests()?;
        for doc in 0..1536 {
            let mut content = "alpha beta".to_string();
            // Real native DEFAULT selection: beta's first full block contains
            // only TF1, but the crossed second block has a larger literal leaf.
            if doc == 150 {
                for _ in 0..19 {
                    content.push_str(" beta");
                }
            }
            if doc % 3 != 2 {
                let tf = match doc {
                    127 => 2000,
                    1024 => 1000,
                    130 => 20,
                    _ => 1,
                };
                for _ in 0..tf {
                    content.push_str(" gamma");
                }
            }
            writer.add_document(doc!(text => content, id => doc as u64))?;
        }
        writer.commit()?;
        if with_deletes {
            writer.delete_term(crate::Term::from_field_u64(id, 0));
            writer.delete_term(crate::Term::from_field_u64(id, 127));
            writer.commit()?;
        }
        Ok((index, text))
    }

    fn indexed_scorers(
        searcher: &crate::Searcher,
        field: crate::schema::Field,
        count: usize,
        boosted: bool,
    ) -> crate::Result<Vec<TermScorer>> {
        use crate::query::{EnableScoring, TermQuery};
        use crate::schema::IndexRecordOption;
        let mut scorers = Vec::new();
        for ordinal in 0..count {
            let text = if ordinal + 1 == count {
                "gamma"
            } else if ordinal % 2 == 0 {
                "alpha"
            } else {
                "beta"
            };
            let query = TermQuery::new(
                crate::Term::from_field_text(field, text),
                IndexRecordOption::WithFreqs,
            );
            let weight =
                query.specialized_weight(EnableScoring::enabled_from_searcher(searcher))?;
            let boost = if boosted && ordinal + 1 != count {
                0.001
            } else {
                1.0
            };
            scorers.push(
                weight
                    .term_scorer_for_test(searcher.segment_reader(0), boost)?
                    .unwrap(),
            );
        }
        Ok(scorers)
    }

    #[test]
    fn wider_native_indexed_default_and_safe_custom_profiles_enclose_every_leaf(
    ) -> crate::Result<()> {
        use crate::query::Bm25Parameters;
        let (index, field) = indexed_widening_fixture(false)?;
        let reader = index.reader()?;
        assert_eq!(reader.searcher().segment_readers().len(), 1);
        for parameters in [
            Bm25Parameters::DEFAULT,
            Bm25Parameters::new(0.9, 0.4)?,
            Bm25Parameters::new(2.0, 1.0)?,
            Bm25Parameters::new(0.0, 0.75)?,
        ] {
            let searcher = reader.searcher().with_bm25_parameters(parameters);
            for count in [3, 32] {
                let scorers = indexed_scorers(&searcher, field, count, true)?;
                for limit in [0, 1, 10] {
                    let (_, evidence) = check_widening(scorers.clone(), 1536, 0.01, limit);
                    assert!(
                        !evidence.regions.is_empty(),
                        "count={count} parameters={parameters:?}"
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    fn wider_native_default_crossed_pair_is_replaced_before_stronger_leaf() -> crate::Result<()> {
        use crate::query::{EnableScoring, TermQuery};
        use crate::schema::IndexRecordOption;
        let (index, field) = indexed_widening_fixture(false)?;
        let searcher = index.reader()?.searcher();
        let mut scorers = Vec::new();
        for (term, boost) in [("alpha", 0.005), ("beta", 0.01), ("gamma", 1.0)] {
            let query = TermQuery::new(
                crate::Term::from_field_text(field, term),
                IndexRecordOption::WithFreqs,
            );
            let weight =
                query.specialized_weight(EnableScoring::enabled_from_searcher(&searcher))?;
            scorers.push(
                weight
                    .term_scorer_for_test(searcher.segment_reader(0), boost)?
                    .unwrap(),
            );
        }
        let selected_sum = scorers[..2]
            .iter()
            .cloned()
            .map(|mut scorer| f64::from(scorer.block_max_score()))
            .sum::<f64>();
        let leaves = scalar_leaves(&scorers);
        let later_leaf = leaves[1].iter().find(|&&(doc, _, _)| doc == 150).unwrap().2;
        let selected_alpha = scorers[0].clone().block_max_score();
        // Derive the threshold from an independent literal later beta leaf,
        // not the new range helper or A's deliberately loose global fallback.
        let later_sum = f64::from(selected_alpha) + f64::from(Score::from_bits(later_leaf));
        assert!(selected_sum < later_sum);
        let threshold = ((selected_sum + later_sum) * 0.5) as Score;
        let (_, evidence) = check_widening(scorers.clone(), 1536, threshold, 0);
        let region = evidence
            .regions
            .iter()
            .find(|region| region.lo == 0)
            .unwrap();
        let rank = region
            .ordinals
            .iter()
            .position(|&ordinal| ordinal == 1)
            .unwrap();
        assert!(region.tight_hi <= 150 && 150 < region.hi);
        assert!(rank < region.tight_first_essential && rank >= region.first_essential);
        assert!(region.selected_ends[rank] < region.hi);
        assert!(Score::from_bits(later_leaf) > region.selected_bounds[rank]);
        assert!(region.bounds[rank] >= Score::from_bits(later_leaf));
        assert!(
            region.bounds[rank] < scorers[1].max_score(),
            "unsaturated native range must be tighter than query-global"
        );
        assert!(
            evidence.progress.iter().any(|event| matches!(
                event,
                CursorProgress::Advance {
                    ordinal: 1,
                    before: 127,
                    doc: 128,
                    ..
                }
            )),
            "promoted scalar beta must cross its full-block boundary"
        );
        Ok(())
    }

    #[test]
    fn wider_legacy_custom_parameters_reuse_existing_global_bound_policy() -> crate::Result<()> {
        use crate::query::{Bm25FieldStatistics, Bm25Parameters, Bm25StatisticsProvider};
        use crate::schema::{Field, Schema, TEXT};
        struct Statistics(Bm25Parameters);
        impl Bm25StatisticsProvider for Statistics {
            fn total_num_tokens(&self, _field: Field) -> crate::Result<u64> {
                Ok(1200)
            }
            fn total_num_docs(&self) -> crate::Result<u64> {
                Ok(1200)
            }
            fn doc_freq(&self, _term: &crate::Term) -> crate::Result<u64> {
                Ok(600)
            }
            fn field_statistics(&self, _field: Field) -> crate::Result<Bm25FieldStatistics> {
                Ok(Bm25FieldStatistics::new(1200, 1200).with_parameters(self.0))
            }
        }
        let mut schema = Schema::builder();
        let field = schema.add_text_field("text", TEXT);
        let term = crate::Term::from_field_text(field, "fixture");
        let norms = vec![1; 1200];
        let weak: Vec<_> = (0..512).map(|doc| (doc, 1)).collect();
        let driver: Vec<_> = (0..300).map(|i| (i * 3, 1 + i % 7)).collect();
        for parameters in [
            Bm25Parameters::new(0.9, 0.4)?,
            Bm25Parameters::new(2.0, 1.0)?,
            Bm25Parameters::new(0.0, 0.75)?,
        ] {
            let weight =
                Bm25Weight::for_terms(&Statistics(parameters), std::slice::from_ref(&term))?;
            assert!(weight.has_safe_score_bounds());
            let scorers = vec![
                TermScorer::create_for_test(&weak, &norms, weight.boost_by(0.001)),
                TermScorer::create_for_test(&driver, &norms, weight.clone()),
                TermScorer::create_for_test(&weak, &norms, weight.boost_by(0.002)),
            ];
            let mut weak_bound = scorers[0].clone();
            assert_eq!(
                weak_bound.block_max_score().to_bits(),
                weak_bound.max_score().to_bits(),
                "nondefault legacy full blocks must retain the existing global policy"
            );
            for limit in [0, 1, 10] {
                let (_, evidence) = check_widening(scorers.clone(), 1200, 0.01, limit);
                assert!(!evidence.regions.is_empty());
            }
        }
        Ok(())
    }

    #[test]
    fn wider_real_indexed_deletions_keep_threshold_and_public_topdocs_exact() -> crate::Result<()> {
        use crate::collector::TopDocs;
        use crate::query::{BooleanQuery, Occur, Query, TermQuery};
        use crate::schema::IndexRecordOption;
        let (index, field) = indexed_widening_fixture(true)?;
        let searcher = index.reader()?.searcher();
        let segment = searcher.segment_reader(0);
        assert_eq!(segment.max_doc(), 1536);
        assert_eq!(segment.num_docs(), 1534);
        let alive = segment.alive_bitset().unwrap();
        let scorers = indexed_scorers(&searcher, field, 3, false)?;
        let leaves = scalar_leaves(&scorers);
        let run = |optimized: bool| {
            let mut callbacks = Vec::new();
            let mut current = Score::NEG_INFINITY;
            let mut best = None;
            let mut callback = |doc, score: Score| {
                callbacks.push((doc, score.to_bits(), current.to_bits()));
                if alive.is_alive(doc) {
                    current = score;
                    best = Some((score.to_bits(), doc));
                }
                current
            };
            if optimized {
                or_maxscore(scorers.clone(), 1536, Score::NEG_INFINITY, &mut callback);
            } else {
                let mut union =
                    BufferedUnionScorer::build(scorers.clone(), SumCombiner::default, 1536);
                for_each_pruning_scorer(&mut union, Score::NEG_INFINITY, &mut callback);
            }
            (callbacks, best.unwrap())
        };
        let expected = run(false);
        begin_widening_evidence();
        let actual = run(true);
        let evidence = finish_widening_evidence(&leaves);
        assert_eq!(actual, expected);
        assert!(actual.0.iter().any(|&(doc, _, _)| doc == 127));
        assert!(actual.1 .1 >= 128, "must retain a later live winner");
        assert!(!evidence.regions.is_empty());
        let query = BooleanQuery::new(
            ["alpha", "beta", "gamma"]
                .into_iter()
                .map(|term| {
                    (
                        Occur::Should,
                        Box::new(TermQuery::new(
                            crate::Term::from_field_text(field, term),
                            IndexRecordOption::WithFreqs,
                        )) as Box<dyn Query>,
                    )
                })
                .collect(),
        );
        begin_widening_evidence();
        let top = searcher.search(&query, &TopDocs::with_limit(1).order_by_score())?;
        let public_evidence = finish_widening_evidence(&leaves);
        assert!(
            !public_evidence.regions.is_empty(),
            "public TopDocs must actually widen"
        );
        assert_eq!(top.len(), 1);
        assert_eq!((top[0].0.to_bits(), top[0].1.doc_id), expected.1);
        assert!(alive.is_alive(top[0].1.doc_id));
        Ok(())
    }

    #[test]
    fn wider_simulated_deleted_winners_and_unsafe_fallback_observers_are_exact() {
        let norms = vec![1; 1200];
        let dense: Vec<_> = (0..512).map(|doc| (doc, 1)).collect();
        let driver: Vec<_> = (0..300)
            .map(|i| {
                (
                    i * 3,
                    if i == 127 || i == 128 || i == 200 {
                        20
                    } else {
                        1
                    },
                )
            })
            .collect();
        let scorers = vec![
            fixture(&dense, &norms, 0.001),
            fixture(&driver, &norms, 1.0),
            fixture(&dense, &norms, 0.002),
        ];
        let leaves = scalar_leaves(&scorers);
        let run = |optimized| {
            let mut callbacks = Vec::new();
            let mut current: Score = 0.01;
            let mut callback = |doc, score: Score| {
                callbacks.push((doc, score.to_bits(), current.to_bits()));
                if doc != 0 && doc != 381 && doc != 384 {
                    current = current.max(score);
                }
                current
            };
            if optimized {
                or_maxscore(scorers.clone(), 1200, 0.01, &mut callback);
            } else {
                let mut union =
                    BufferedUnionScorer::build(scorers.clone(), SumCombiner::default, 1200);
                for_each_pruning_scorer(&mut union, 0.01, &mut callback);
            }
            callbacks
        };
        let expected = run(false);
        begin_widening_evidence();
        let actual = run(true);
        let evidence = finish_widening_evidence(&leaves);
        assert_eq!(actual, expected);
        assert!(actual.iter().any(|&(doc, _, _)| doc == 600));
        assert!(!evidence.regions.is_empty());
        for boost in [0.0, -1.0, Score::INFINITY, Score::NAN] {
            let exceptional = fixture(&dense, &norms, boost);
            let (_, evidence) = check_widening(
                vec![scorers[0].clone(), exceptional, scorers[1].clone()],
                1200,
                Score::NEG_INFINITY,
                0,
            );
            assert!(evidence.regions.is_empty());
            assert!(evidence.progress.is_empty());
            assert!(evidence.skips.is_empty());
        }
    }

    fn check_scheduled_thresholds(
        scorers: Vec<TermScorer>,
        max_doc: DocId,
        initial: Score,
        schedule: &[(DocId, Score)],
    ) -> (Vec<(DocId, u32)>, Vec<GlobalPrefixState>, Vec<RegionChoice>) {
        assert!(schedule.windows(2).all(|pair| pair[0].1 <= pair[1].1));
        let run = |optimized: bool| {
            let mut accepted = Vec::new();
            let mut current = initial;
            let mut callback = |doc, score: Score| {
                accepted.push((doc, score.to_bits()));
                for &(from, planned) in schedule {
                    if from <= doc {
                        current = current.max(planned);
                    }
                }
                current
            };
            if optimized {
                or_maxscore(scorers.clone(), max_doc, initial, &mut callback);
            } else {
                let mut union =
                    BufferedUnionScorer::build(scorers.clone(), SumCombiner::default, max_doc);
                for_each_pruning_scorer(&mut union, initial, &mut callback);
            }
            assert!(accepted.windows(2).all(|pair| pair[0].0 < pair[1].0));
            accepted
        };
        let expected = run(false);
        GLOBAL_PREFIX_STATES.with(|states| *states.borrow_mut() = Some(Vec::new()));
        GLOBAL_CHOICES.with(|choices| *choices.borrow_mut() = Some(Vec::new()));
        let actual = run(true);
        let states = GLOBAL_PREFIX_STATES.with(|states| states.borrow_mut().take().unwrap());
        let choices = GLOBAL_CHOICES.with(|choices| choices.borrow_mut().take().unwrap());
        assert_eq!(actual, expected);
        assert!(states.windows(2).all(|pair| pair[0].2 <= pair[1].2));
        assert!(states.iter().all(|state| state.4 <= scorers.len()));
        (actual, states, choices)
    }

    #[test]
    fn cached_global_prefix_preserves_equality_neighbors_and_large_threshold_jumps() {
        let norms = vec![1; 2048];
        let postings: Vec<_> = (0..2048).map(|doc| (doc, 1)).collect();
        for native in [false, true] {
            for num_terms in [4, 32] {
                let boosts = [0.001, 0.002, 0.003, 1.0];
                let mut scorers: Vec<_> = boosts
                    .iter()
                    .map(|&boost| {
                        let weight = if native {
                            Bm25Weight::for_native_block_bounds(1.0).boost_by(boost)
                        } else {
                            Bm25Weight::new_without_explain(1.0, 1.0).boost_by(boost)
                        };
                        assert!(weight.has_safe_score_bounds());
                        TermScorer::create_for_test(&postings, &norms, weight)
                    })
                    .collect();
                for _ in 4..num_terms {
                    let weight = if native {
                        Bm25Weight::for_native_block_bounds(1.0).boost_by(0.125)
                    } else {
                        Bm25Weight::new_without_explain(1.0, 1.0).boost_by(0.125)
                    };
                    scorers.push(TermScorer::create_for_test(&postings, &norms, weight));
                }
                let generic = crate::query::score_combiner::ScoreSumUpperBound::new(num_terms);
                let mut sum = 0.0f64;
                let bounds: Vec<_> = scorers[..3]
                    .iter()
                    .map(|scorer| {
                        sum += f64::from(scorer.max_score());
                        generic.score(sum)
                    })
                    .collect();
                let schedule = [
                    (0, bounds[0].next_down()),
                    (128, bounds[0]),
                    (256, bounds[0].next_up()),
                    (384, bounds[1].next_down()),
                    (512, bounds[1]),
                    (640, bounds[1].next_up()),
                    (768, bounds[2].next_down()),
                    (896, bounds[2]),
                    (1024, bounds[2].next_up()),
                    (1408, 0.25),
                    (1800, Score::INFINITY),
                ];
                for initial in [-Score::INFINITY, -1.0, -0.0, 0.0] {
                    for reversed in [false, true] {
                        let ordered = if reversed {
                            scorers.iter().rev().cloned().collect()
                        } else {
                            scorers.clone()
                        };
                        let (trace, states, _) =
                            check_scheduled_thresholds(ordered, 2048, initial, &schedule);
                        assert_eq!(trace.first().unwrap().0, 0);
                        assert_eq!(trace.last().unwrap().0, 1800);
                        for count in [0, 1, 2, 3, num_terms] {
                            assert!(states.iter().any(|state| state.2 == count), "{states:?}");
                        }
                        for &bound in &bounds {
                            assert!(states.iter().any(|state| state.1 == bound.to_bits()));
                        }
                        let last = states.last().unwrap();
                        assert_eq!(last.1, Score::INFINITY.to_bits());
                        assert_eq!(last.4, num_terms);
                        assert!(states.len() > num_terms.min(10));
                    }
                }
                for initial in [Score::INFINITY, Score::NAN] {
                    assert!(check(scorers.clone(), 2048, initial, 0).is_empty());
                }
            }
        }
    }

    #[test]
    fn cached_certificate_survives_density_rejection_widening_and_rejection_again() {
        let norms = vec![1; 4096];
        let weak: Vec<_> = (0..300).map(|i| (1 + 10 * i, 1)).collect();
        let high_cost: Vec<_> = (0..200).map(|doc| (doc, 1)).collect();
        let driver: Vec<_> = std::iter::once((0, 1))
            .chain((0..128).map(|i| (2000 + 15 * i, 1)))
            .collect();
        // A finite tiny block caps the first widened region at 2160. Its tail
        // still lies beyond the driver's full block cap 3891, ensuring the
        // rising threshold is observed in a third preparation, not swallowed
        // by the first widened region. Keep all three density witnesses.
        let tiny: Vec<_> = (0..128)
            .map(|i| (17 * i, 1))
            .chain(std::iter::once((3999, 1)))
            .collect();
        let scorers = vec![
            fixture(&weak, &norms, 0.01),
            fixture(&high_cost, &norms, 0.02),
            fixture(&driver, &norms, 1.0),
            fixture(&tiny, &norms, 0.1),
        ];
        // Ground the geometry in the actual admitted bounds and preference,
        // rather than assuming that boosts determine the rank by themselves.
        let maxima: Vec<_> = scorers.iter().map(TermScorer::max_score).collect();
        let generic = crate::query::score_combiner::ScoreSumUpperBound::new(4);
        assert!(generic.score(f64::from(maxima[0])) <= 0.04);
        assert!(generic.score(f64::from(maxima[0]) + f64::from(maxima[1])) > 0.04);
        let mut ranks: Vec<_> = (0..scorers.len()).collect();
        ranks.sort_unstable_by(|&left, &right| {
            let priority = |ordinal: usize| {
                f64::from(maxima[ordinal]) / f64::from(scorers[ordinal].size_hint().max(1))
            };
            priority(left)
                .total_cmp(&priority(right))
                .then_with(|| left.cmp(&right))
        });
        assert_eq!(ranks, [0, 1, 3, 2]);
        let first_three = f64::from(maxima[0]) + f64::from(maxima[1]) + f64::from(maxima[3]);
        assert!(generic.score(first_three) <= 0.4);
        assert!(generic.score(first_three + f64::from(maxima[2])) > 0.4);
        let mut tiny_block = scorers[3].clone();
        tiny_block.seek_block(0);
        assert_eq!(tiny_block.last_doc_in_block(), 2159);
        assert!(tiny_block.block_max_score() > 0.04);
        let (trace, states, choices) =
            check_scheduled_thresholds(scorers, 4096, 0.04, &[(3800, 0.4)]);
        assert_eq!(trace.first().unwrap().0, 0);
        assert_eq!(trace.last().unwrap().0, 3905);
        let rejected = choices
            .iter()
            .position(|&(_, certified, effective, cost)| {
                certified == 1 && effective == 0 && cost == 200
            })
            .unwrap();
        let widened = choices
            .iter()
            .position(|&(_, certified, effective, cost)| {
                certified == 1 && effective == 1 && cost == 129
            })
            .unwrap();
        let rejected_again = choices
            .iter()
            .position(|&(_, certified, effective, cost)| {
                certified == 3 && effective == 0 && cost == 129
            })
            .unwrap();
        assert!(
            rejected < widened && widened < rejected_again,
            "{choices:?}"
        );
        assert_eq!(choices[rejected].0, 0);
        assert_eq!(choices[widened].0, 2160);
        assert_eq!(choices[rejected_again].0, 3891);
        assert!(states.iter().any(|state| state.2 == 3));
        assert_eq!(states.last().unwrap().4, 3);
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
        // A finite first block forces another preparation after the high-cost
        // term exhausts. A two-posting tail would now legitimately widen to
        // max_doc and finish before another density decision can be observed.
        let mut rare: Vec<_> = (0..128).map(|i| (i * 20, 1)).collect();
        rare.push((3500, 1));
        let scorers = vec![
            fixture(&weak, &norms, 0.01),
            fixture(&high_cost, &norms, 0.02),
            fixture(&rare, &norms, 1.0),
        ];
        // The first global maximum fits .04, but the second does not. The
        // 300-posting optional term is too small for a global certificate while
        // the 200-posting remainder is live. After actual traversal exhausts
        // that remainder, another preparation accepts the same global prefix
        // against the 129-posting sparse remainder.
        GLOBAL_CHOICES.with(|choices| *choices.borrow_mut() = Some(Vec::new()));
        let accepted = check(scorers.clone(), 4096, 0.04, 0);
        let choices = GLOBAL_CHOICES.with(|choices| choices.borrow_mut().take().unwrap());
        assert_eq!(
            accepted.iter().map(|&(doc, _)| doc).collect::<Vec<_>>(),
            rare.iter().map(|&(doc, _)| doc).collect::<Vec<_>>()
        );
        assert!(choices.iter().all(|&(_, certified, _, _)| certified == 1));
        let rejected = choices
            .iter()
            .position(|&(_, _, selected, cost)| selected == 0 && cost == 200)
            .unwrap();
        let widened = choices
            .iter()
            .position(|&(_, _, selected, cost)| selected == 1 && cost == 129)
            .unwrap();
        assert!(rejected < widened, "region choices: {choices:?}");
        assert!(
            choices[widened].0 > choices[rejected].0,
            "must observe a later preparation"
        );
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
    }
}
