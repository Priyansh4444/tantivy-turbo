# Two-term OR: fixed execution and local certificates

Design only, grounded at fork `0e9329dfe782187a8feda0024102e4aabf517c28`.
No source changes, builds, tests, profiles, benchmarks, or child tasks were run.
throughput checkpoint: n/a, read-only investigation

## Caller usage first

The only production caller of `two_term_or_maxscore` is the two-clause arm of
`BooleanWeight::for_each_pruning`. It has already established actual TermScorers
and a combiner supporting Block-WAND and incoming f32 leaves summed in f64.
COUNT, unpruned scored enumeration, non-summing combiners, mixed scorers,
exclusions, and minimum-should-match constraints retain their existing routes.
The proposed boundary is:

```rust
match term_scorers.len() {
    2 => two_term_or_maxscore(term_scorers, reader.max_doc(), threshold, callback),
    3..=32 => or_maxscore(term_scorers, reader.max_doc(), threshold, callback),
    _ => block_wand(term_scorers, threshold, callback),
}
```

Passing physical `max_doc`, rather than live `num_docs`, enables a safe canonical
fallback and finite tail caps. The only other direct call found is the module's
test helper; it must pass its fixture `max_doc`. No public API is added.

Before ANY warm-up/cursor movement, use the same admission as `or_maxscore`:
non-NaN threshold; both weights have safe score bounds; each immutable maximum
is finite and strictly positive. Otherwise call BufferedUnionScorer with
SumCombiner and the physical max_doc. This is categorical preservation of
signed/exceptional arithmetic, not a heuristic selector.

## Existing behavior that must remain useful

`block_wand_union.rs::two_term_or_maxscore` first enumerates 128 union candidates.
It immediately transfers to `block_wand_intersection` if BOTH global maxima are
at/below threshold. After warm-up it drives the sole globally essential term
when the other cannot win alone; otherwise it returns to general Block-WAND.
The global single-essential branch walks every essential posting, scores it,
tests `score + optional.max_score()`, conditionally seeks/scores the optional,
and advances. It has no whole-block rejection or optional local bound.

The historical decision table is decisive negative evidence:

* Specializing two-term sorting/advance alone lost: geomean 1.004x slower.
* A small dense decoder bounds-check change lost around 1.5% in both orders.
* The warm/global essential/intersection policy was accepted, improving the
  old 20-query Java comparison geomean from 0.451 to 0.372 and making
  `the american` 217.8 us versus Java 476.6 us. Those are old fixtures and Java
  numbers, not evidence of a fresh lucene-rs win.

Thus a fixed kernel must retain the sparse global driver policy, not just replace
it with a generic union having two elements. The current remaining port gap is
2.690/2.712 mean ratios over 198 broad two-term ORs. The old experiment and
existing five 3+ profile do not explain two-term cycles. Root has now supplied
a fresh trace in `baseline-two-perf-report.txt`, covering expensive queries
481 `american south`, 843 `home business`, 673 `times union`, 383 `interest only`,
and 13 `the of`: 55.93% sampled self cycles in generic block_wand, 16.43% in
BlockSegmentPostings::seek, 7.47% in intersection, 6.88% block_max_score, and
5.30% load_block, with no lost samples. This directly favors replacing the
GENERIC FALLBACK first. It does not prove all WAND self cycles are bookkeeping;
that symbol also contains inlined leaf arithmetic. The source permits a causal
mechanism removing repeated pivot/bounds/alignment, while post-profile and
paired latency must establish whether those instructions dominated.

Actual source constraints:

* TermScorer is about 2 KiB, and doc() dereferences decoded postings at a cursor.
* `seek_block` changes the selected block and its bound cache but leaves doc()
  and freq() in the old decoded block. Only actual seek/advance own movement.
* `SegmentPostings::seek` returns early when decoded doc equals target. After
  a shallow move, reconciliation must use a floor strictly above the stale doc,
  as the current 3+ engine does. Do not call seek(stale_doc) to load a new block.
* Tail selected last-doc TERMINATED does not prove remaining docs after a floor.
  Reconcile tail at the floor and retain the originally selected cap.
* ScoreTopK rejects deletions before updating the threshold. Physical IDs still
  define every certificate; the new kernel must not invent a live-ID horizon.

## Design A: fixed-two local region kernel (recommended first isolated attempt)

The full shape retains the first 128 candidates and both-global-optional
intersection transfer, then uses fixed two-term region execution. The FIRST
ISOLATED ATTEMPT replaces ONLY the final `block_wand(scorers, threshold,
callback)` fallback with `two_term_regions(scorers, max_doc, threshold,
callback)`. Retain the entire existing global single-essential branch unchanged
for this attempt. This attacks the fresh trace's 55.93% generic fallback and its
associated seek work without bundling a single-essential optimization.

The admission/fallback boundary is still established before warm-up so the new
region helper can trust its safe domain. Add no sorted metadata Vec, heap,
contribution Vec, prefix Vec, or per-region allocation. Applying the region
kernel to the old single-essential branch is a later independent unit, informed
by its own profile and rejection evidence.

### Type/signature/module sketch

Keep implementation private in `block_wand_union.rs` initially. Avoid a new
module until the resulting function is truly large enough to need one.

```rust
// Incoming scorer order is NEVER reordered. These records are stack metadata.
struct TwoTermState {
    doc: DocId,
    global_max: Score,
    cost: u32,
}

enum TwoTermMode {
    Union,
    Essential { ordinal: usize, optional_required: bool },
}

struct TwoTermRegion {
    hi: DocId,                 // physical half-open end
    maxima: [Score; 2],        // certificate valid only below hi
    mode: TwoTermMode,
}

fn two_term_or_maxscore(
    scorers: Vec<TermScorer>, max_doc: DocId, threshold: Score,
    callback: &mut dyn FnMut(DocId, Score) -> Score,
) { /* not implemented */ }

fn two_term_regions(
    scorers: Vec<TermScorer>, max_doc: DocId, threshold: Score,
    callback: &mut dyn FnMut(DocId, Score) -> Score,
) { /* not implemented */ }
```

Only actual `seek`/`advance` methods updating `TwoTermState.doc` may mutate
scorers. Reuse the current ClauseState ownership pattern with two stack records,
not a wrapper that scatters large scorer fields among hot records. All shallow,
bound, and scoring operations assert doc mirror coherence in debug builds.

The region builder and executor can be inlined local functions or plain helper
functions. Do not expose a builder/iterator lifecycle API to callers. A helper
that merely forwards `scorer.doc()` would add no depth and should be deleted.

### Certificate policy and modes

1. **Globally single-essential:** ignore the optional's block boundary and use
   its immutable global maximum across the essential's block. This preserves
   the existing route for EVERY density ratio. Do not impose the 3+ global
   optional 2x cost gate here: the historically accepted two-term route did
   not have that gate. A selected essential local maximum plus optional global
   maximum can reject a whole essential block. This is safe but sometimes loose.
2. **No globally optional term:** select both local blocks at max(floor, doc),
   cap hi at the smaller finite block end+1 and physical max_doc, and zero a
   term whose actual decoded doc is already >=hi. Preserve tail reconciliation.
3. If the rounded f64 sum of both maxima is <=threshold, skip the region.
4. If both individual maxima exceed threshold, merge two essential streams.
5. If exactly one individual maximum exceeds threshold, drive that stream;
   the other cannot produce a winner alone.
6. If neither can win alone but their sum can, choose the cheaper live stream
   as essential and require optional membership. This is LOCAL intersection,
   not permanent global OR-to-AND conversion. Drop it at hi.

When global maxima both fall <=threshold after a callback, transfer to the
existing global `block_wand_intersection` with actual cursors, as today. With
exactly two positive finite operands, its f64 accumulation is commutative and
matches incoming-order leaf sum. The existing implementation may batch the
leader without reconciling its SegmentPostings.cur; hand-off to it is terminal,
not a callable subroutine that later returns to the OR kernel.

### Pseudocode

```text
admit safe domain or canonical union fallback BEFORE movement
cache two immutable maxima/costs and decoded docs
warm 128 ascending union candidates, preserving strict score > threshold
  leaves = incoming [a-or-zero, b-or-zero]
  score = (f64(leaf0) + f64(leaf1)) as f32
  advance matching actual cursors and mirrors
  if both global maxima <= threshold: terminal global-intersection hand-off

floor = min(cached decoded docs)
while floor < physical max_doc:
  establish region using the policy above
  if region sum <= threshold: floor = hi; continue
  actual-seek essential cursors whose decoded docs < floor
  dispatch once into Union or Essential loop (no generic n-way scan)
  Union:
    d = min(doc0, doc1); stop if d >= hi
    score only matching terms, fold f64 leaves, callback iff score > threshold
    advance both matching cursors
  Essential:
    d = cached essential doc; stop if d >= hi
    essential leaf = actual score()
    bound = round_f32(f64(leaf) + f64(optional region maximum))
    if bound > threshold:
      actual seek optional iff optional doc < d
      if optional matches: obtain its exact leaf
      if required and missing: reject
      else publish exact incoming f64 sum iff > threshold
    advance actual essential cursor and mirror
  after callback, both global optional => terminal intersection hand-off
  otherwise retain conservative region mode to hi (first attempt)
  floor = hi; discard every local certificate
```

Do not use `threshold - optional_max` as a leader cutoff. Rounded subtraction
can lose a candidate at a float midpoint. All bounds are forward nonnegative
f64 sums converted monotonically to f32. ScoreSumUpperBound::new(2) has factor 1,
so direct f64 sum is sufficient but using the helper preserves familiar proof.
Candidate scores never use bound inflation. No approximation, score
quantization, minimum competitive epsilon, or reordered mutable accumulation.

Repartitioning on rising threshold is deliberately deferred in attempt A. The
region mode remains conservative: additional essential docs may be visited,
but no winner is missed. This separates local certificates/fixed dispatch from
threshold-driven specialization and prevents one attempt having two causes.

### Causal mechanism and costs

The fresh two-term profile places most sampled cycles in precisely the generic
fallback selected for the first isolated attempt. The structural opportunity
is replacing per-candidate generic pivot search,
doc sorting, shallow bound summation, and scorer alignment in the fallback
with one certificate per intersected block range and a direct two-doc merge.
New local single-essential/local-intersection ranges can stop enumerating
noncompetitive one-term hits that are still globally competitive. Global
single-essential blocks that cannot beat the threshold can be skipped before
essential BM25 arithmetic. Cached docs remove repeated large-scorer doc reads.

This can remove a large amount of work only if real query thresholds admit
large local regions or block skips. If all local maxima remain above threshold,
fixed dispatch reduces overhead but is unlikely to erase a 2.7x gap alone.
If global optional max dominates every sum, global single-essential block
rejection will be ineffective; record that result rather than calling the
metadata speedup causal evidence of enough pruning.

Scratch: two 12-byte metadata records, two f32 leaves (8 bytes), region bound
array (8), hi (4), small enum/ordinals and threshold locals. Exact aggregate
layout should be measured; do not publish a guessed struct size. No allocation
beyond the incoming scorer Vec, unlike general Block-WAND's wrapper Vec.
Cost is O(1) per candidate and O(1) per region, with a few mode tests moved
outside loops. Code duplication between Union/Essential bodies should stay
small; share actual leaf publication, not a generic per-term framework.

### Rejection cases and acceptance tests

Likely losses: tiny postings where region setup exceeds the entire query;
dense comparable costs with many staggered local block ends; globally optional
loose bounds preventing any rejection; configured BM25 transform/tail scans
making every local-bound setup costly; all-union regions with low threshold;
code-size/I-cache growth from duplicated loops. Preserve global single-essential
cap omission rather than solving these by a query-id selector.

Independent meaningful tests use exhaustive BufferedUnionScorer+SumCombiner
and compare EVERY callback's (doc, raw score bits) under the same monotone heap:

* Warm-up ends at exactly candidates 127/128/129, including a threshold rise
  that invokes global intersection on the final warm candidate.
* A term locally optional in one region becomes essential next region; later
  single-term winner must survive. Both incoming orders.
* Local intersection region followed by a winner missing the old required
  clause. Prevent accidental permanent OR-to-AND conversion.
* Complete blocks, staggered block caps, dense bitset blocks, tails, one stream
  exhausted, and actual doc beyond region end. Include a shallow-selected tail
  whose decoded position is stale below floor, and no remaining real tail doc.
* Threshold just below/equal/above a rounded two-leaf sum; score equal to heap
  threshold does not callback, equal top scores keep earlier physical doc.
* Callback rejecting a deleted winner leaves threshold unchanged; deleted IDs
  still extend physical max_doc. Real index test with trailing deletions.
* Negative/zero/NaN/infinite boosts and exceptional threshold take canonical
  fallback before movement. Reuse existing numeric cases, but assert full bits
  or class for exceptional floats rather than near-equality.
* Seeded random two-list traces, explicit one-term winners, both reversed orders,
  both configured profiles and all frozen correctness gates.

Existing module randomized checks use near-equality and suppress some equal
checkpoint scores; they are insufficient as the new raw-bit oracle. Root should
retain them, then add a compact independent exact trace harness for this unit.

## Design B: route two clauses through existing generic local MaxScore

Structurally distinct: widen or_maxscore admission to 2..32 and route the
two-clause BooleanWeight arm into it with physical max_doc. Delete the old
two_term_or_maxscore only after callers/tests migrate, or retain it for fallback
in a clearly isolated experimental trial. Type surface would not grow at all.

Benefits: the exact original-order arithmetic, unsafe-domain fallback,
conservative bounds, cursor mirror ownership, half-open regions and independent
3+ trace tests already exist. The mechanism trial is very small and can answer
whether local regions alone dominate the gap.

Costs: three scratch allocations plus generic record/prefix/leaf loops, sorting
two preference records, optional reverse scan, essential suffix min-scan and
contribution clearing for every candidate. It loses the warm-128 phase and
terminal global intersection fast path unless more branches are bolted on.
Most importantly, the generic global optional density gate may keep a dense
term essential in workloads where the accepted fixed-two global route drives
one sparse term. Earlier 3+ failures 401/719/791/691/909/817 show why arbitrary
loosening of global driver policy is not a harmless cleanup.

Verdict: worthwhile tiny causal experiment, not the favored production design.
Keep its attempt source/binary/receipt distinct and reject it if expensive
two-term IDs regress. If it wins materially, graft its proven region policy
into fixed-two attempt A; do not graft all generic scratch management.
For a fair first comparison, experiment B should likewise replace only the
post-warm generic fallback with `or_maxscore` admitted for two terms; preserve
the warm/global branches in both A and B. Routing all two-term queries directly
at BooleanWeight is a distinct whole-shape trial, not the same mechanism.

## Design C: block-batched global single-essential kernel (second fixed attempt)

Structurally distinct from both streaming kernels: retain the entire current
warm/global routing, replace only global single-essential walking with batches
of <=128 essential docs/leaves. The port's `TermScorer::next_docs_and_scores`
separates norm gathering from pure arithmetic and visits optional clauses over
sorted batches. Tantivy's intersection already shows score-first candidate
arrays, but manipulating `block_cursor` alone leaves SegmentPostings.cur stale.

Use a private cursor-owning API rather than exporting cur or calling a public
block_cursor fast loop and pretending the TermScorer moved:

```rust
// SegmentPostings owns cur and position-prefix coherence.
fn read_postings_before(&mut self, hi: DocId,
    docs: &mut [DocId; COMPRESSION_BLOCK_SIZE],
    freqs: &mut [u32; COMPRESSION_BLOCK_SIZE]) -> usize { /* not implemented */ }

// TermScorer owns exact norm/BM25 evaluation and actual cursor advancement.
fn read_scores_before(&mut self, hi: DocId,
    docs: &mut [DocId; COMPRESSION_BLOCK_SIZE],
    scores: &mut [Score; COMPRESSION_BLOCK_SIZE]) -> usize { /* not implemented */ }
```

Contract: consume ascending postings starting at current actual doc below hi,
return n<=128, leave actual decoded doc at the first unconsumed posting (>=hi
or the next batch) and preserve future seek/advance/term_freq/positions behavior.
Do not add separate caller `finish_batch` that can be forgotten. Score gather
must use unchanged `Bm25Weight::score` arithmetic. Do not transplant port's
algebraic x/norm-inverse form without a separate exact-bit proof.

For each essential batch, retain the score from the exact leaf kernel, forward
bound against optional global maximum at batch-entry threshold, then iterate
survivors in ascending doc order and recheck against current threshold before
actual optional seek/publication. Reusing the earlier lower threshold only
produces extra survivors. If threshold makes both global terms optional mid
batch, finish/check the already-consumed suffix in ascending order BEFORE
terminal intersection hand-off; otherwise later consumed hits would disappear.

Scratch minimum docs128*4 + scores128*4 = 1 KiB; a separated norm gather needs
another128 bytes of norm IDs or512 bytes of f32 normalization, depending on the
unchanged score kernel's real input. Reuse fixed stack arrays, not 4096-score
windows or pooled 80.5 KiB port scratch. New API/ownership changes span postings
and TermScorer, so this should be a separate attempt and tests must prove cursor
state after empty, partial, complete, boundary, tail and deleted-doc batches.

Verdict: better chance to remove arithmetic/cursor dependency costs when region
pruning cannot shrink candidate counts, but larger proof surface. Attempt it
after A/B establish whether pruning or arithmetic is the residual bottleneck.

## Synthesis and verification recommendation

Fresh two-term fallback dominance favors the fallback-only form of A as the
first isolated attempt. B remains a cheap rejecting comparison with the same
preserved outer warm/global policy. C is the next independently measurable
mechanism if arithmetic or single-essential walking becomes the residual
bottleneck; do not bundle it into A. Retain root's fresh two-term trace as the
baseline for post-profile comparison. Do not redo sorting-only or tiny decoder
bounds-check experiments.

Root owns fresh measurements: all 198 broad two-term OR and 4 Wiki OR remain in
the ruler, plus all 103 3+ OR and unaffected TERM/AND/PHRASE controls. Keep the
historical bad global-policy IDs visible. Pilot is selection evidence only;
full AB/BA candidate/baseline and self-controls, actual fixed-port AB/BA plus
self-controls, exact payload/source/bin identities, configured correctness,
unchanged index hashes and measured binary/scratch/RSS define acceptance.
No claim of victory follows from Rust syntax, fewer allocations, or compilation.

## Source anchors inspected

* `search/bench/lucene-rs-or-hillclimb-oct07/grounding.md` in full.
* `search/bench/or-top10-decision.tsv` in full.
* Fork `src/query/boolean_query/block_wand_union.rs` current route/general WAND.
* Fork `boolean_weight.rs` caller and scorer selection; `or_maxscore.rs` full
  production engine and independent trace helper.
* Fork `block_wand_intersection.rs` full production batching/global termination.
* Fork `score_combiner.rs` arithmetic and ScoreSumUpperBound.
* Fork `term_query/term_scorer.rs` full production API and `SegmentPostings`
  construction/advance/seek/doc and cursor privacy.
* Port `src/search/bulk.rs` MaxScoreBulkScorer through partition_scorers;
  `src/search/term_scorer.rs` complete scalar/shallow/batch implementation.
