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

    #[inline]
    fn advance(&mut self, scorers: &mut [TermScorer]) {
        self.assert_coherent(scorers);
        self.doc = scorers[self.ordinal].advance();
        self.assert_coherent(scorers);
    }
}

#[cfg(test)]
type RegionChoice = (DocId, usize, usize, u64);

#[cfg(test)]
type GlobalPrefixState = (DocId, u32, usize, u64, usize);

#[cfg(test)]
#[derive(Debug)]
struct DensityState {
    lo: DocId,
    certified: usize,
    effective: usize,
    minimum: u32,
    remaining: u64,
    winner_rank: usize,
    winner_ordinal: Option<usize>,
    rebuild_reason: u8,
    terminated: u32,
}

#[cfg(test)]
thread_local! {
    // Test-only region evidence; no observer or counter exists in release.
    static GLOBAL_CHOICES: std::cell::RefCell<Option<Vec<RegionChoice>>> =
        const { std::cell::RefCell::new(None) };
    static GLOBAL_PREFIX_STATES: std::cell::RefCell<Option<Vec<GlobalPrefixState>>> =
        const { std::cell::RefCell::new(None) };
    static DENSITY_STATES: std::cell::RefCell<Option<Vec<DensityState>>> =
        const { std::cell::RefCell::new(None) };
    static BATCH_LENGTHS: std::cell::RefCell<Option<Vec<usize>>> =
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
    // Fixed rank costs and forward cursors make these drained-boundary facts
    // reusable without any extra work in seek/advance/batch ownership.
    let mut min_certified_cost = u32::MAX;
    let mut max_live_rank = num_terms;
    let mut live_max_ready = false;
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
            min_certified_cost = min_certified_cost.min(clauses[certified_global].cost);
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
            assert_eq!(
                min_certified_cost,
                clauses[..certified_global]
                    .iter()
                    .map(|clause| clause.cost)
                    .min()
                    .unwrap_or(u32::MAX)
            );
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
            // A live maximum stays maximal until the growing prefix excludes
            // it or its actual cursor reaches EOF. Metadata selection alone
            // cannot invalidate this winner or make another clause live again.
            let winner_invalid = max_live_rank < num_terms
                && (max_live_rank < globally_optional || clauses[max_live_rank].doc == TERMINATED);
            #[cfg(test)]
            let rebuild_reason = u8::from(!live_max_ready)
                | (u8::from(live_max_ready && max_live_rank < globally_optional) << 1)
                | (u8::from(
                    live_max_ready
                        && max_live_rank < num_terms
                        && clauses[max_live_rank].doc == TERMINATED,
                ) << 2);
            if !live_max_ready || winner_invalid {
                max_live_rank = num_terms;
                for (rank, clause) in clauses.iter().enumerate().skip(globally_optional) {
                    if clause.doc != TERMINATED
                        && (max_live_rank == num_terms || clause.cost > clauses[max_live_rank].cost)
                    {
                        max_live_rank = rank;
                    }
                }
                live_max_ready = true;
            }
            let remaining_cost = if max_live_rank < num_terms {
                u64::from(clauses[max_live_rank].cost)
            } else {
                0
            };
            #[cfg(test)]
            {
                observed_remaining_cost = remaining_cost;
            }
            // A looser global certificate is useful when it leaves a much
            // sparser candidate driver. For comparable-density clauses, keep
            // tight local bounds so another dense term need not be essential.
            let sparse_remainder = u64::from(min_certified_cost) >= 2 * remaining_cost;
            #[cfg(test)]
            {
                // Independent old from-zero facts; never derive the oracle
                // from the cached winner, minimum, or effective density choice.
                let naive_remaining = clauses[certified_global..]
                    .iter()
                    .filter(|clause| clause.doc != TERMINATED)
                    .map(|clause| u64::from(clause.cost))
                    .max();
                let naive_sparse = clauses[..certified_global]
                    .iter()
                    .all(|clause| u64::from(clause.cost) >= 2 * naive_remaining.unwrap_or(0));
                assert_eq!(remaining_cost, naive_remaining.unwrap_or(0));
                assert_eq!(sparse_remainder, naive_sparse);
                if let Some(naive_maximum) = naive_remaining {
                    assert!(max_live_rank >= certified_global && max_live_rank < num_terms);
                    assert_ne!(clauses[max_live_rank].doc, TERMINATED);
                    assert_eq!(u64::from(clauses[max_live_rank].cost), naive_maximum);
                } else {
                    assert_eq!(max_live_rank, num_terms);
                }
                DENSITY_STATES.with(|states| {
                    if let Some(states) = states.borrow_mut().as_mut() {
                        states.push(DensityState {
                            lo,
                            certified: certified_global,
                            effective: if sparse_remainder {
                                certified_global
                            } else {
                                0
                            },
                            minimum: min_certified_cost,
                            remaining: remaining_cost,
                            winner_rank: max_live_rank,
                            winner_ordinal: (max_live_rank < num_terms)
                                .then(|| clauses[max_live_rank].ordinal),
                            rebuild_reason,
                            terminated: clauses.iter().fold(0u32, |mask, clause| {
                                mask | if clause.doc == TERMINATED {
                                    1u32 << clause.ordinal
                                } else {
                                    0
                                }
                            }),
                        });
                    }
                });
            }
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
    fn density_cache_observes_exhaustion_and_rejection_restoration_rejection() {
        DENSITY_STATES.with(|states| *states.borrow_mut() = Some(Vec::new()));
        // The existing independent complete-callback fixture stays unchanged.
        cached_certificate_survives_density_rejection_widening_and_rejection_again();
        let states = DENSITY_STATES.with(|states| states.borrow_mut().take().unwrap());
        let rejected = states
            .iter()
            .position(|state| {
                state.certified == 1 && state.effective == 0 && state.remaining == 200
            })
            .unwrap();
        let restored = states
            .iter()
            .position(|state| {
                state.certified == 1 && state.effective == 1 && state.remaining == 129
            })
            .unwrap();
        let rejected_again = states
            .iter()
            .position(|state| {
                state.certified == 3 && state.effective == 0 && state.remaining == 129
            })
            .unwrap();
        assert!(
            rejected < restored && restored < rejected_again,
            "{states:?}"
        );
        assert_eq!(states[rejected].minimum, 300);
        assert_eq!(states[restored].minimum, 300);
        assert_eq!(states[rejected_again].minimum, 2);
        assert_eq!(states[restored].rebuild_reason, 0b100);
        assert_eq!(states[rejected_again].rebuild_reason, 0);
        assert!(states.iter().any(|state| state.rebuild_reason == 0b001));
        assert!(states.iter().any(|state| state.rebuild_reason == 0));
    }

    #[test]
    fn density_cache_prefix_exclusion_preserves_native_legacy_n4_n32_neighbors() {
        DENSITY_STATES.with(|states| *states.borrow_mut() = Some(Vec::new()));
        cached_global_prefix_preserves_equality_neighbors_and_large_threshold_jumps();
        let states = DENSITY_STATES.with(|states| states.borrow_mut().take().unwrap());
        assert!(!states.is_empty());
        assert!(states
            .iter()
            .all(|state| state.minimum == 2048 && state.remaining == 2048));
        // All real costs tie. The first live suffix rank is retained until
        // a numerically accepted global-prefix append actually excludes it.
        assert!(states.iter().any(|state| state.rebuild_reason == 0b010));
        assert!(states.iter().any(|state| state.rebuild_reason == 0));
        assert!(states.iter().any(|state| state.certified == 1));
        assert!(states.iter().any(|state| state.certified == 3));
        assert!(states
            .iter()
            .all(|state| state.winner_rank == state.certified));
    }

    #[test]
    fn density_cache_tied_nonwinner_eof_equal_rank_and_prefix_pass_are_real() {
        let norms = vec![1; 4096];
        let weak: Vec<_> = (0..800).map(|doc| (doc, 1)).collect();
        let winner: Vec<_> = (0..400).map(|i| (600 + 2 * i, 1)).collect();
        let tied: Vec<_> = (0..400).map(|doc| (doc, 1)).collect();
        let driver: Vec<_> = std::iter::once((0, 1))
            .chain((0..256).map(|i| (2000 + 7 * i, 1)))
            .collect();
        let last: Vec<_> = (0..128)
            .map(|i| (27 * i, 1))
            .chain(std::iter::once((4000, 1)))
            .collect();
        let scorers = vec![
            fixture(&weak, &norms, 0.001),
            fixture(&winner, &norms, 0.1),
            fixture(&tied, &norms, 0.2),
            fixture(&driver, &norms, 1.0),
            fixture(&last, &norms, 10.0),
        ];
        let costs: Vec<_> = scorers.iter().map(TermScorer::size_hint).collect();
        assert_eq!(costs, [800, 400, 400, 257, 129]);
        let maxima: Vec<_> = scorers.iter().map(TermScorer::max_score).collect();
        let mut ranks: Vec<_> = (0..5).collect();
        ranks.sort_unstable_by(|&left, &right| {
            (f64::from(maxima[left]) / f64::from(costs[left]))
                .total_cmp(&(f64::from(maxima[right]) / f64::from(costs[right])))
                .then_with(|| left.cmp(&right))
        });
        assert_eq!(ranks, [0, 1, 2, 3, 4]);
        let bound = crate::query::score_combiner::ScoreSumUpperBound::new(5);
        let prefix =
            |count| bound.score(maxima[..count].iter().map(|&score| f64::from(score)).sum());
        assert!(prefix(1) <= 0.01 && prefix(2) > 0.01);
        assert!(prefix(3) <= 0.8 && prefix(4) > 0.8);
        assert!(prefix(4) <= 3.0 && prefix(5) > 3.0);
        let mut end = scorers[4].clone();
        end.seek_block(0);
        assert_eq!(end.last_doc_in_block(), 3429);
        DENSITY_STATES.with(|states| *states.borrow_mut() = Some(Vec::new()));
        let (accepted, _, _) =
            check_scheduled_thresholds(scorers, 4096, 0.01, &[(3000, 0.8), (3700, 3.0)]);
        let states = DENSITY_STATES.with(|states| states.borrow_mut().take().unwrap());
        assert_eq!(accepted.last().unwrap().0, 4000);
        let nonwinner_eof = states
            .iter()
            .position(|state| {
                state.certified == 1
                    && state.winner_ordinal == Some(1)
                    && state.terminated & (1 << 2) != 0
            })
            .unwrap();
        assert_eq!(states[nonwinner_eof].rebuild_reason, 0);
        assert_eq!(states[nonwinner_eof].remaining, 400);
        assert_eq!(states[nonwinner_eof].terminated & (1 << 1), 0);
        let winner_eof = states
            .iter()
            .position(|state| {
                state.certified == 1
                    && state.winner_ordinal == Some(3)
                    && state.rebuild_reason == 0b100
            })
            .unwrap();
        assert!(nonwinner_eof < winner_eof);
        assert_eq!(states[winner_eof].lo, 2883);
        assert_eq!(states[winner_eof].remaining, 257);
        assert_ne!(states[winner_eof].terminated & (1 << 1), 0);
        let equal_rank = states
            .iter()
            .position(|state| {
                state.certified == 3 && state.winner_rank == 3 && state.rebuild_reason == 0
            })
            .unwrap();
        assert!(winner_eof < equal_rank);
        assert_eq!(states[equal_rank].lo, 3430);
        assert_eq!(states[equal_rank].minimum, 400);
        let excluded = states
            .iter()
            .position(|state| {
                state.certified == 4
                    && state.winner_ordinal == Some(4)
                    && state.rebuild_reason == 0b010
            })
            .unwrap();
        assert!(equal_rank < excluded);
        assert_eq!(states[excluded].lo, 3779);
        assert_eq!(states[excluded].minimum, 257);
        assert_eq!(states[excluded].remaining, 129);
        assert_eq!(
            states[excluded].terminated & (1 << 3),
            0,
            "prefix must exclude a still-live winner, not reuse EOF as the cause"
        );
    }

    #[test]
    fn density_cache_initial_exhausted_suffix_returns_exact_zero_once() {
        let norms = vec![1; 512];
        let weak: Vec<_> = (0..300).map(|doc| (doc, 1)).collect();
        let other: Vec<_> = (0..130).map(|doc| (doc, 1)).collect();
        let mut first = fixture(&other, &norms, 1.0);
        let mut second = fixture(&other, &norms, 2.0);
        while first.advance() != TERMINATED {}
        while second.advance() != TERMINATED {}
        DENSITY_STATES.with(|states| *states.borrow_mut() = Some(Vec::new()));
        let accepted = check(
            vec![fixture(&weak, &norms, 0.01), first, second],
            512,
            0.04,
            0,
        );
        let states = DENSITY_STATES.with(|states| states.borrow_mut().take().unwrap());
        assert!(accepted.is_empty());
        assert_eq!(states.len(), 1);
        let state = &states[0];
        assert_eq!(state.certified, 1);
        assert_eq!(state.effective, 1);
        assert_eq!(state.minimum, 300);
        assert_eq!(state.remaining, 0);
        assert_eq!(state.winner_rank, 3);
        assert_eq!(state.winner_ordinal, None);
        assert_eq!(state.rebuild_reason, 0b001);
        assert_eq!(state.terminated, 0b110);
        // With no live suffix and a globally certified prefix, every local
        // region is already certified. max_doc becomes its cap and ends this
        // query; a second empty-cache reuse witness would be unrealizable.
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
        let tiny = [(0, 1), (3999, 1)];
        let scorers = vec![
            fixture(&weak, &norms, 0.01),
            fixture(&high_cost, &norms, 0.02),
            fixture(&driver, &norms, 1.0),
            fixture(&tiny, &norms, 0.001),
        ];
        let (trace, states, choices) =
            check_scheduled_thresholds(scorers, 4096, 0.04, &[(3800, 0.07)]);
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
