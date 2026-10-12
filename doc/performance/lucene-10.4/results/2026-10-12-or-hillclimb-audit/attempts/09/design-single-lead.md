# Mechanism09 candidate: drain a certified single-lead span

Ground and Sketch only. Production sources unchanged; no build, test, profile,
latency, or compression job run by this designer.

Update from root:08 passes its19 fixtures and three1227 exact gates, but its
pilot is12.79%/13.55% slower in opposite orders. Root rejects08; mechanism09's
implementation parent is accepted06. Bitmap appears below only as comparative
context. Its storage and gather code are not part of this implementation sketch.

throughput checkpoint: n/a, read-only investigation

## Recommendation and caller usage

This is a plausible, bounded next experiment. A suffix can remain certified
essential while only its earliest clause has any postings in an initial physical
span. Process that span with the existing exact single-term batch reader, retaining
the same optional prefix, numerical certificate, and candidate completion body.
Do not make the other essential clauses optional and do not repartition.

At a fully drained replay boundary inside `[lo, hi)`:

1. Read the two smallest cursor docs in the existing essential suffix, without
   changing its preference order. A tie occupies both first and second positions.
2. For a unique earliest clause, let `up_to = min(hi, second_doc)`. If there is
   only one essential clause, use `hi`, preserving accepted03 behavior.
3. For a suffix of at least two, select single-lead replay only if the *capped*
   span `up_to - first_doc >= 2048`. Otherwise use accepted06's existing
   multi-essential route. The subtraction is performed only after `first_doc <
   up_to`; document IDs stay unsigned.
4. Drain that lead through repeated existing batches below `up_to`. Complete
   every retained row against the current callback threshold before another fill,
   selection, gather, or region transition.
5. Once the lead reaches `up_to`, or a zero fill reconciles it beyond the span,
   return to the drained dispatch boundary. Other essential cursors stayed fixed.

Conceptual caller flow, not implementation:

```text
establish accepted region certificate and first_essential
reconcile essential cursors below lo, exactly as today
replay = Drained
while region still has candidates:
    if replay is Drained:
        choose single lead, existing multi route, or region done
    obtain ONE candidate from the active replay source
    run EXISTING optional probes, original-order sum, strict callback
    retire this candidate from its replay source
    only now may a source become Drained or the region finish
assert no retained candidates
lo = hi
```

The 2048 cutoff is an initial fixed choice grounded in the port, not a measured
optimum. Capping the gap before testing worthiness avoids choosing a long gap
whose numerical region has only a few physical addresses left. That is a small,
explicit difference from the port's uncapped eligibility test; test this boundary.
No new density signal or tunable selector is part of this experiment.

## Grounding: what the actual sources do

Accepted06 is `a674aaf2f6187f3010a135dc25fdacf9cab27e53` in
`search/tantivy-pr2937`. Active08 is the dirty working
`search/tantivy-or-hillclimb-oct07` source on that HEAD; it is not an accepted
dependency. Root supplied the current measurements and acceptance status; this
design makes no new performance claim.

* The fixed port's `INNER_WINDOW_SIZE` is 4096. Its
  `bulk.rs::score_inner_window` at lines 223–233 calls the single-clause route
  when the heap has no second essential, or when the second essential doc is at
  least 2048 beyond the first. It caps `up_to` at the outer boundary and second
  cursor. Lines 236–252 repeatedly batch the one driver, score nonessentials,
  update its mirror, and repair the heap. The competing multi route at lines
  255–299 gathers a 4096-address bitmap/sum window. We need no heap to borrow
  the physical-span observation.
* Accepted06 `or_maxscore.rs` lines 236–275 reconciles essentials and batches
  only when `num_terms - first_essential == 1`. Otherwise it scans the full
  essential suffix for every candidate minimum. Lines 281–295 score matching
  essentials; lines 325–331 advance them. A temporary single lead can bypass
  all three suffix scans while retaining the exact same certified partition.
* The same file lines 297–323 probes optionals in reverse preference order,
  then folds original incoming leaves in f64 and casts once to `Score`, calling
  the callback only for `score > threshold`. This entire completion body stays
  common. Port `score_non_essential_clauses` at lines 341–368 has batch filtering
  and a different accumulation structure; it is not the arithmetic template.
* `ClauseState::fill_scored_batch` updates `doc` from the real scorer after
  reading. `term_scorer.rs` lines 13–33 and 51–69 define the existing fixed
  128-posting scratch and scoring fill. `segment_postings.rs` lines 25–75 owns
  `cur`, reconciles shallow selection, copies one decoded block suffix below
  the exclusive cap, and leaves the first unread doc. This is the ownership
  boundary to reuse, rather than adding cursor operations to the OR dispatcher.
* `bm25.rs` lines 575–602 gathers normalization components then evaluates the
  literal scalar leaf expression, including native Lucene BM25. Keep this API;
  never score a retained doc via a cursor that has already advanced.
* Active08 `ExactLeafWindow::gather_window` lines 74–146 selects a minimum,
  caps a 128-*physical-address* window, batches matching essentials, and scatters
  exact leaves into incoming-order rows. `pop_candidate` lines 149–164 owns
  ascending replay. Its main loop explicitly drains rows even when all actual
  cursors have reached `hi` or `TERMINATED`. Mechanism09 must preserve that rule.

Source references above are local paths beneath
`/home/pronsh/Coding/playground/search/`; line numbers describe the inspected
accepted source and active08 snapshot, not future edits.

## Private data shape, signatures, and module map

Keep `ClauseState`, `ScoredTermBatch`, and original `contributions`. No fields are added to the
per-clause record. No heap, dynamic document order, allocation per region,
alternative score representation, or index data is needed.

The smallest useful new metadata is a short-lived lead span plus one exhaustive
replay owner, replacing the current boolean/index state rather than adding a
parallel collection of flags:

```rust
// Private to or_maxscore.rs; illustrative signatures, not compiled code.
struct SingleLeadSpan {
    rank: usize,       // index in existing clauses, not incoming ordinal
    up_to: DocId,      // exclusive; fixed for this span, <= region hi
    len: usize,        // currently retained batch length, <= 128
    next: usize,       // unretired row; next <= len
}

enum ReplaySource {
    Drained,
    SingleLead(SingleLeadSpan),
    ScalarMulti { doc: DocId },
}

enum EssentialDispatch {
    Done,
    SingleLead { rank: usize, up_to: DocId },
    ScalarMulti { doc: DocId },
}

fn choose_essential_dispatch(
    clauses: &[ClauseState], first_essential: usize, hi: DocId,
) -> EssentialDispatch { /* not implemented: one O(e) two-min pass */ }
```

`ScalarMulti` owns the selected scalar candidate doc. Complete it and advance
every matching essential cursor before returning to `Drained`. Carrying the doc
computed by the two-min scan avoids repeating accepted06's minimum scan. No
retained matrix is introduced.

`SingleLead` is the exclusive owner of `batch.docs[next..len]` and corresponding
scores. The query-level `ScoredTermBatch` remains the storage. A `next == len`
span may refill the *same* lead below the same
cap; it does not authorize redispatch. Only reaching the cap/exhaustion with all
rows retired ends the span. These invariants delete the dangerous shortcut
"all cursors beyond hi implies region done."

Do not add a public iterator or generic framework. Keep selection private and
the refill/replay branch beside the current shared optional/callback body in
`or_maxscore.rs`. Reader and scorer APIs remain unchanged. Avoid extracting the
common completion body into a helper requiring the whole query state merely to
save lines: that would expose more state than it hides.

Performing a two-min pass on every rejected scalar dispatch could add work to
dense regions. Obtain the current minimum from that same pass and carry it in
`ScalarMulti`, so fallback does not also repeat its existing minimum scan. This
is reuse of the selection result, not a second candidate-generation mechanism.

## Boundaries, progress, and exactness proof

**Certificate stays fixed.** Let `E = clauses[first_essential..]` and let the
optional prefix bound certify that optional-only documents cannot exceed the
threshold used to form the region. Callback thresholds are monotonic, as in the
existing contract. Keeping that certificate for later, higher thresholds is
conservative. No optional clause is dropped and no essential is reclassified.

**Unique lead span.** Let `d1` be the lead's first unread doc and `d2` the smallest
first unread doc among all other essentials. For any `d < min(hi, d2)`, every
other essential cursor is strictly beyond `d`; sorted postings mean those clauses
cannot match there. Candidate docs are therefore precisely the lead's postings
in that half-open span. A two-way first-doc tie gives `d1 == d2`, rejects the
single-lead route, and is handled by the existing multi route. Equal later docs
at `d2` are excluded from all lead batches and replayed with all their matching
essential leaves by the next multi dispatch.

**Optional scoring is unchanged.** Clear `contributions`, store each retained
lead f32 leaf at `clauses[rank].ordinal`, and start `known = f64::from(leaf)`.
Other essentials contribute zero in this span, so this equals existing scalar
or matrix known accumulation. Probe every original optional rank through the
same bound/seek/score loop using the current threshold. Fold all incoming-order
slots in f64, cast once to f32, and test strict `>` immediately before callback.
Preserve midpoint rounding and tie behavior; never publish the pruning-known
sum instead of the original-order fold.

**Cursor and retained-row ownership are separate.** A successful fill may move
the lead to the next block, cap, or `TERMINATED` while 128 rows remain. Those
rows still complete in ascending order. Do not call `advance()` for each retained
row: the batch reader already consumed it. Optional seeks can advance only
optional cursors, not another essential driver. With no concurrent driver
mutations, the second-doc cap remains valid throughout the span.

**Progress.** First require `d1 < hi`. For selected spans `up_to > d1`. Each
nonempty fill retains at least one candidate and consumes a decoded suffix.
Drain those rows, then refill only if the lead is still below the cap. A zero
fill means the reader reconciled to a doc at/above the cap or termination; end
the span and recompute the lead instead of repeatedly issuing the same empty
fill. A reconciliation that jumps beyond the second cursor emits nothing and
safely lets the next dispatch select the new minimum. Cursor assertions remain
with `ClauseState` mutations. Never expire the numerical region on an empty
batch unless *all* essentials are beyond `hi` and no rows remain.

**Exhaustion and sentinel.** If every essential is terminated/outside `hi`,
dispatch `Done` only from `Drained`. If one remains live, other cursors may give
`d2 = TERMINATED`; `up_to = hi`, with the same cutoff for the multi-essential
case. `DocId` is u32; cap before subtraction, never add the sentinel, and retain
the accepted physical `max_doc`/block-end rules. Two exhausted cursor ties do
not create a span because `d1 >= hi` is checked first.

**Signed/exceptional domains and deletion.** The existing admission check runs
before any cursor mutation and rejects unsafe/signed leaf domains to independent
canonical union. Mechanism09 is entirely downstream of it. Negative initial
thresholds with admitted nonnegative leaves retain the same contract. Do not
copy the port's signed i32 gap expression or its inclusive collector conventions.
Physical document addresses include deleted docs; callback-controlled rejection
keeps the current threshold unchanged, exactly as accepted03 fixtures model.

## Tradeoffs against simply keeping08 bitmap windows

| Property | Single-lead span | Existing08 bitmap route |
| --- | --- | --- |
| Selection | One O(e) two-min scan per span, then fixed driver refills | O(e) minimum per 128 physical addresses |
| Sparse lead with large physical gaps | Up to128 postings per reader fill regardless of spread below cap | May gather very few postings per physical window |
| Row preparation | Existing O(n) contributions clear per candidate | Zero row on first touch plus essential scatter and bitmap pop |
| Pending ownership | One batch, fixed rank/cap, no matrix touch | Exact incoming leaf matrix plus match bits |
| Ties/interleaving | Uses existing multi fallback | Naturally handles overlapping essential docs |
| Unfavorable case | Extra second-min comparison; regions shorter than cutoff get no benefit | Sparse/disjoint rows may repeatedly gather few docs |
| Scratch | Reuses batch and contributions; O(1) span metadata | Existing bounded128-by-n matrix; mechanism09 does not shrink it |

A physical gap alone does not prove enough postings or enough eliminated work.
One posting 2048 addresses before its runner-up can still lose to extra dispatch
logic. Optional probes, per-candidate incoming clearing/folding, local-bound
construction, and region caps remain. Thus this does not justify a speed promise
or a prediction that it closes the existing fixed-port gap. It targets a specific
avoidable multi-driver cost and is structurally smaller than a second heap route.

In accepted06, an essential lead's own selected block end usually caps the
numerical region, so do not assume many batch refills or a2048-address opportunity
in every dense block. A full dense128-posting block can end long before the
cutoff. Fixtures that assert activation must space the lead postings enough to
leave a capped gap of2048 or more, and assert the certified suffix length too.
The cutoff can therefore have low activation; that is a measured rejection
reason, not permission to silently widen bounds or lower the cutoff mid-trial.

## Minimal distinguishing fixtures and root-owned acceptance

Use the existing independent `check`/`trace` union oracle, comparing every callback
doc and score bits, with test-only evidence that the intended branch activated.
Do not validate only final top-k heaps or use mechanism09 as its own oracle.

1. **Gap and tie classifier:** suffix length >=2, unique gaps2047/2048/2049,
   exact first-doc tie, exhausted runner-up, all exhausted, and a large uncapped
   gap with `hi - d1 = 2047`. Classifier metadata can cover values near the u32
   sentinel without allocating enormous fieldnorm fixtures.
2. **Temporary-lead transition:** multiple certified essentials, lead posting
   lengths127/128/129 across blocks/regions below second doc; lead and multiple other essentials all
   match at second doc. Include cap at second doc and a region `hi` strictly
   before it. Assert no duplicate/lost doc and exact shared-boundary leaf fold.
3. **Pending rows outlive cursor:** final fill leaves lead at `TERMINATED` or past
   `hi`, while earlier retained rows contain both an early winner and later winner.
   Top1 threshold growth inside that batch must preserve the later callback.
   Include callback-modeled deleted early winners and fixed thresholds.
4. **Probe and rounding contract:** at least one dense optional plus absent
   optional docs in a selected span, varied norms/frequencies, original incoming
   order different from pruning order, midpoint leaves and threshold equality/
   adjacent f32 values. Assert every branch completes with strict current threshold.
5. **Replay alternation:** selected lead span -> tied/scalar multi candidate ->
   a selected span with a different driver, reusing dirty contribution storage.
   Never overwrite batch while lead rows remain or redispatch before the current
   callback and scalar matching-cursor advances finish.

Retain existing unsafe-domain, tail/block, dynamic threshold, and independent
raw-trace fixtures rather than duplicating them. Root owns targeted tests,
release build, exact default/two-config gates, frozen index/compression checks,
mixed pilot, and AB/BA query-median acceptance ruler for301 OR/1033+ OR with
whole-suite negative controls. Reject the mechanism on gate failure or mixed
pilot regression; measurements must also reveal whether selected spans occur.

Implement against accepted06;08 was rejected by root. This design does
not carry attempt07 threshold repartition, region-expiry changes, or any broader
bound/order/heap optimization forward.
