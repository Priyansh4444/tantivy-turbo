use crate::query::score_combiner::{ScoreSumThresholdCutoff, SmallScoreSumUpperBound, SumCombiner};
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
thread_local! {
    // Test-only region evidence; no observer or counter exists in release.
    static GLOBAL_CHOICES: std::cell::RefCell<Option<Vec<RegionChoice>>> =
        const { std::cell::RefCell::new(None) };
    static GLOBAL_PREFIX_STATES: std::cell::RefCell<Option<Vec<GlobalPrefixState>>> =
        const { std::cell::RefCell::new(None) };
    static BATCH_LENGTHS: std::cell::RefCell<Option<Vec<usize>>> =
        const { std::cell::RefCell::new(None) };
    // Start every query on coarse regions so fixtures exercise that path.
    static FORCE_COARSE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    // Hand every coarse continuation back to fine regions after two regions.
    static FORCE_REVERT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static REVERTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Regions on the fine (every clause's block) geometry before a query may
/// switch to coarse covering bounds.
const COARSE_PROBE_REGIONS: u64 = 32;

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
    let mut optional_cutoff = ScoreSumThresholdCutoff::new(threshold);
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
    let mut window = None;
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
    // Fine regions follow every clause's block, as before. Once the observed
    // region bookkeeping (O(terms) per region) outweighs candidate work, the
    // rest of the segment continues on coarse covering-bound regions.
    let mut previous_essential = 0;
    let mut fine_regions = 0u64;
    let mut candidate_count = 0u64;
    // Covering bounds can instead make dense clauses essential. Coarse regions
    // hand the query back when that multiplies the fine candidate density.
    let first_lo = lo;
    let mut coarse_allowed = true;
    #[cfg(test)]
    let force_coarse = FORCE_COARSE.with(|force| force.get());
    #[cfg(not(test))]
    let force_coarse = false;

    while lo < max_doc {
        // Like Lucene's outer windows, target 16 candidates per clause per
        // region; fewer means region bookkeeping dominates.
        if coarse_allowed
            && (force_coarse
                || (fine_regions >= COARSE_PROBE_REGIONS
                    && candidate_count < fine_regions * 16 * num_terms as u64))
        {
            let fine_span = u64::from(lo - first_lo);
            let back = or_maxscore_coarse(
                CoarseContinuation {
                    scorers,
                    clauses,
                    prefix,
                    contributions,
                    batch,
                    window,
                    lo,
                    max_doc,
                    threshold,
                    optional_cutoff,
                    certified_global,
                    certified_global_sum,
                    #[cfg(test)]
                    global_appends,
                    previous_essential,
                    candidate_count,
                    upper_bound,
                    fine_span,
                },
                callback,
            );
            let Some(back) = back else {
                return;
            };
            CoarseContinuation {
                scorers,
                clauses,
                prefix,
                contributions,
                batch,
                window,
                lo,
                threshold,
                optional_cutoff,
                certified_global,
                certified_global_sum,
                previous_essential,
                candidate_count,
                ..
            } = back;
            #[cfg(test)]
            {
                global_appends = back.global_appends;
            }
            coarse_allowed = false;
            continue;
        }
        fine_regions += 1;
        debug_assert!(clauses
            .iter()
            .all(|clause| clause.doc == scorers[clause.ordinal].doc()));
        while certified_global < num_terms {
            let next_sum = certified_global_sum + f64::from(clauses[certified_global].global_max);
            // Exact cached form of `upper_bound.score(next_sum) > threshold`.
            debug_assert_eq!(
                upper_bound.score_is_at_most(next_sum, &optional_cutoff),
                upper_bound.score(next_sum) <= threshold
            );
            if !upper_bound.score_is_at_most(next_sum, &optional_cutoff) {
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
            && upper_bound.score_is_at_most(prefix[first_essential + 1], &optional_cutoff)
        {
            first_essential += 1;
        }
        previous_essential = first_essential;
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
        if num_terms - first_essential > 1
            && dense_essential_windows(&clauses[first_essential..], max_doc)
        {
            let mut state = Pruning {
                threshold,
                optional_cutoff,
                candidate_count,
                callback: &mut *callback,
            };
            score_essential_windows(
                &mut clauses,
                &mut scorers,
                first_essential,
                &prefix,
                hi,
                window.get_or_insert_with(EssentialWindow::new),
                &mut batch,
                &mut contributions,
                &upper_bound,
                &mut state,
            );
            threshold = state.threshold;
            optional_cutoff = state.optional_cutoff;
            candidate_count = state.candidate_count;
            debug_assert!(clauses
                .iter()
                .all(|clause| clause.doc == scorers[clause.ordinal].doc()));
            lo = hi;
            continue;
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
                    // Drop leaves the first optional test below rejects; the
                    // threshold only rises, so survivors are retested there.
                    let floor = upper_bound
                        .max_rejected_leaf(prefix[first_essential], &optional_cutoff);
                    let kept = batch.retain_scores_above(batch_len, floor);
                    candidate_count += (batch_len - kept) as u64;
                    batch_len = kept;
                    if batch_len == 0 {
                        continue;
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
            candidate_count += 1;
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
                let optional_sum = known + prefix[rank + 1];
                let rejected = upper_bound.score_is_at_most(optional_sum, &optional_cutoff);
                #[cfg(test)]
                debug_assert_eq!(rejected, upper_bound.score(optional_sum) <= threshold);
                if rejected {
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
                    optional_cutoff = ScoreSumThresholdCutoff::new(threshold);
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

/// Query state at the switch from fine to coarse regions, moved by value.
struct CoarseContinuation {
    scorers: Vec<TermScorer>,
    clauses: Vec<ClauseState>,
    prefix: Vec<f64>,
    contributions: Vec<Score>,
    batch: ScoredTermBatch,
    window: Option<EssentialWindow>,
    lo: DocId,
    max_doc: DocId,
    threshold: Score,
    optional_cutoff: ScoreSumThresholdCutoff,
    certified_global: usize,
    certified_global_sum: f64,
    #[cfg(test)]
    global_appends: usize,
    previous_essential: usize,
    candidate_count: u64,
    upper_bound: SmallScoreSumUpperBound,
    /// Doc IDs the fine regions covered, all yielding `candidate_count`.
    fine_span: u64,
}

/// The rest of `or_maxscore` on coarse regions, out of line so the fine loop
/// keeps its accepted code generation.
#[inline(never)]
fn or_maxscore_coarse(
    state: CoarseContinuation,
    callback: &mut dyn FnMut(DocId, Score) -> Score,
) -> Option<CoarseContinuation> {
    let CoarseContinuation {
        mut scorers,
        mut clauses,
        mut prefix,
        mut contributions,
        mut batch,
        mut window,
        mut lo,
        max_doc,
        mut threshold,
        mut optional_cutoff,
        mut certified_global,
        mut certified_global_sum,
        #[cfg(test)]
        mut global_appends,
        mut previous_essential,
        mut candidate_count,
        upper_bound,
        fine_span,
    } = state;
    let switch_lo = lo;
    let fine_candidates = candidate_count;
    let num_terms = scorers.len();
    let mut min_span = 1u32;
    let mut region_count = 0u64;
    while lo < max_doc {
        // Revert when coarse regions yield over twice the fine candidates per
        // doc ID. The fine span is nonempty after its probe regions.
        let coarse_span = u64::from(lo - switch_lo);
        #[cfg(test)]
        let force_revert = FORCE_REVERT.with(|force| force.get()) && region_count >= 2;
        #[cfg(not(test))]
        let force_revert = false;
        if force_revert
            || (region_count >= COARSE_PROBE_REGIONS
                && fine_span > 0
                && u128::from(candidate_count - fine_candidates) * u128::from(fine_span)
                    > 2 * u128::from(fine_candidates.max(1)) * u128::from(coarse_span))
        {
            #[cfg(test)]
            REVERTS.with(|reverts| reverts.set(reverts.get() + 1));
            return Some(CoarseContinuation {
                scorers,
                clauses,
                prefix,
                contributions,
                batch,
                window,
                lo,
                max_doc,
                threshold,
                optional_cutoff,
                certified_global,
                certified_global_sum,
                #[cfg(test)]
                global_appends,
                previous_essential,
                candidate_count,
                upper_bound,
                fine_span,
            });
        }
        debug_assert!(clauses
            .iter()
            .all(|clause| clause.doc == scorers[clause.ordinal].doc()));
        while certified_global < num_terms {
            let next_sum = certified_global_sum + f64::from(clauses[certified_global].global_max);
            // Exact cached form of `upper_bound.score(next_sum) > threshold`.
            debug_assert_eq!(
                upper_bound.score_is_at_most(next_sum, &optional_cutoff),
                upper_bound.score(next_sum) <= threshold
            );
            if !upper_bound.score_is_at_most(next_sum, &optional_cutoff) {
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
            return None;
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
        let (hi, first_essential) = coarse_region(
            &mut clauses,
            &mut scorers,
            &mut prefix,
            lo,
            max_doc,
            globally_optional,
            previous_essential,
            &mut min_span,
            &mut region_count,
            candidate_count,
            &upper_bound,
            &optional_cutoff,
        );
        previous_essential = first_essential;
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
        if num_terms - first_essential > 1
            && dense_essential_windows(&clauses[first_essential..], max_doc)
        {
            let mut state = Pruning {
                threshold,
                optional_cutoff,
                candidate_count,
                callback: &mut *callback,
            };
            score_essential_windows(
                &mut clauses,
                &mut scorers,
                first_essential,
                &prefix,
                hi,
                window.get_or_insert_with(EssentialWindow::new),
                &mut batch,
                &mut contributions,
                &upper_bound,
                &mut state,
            );
            threshold = state.threshold;
            optional_cutoff = state.optional_cutoff;
            candidate_count = state.candidate_count;
            debug_assert!(clauses
                .iter()
                .all(|clause| clause.doc == scorers[clause.ordinal].doc()));
            lo = hi;
            continue;
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
                    // Drop leaves the first optional test below rejects; the
                    // threshold only rises, so survivors are retested there.
                    let floor = upper_bound
                        .max_rejected_leaf(prefix[first_essential], &optional_cutoff);
                    let kept = batch.retain_scores_above(batch_len, floor);
                    candidate_count += (batch_len - kept) as u64;
                    batch_len = kept;
                    if batch_len == 0 {
                        continue;
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
            candidate_count += 1;
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
                let optional_sum = known + prefix[rank + 1];
                let rejected = upper_bound.score_is_at_most(optional_sum, &optional_cutoff);
                #[cfg(test)]
                debug_assert_eq!(rejected, upper_bound.score(optional_sum) <= threshold);
                if rejected {
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
                    optional_cutoff = ScoreSumThresholdCutoff::new(threshold);
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
    None
}

/// Coarse region setup, kept out of line so the fine (accepted) region loop
/// keeps its code layout. Returns the region end and the first essential rank.
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn coarse_region(
    clauses: &mut [ClauseState],
    scorers: &mut [TermScorer],
    prefix: &mut [f64],
    lo: DocId,
    max_doc: DocId,
    globally_optional: usize,
    previous_essential: usize,
    min_span: &mut u32,
    region_count: &mut u64,
    candidate_count: u64,
    upper_bound: &SmallScoreSumUpperBound,
    optional_cutoff: &ScoreSumThresholdCutoff,
) -> (DocId, usize) {
    let num_terms = clauses.len();
    // Only the previous essential leads end a region; optional clauses are
    // covered by one range bound instead of capping it with their blocks.
    let mut lead = previous_essential.min(num_terms - 1).max(globally_optional);
    let mut hi = max_doc;
    for clause in &clauses[lead..] {
        if clause.doc == TERMINATED {
            continue;
        }
        let scorer = &mut scorers[clause.ordinal];
        scorer.seek_block(lo.max(clause.doc));
        hi = hi.min(scorer.last_doc_in_block().min(max_doc - 1) + 1);
    }
    *region_count += 1;
    if num_terms - lead > 1 {
        if candidate_count < *region_count * 32 * num_terms as u64 {
            *min_span = (*min_span * 2).min(4096);
        } else {
            *min_span = 1;
        }
        hi = hi.max(lo.saturating_add(*min_span).min(max_doc));
    }
    // An optional clause's covering bound stays within one aligned group of
    // its blocks, so a sparse lead cannot loosen a dense clause's bound
    // across the segment, yet sparse clauses still allow wide regions.
    for clause in &clauses[globally_optional..lead] {
        if clause.doc == TERMINATED || clause.doc >= hi {
            continue;
        }
        let end = scorers[clause.ordinal].coarse_block_end(lo.max(clause.doc));
        hi = hi.min(end.min(max_doc - 1) + 1);
    }
    let mut first_essential_coarse;
    loop {
        debug_assert!(hi > lo);
        for (rank, clause) in clauses.iter_mut().enumerate() {
            // A real loaded position beyond the interval cannot contribute.
            // A stale position below lo does not justify a zero bound.
            clause.local_max = if clause.doc >= hi {
                0.0
            } else if rank < globally_optional {
                clause.global_max
            } else {
                scorers[clause.ordinal].range_max_score(lo, hi)
            };
            prefix[rank + 1] = prefix[rank] + f64::from(clause.local_max);
        }
        first_essential_coarse = 0;
        while first_essential_coarse < num_terms
            && upper_bound.score_is_at_most(prefix[first_essential_coarse + 1], optional_cutoff)
        {
            first_essential_coarse += 1;
        }
        if first_essential_coarse >= lead {
            break;
        }
        // Looser covering bounds promoted clauses to essential. Let their
        // current blocks end the region too, then repartition.
        let mut capped = hi;
        for clause in &clauses[first_essential_coarse..lead] {
            if clause.doc == TERMINATED || clause.doc >= hi {
                continue;
            }
            let scorer = &mut scorers[clause.ordinal];
            scorer.seek_block(lo.max(clause.doc));
            capped = capped.min(scorer.last_doc_in_block().min(max_doc - 1) + 1);
        }
        lead = first_essential_coarse;
        if capped >= hi {
            break;
        }
        hi = capped;
    }
    (hi, first_essential_coarse)
}

/// Doc IDs covered by one multiple-essential window, as in Lucene's
/// `MaxScoreBulkScorer`.
const ESSENTIAL_WINDOW: u32 = 4096;

/// Expected essential postings per window below which a doc-at-a-time merge
/// beats per-window batch setup and the bitmap scan.
const MIN_WINDOW_POSTINGS: u64 = 32;

/// Whether the essential clauses' average density fills a window enough to
/// amortize its fixed costs.
fn dense_essential_windows(essential: &[ClauseState], max_doc: DocId) -> bool {
    let postings: u64 = essential.iter().map(|clause| u64::from(clause.cost)).sum();
    postings * u64::from(ESSENTIAL_WINDOW) >= MIN_WINDOW_POSTINGS * u64::from(max_doc)
}

/// Reusable scratch for `score_essential_windows`. Between windows every
/// bit and sum is zero.
struct EssentialWindow {
    matches: Box<[u64]>,
    sums: Box<[f64]>,
    /// Each essential clause's (doc, leaf) postings in the window, in clause
    /// order; `ends[slot]` closes clause `slot`'s run.
    postings: Vec<(DocId, Score)>,
    ends: Vec<usize>,
    cursors: Vec<usize>,
}

impl EssentialWindow {
    fn new() -> Self {
        Self {
            matches: vec![0; ESSENTIAL_WINDOW as usize / 64].into_boxed_slice(),
            sums: vec![0.0; ESSENTIAL_WINDOW as usize].into_boxed_slice(),
            postings: Vec::new(),
            ends: Vec::new(),
            cursors: Vec::new(),
        }
    }
}

/// Mutable query state shared by the region loops and the window scorer.
struct Pruning<'a> {
    threshold: Score,
    optional_cutoff: ScoreSumThresholdCutoff,
    candidate_count: u64,
    callback: &'a mut dyn FnMut(DocId, Score) -> Score,
}

/// Scores a region with several essential clauses. Their postings are
/// batch-scored into doc windows: per doc the f64 sum accumulates in clause
/// order, exactly as a doc-at-a-time merge would. Optional clauses are then
/// probed per candidate in doc order with the same tests. Only candidates the
/// complete sum cannot reject recover their leaves for the canonical
/// ordinal-order sum that is published.
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn score_essential_windows(
    clauses: &mut [ClauseState],
    scorers: &mut [TermScorer],
    first_essential: usize,
    prefix: &[f64],
    hi: DocId,
    window: &mut EssentialWindow,
    batch: &mut ScoredTermBatch,
    contributions: &mut [Score],
    upper_bound: &SmallScoreSumUpperBound,
    state: &mut Pruning,
) {
    let num_essential = clauses.len() - first_essential;
    debug_assert!(num_essential > 1);
    let mut optional_hits = [(0usize, 0.0 as Score); 32];
    // `known <= first_floor` exactly when the first test below rejects.
    let mut first_floor =
        upper_bound.max_rejected_sum(prefix[first_essential], &state.optional_cutoff);
    loop {
        let start = clauses[first_essential..]
            .iter()
            .map(|clause| clause.doc)
            .min()
            .unwrap_or(TERMINATED);
        if start >= hi {
            break;
        }
        let end = hi.min(start.saturating_add(ESSENTIAL_WINDOW));
        window.postings.clear();
        window.ends.clear();
        for clause in &mut clauses[first_essential..] {
            while clause.doc < end {
                let len = clause.fill_scored_batch(scorers, start, end, batch);
                if len == 0 {
                    break;
                }
                for (&doc, &leaf) in batch.docs[..len].iter().zip(&batch.scores[..len]) {
                    let offset = (doc - start) as usize;
                    window.matches[offset / 64] |= 1u64 << (offset % 64);
                    window.sums[offset] += f64::from(leaf);
                }
                window.postings.extend(
                    batch.docs[..len]
                        .iter()
                        .copied()
                        .zip(batch.scores[..len].iter().copied()),
                );
            }
            window.ends.push(window.postings.len());
        }
        window.cursors.clear();
        window.cursors.push(0);
        window
            .cursors
            .extend_from_slice(&window.ends[..num_essential - 1]);
        let words = ((end - start) as usize).div_ceil(64);
        for word_index in 0..words {
            let mut word = std::mem::take(&mut window.matches[word_index]);
            while word != 0 {
                let offset = word_index * 64 + word.trailing_zeros() as usize;
                word &= word - 1;
                let doc = start + offset as DocId;
                let mut known = std::mem::take(&mut window.sums[offset]);
                state.candidate_count += 1;
                if known <= first_floor {
                    continue;
                }
                let mut num_hits = 0;
                let mut competitive = true;
                for rank in (0..first_essential).rev() {
                    let optional_sum = known + prefix[rank + 1];
                    let rejected =
                        upper_bound.score_is_at_most(optional_sum, &state.optional_cutoff);
                    #[cfg(test)]
                    debug_assert_eq!(rejected, upper_bound.score(optional_sum) <= state.threshold);
                    if rejected {
                        competitive = false;
                        break;
                    }
                    let clause = &mut clauses[rank];
                    if clause.doc < doc {
                        clause.seek(scorers, doc);
                    }
                    if clause.doc == doc {
                        clause.assert_coherent(scorers);
                        let leaf = scorers[clause.ordinal].score();
                        clause.assert_coherent(scorers);
                        optional_hits[num_hits] = (clause.ordinal, leaf);
                        num_hits += 1;
                        known += f64::from(leaf);
                    }
                }
                // The canonical sum is at most score(known); see the
                // optional tests above, here with no clause left to add.
                if !competitive || upper_bound.score_is_at_most(known, &state.optional_cutoff) {
                    continue;
                }
                contributions.fill(0.0);
                for (slot, clause) in clauses[first_essential..].iter().enumerate() {
                    let run = &window.postings[window.cursors[slot]..window.ends[slot]];
                    let position = run.partition_point(|&(posting, _)| posting < doc);
                    window.cursors[slot] += position;
                    if let Some(&(posting, leaf)) = run.get(position) {
                        if posting == doc {
                            contributions[clause.ordinal] = leaf;
                        }
                    }
                }
                for &(ordinal, leaf) in &optional_hits[..num_hits] {
                    contributions[ordinal] = leaf;
                }
                let score = contributions
                    .iter()
                    .map(|&leaf| f64::from(leaf))
                    .sum::<f64>() as Score;
                if score > state.threshold {
                    let next_threshold = (state.callback)(doc, score);
                    debug_assert!(next_threshold >= state.threshold);
                    state.threshold = next_threshold;
                    state.optional_cutoff = ScoreSumThresholdCutoff::new(state.threshold);
                    first_floor = upper_bound
                        .max_rejected_sum(prefix[first_essential], &state.optional_cutoff);
                }
            }
        }
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

    #[test]
    fn coarse_regions_match_exhaustive_union_on_every_fixture() {
        // The three region-trace tests assert fine geometry and are excluded.
        FORCE_COARSE.with(|force| force.set(true));
        single_driver_batches_replay_every_dynamic_threshold_exactly();
        exact_traces_at_full_block_and_tail_boundaries();
        deferred_optional_clause_becomes_essential_after_shallow_full_blocks();
        globally_optional_dense_terms_cover_sparse_essential_gaps_and_tails();
        threshold_growth_promotes_local_optionals_to_global_certificates();
        similar_density_terms_keep_local_certificates_until_both_are_optional();
        exact_midpoint_leaves_preserve_incoming_order_and_strict_ties();
        duplicates_disparate_boosts_exhaustion_and_sparse_regions();
        exact_growing_threshold_traces_for_mixed_dense_and_sparse_terms();
        unsafe_domains_use_exhaustive_canonical_sum_before_any_seek();
        FORCE_COARSE.with(|force| force.set(false));
    }

    #[test]
    fn reverted_coarse_regions_match_exhaustive_union_on_every_fixture() {
        FORCE_COARSE.with(|force| force.set(true));
        FORCE_REVERT.with(|force| force.set(true));
        REVERTS.with(|reverts| reverts.set(0));
        single_driver_batches_replay_every_dynamic_threshold_exactly();
        exact_traces_at_full_block_and_tail_boundaries();
        deferred_optional_clause_becomes_essential_after_shallow_full_blocks();
        globally_optional_dense_terms_cover_sparse_essential_gaps_and_tails();
        threshold_growth_promotes_local_optionals_to_global_certificates();
        similar_density_terms_keep_local_certificates_until_both_are_optional();
        exact_midpoint_leaves_preserve_incoming_order_and_strict_ties();
        duplicates_disparate_boosts_exhaustion_and_sparse_regions();
        exact_growing_threshold_traces_for_mixed_dense_and_sparse_terms();
        unsafe_domains_use_exhaustive_canonical_sum_before_any_seek();
        many_mixed_density_terms_match_exhaustive_union_in_both_geometries();
        assert!(REVERTS.with(|reverts| reverts.get()) > 0);
        FORCE_REVERT.with(|force| force.set(false));
        FORCE_COARSE.with(|force| force.set(false));
    }

    #[test]
    fn many_mixed_density_terms_match_exhaustive_union_in_both_geometries() {
        // Twelve terms from dense to sparse over many full blocks, with varied
        // frequencies and norms, so coarse groups, tails and shrinks all occur.
        let max_doc = 40_000;
        let mut state = 0x9E37_79B9u32;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state
        };
        let norms: Vec<u32> = (0..max_doc).map(|_| 1 + next() % 40).collect();
        let scorers: Vec<_> = (0..12u32)
            .map(|term| {
                let stride = [2, 3, 5, 9, 17, 33, 65, 129, 300, 700, 2000, 9000][term as usize];
                let mut postings = Vec::new();
                for doc in 0..max_doc {
                    if next() % stride == 0 {
                        postings.push((doc, 1 + next() % 6));
                    }
                }
                fixture(&postings, &norms, 0.5 + term as Score * 0.25)
            })
            .collect();
        for force in [false, true] {
            FORCE_COARSE.with(|cell| cell.set(force));
            for limit in [1, 10, 100] {
                check(scorers.clone(), max_doc, -Score::INFINITY, limit);
                check(scorers.iter().rev().cloned().collect(), max_doc, 0.0, limit);
            }
        }
        FORCE_COARSE.with(|cell| cell.set(false));
    }
}
