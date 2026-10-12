# Region-local preference partition: design package

## Ground and intended caller

The existing caller stays unchanged. `BooleanWeight::for_each_pruning` calls
`or_maxscore(term_scorers, reader.max_doc(), threshold, callback)` only for an
actual 3..=32-term union and a summing combiner with `SUMS_IN_F64`. Two-term OR,
COUNT, other query shapes, unsupported combiners, and >32-term OR retain their
current routes. `or_maxscore` keeps its current pre-mutation numerical admission
and exhaustive fallback. No public API, index format, codec, or stored bound
changes are proposed.

This package was grounded in the complete current `or_maxscore.rs`, the actual
`ScoreSumUpperBound` and `SumCombiner`, the caller in `boolean_weight.rs`, the
`TermScorer` scoring/bound/movement methods, postings shallow-selection and actual
seek behavior, the TopDocs deletion callback, and port `bulk.rs` through its
partition implementation. The supplied grounding/profile is the causal input;
this agent ran no build, test, timing, or profiling job.

Current fixed global score/cost preference controls both region-bound prefixes
and candidate essential suffixes. This can be a poor approximation of the best
local choice: a globally valuable dense term can have a weak bound in this block,
while the globally low-priority neighbor has a large local impact. The current
prefix must stop at the latter and leaves the former essential as well. The port
sorts by local maximum/cost at each outer window, so it can make that locally weak
dense term optional. This eliminates actual dense candidate visits, score calls,
and movement, rather than making only their metadata loads cheaper.

The existing global-density policy is independent and must remain byte-for-byte
in intent. Historical failures 401/719/791/691 and 909/817 must stay visible. A
local preference must not overwrite global order or alter which globally optional
terms omit their block-end cap.

## One isolated first attempt

**First attempt: local-order permutation only.** Keep region construction,
global-density widening, threshold update timing, candidate scoring, cursor
ownership, and exact incoming-order score replay unchanged. Within each completed
region certificate, build a bounded stack permutation of the existing compact
records. Preserve the certified global optional prefix in its fixed global
order; sort only the remaining records by local maximum/cost. Build the existing
non-subtractive inclusive prefix in that permutation and run the existing scalar
candidate loop through it.

Do not combine this attempt with immediate threshold repartition, a new
single-driver kernel, batching, heaps, cached global prefixes, wider regions,
required-clause intersections, sparse replay, or a changed two-term route. Their
shapes are described below to make alternatives reviewable, not to stack them
into the first measurement.

## Caller-first usage and private sketch

```rust
// Caller unchanged; private implementation adds one bounded local permutation.
or_maxscore(term_scorers, reader.max_doc(), threshold, callback);

// Inside the existing outer region loop, after every local_max has been certified
// and doc>=hi cleanup has completed:
let mut local_order = LocalOrder::new(num_terms); // once per query, not per region
local_order.rebuild(&clauses, globally_optional);
let first_essential = partition_prefix(
    &clauses, local_order.as_slice(), &mut prefix, &upper_bound, threshold,
);
// Run the existing candidate/optional/replay/advance algorithm using local_order.
```

```rust
// Existing ClauseState and scorer arrays retain fixed global/incoming identities.
struct LocalOrder {
    // n<=32 is established at the private routing boundary. Entries are global
    // ClauseState positions, not incoming scorer ordinals.
    ranks: [u8; 32],
    len: u8,
}
impl LocalOrder {
    fn new(num_terms: usize) -> Self { /* not implemented */ }
    fn rebuild(&mut self, clauses: &[ClauseState], globally_optional: usize) {
        /* not implemented: see pseudocode */
    }
    fn as_slice(&self) -> &[u8] { /* not implemented */ }
}

fn partition_prefix(
    clauses: &[ClauseState], local_order: &[u8], prefix: &mut [f64],
    bound: &ScoreSumUpperBound, threshold: Score,
) -> usize { /* not implemented */ }
```

`LocalOrder` is optional as a named type; a local `[u8; 32]` plus the admitted
length is equally sound and smaller if only two uses arise. It hides no scorer
mutation and exposes no new lifetime protocol. Avoid a full `OrEngine` object or
private methods merely passing the same arrays through many levels. Keep one
module, `boolean_query/or_maxscore.rs`; no postings or term-scoring changes.

Production signatures above are sketches, not mandatory extra call frames.
Inline a simple partition construction into the existing region loop if that is
more readable. `prefix` is still one existing allocation of n+1 doubles,
`contributions` one existing allocation of n floats, `clauses` one existing
allocation of n compact records. The order is stack storage, 32 bytes plus its
length/padding, no per-region allocation. `usize::from(u8)` is lossless. Creating
u8 ranks must use the proven admitted range rather than a general unchecked
external narrowing cast.

## Local comparator and rebuild pseudocode

```
for rank in 0..n:
    order[rank] = rank
keep order[..globally_optional] fixed
sort order[globally_optional..n] in ascending local_max / max(cost,1)
    ties: original incoming ordinal
prefix[0] = 0
for local_rank in 0..n:
    prefix[local_rank+1] = prefix[local_rank]
        + f64(clauses[order[local_rank]].local_max)
first_essential = longest prefix whose conservative rounded bound <= threshold
```

An allocation-free unstable slice sort is acceptable with the ordinal tie break.
An insertion sort is also simple for <=32 elements but has O(n^2) worst case;
measure rather than choosing it for theoretical cache friendliness. The exact
heuristic ratio is not an arithmetic contract. Comparisons can use the port-like
f64 division, or compare f64(local_max_A)*f64(max(cost_B,1)) to the reciprocal
product to avoid division inside the comparator. All sanitized local bounds are
finite nonnegative f32 and costs are u32, so the cross products fit finite f64.
Cross products can produce a different priority at almost-equal floating ratios;
that changes only optimization order and must be stated in evidence. Start with
one comparator, do not benchmark two then silently report only the winner.

Why pin the global prefix: every one of these terms already has a global
certificate, its combined upper bound <= threshold, and the density guard
justified dropping its physical block cap. Allowing a fresh local sort to move
such a term behind an intervening higher bound can accidentally make it essential
again. Correctness would survive its global bound, but the change would revive a
dense driver and defeat the protected widening policy. Pinning the whole certified
prefix makes those terms remain optional. Sort the remaining suffix only.

A globally optional rank may have zero local_max because its loaded decoded doc
is beyond hi. Keep the global prefix membership even then; it costs at most a few
zero additions and avoids a second membership representation. A zero-bound suffix
term will sort before positive bounds. Negative thresholds can still make all
zero-score matching clauses essential: keep the ordinary conservative comparison,
not a shortcut that discards every zero-bound clause.

## Candidate-loop integration

```
seek every local essential clause below lo using ClauseState::seek
loop:
    doc = minimum mirrored doc over local essential suffix
    if doc >= hi: end region
    clear incoming contributions
    known = 0
    for each local essential rank matching doc:
        score its incoming scorer; write contributions[ordinal]
        known += f64(leaf)
    for local optional rank from first_essential-1 down to 0:
        if bound(known + prefix[rank+1]) <= threshold: reject candidate
        else reconcile optional cursor using actual seek(doc), score if equal
    if not rejected:
        score = incoming-order f64 fold of original f32 leaves, cast once to f32
        if score > threshold: threshold = callback(doc, score)
    advance every matching member of the SAME local essential suffix
lo = hi; discard local certificate/permutation and reconstruct next region
```

Only metadata reads gain one bounded `order[rank]` indirection. Movements continue
through `ClauseState::seek/advance`, which own the mirror update and debug equality
checks. Scorers remain in original incoming order; the local sort never moves a
2,072-byte scorer. The cached `ordinal` maps both scorers and contribution leaves.
The ordinal tie break makes partition evidence deterministic.

## Exact arithmetic and certificate proof

For each physical half-open region [lo,hi), an ordinary local_max bounds each
clause's contribution throughout the region, with hi capped by every selected
non-global block end. Global optional ranks use their global maximum and need no
block cap. A decoded position >=hi proves no hit in the region and is zeroed;
a stale decoded position <lo does not prove absence and must never be zeroed.
Fallback to the existing global maximum handles unusable local bounds.

Let P be any optional prefix in the local permutation. For an accepted eligible
nonnegative score domain, each leaf in P is at most its corresponding maximum.
`ScoreSumUpperBound::new(original_n)` conservatively covers different orders and
groupings of the recursive f64 sum, with outward rounding followed by monotone
f32 conversion. If the conservative bound of P is <= threshold, a document with
no essential clause cannot have a published score > threshold. Therefore the
union of essential docs is a complete candidate set. Prefixes are constructed by
addition from zero, never subtraction of total-minus-suffix. The optional test is
inclusive `known + prefix[rank+1]`, including the next term to be probed, so it
cannot exclude a candidate before accounting for an unexamined leaf.

The local order is used only for known-bound arithmetic and traversal. Published
scores remain `contributions.iter()` in the original incoming order: original
f32 BM25 leaves are converted individually to f64, folded sequentially, then cast
once to f32. No port preference-order f64 accumulator is published, no f32 partial
sum is published, no leaf kernel is changed. Midpoint, duplicate, disparate-boost,
negative/NaN/infinite fallback, configured BM25, and strict ties keep the existing
arithmetic contract. Scorer eligibility is checked before any shallow mutation.

## Cursor, tail, dynamic threshold, deletion, exhaustion

- Shallow `seek_block` changes skip selection/cache and can leave decoded doc/freq
  stale. Local sorting moves only integer indices; it neither loads nor legitimizes
  a stale decoder. Any newly essential clause is actually sought to lo before
  scoring/advance. Optional clauses are sought to each candidate before scoring.
- Keep the current tail reconciliation: selected end TERMINATED and doc<lo triggers
  an actual seek(lo); preserve the selected last_doc value when computing the cap.
  Do not replace this with `has_remaining_docs` or a sorted-doc inference.
- Prefix membership is region-local. At the next region, a formerly optional
  clause may become essential and must be reconciled through actual seek. Sorting
  it locally is precisely the new behavior the independent fixture must exercise.
- Current threshold updates remain immediate for candidate tests but do not
  repartition until the next region. Since threshold only rises, a prefix valid
  at region start stays optional. Keeping too many essentials wastes work but
  never removes a valid hit. Do not retain a permutation/certificate across hi.
- The physical range still uses max_doc, including deleted IDs. Deletion filtering
  stays in the collector callback; a deleted candidate returns the same threshold,
  so it cannot trigger a new pruning certificate. Ascending doc order is preserved
  by the minimum of an essential union and actual monotone cursor moves. Thus
  strict score > threshold preserves lower-doc tie winners.
- TERMINATED records get zero bounds and can occur anywhere in fixed global order.
  Local zero-prefix placement is harmless. Keep them in the original scorer array
  so score ordinals and original_n error factor never shrink on exhaustion.

## Why this could materially reduce the gap, and why it might fail

The plausible large gain is changing the number of dense essential postings
visited. A locally weak dense term can change from essential to optional, replacing
thousands of scalar score/advance operations with candidate-target seeks. This can
save much more than the previous 5% metadata refinement on queries where global
and local priorities diverge. It can also create a one-essential regime for a
future independently measured driver kernel. The port demonstrates the mechanism,
not its transfer gain: Tantivy's tight 128-posting region construction means many
more region sorts than the port's adaptive/hierarchical windows.

Nothing in the current trace proves that local/global order divergence dominates
our five expensive queries. Repeated region cleanup/prefix stores are already hot;
adding a sort and an indirection can make that bottleneck worse. If local maxima
are proportional to their globals, the order often remains unchanged and the
candidate set is identical. Many three-clause queries also need two or three
clauses for top-10, so no priority can discard meaningful candidate work.
Configured bounds can be close to global and provide little local discrimination.
At <=32 clauses the sort cost is bounded, but not free. This attempt also cannot
improve the 198 two-term OR route. It cannot alone establish an all-OR win while
that larger group is roughly 2.7x slower than the port.

Reject if paired primary 3+ timings are neutral/regressive beyond self-control
spread, if any exact gate differs, if source/input guards move, or if known dense
failure IDs regress beyond controls. An overall 1% improvement with 2.3x remaining
gap is a measured small win, not victory. Preserve its failed source/binary/receipt
before reverting. Do not add windows/batches merely to make this design survive.

## Independent verification and causal diagnostics

1. Keep all ten existing buffered-union raw-bit callback fixtures. Their oracle
   scores every matching doc with canonical SumCombiner and independently evolves
   the threshold. Incoming-order midpoint fixtures are especially important after
   preference changes; test several incoming permutations and thresholds equal to
   and one ULP around the exact score.
2. Add a two-region local-priority reversal fixture: two same-df/same-boost dense
   clauses tie globally (fixed incoming ordinal decides), but clause A has TF100
   in its first full block and TF1 in its second, while B has TF1 then TF100. Add
   a sparse higher-priority third clause and use norms fixed at 1. Choose a fixed
   threshold strictly between the weak and strong local maxima, then run Top-1 and
   Top-10 growing thresholds. Assert the COMPLETE callback `(doc, raw score bits)`
   trace equals independent buffered union. A cfg(test)-only partition observer
   should additionally prove that the dense optional identity changes between
   regions, rather than testing only the final heap on a never-reached branch.
3. Extend that fixture with a weak first two full blocks and strong third block
   after an optional clause's decoder was left stale; use doc gaps and lengths
   127/128/129/255/256/257. Assert literal known docs around 255/256 and final tail,
   as well as the whole independent trace. This validates promotions caused by
   LOCAL REORDER, not merely the already-existing threshold behavior.
4. Keep global prefix density fixtures and the exhausted-high-cost observer.
   For a globally certified dense prefix plus locally reordered suffix, prove the
   global prefix remains optional and the widening choice matches the baseline.
5. Use actual deleted documents in an integration corpus where deleted early
   high-score docs would otherwise fill Top-1. Compare exact ranked DocAddresses
   and score bits with independent exhaustive collection. Unit callback can
   separately return an unchanged threshold for selected deleted IDs.
6. Default and two configured-profile full 1,227-query exact gates; every timed
   checksum, actual port comparisons, complete 1,221 query schedule, AB/BA and
   both self-controls; retain all negative controls and old failure IDs.

For diagnosis only, cfg(test) or a separately compiled non-timing instrumented
worker can count regions, local order changes, essential-count histograms,
essential score/advance calls, optional seeks, and admitted single-driver regions.
These show whether the causal hypothesis occurred; they must compile out of the
measured release binary. Compare exhaustive-verified baseline/candidate counts,
then preserve instrumented binary/source identities separately. Do not infer
candidate clears dominate from region prefix-store samples. New release profile
should inspect whether lost essential scalar work outweighs sort/region cost.

## Structurally distinct alternatives, not first-attempt additions

### B. Threshold-only in-region promotion in fixed preference order

This is the smallest independent alternative. No sorting/permutation is needed.
Existing region prefixes stay fixed; after scoring/advancing the current candidate,
raise first_essential while the next inclusive prefix bound <= new threshold.
Only do this after a callback raises threshold past the next prefix boundary; do
not rebuild local bounds or execute shallow seeks. Essential membership only
shrinks, so the next essential minimum cannot revisit the current doc. A demoted
clause remains alive as an optional and is sought to future candidates as needed.
If all terms become optional, skip directly to hi. Keep global widening until the
next ordinary region, so the historical density guard is unchanged.

Sketch: `promote_local_prefix(prefix, bound, threshold, &mut first_essential)`
returns no heap or scorer object. A stored next conservative prefix score gates
calls; treat equality as crossing because candidates are accepted strictly >.
**Critical sequence**: advance every matching OLD essential at the current doc
BEFORE changing first_essential. Otherwise a newly demoted scorer can retain the
current doc while other cursor state/driver assumptions change. There is no extra
allocation and O(n) total prefix growth per region. On steady thresholds it is one
predictable post-callback branch; while filling top-K it may matter little because
most callbacks do not yet raise the threshold. Top-1 within a large sparse tail
is the strongest fixture. This can save late-region essential work but is likely
smaller than a batching/window change because tight regions can end quickly.

Oracle fixture: first accepted doc raises threshold above a dense-prefix bound
inside a single >128-doc-ID sparse-tail region; a later sparse high-TF winner must
still be returned. Verify complete raw-bit trace, exact crossing equality/ULPs,
old-essential demotion at a shared candidate, stale optional probes after demotion,
Top-K threshold jumps crossing several prefixes, no-callback region, and unchanged
threshold on deleted candidates. Prove the cfg(test) first_essential changes before
hi. Benchmark as its OWN source revision, not bundled with first local ordering.

### C. Explicit scalar single-essential driver

Retain a fixed region partition. If the essential suffix has exactly one clause,
walk its mirrored doc directly until hi, score it once, probe optional clauses,
replay incoming leaves exactly, callback, and advance that one cursor. This removes
per-candidate essential min/match/advance loops and their branch/index traffic.
It uses no window bitmap, array of batched scores, or new postings API. A broader
variant computes the second-smallest essential doc once and drives the unique
first clause only until min(second_doc,hi); every other essential's leaf is zero
in that half-open subrange. That variant is a separate attempt because its extra
minimum search can hurt dense unions.

The kernel should own the whole current candidate operation behind ONE private
helper or one inline branch, not accept many loose leaf/bound/mirror parameters
through pass-through functions. Keep `ClauseState::seek/advance` as actual movement
owners. A named reusable OrScratch/engine is justified only if helper signatures
start leaking many unrelated arrays. Do not duplicate exact optional/replay code
in two branches without a common small scoring routine; maintaining two versions
of numerical pruning invites drift.

An essential suffix with zero-bound or TERMINATED members is not automatically a
single driver: at negative thresholds a zero-score matching clause is competitive.
The first safe admission is exactly one essential rank, not merely one positive
bound. With rising thresholds, dispatch only between candidates AFTER old-matching
advances. Half-open second-doc routing must return to the union at equality.
Single-driver benefit is constant-factor scalar overhead, not the port's actual
two-pass batch BM25 arithmetic. It probably cannot halve total cost unless most
current visits are already in one-essential regions. Use the histogram diagnostic
to decide; do not claim a 2.3x closure merely from a cleaner loop.

Oracle fixture: one sparse essential and 2..31 dense optionals, large gaps, tails,
matching optional/essential docs, original-order midpoint leaves, callbacks raising
threshold inside the driver run, zero/negative initial thresholds, exact hi and
second-doc coincidence. Compare every callback bit, not only returned top-K.

### Rejected composite shapes

- Copy port MaxScoreBulkScorer wholesale: pooled 80.5KiB window arrays and scorer-
  preference-order score accumulation violate this unit's compactness and exact
  incoming-order publication contract. Batching needs its own cursor-owned API.
- Sort the ClauseState records in place per region: easier inner loops, but loses
  fixed global order or needs a second restore sort, and mixes policy certificates.
- Cache local sorted order across regions without recomputing changed bounds:
  stale maxima can preserve an order heuristic safely, but stale prefixes are not
  certificates. It hides the very lifetime boundary this design must keep explicit.
- Global persistent prefix cache: independent from local selection and does not
  address candidate density. It is another mechanism with separate exhaustion
  policy tests, not a required dependency of the local-order trial.
- New doc heap for every region: for <=32 compact records, construction/update and
  extra metadata can cost more than the min scan. Test only with actual essential-
  count evidence, not the port's different heap/window workload.

## Synthesis recommendation

Measure A independently because its causal upside is removing dense work and its
source change is bounded in one private module. B is the smallest next isolated
attempt if A does not improve; C requires observed one-essential dominance before
implementation. If more than one direction repeatedly needs window/codec/batch
special cases to be competitive, scrap the scalar design and re-ground a dedicated
cursor-owned batch scorer rather than accreting flags into this loop.

Rubric self-assessment (not a measured acceptance): exactness/proof 5; potential
large cost reduction 3; compactness 5; ownership/API simplicity 4; independent
verification 5. The main weakness is actual candidate-density benefit is unproven
and region sorting makes an already-hot phase more expensive. Root must judge it
against the structurally different batch and prefix candidates before choosing.
