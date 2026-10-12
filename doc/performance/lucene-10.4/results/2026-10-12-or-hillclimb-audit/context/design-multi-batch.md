# Next isolated mechanism: 128-doc exact-leaf bitmap windows

Read-only design. Grounded in the current production region/optional/replay loop, the isolated attempt03 private batch APIs and its implementation receipt. No shared-main/attempt03 production edits, builds, tests, profiles, timings, commits or child tasks. Attempt03 has compiled under root control but its performance/correctness acceptance is not assumed.

## Caller first and isolation contract

For admitted3..32 term unions only, keep the existing safe-domain fallback, fixed preference ordering, global-density policy, half-open certified region, prefix construction and `first_essential` unchanged. After reconciling essential floors, dispatch regions with `num_terms - first_essential >= 2` to bitmap gathering. Exactly-one-essential regions retain whatever single-essential implementation exists in the measured base: attempt03 batching only if separately accepted; otherwise the old scalar driver. Two-term routing is unchanged.

This is one mechanism: replace repeated multi-essential candidate min/match/advance scans with block-batched exact leaf gathering into a128-doc window. Optional clauses remain scalar per candidate with the current dynamic callback threshold. No optional batching, clause repartition, region widening, adaptive window sizing, new dense-driver admission or collector specialization.

Call-site shape:

```text
for each certified region:
    reconcile essentials at region floor exactly as today
    if essential suffix length >= 2:
        lazily acquire one query-owned ExactLeafWindow(n)
        while an essential doc remains below region_hi:
            gather one window from authoritative first-unread cursors
            for each bitmap candidate in ascending doc order:
                incoming row already contains exact essential f32 leaves
                known starts with that candidate's essential f64 pruning sum
                execute today's scalar optional probe loop, writing into row
                publish today's incoming-order f64 fold / single f32 cast
                if score > current threshold: callback, update threshold
    else:
        existing single-essential branch of the selected base
```

A practical implementation can share today's optional/replay body by conditionally binding `(doc, known, contributions: &mut [Score])`: the bitmap branch returns its initialized incoming-order matrix row, the scalar branch returns the existing cleared contributions slice. Gather only when no prior window rows remain. Do not duplicate or reimplement the arithmetic/pruning publication body unless Rust borrowing demonstrably requires it.

Use a new isolated candidate worktree and explicitly name its base. If attempt03 is rejected, copying just its private batch foundation into the new candidate is a prerequisite to this mechanism, not permission to keep its single-essential dispatch. That foundation must still be included in the new complete verification and measurement. Report cumulative comparisons separately from this mechanism's own delta.

## The retained exact-leaf state

```rust
const OR_WINDOW_DOCS: usize = 128;

// private to or_maxscore.rs; no export/public API.
struct ExactLeafWindow {
    incoming_leaves: Vec<Score>, // len exactly OR_WINDOW_DOCS * num_terms
    essential_known: [f64; OR_WINDOW_DOCS],
    matches: [u64; OR_WINDOW_DOCS / 64],
    base_doc: DocId,
    num_terms: usize,
}

impl ExactLeafWindow {
    fn new(num_terms: usize) -> Self; // allocate once at first multi region
    fn is_empty(&self) -> bool;
    fn push_essential_leaf(&mut self, doc: DocId, ordinal: usize, leaf: Score);
    fn pop_candidate(&mut self) -> Option<(DocId, f64, &mut [Score])>;
}
```

The ownership-rich `fill_scored_batch` on ClauseState/TermScorer/SegmentPostings stays unchanged. `ExactLeafWindow` never touches block_cursor, cur, BM25 state, skip metadata or actual scoring. It only owns already computed leaves, union membership and pruning-known sums. An ordinal belongs at this OR scatter site, never in the term batch.

Keep the min/hi/gather loop at the call site initially rather than adding a forwarding method with scorer/clauses/bounds/callback parameters. A private `gather_window` method is reasonable only if it hides the complete bitmap/scatter phase and is easier to read than the local loop; do not create shallow API wrappers around every bit operation.

The row returned by `pop_candidate` is a mutable slice in **original incoming scorer order**, not pruning preference order. Rust's mutable borrow of that row prevents a refill/reset while the current row remains in use. The loop calls refill only on the following iteration after row replay ends. This encodes the crucial no-rewind/no-overwrite ownership boundary without a second doc mirror or externally mutable cursor index.

## Gathering one certified physical window

1. Once the previous window has no pending rows, find the minimum authoritative essential ClauseState.doc. This costs one O(e) scan per window, not per candidate. If it is>=region_hi, the region is complete.
2. Set `base_doc = minimum`, `end = min(region_hi, base_doc.saturating_add(128))`. All IDs are physical DocIds; end is exclusive. No live-document renumbering. Since base_doc<region_hi, end>base_doc.
3. Matches must be zero before gathering. Old matrix/known data may remain; they are reset on first touch of a new candidate row.
4. Iterate essential clauses in their current fixed **preference order**, exactly like today's scalar known-score loop. If clause.doc>=end, skip the clause entirely. Otherwise call its owned `fill_scored_batch(base_doc, end, batch)` and scatter each returned `(doc, f32 leaf)` using its incoming ordinal. Repeat only while its authoritative doc remains<end.
5. At first touch of an offset, zero that entire incoming row and set essential_known[offset]=0, then set its match bit. Store the leaf at row[ordinal], and add `f64::from(leaf)` to essential_known[offset]. If another essential matches the same doc, do not clear its row again.
6. After all essential clauses have been gathered, every one has an authoritative doc>=end (orTERMINATED), and the bitmap is exactly the essential union in `[base_doc,end)`. No callback or optional scoring has run during gathering.
7. Enumerate matches using trailing_zeros in word0 then word1, popping bits in ascending offset order. Every pop returns the physical doc, its retained essential known sum and initialized incoming row. Run today's optional/replay body immediately for that row, using the current threshold. No scalar essential score/advance calls during replay.

The same-format current region caps each essential at its selected block end; therefore most clause fills in a window need only one remaining block suffix. A while loop around the owned API is still sound and simple, and cannot consume a doc>=end. It must not call the helper repeatedly merely to get an empty batch when mirror.doc already proves the clause has no candidate in this window.

If a reconciled shallow selection produces no bitmap rows, continue from the newly authoritative minimum essential doc. This must strictly increase to>=end orterminate; assert every essential mirror>=end at gather completion in debug. Avoid walking all empty physical windows: base is always the next minimum doc, so huge sparse gaps jump directly.

## Why zero-on-first-touch is the bounded clearing strategy

The current scalar engine clears n contributions for every essential union candidate. First-touch clearing performs exactly that width of initialization once per actual union candidate row, with no full128×n clear per window. Repeated overlapping essential matches do not clear again. Unused rows are neither read nor cleared. A row from an old window becomes visible only after its first new essential hit clears it, so old optional leaves cannot survive into published scores.

Pop can consume the bitmap bits in place. At the next gather, empty matches marks every new row as needing initialization; no separate touched-row list, generation array or full end-window clearing is needed. A freshly constructed Vec is zeroed once at query allocation as Rust's safe initialized storage; do not introduce unsafe uninitialized buffers just to avoid that one-time cost.

Maintain separate fixed known values only for essential contributions. Per-candidate optional known is a local f64 copied from essential_known and mutated by today's reverse optional loop. Matrix row publication never reads that pruning accumulator. There is no compact survivor list or sorted docs array: the128-bit bitmap itself supplies membership and ordering.

## Cursor and region certificate proof

The batch APIs are the sole real mutation owners. SegmentPostings loads/reconciles shallow state, owns cur and block advance, and returns exact first-unread doc. ClauseState copies that actual doc into its mirror after every fill. The window does not call scorer.score() after a scorer has advanced ahead; retained f32 leaves are the authority for replay.

A full batch may advance/load the next block and invalidate that scorer's mutable block-bound cache. This does not invalidate an already copied numerical certificate for earlier docs: all gathered hits satisfy doc<window_end<=region_hi, and the prefix values remain the bounds certified for that physical region before gathering. During replay, do not request/rebuild bounds or inspect last_doc_in_block to amend the current region. The old selected block may no longer be physically loaded.

When a window is drained, cursors are ready at first unread docs. The next window either begins at their new minimum within the same region, or the region ends and sets lo=the previously certified hi. Only then compute new shallow selections, reconcile tails, rebuild local maxima/prefix and repartition. Old window rows are no longer candidates; their storage may remain dirty safely.

Optional cursors remain deferred exactly as today. On a candidate, if optional.doc<candidate_doc, ClauseState.seek establishes the actual cursor; only a matching actual cursor is scored. Candidates are ascending, so optional probes never rewind. An optional may become essential in a later region only through the existing floor reconciliation. Global-density widening policy and tail last_doc handling remain unchanged.

Early loop termination on an essential mirror>=region_hi while bitmap rows remain is forbidden: those mirrors point past prefetched hits. The replay state, not actual cursor minimum, governs whether candidates remain. Refilling or discarding a window before all rows are processed is likewise forbidden unless a separately proved all-rows upper bound allows it; such a new optimization is outside this unit.

## Callback/score/deletion proof

Gathering essential leaves in preference-clause order means each row's `essential_known` receives the same sequence of matching f64 leaf additions as today's scalar essential loop. This keeps existing internal bound comparisons identical as well as conservative. Incoming absent leaves are zero and matching leaves are retained literally as f32. Optional leaves enter their original row ordinals. Published score is the unchanged incoming-order f64 fold with one final f32 cast; a running wrapper-order sum cannot replace it.

Callbacks occur in precisely the essential union's ascending doc order, after today's safe optional pruning and strict exact `score > threshold` check. Before every candidate, the threshold is the value returned by the last callback, not the value captured at window start. Thus Top1/Top10 threshold growth, ties and collector-owned deletion callbacks retain complete observable raw-bit traces. Gathering before threshold growth only changes when essential leaves are computed; today's fixed region partition already computes those essentials for every candidate and does not reclassify them midway through a region.

The batch does not filter deleted docs. Deleted IDs remain in physical bounds and may reach the generic callback; the Top-K collector discards them and returns its existing threshold. Use max_doc, never num_docs. NaN/signed/unsupported score domains still fall back before any batch cursor mutations. No fresh arithmetic eligibility is inferred from matrix storage.

## Scratch and allocation accounting

At32 clauses, leaf matrix128×32×4 =16,384B. Essential known128×8 =1,024B. Matches2×8 =16B. Existing ScoredTermBatch =2,048B. Existing ClauseState/prefix/contributions array payload max1,160B. **Combined maximum array payload20,632B.** Matrix allocation is one additional query-time Vec allocation; existing three allocations remain, yielding four/96B Vec headers. Small window metadata base_doc/num_terms adds ordinary scalar/alignment bytes; root should print actual container/total layout in an oracle fixture.

At3 clauses, the matrix is1,536B, not16KiB; known/bitmap/batch are fixed3,088B. Existing array payload116B means combined4,740B. At32 max20.15KiB versus the port's80.5KiB pooled window payload, but these are different execution strategies/work and this does not establish RSS superiority. Matrix memory is lazily allocated only when a multi-essential region is actually dispatched; no cost to a query that remains scalar/single-essential throughout. The fixed known/bitmap container's stack layout and any compiler initialization cost still require measurement.

No index writes/codec changes, no per-window or per-region allocation, no pooling. Bitmap128 is intentionally small enough to bound exact leaf retention and keep rows cache-friendly. W256 and ordinal-major matrices are separate experiments, not changes slipped into this unit. Do not add a candidate offset list512B that bitmap enumeration does not need.

## Dispatcher policy and expected losses

First measured mechanism uses one transparent structural rule: suffix length>=2. No unmeasured density/hit-rate prediction or adaptive window heuristic. Its negative controls include exactly-one-essential regions, existing two-term path, and nonOR routes. This proves the mechanism broadly before guessing workload selectors. If tiny/sparse windows regress, retain the rejected prototype and use observed diagnostics to design a separate dispatch adjustment.

Likely wins: multiple dense/moderately dense essentials produce many union candidates per128 physical addresses, avoiding O(e) min/match/advance bookkeeping for each; shared docs gather each term's norms/arithmetic contiguously; matrix exact row replay remains sequential. Compared with attempt03 this removes bookkeeping on the expensive multi-essential branch, not only batching one driver.

Likely losses: sparse/disjoint essentials produce1-2 candidates per physical window, making per-clause fill/setup/scatter more expensive than scalar scans; a selected block's region ends immediately, creating tiny windows; per-leaf scatter with row stride and matrix initialization cost; hot code growth in already heavily inlined BooleanWeight; optional scalar seeks or region prefix rebuilding dominate and do not improve. A matrix cannot recover the port's richer broader impact certificate without an independent policy/codec change.

This mechanism does not remove the traced per-region prefix construction and does not touch two-term OR. It has substantially more causal potential than compact metadata alone, but no demonstrated ability to close the full2.3x/allOR gap until measured. Existing no-own-heavy-job timing discipline, AB/BA controls and unchanged ruler remain mandatory.

## Independent runtime gates to add

1. Full raw callback traces against BufferedUnionScorer on disjoint/overlapping dense essentials, n3/n32, bounded128 windows, lengths127/128/129/255/256/257, clipping region ends and huge doc gaps; compare published bits and ascending callbacks, not only finalTopK.
2. Dirty-row reuse adversary: doc offset7 receives multiple essentials+optionals in one window; in the next window the same offset has only one different essential. Literal expected score must exclude old leaves. Also use every offset0/63/64/127 to prove bitmap ordering/reset.
3. Region expiry/stale decoder: deferred optional becomes essential after several shallow full-block skips, a gathered full batch loads a following block before replay, and a tail exhausts below floor. Record actual first-unread mirror coherence and compare whole independent traces. No stale scorer.score() calls are needed in the new path.
4. Midpoint/incoming-order adversary: pruning order unlike incoming order, different leaf magnitudes, exact float-neighbor thresholds and boosted native/legacy profiles. This catches port-style running sums or row ordinal mistakes.
5. Threshold/deletion trace: early winners raise Top1 threshold within the same128-doc window; rejected deleted winners return unchanged threshold; later genuine stronger hits must appear. Fixed-threshold limit0 and limits1/10 cover both callback patterns. Prove the matrix path actually ran using cfg(test) observation, not only test names.
6. Scratch/progress oracle: at least one window contains>1 essential and>1 union candidate, then an empty/clipped window or fully exhausted suffix; candidate consumption terminates and actualsizeof/Vec lengths remain bounded. Print layout receipt within the semantic fixture.
7. Keep attempt03 cursor/positions/SkipFreq and exact batch-leaf tests if its APIs are included; keep current complete raw-bit fixtures, all default/two-config frozenquery gates and appropriate integration tests.

Accept only after correctness gates and a pilot, then full balanced query/controls against the explicitly named source base, with per-query/no-query-exclusion reports. Preserve known dense policy regressions401/719/791/691/909/817. A fresh fixedport comparison establishes progress toward user target separately from this local mechanism delta. No assertion of allOR victory while the two-term route remains outside the change.
