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

// A retained batch owns its pending docs even after the real cursor has moved
// beyond the fixed span. Only drained replay may select another driver.
struct SingleLeadSpan {
    rank: usize,
    up_to: DocId,
    len: usize,
    next: usize,
}

enum ReplaySource {
    Drained,
    SingleLead(SingleLeadSpan),
    ScalarMulti { doc: DocId },
}

#[derive(Debug, PartialEq)]
enum EssentialDispatch {
    Done,
    SingleLead { rank: usize, up_to: DocId },
    ScalarMulti { doc: DocId },
}

fn choose_essential_dispatch(
    clauses: &[ClauseState],
    first_essential: usize,
    hi: DocId,
) -> EssentialDispatch {
    let essentials = &clauses[first_essential..];
    if let [single] = essentials {
        return if single.doc < hi {
            EssentialDispatch::SingleLead {
                rank: first_essential,
                up_to: hi,
            }
        } else {
            EssentialDispatch::Done
        };
    }
    let mut first = TERMINATED;
    let mut second = TERMINATED;
    let mut lead = first_essential;
    for (offset, clause) in essentials.iter().enumerate() {
        if clause.doc < first {
            second = first;
            first = clause.doc;
            lead = first_essential + offset;
        } else if clause.doc < second {
            // Equality with first occupies the second minimum too: a tie
            // cannot certify a span with just one matching essential.
            second = clause.doc;
        }
    }
    if first >= hi {
        return EssentialDispatch::Done;
    }
    let up_to = hi.min(second);
    // Cap before subtracting. Physical width does not promise 128 postings;
    // this fixed trial cutoff reuses our existing decoded batch granularity.
    if first < up_to && up_to - first >= 128 {
        EssentialDispatch::SingleLead { rank: lead, up_to }
    } else {
        // Reuse the minimum from this pass; scalar replay must not scan again.
        EssentialDispatch::ScalarMulti { doc: first }
    }
}

#[cfg(test)]
#[derive(Debug, Default)]
struct LeadEvidence {
    dispatches: Vec<(usize, EssentialDispatch)>,
    fills: Vec<LeadFillEvidence>,
    candidates: Vec<LeadCandidateEvidence>,
}

#[cfg(test)]
#[derive(Debug)]
struct LeadFillEvidence {
    essentials: usize,
    rank: usize,
    ordinal: usize,
    up_to: DocId,
    docs: Vec<DocId>,
    leaves: Vec<Score>,
    unread: DocId,
}

#[cfg(test)]
#[derive(Debug)]
struct LeadCandidateEvidence {
    doc: DocId,
    lead: Option<(usize, DocId)>,
    ordinals: Vec<usize>,
    incoming: Vec<Score>,
    known: f64,
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
    static LEAD_EVIDENCE: std::cell::RefCell<Option<LeadEvidence>> =
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
        let mut replay = ReplaySource::Drained;
        'region: loop {
            let (doc, mut known) = loop {
                match &mut replay {
                    ReplaySource::Drained => {
                        let dispatch = choose_essential_dispatch(&clauses, first_essential, hi);
                        #[cfg(test)]
                        LEAD_EVIDENCE.with(|evidence| {
                            if let Some(evidence) = evidence.borrow_mut().as_mut() {
                                let observed = match &dispatch {
                                    EssentialDispatch::Done => EssentialDispatch::Done,
                                    EssentialDispatch::SingleLead { rank, up_to } => {
                                        EssentialDispatch::SingleLead {
                                            rank: *rank,
                                            up_to: *up_to,
                                        }
                                    }
                                    EssentialDispatch::ScalarMulti { doc } => {
                                        EssentialDispatch::ScalarMulti { doc: *doc }
                                    }
                                };
                                evidence
                                    .dispatches
                                    .push((num_terms - first_essential, observed));
                            }
                        });
                        replay = match dispatch {
                            EssentialDispatch::Done => break 'region,
                            EssentialDispatch::SingleLead { rank, up_to } => {
                                ReplaySource::SingleLead(SingleLeadSpan {
                                    rank,
                                    up_to,
                                    len: 0,
                                    next: 0,
                                })
                            }
                            EssentialDispatch::ScalarMulti { doc } => {
                                ReplaySource::ScalarMulti { doc }
                            }
                        };
                    }
                    ReplaySource::SingleLead(span) => {
                        debug_assert!(span.next <= span.len);
                        if span.next == span.len {
                            if clauses[span.rank].doc >= span.up_to {
                                replay = ReplaySource::Drained;
                                continue;
                            }
                            span.len = clauses[span.rank].fill_scored_batch(
                                &mut scorers,
                                lo,
                                span.up_to,
                                &mut batch,
                            );
                            span.next = 0;
                            #[cfg(test)]
                            BATCH_LENGTHS.with(|lengths| {
                                if let Some(lengths) = lengths.borrow_mut().as_mut() {
                                    lengths.push(span.len);
                                }
                            });
                            #[cfg(test)]
                            LEAD_EVIDENCE.with(|evidence| {
                                if let Some(evidence) = evidence.borrow_mut().as_mut() {
                                    evidence.fills.push(LeadFillEvidence {
                                        essentials: num_terms - first_essential,
                                        rank: span.rank,
                                        ordinal: clauses[span.rank].ordinal,
                                        up_to: span.up_to,
                                        docs: batch.docs[..span.len].to_vec(),
                                        leaves: batch.scores[..span.len].to_vec(),
                                        unread: clauses[span.rank].doc,
                                    });
                                }
                            });
                            if span.len == 0 {
                                debug_assert!(clauses[span.rank].doc >= span.up_to);
                                replay = ReplaySource::Drained;
                                continue;
                            }
                        }
                        let doc = batch.docs[span.next];
                        debug_assert!(lo <= doc && doc < span.up_to && span.up_to <= hi);
                        contributions.fill(0.0);
                        let leaf = batch.scores[span.next];
                        contributions[clauses[span.rank].ordinal] = leaf;
                        break (doc, f64::from(leaf));
                    }
                    ReplaySource::ScalarMulti { doc } => {
                        contributions.fill(0.0);
                        let mut known = 0.0f64;
                        for clause in &clauses[first_essential..] {
                            if clause.doc == *doc {
                                clause.assert_coherent(&scorers);
                                let leaf = scorers[clause.ordinal].score();
                                clause.assert_coherent(&scorers);
                                contributions[clause.ordinal] = leaf;
                                known += f64::from(leaf);
                            }
                        }
                        break (*doc, known);
                    }
                }
            };
            #[cfg(test)]
            LEAD_EVIDENCE.with(|evidence| {
                if let Some(evidence) = evidence.borrow_mut().as_mut() {
                    let lead = match &replay {
                        ReplaySource::SingleLead(span) => Some((span.rank, span.up_to)),
                        _ => None,
                    };
                    evidence.candidates.push(LeadCandidateEvidence {
                        doc,
                        lead,
                        ordinals: clauses[first_essential..]
                            .iter()
                            .map(|clause| clause.ordinal)
                            .collect(),
                        incoming: contributions.clone(),
                        known,
                    });
                }
            });
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
            match &mut replay {
                ReplaySource::SingleLead(span) => span.next += 1,
                ReplaySource::ScalarMulti { .. } => {
                    for clause in &mut clauses[first_essential..] {
                        if clause.doc == doc {
                            clause.advance(&mut scorers);
                        }
                    }
                    replay = ReplaySource::Drained;
                }
                ReplaySource::Drained => unreachable!("candidate owns its retirement"),
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

    fn observed_lead_check(
        scorers: Vec<TermScorer>,
        max_doc: DocId,
        threshold: Score,
        limit: usize,
    ) -> (Vec<(DocId, u32)>, LeadEvidence) {
        LEAD_EVIDENCE.with(|evidence| *evidence.borrow_mut() = Some(LeadEvidence::default()));
        let accepted = check(scorers.clone(), max_doc, threshold, limit);
        let evidence = LEAD_EVIDENCE.with(|evidence| evidence.borrow_mut().take().unwrap());
        assert_lead_evidence(&evidence, &scorers);
        (accepted, evidence)
    }

    fn scalar_leaf(scorer: &TermScorer, doc: DocId) -> Score {
        let mut original = scorer.clone();
        // Forward-only seekers cannot recover an original first doc after the
        // candidate. Such a clause is absent there, not a reason to seek back.
        if original.doc() > doc {
            return 0.0;
        }
        if original.seek(doc) == doc {
            original.score()
        } else {
            0.0
        }
    }

    fn assert_lead_evidence(evidence: &LeadEvidence, scorers: &[TermScorer]) {
        assert!(evidence
            .candidates
            .windows(2)
            .all(|pair| pair[0].doc < pair[1].doc));
        for fill in &evidence.fills {
            assert!(fill.rank < scorers.len());
            assert!(fill.docs.len() <= 128);
            assert_eq!(fill.docs.len(), fill.leaves.len());
            assert!(fill.docs.windows(2).all(|pair| pair[0] < pair[1]));
            assert!(fill
                .docs
                .iter()
                .all(|&doc| doc < fill.up_to && doc < fill.unread));
            if fill.docs.is_empty() {
                assert!(fill.unread >= fill.up_to);
            }
            for (&doc, &leaf) in fill.docs.iter().zip(&fill.leaves) {
                assert_eq!(
                    leaf.to_bits(),
                    scalar_leaf(&scorers[fill.ordinal], doc).to_bits()
                );
            }
        }
        for candidate in &evidence.candidates {
            let expected: Vec<_> = scorers
                .iter()
                .enumerate()
                .map(|(ordinal, scorer)| {
                    if candidate.ordinals.contains(&ordinal) {
                        scalar_leaf(scorer, candidate.doc)
                    } else {
                        0.0
                    }
                })
                .collect();
            assert_eq!(
                candidate
                    .incoming
                    .iter()
                    .map(|leaf| leaf.to_bits())
                    .collect::<Vec<_>>(),
                expected
                    .iter()
                    .map(|leaf| leaf.to_bits())
                    .collect::<Vec<_>>()
            );
            let known = candidate
                .ordinals
                .iter()
                .map(|&ordinal| f64::from(expected[ordinal]))
                .sum::<f64>();
            assert_eq!(candidate.known.to_bits(), known.to_bits());
            if let Some((rank, up_to)) = candidate.lead {
                assert!(candidate.doc < up_to);
                // Rank is a preference position, not an incoming ordinal.
                assert!(rank < scorers.len());
                assert_eq!(expected.iter().filter(|&&leaf| leaf != 0.0).count(), 1);
            }
        }
    }

    fn metadata(docs: &[DocId]) -> Vec<ClauseState> {
        docs.iter()
            .enumerate()
            .map(|(ordinal, &doc)| ClauseState {
                ordinal,
                global_max: 1.0,
                cost: 1,
                doc,
                local_max: 1.0,
            })
            .collect()
    }

    #[test]
    fn temporary_lead_capped_cutoff_ties_and_sentinel_classification() {
        for width in [127, 128, 129] {
            let clauses = metadata(&[10, 10 + width, 900]);
            let expected = if width < 128 {
                EssentialDispatch::ScalarMulti { doc: 10 }
            } else {
                EssentialDispatch::SingleLead {
                    rank: 0,
                    up_to: 10 + width,
                }
            };
            assert_eq!(choose_essential_dispatch(&clauses, 0, 1000), expected);
        }
        assert_eq!(
            choose_essential_dispatch(&metadata(&[10, 1000, 2000]), 0, 137),
            EssentialDispatch::ScalarMulti { doc: 10 }
        );
        assert_eq!(
            choose_essential_dispatch(&metadata(&[10, 10, 1000]), 0, 2000),
            EssentialDispatch::ScalarMulti { doc: 10 }
        );
        assert_eq!(
            choose_essential_dispatch(&metadata(&[TERMINATED, 10, TERMINATED]), 0, 138),
            EssentialDispatch::SingleLead {
                rank: 1,
                up_to: 138
            }
        );
        assert_eq!(
            choose_essential_dispatch(&metadata(&[TERMINATED; 3]), 0, TERMINATED),
            EssentialDispatch::Done
        );
        assert_eq!(
            choose_essential_dispatch(&metadata(&[0, 0, 10]), 2, 11),
            EssentialDispatch::SingleLead { rank: 2, up_to: 11 }
        );
        assert_eq!(
            choose_essential_dispatch(
                &metadata(&[TERMINATED - 128, TERMINATED, TERMINATED]),
                0,
                TERMINATED
            ),
            EssentialDispatch::SingleLead {
                rank: 0,
                up_to: TERMINATED
            }
        );
    }

    #[test]
    fn temporary_lead_explicit_runner_boundary_retains_exact_block_and_tail_rows() {
        let norms = vec![1; 1024];
        for length in [127, 128, 129] {
            let runner = length * 2;
            let lead: Vec<_> = (0..length)
                .map(|i| (i * 2, 1 + i % 5))
                .chain([(runner, 11), (900, 2)])
                .collect();
            let others = [(runner, 7), (800, 3)];
            for hi in [runner, runner + 1] {
                let mut scorers = vec![
                    fixture(&lead, &norms, 1.0),
                    fixture(&others, &norms, 0.5),
                    fixture(&others, &norms, 0.25),
                ];
                let mut clauses = metadata(&[0, runner, runner]);
                let dispatch = choose_essential_dispatch(&clauses, 0, hi);
                assert_eq!(
                    dispatch,
                    EssentialDispatch::SingleLead {
                        rank: 0,
                        up_to: runner
                    }
                );
                let mut batch = ScoredTermBatch::new();
                let mut retained = Vec::new();
                while clauses[0].doc < runner {
                    let len = clauses[0].fill_scored_batch(&mut scorers, 0, runner, &mut batch);
                    assert!(len > 0);
                    assert!(clauses[0].doc >= runner || clauses[0].doc > batch.docs[len - 1]);
                    for i in 0..len {
                        let doc = batch.docs[i];
                        assert!(doc < runner);
                        assert_eq!(
                            batch.scores[i].to_bits(),
                            scalar_leaf(&fixture(&lead, &norms, 1.0), doc).to_bits()
                        );
                        retained.push(doc);
                    }
                }
                assert_eq!(retained, (0..length).map(|i| i * 2).collect::<Vec<_>>());
                assert_eq!(
                    clauses.iter().map(|clause| clause.doc).collect::<Vec<_>>(),
                    [runner; 3]
                );
                let next = choose_essential_dispatch(&clauses, 0, hi);
                assert_eq!(
                    next,
                    if hi == runner {
                        EssentialDispatch::Done
                    } else {
                        EssentialDispatch::ScalarMulti { doc: runner }
                    }
                );
                // A separate full-query oracle covers tie coalescing, future
                // postings and actual numerical-region transitions.
                check(
                    vec![
                        fixture(&lead, &norms, 1.0),
                        fixture(&others, &norms, 0.5),
                        fixture(&others, &norms, 0.25),
                    ],
                    1024,
                    -Score::INFINITY,
                    0,
                );
            }
        }
    }

    #[test]
    fn temporary_lead_pending_cursor_exhaustion_replays_later_winner() {
        let norms = vec![1; 512];
        let lead: Vec<_> = (0..127)
            .map(|i| (i * 2, if i == 100 { 30 } else { 1 }))
            .collect();
        let mut exhausted = fixture(&[(450, 1)], &norms, 0.2);
        assert_eq!(exhausted.advance(), TERMINATED);
        let scorers = vec![fixture(&lead, &norms, 1.0), exhausted.clone(), exhausted];
        let (accepted, evidence) = observed_lead_check(scorers.clone(), 512, -Score::INFINITY, 1);
        assert_eq!(
            accepted.iter().map(|&(doc, _)| doc).collect::<Vec<_>>(),
            [0, 200]
        );
        assert!(evidence.fills.iter().any(|fill| fill.essentials >= 2
            && fill.docs.len() == 127
            && fill.unread == TERMINATED));
        assert_eq!(evidence.candidates.len(), 127);
        let mut at_capacity = vec![scorers[0].clone()];
        at_capacity.extend(std::iter::repeat_n(scorers[1].clone(), 31));
        let (capacity_trace, capacity_evidence) =
            observed_lead_check(at_capacity, 512, -Score::INFINITY, 1);
        assert_eq!(capacity_trace, accepted);
        assert!(capacity_evidence
            .fills
            .iter()
            .any(|fill| fill.essentials == 32
                && fill.docs.len() == 127
                && fill.unread == TERMINATED));
        let run = |optimized: bool| {
            let mut accepted = Vec::new();
            let mut current = -Score::INFINITY;
            let mut callback = |doc, score: Score| {
                accepted.push((doc, score.to_bits()));
                if doc != 0 {
                    current = current.max(score);
                }
                current
            };
            if optimized {
                or_maxscore(scorers.clone(), 512, -Score::INFINITY, &mut callback);
            } else {
                let mut union =
                    BufferedUnionScorer::build(scorers.clone(), SumCombiner::default, 512);
                for_each_pruning_scorer(&mut union, -Score::INFINITY, &mut callback);
            }
            accepted
        };
        assert_eq!(run(true), run(false));
        assert_eq!(
            run(true).iter().map(|&(doc, _)| doc).collect::<Vec<_>>(),
            [0, 2, 200]
        );
        eprintln!(
            "OR_LEAD_LAYOUT span={} replay={} batch={} no_additional_heap_allocation",
            std::mem::size_of::<SingleLeadSpan>(),
            std::mem::size_of::<ReplaySource>(),
            std::mem::size_of::<ScoredTermBatch>()
        );
    }

    #[test]
    fn temporary_lead_scalar_tie_different_lead_and_dirty_optional_roles() {
        let norms = vec![1; 1024];
        let a = [(0, 1), (2, 3), (300, 5), (900, 11)];
        let b = [(300, 7), (302, 2), (304, 3), (700, 5)];
        let c = [(300, 3), (700, 2), (900, 9)];
        let weak: Vec<_> = (0..1024)
            .filter(|doc| doc % 3 != 0)
            .map(|doc| (doc, 1))
            .collect();
        let scorers = vec![
            fixture(&a, &norms, 1.0),
            fixture(&weak, &norms, 0.001),
            fixture(&b, &norms, 1.2),
            fixture(&c, &norms, 0.9),
        ];
        let (_, evidence) = observed_lead_check(scorers.clone(), 1024, 0.01, 0);
        assert!(evidence.dispatches.iter().any(|&(n, ref choice)| n >= 2
            && matches!(choice, EssentialDispatch::SingleLead { up_to: 300, .. })));
        assert!(evidence
            .dispatches
            .iter()
            .any(|&(_, ref choice)| matches!(choice, EssentialDispatch::ScalarMulti { doc: 300 })));
        let first = evidence
            .candidates
            .iter()
            .find(|candidate| candidate.doc == 0)
            .unwrap()
            .lead
            .unwrap()
            .0;
        let next = evidence
            .candidates
            .iter()
            .find(|candidate| candidate.doc == 302)
            .unwrap()
            .lead
            .unwrap()
            .0;
        assert_ne!(first, next);
        for threshold in [-Score::INFINITY, 0.01, 0.5, 2.0] {
            for limit in [0, 1, 10] {
                check(scorers.clone(), 1024, threshold, limit);
            }
        }
        // Existing role-change fixtures remain in the file. This corpus adds
        // repeated lead/scalar alternation under different incoming orders.
        check(scorers.iter().rev().cloned().collect(), 1024, 0.01, 1);
    }

    #[test]
    fn temporary_multi_lead_midpoint_optional_leaves_in_native_and_legacy_profiles() {
        let norms = vec![1; 512];
        let lead = [(0, 1), (2, 1), (200, 1000)];
        let runner = [(300, 1000), (400, 1)];
        let tiny = [(0, 1), (2, 1), (200, 1), (300, 1), (400, 1)];
        let boosts = [
            1.0,
            1.0,
            2.0f32.powi(-24),
            2.0f32.powi(-53),
            2.0f32.powi(-53),
        ];
        for native in [false, true] {
            let make = |postings: &[(DocId, u32)], boost| {
                if native {
                    TermScorer::create_for_test(
                        postings,
                        &norms,
                        // With norm1/TF1, DEFAULT's literal subtraction
                        // gives an exact unit leaf for weight2.2. Power-of-two
                        // boosts retain the same midpoint leaves as legacy.
                        Bm25Weight::for_native_block_bounds(1.0).boost_by(2.2 * boost),
                    )
                } else {
                    fixture(postings, &norms, boost)
                }
            };
            let scorers = vec![
                make(&lead, boosts[0]),
                make(&runner, boosts[1]),
                make(&tiny, boosts[2]),
                make(&tiny, boosts[3]),
                make(&tiny, boosts[4]),
            ];
            for (ordinal, &leaf) in boosts.iter().enumerate() {
                let doc = if ordinal == 1 { 400 } else { 0 };
                assert_eq!(
                    scalar_leaf(&scorers[ordinal], doc).to_bits(),
                    leaf.to_bits(),
                    "native={native} ordinal={ordinal}"
                );
            }
            for order in [[0, 1, 2, 3, 4], [3, 4, 2, 0, 1], [2, 0, 1, 4, 3]] {
                let ordered: Vec<_> = order.iter().map(|&i| scorers[i].clone()).collect();
                let expected = ordered
                    .iter()
                    .map(|scorer| f64::from(scalar_leaf(scorer, 0)))
                    .sum::<f64>() as Score;
                // The high-TF later lead keeps its local bound above every
                // tested threshold, while the runner remains another essential.
                assert!(scalar_leaf(&scorers[0], 200) > expected.next_up());
                for threshold in [expected.next_down(), expected, expected.next_up()] {
                    for limit in [0, 1, 10] {
                        let (accepted, evidence) =
                            observed_lead_check(ordered.clone(), 512, threshold, limit);
                        assert!(
                            evidence
                                .fills
                                .iter()
                                .any(|fill| fill.essentials >= 2 && fill.docs.contains(&0)),
                            "native={native} order={order:?} threshold={threshold:?} \
                             evidence={evidence:?}"
                        );
                        let first = accepted.iter().find(|&&(doc, _)| doc == 0);
                        if threshold < expected {
                            assert_eq!(first.unwrap().1, expected.to_bits());
                        } else {
                            assert!(first.is_none());
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn temporary_lead_actual_indexed_deletes_keep_physical_pending_rows() -> crate::Result<()> {
        use crate::collector::TopDocs;
        use crate::indexer::NoMergePolicy;
        use crate::query::{BooleanQuery, BoostQuery, EnableScoring, Query, TermQuery};
        use crate::schema::{IndexRecordOption, Schema, TEXT};
        use crate::{DocAddress, Index, Term};
        let mut schema = Schema::builder();
        let text = schema.add_text_field("text", TEXT);
        let marker = schema.add_text_field("marker", TEXT);
        let index = Index::create_in_ram(schema.build());
        let mut writer = index.writer_for_tests()?;
        writer.set_merge_policy(Box::new(NoMergePolicy));
        for doc in 0..400 {
            let (term, frequency) = if doc < 128 {
                (
                    "alpha",
                    if [0, 63].contains(&doc) {
                        30
                    } else if doc == 127 {
                        25
                    } else {
                        1
                    },
                )
            } else if doc >= 256 {
                ("beta gamma", 1)
            } else {
                ("filler", 1)
            };
            let used = if doc >= 256 { 2 * frequency } else { frequency };
            let content = std::iter::repeat_n(term, frequency)
                .chain(std::iter::repeat_n("filler", 90 - used))
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
        assert_eq!(segment.max_doc(), 400);
        assert_eq!(segment.num_docs(), 398);
        let queries: Vec<_> = ["alpha", "beta", "gamma"]
            .iter()
            .map(|term| {
                TermQuery::new(
                    Term::from_field_text(text, term),
                    IndexRecordOption::WithFreqs,
                )
            })
            .collect();
        let boosts = [4.0, 0.001, 0.002];
        let scorers: Vec<_> = queries
            .iter()
            .zip(boosts)
            .map(|(query, boost)| {
                query
                    .specialized_weight(EnableScoring::enabled_from_searcher(&searcher))?
                    .term_scorer_for_test(segment, boost)?
                    .ok_or_else(|| crate::TantivyError::InvalidArgument("missing test term".into()))
            })
            .collect::<crate::Result<_>>()?;
        let run = |optimized: bool| {
            let mut accepted = Vec::new();
            let mut current = -Score::INFINITY;
            let mut callback = |doc, score: Score| {
                accepted.push((doc, score.to_bits()));
                if segment.alive_bitset().unwrap().is_alive(doc) {
                    current = current.max(score);
                }
                current
            };
            if optimized {
                or_maxscore(scorers.clone(), 400, -Score::INFINITY, &mut callback);
            } else {
                let mut union =
                    BufferedUnionScorer::build(scorers.clone(), SumCombiner::default, 400);
                for_each_pruning_scorer(&mut union, -Score::INFINITY, &mut callback);
            }
            accepted
        };
        LEAD_EVIDENCE.with(|evidence| *evidence.borrow_mut() = Some(LeadEvidence::default()));
        let accepted = run(true);
        let evidence = LEAD_EVIDENCE.with(|evidence| evidence.borrow_mut().take().unwrap());
        assert_eq!(accepted, run(false));
        assert_lead_evidence(&evidence, &scorers);
        assert!(accepted.iter().any(|&(doc, _)| doc == 0));
        assert!(accepted.iter().any(|&(doc, _)| doc == 63));
        assert!(accepted.iter().any(|&(doc, _)| doc == 127));
        assert!(evidence.fills.iter().any(|fill| fill.essentials >= 2
            && fill.docs.len() == 128
            && fill.docs.contains(&63)
            && fill.unread >= fill.up_to));
        let query = BooleanQuery::union(
            queries
                .into_iter()
                .zip(boosts)
                .map(|(query, boost)| {
                    Box::new(BoostQuery::new(Box::new(query), boost)) as Box<dyn Query>
                })
                .collect(),
        );
        let top = searcher.search(&query, &TopDocs::with_limit(1))?;
        assert_eq!(top[0].1, DocAddress::new(0, 127));
        let winner = accepted.iter().find(|&&(doc, _)| doc == 127).unwrap().1;
        assert_eq!(top[0].0.to_bits(), winner);
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
