# Posting-bounded essential merge, v2: caller first

Design package only; bodies below are **not implemented**. This is the second whole-shape arena direction alongside SingleLead span dispatch. It consumes sorted per-essential posting batches with a small fixed heap, rather than grouping leaves by physical addresses.

```text
admit existing finite/safe 3..32-term domain, or unchanged exhaustive fallback
prepare existing [lo, hi), fixed preference order, prefix and essential suffix
reconcile the essential floors through today's authoritative cursor API

if essential_count == 1:
    existing accepted-base single-essential batch replay
else:
    merge = query_heap.get_or_insert_with(|| EssentialMergeScratch::new(n))
    (optionals, essentials) = clauses.split_at_mut(first_essential)
    region = merge.region(essentials, lo, hi, scorers, shared_batch)
    while (doc, known) = region.next_candidate(scorers, shared_batch, incoming_row):
        run existing reverse optional probes on optionals, with CURRENT threshold
        fold incoming_row in ORIGINAL incoming order in f64, then cast ONCE to f32
        invoke callback only if exact score > CURRENT threshold
        use its returned threshold immediately for the next candidate
    // Region is drained; its borrows end before any new certificate is prepared.
lo = hi
```

The existing contribution vector is the incoming row. The merger returns only `(doc, pruning_known)` and writes literal essential leaves to that row; it neither publishes scores nor owns thresholds/callbacks. It borrows only the essential ClauseState suffix for the region. Its transient scorer/scratch borrows end at each `next_candidate` return, so scalar optional scoring can legally borrow the whole incoming-order scorer array. Splitting metadata is sufficient; splitting the differently ordered scorers would be wrong.

No explicit pop/refill/finish chores appear in the caller. No callback-driven region expiration, repartition, dynamic ordering, optional batching, physical span heuristic or new score algebra is included. A region drains fully even if every authoritative cursor has reached `hi` or TERMINATED while leaves remain pending. Two-term OR and public interfaces remain unchanged.

## Grounded source and base choice

Canonical accepted06 is `/home/pronsh/Coding/playground/search/tantivy-pr2937`, HEAD `a674aaf2f`; active08 is `/home/pronsh/Coding/playground/search/tantivy-or-hillclimb-oct07`. Root's latest 08 verdict is rejection: all 19 targeted fixtures and three 1227-case frozen gates passed, but pilot was 12.79%/13.55% slower in the two orders, with sparse worst queries around 2x. Those are root-provided status inputs, not runs by this candidate. The next implementation parent is accepted06; root will revert08 after diagnostic profiling. This design therefore replaces accepted06's scalar multi suffix only and keeps no 08 matrix. Root's accepted06 actual-port long-OR result remains about 1.74–1.81x slower. This sketch makes no speed claim.

I read prior `design-multi-heap.md` in full and re-grounded the following actual source:

| Constraint | Source evidence | Design consequence |
| --- | --- | --- |
| 3..32 route and separate two-term route | accepted06 `src/query/boolean_query/boolean_weight.rs:693–700` | Same dispatch admission, heap fixed maximum32. |
| Scorers remain incoming order; ClauseState is frozen pruning order | accepted06 `or_maxscore.rs:9–51`, `:119–136` | Heap rank is suffix preference slot; row scatter uses ClauseState.ordinal. |
| Existing certified region bounds and density policy | accepted06 `or_maxscore.rs:146–257` | Entire preparation unchanged, including exhausted-remainder cost filter, shallow/tail reconciliation and inclusive prefix. |
| Single-essential batching already accepted; multi suffix remains scalar | accepted06 `or_maxscore.rs:259–348` | No batch prerequisite to import; replace only multi suffix mechanism. |
| 08 retains incoming leaves in 128 physical rows and batches each clipped window | active08 `or_maxscore.rs:47–162`, `:372–461` | Rejected08 is diagnostic evidence; none of its matrix/bitmap merger is carried into the next accepted06-based candidate. |
| Batch owns reconciliation and authoritative first unread | both trees `postings/segment_postings.rs:25–74`, `term_query/term_scorer.rs:51–69` | Buffered heads cannot masquerade as ClauseState.doc; no decoder internals leaked. |
| Literal leaf arithmetic preserved in both scoring profiles | both trees `query/bm25.rs:568–614` | Reuse current ScoredTermBatch and score_batch unchanged. |
| Actual port uses two-head separation, not universally dense aggregation | `lucene-rs-idf-fix-oct06/src/search/bulk.rs:57`, `:223–302` | Its 4096 physical window gives a 2048 gap trigger; this is the distinct all-head posting merge alternative. |

The port's `top2` gap branch consumes its sole lead below the second head, even when the partition contains multiple essentials; its remaining multi branch aggregates f64 window scores by wrapper visit order. Copying its score aggregation would violate Tantivy's original incoming leaf fold. Its pooled physical-window scratch, collector filtering and callback/repartition behavior are not imported here.

Workflow principles that changed the sketch: **Model the Domain** made pending term suffixes separate from authoritative cursors; **Minimize Reader Load** removed stored first-rank metadata and repeated clause parameters by borrowing a fixed essential slice for a scoped region. This candidate is read-only arena work, not an independent implementation/measurement run. Throughput checkpoint: n/a, read-only investigation.

## Private data and signatures

All merge types stay private in `or_maxscore.rs`; no new exported module. Prefer ordinary usize indices over packed bytes in the first trial: the small layout saving is not the mechanism, and 128-length cursor states remain easy to read.

```rust
const POSTING_BATCH: usize = crate::postings::compression::COMPRESSION_BLOCK_SIZE;
const MAX_OR_TERMS: usize = 32;

struct EssentialLeafBuffer {
    docs: [DocId; POSTING_BATCH],
    leaves: [Score; POSTING_BATCH],
    next: usize, // 0 <= next <= len <= POSTING_BATCH
    len: usize,
}

#[derive(Clone, Copy)]
struct Head {
    doc: DocId,
    slot: usize, // relative position within THIS fixed essential suffix
}

struct EssentialMergeScratch {
    buffers: Vec<EssentialLeafBuffer>, // exactly n slots, allocate lazily once
    heap: [Head; MAX_OR_TERMS],         // initialized storage; active prefix only
}

// Ephemeral region capability. Only this value can consume its buffers/heap.
// No reference to scorers, shared_batch, incoming_row, threshold or callback.
struct EssentialRegion<'a> {
    scratch: &'a mut EssentialMergeScratch,
    essentials: &'a mut [ClauseState],
    floor: DocId,
    hi: DocId,
    heap_len: usize,
}

impl EssentialMergeScratch {
    fn new(num_terms: usize) -> Self { /* not implemented */ }

    fn region<'a>(
        &'a mut self,
        essentials: &'a mut [ClauseState],
        lo: DocId,
        hi: DocId,
        scorers: &mut [TermScorer],
        shared: &mut ScoredTermBatch,
    ) -> EssentialRegion<'a> { /* not implemented */ }
}

impl EssentialRegion<'_> {
    // row.len() is the admitted original term count; no row returned by reference.
    fn next_candidate(
        &mut self,
        scorers: &mut [TermScorer],
        shared: &mut ScoredTermBatch,
        row: &mut [Score],
    ) -> Option<(DocId, f64)> { /* not implemented */ }

    // Private owner chores, never caller protocol.
    fn refill(&mut self, slot: usize, scorers: &mut [TermScorer],
        shared: &mut ScoredTermBatch) { /* not implemented */ }
    fn heapify(&mut self) { /* not implemented */ }
    fn replace_or_remove_top(&mut self, next: Option<Head>) { /* not implemented */ }
}
```

`Head` ordering is lexicographic `(doc, slot)`; lower preference slots win equal-doc ties. Active heap length gives membership, not a packed doc sentinel. Fields below len/heap_len are initialized, and unused payload is never read. No unsafe access or global pool is needed. `refill` and heap primitives stay local methods because they carry the essential owner invariant; do not build a reusable generic heap adapter for one 32-head consumer.

`region` fills all active slots once and heapifies. Its lifetime prevents another region from borrowing scratch concurrently and preserves the same essential suffix throughout replay. At construction each active slot's counters are reset before fill; old payload outside len remains dirty. A debug-only Drop assertion may require `heap_len == 0` to catch accidental early caller breaks; Drop performs no drain or scorer mutation. Release correctness comes from the sole caller's exhausted while loop, not a destructor side effect. No finish method is needed.

## State and ownership proof

For essential slot s, remaining postings below hi are exactly:

1. retained `buffers[s].docs[next..len]` with corresponding literal leaves, already consumed from the actual postings cursor;
2. first-unread actual postings at/after `essentials[s].doc`, not yet consumed by batching.

The two sets are disjoint and exhaustive. `ClauseState.doc == scorers[ordinal].doc()` always denotes authoritative first unread. A buffered head can be smaller; never overwrite this mirror with replay state. The heap contains exactly one head for each nonempty buffer and none for empty buffers. No hidden absent empty buffer may have unread hits below hi at a public method boundary: drained owners are refilled synchronously before return, or their authoritative doc is proven >=hi/TERMINATED.

Construction:

```text
assert essentials.len() >= 2; assert lo < hi
for each slot in ascending fixed suffix order:
    set next=len=0
    if actual doc < hi:
        fill through ClauseState.fill_scored_batch(floor=lo, up_to=hi)
        copy returned docs/leaves prefixes from shared scratch to this owner buffer
    if buffer nonempty: append (docs[0],slot)
heapify active heads
```

The batch API returns at most one decoded block suffix, never physical addresses absent from that posting list. If a term has no hits below hi, zero is allowed only with authoritative doc>=hi/TERMINATED after reconciliation. It must not produce a zero-progress spin. All returned docs are >=lo, <hi and strictly increasing, all copied leaves remain valid when the decoder/shared scratch is overwritten.

Candidate step:

```text
if heap_len == 0:
    assert all buffers empty and all essential first-unread docs >= hi
    return None
candidate = heap[0].doc
row.fill(0f32)
known = 0f64
while heap_len != 0 and heap[0].doc == candidate:
    slot = heap[0].slot
    copy this leaf into row[essentials[slot].ordinal]
    known += f64(this leaf)                 // slot-tie order is fixed preference order
    consume this owner's buffered head
    if its suffix is now empty and authoritative doc < hi:
        refill this owner with floor=max(region floor, authoritative doc), up_to=hi
    replace root by its next head, or remove it if exhausted in region
    sift root down using (doc,slot)
return (candidate, known)
```

Replacing/removing the root is sufficient because a same-owner replacement strictly increases doc. An unseen refilled head must also be >candidate; strictly increasing postings prove this. Other same-candidate owners therefore remain ahead of every replacement, and increasing slot breaks their ties. Every matching essential is gathered exactly once before return, with the same known addition order as accepted06/08. There is no matched-owner list to sort and no O(e) scan per candidate. Heapify once per region is O(e); every consumed matching posting costs O(log e) metadata repair. e means essential suffix length, including exhausted slots.

Refill is only on drained owners, never before their last retained leaf is copied, and always stops at current hi. Numerical certificates are held locally and survive the decoder moving to its next block. Under today's region rule hi is capped by every nonglobal term's selected block end; essentials lie outside the global optional prefix. Consequently ordinary preparation generally emits at most one decoded block suffix per essential in the region. This is a strict limit, not a promise of 128 hits: the earliest selected block end may belong to another essential or a locally optional nonglobal clause. A dense locally optional block may cap hi before a sparse essential has more than 1–2 hits, and an exhausted or later-head essential may emit zero. No buffer crosses hi to finish a decoded block merely because it has free capacity. The serializer/decoder still owns 128-posting blocks; the merge buffer holds only their certified current-region suffix prefix. Heap batching cannot cure repeated short certified regions without a different region-certificate mechanism, which is excluded. Keep guarded refill because the existing batch contract is one suffix per call and shallow reconciliation/tails must be handled by the owner, not encoded as a one-fill assumption. No pending hit is allowed to cross the region boundary.

Induction: construction preserves the two-set partition and installs all heads; each candidate removes its minimum equal-doc prefix and restores all owner heads before returning. Candidates are strictly ascending; equal-doc leaves coalesce into one publication row; heap exhaustion means all below-hi essential postings were consumed. Only then can rank roles, prefix, local bounds or region floor change. A callback threshold rise changes the next optional/publish decision immediately, but never discards or reclassifies pending leaves.

## Exact scoring, threshold and deletion obligations

Use the current finite safe indexed-TF admission before any cursor mutation. Test positive indexed frequencies for both legacy and native BM25 profiles and finite valid query boosts/parameters; synthetic impossible zero-TF midpoint tricks do not prove this domain. Retain signed/exceptional fallback tests and zero/unsafe maxima admission behavior.

Each retained Score is the actual f32 leaf from unchanged score_batch. `known` is pruning-only: exact ascending fixed preference slots for matching essentials, then unchanged reverse optional probes. Use SmallScoreSumUpperBound with original n and inclusive `prefix[rank+1]`. Publication always folds the full incoming row, including literal zero for nonmatches, in incoming f64 order with one final f32 cast. No cached region threshold, subtraction bound, widened leaf, reordered publication, or heap-known publication.

Optional terms were not prefetched into buffers in this fixed region. Their original scalar seek/score behavior therefore remains coherent. Row is cleared once per candidate, preventing optional leaves from the previous candidate surviving. Reclassifying a buffered essential as optional before drain would lose leaves behind the actual cursor; forbid it structurally by the fixed suffix borrow and caller sequence.

Do not filter deleted postings during fill or merge. Physical max_doc and boundaries remain physical. The caller/collector owns deletion rejection and may return an unchanged threshold. Full callback trace fixtures must cover that behavior, because final Top-K agreement alone can hide duplicated, omitted or out-of-order callbacks.

## Module map and removal scope

| Owner | Responsibility |
| --- | --- |
| or_maxscore.rs | Private scratch/region/head/buffer types, dispatch, exact essential row assembly. Existing region certificates and optional/publish block stay shared. |
| ClauseState | Existing authoritative mutation/mirror update; seek and fill_scored_batch. No heap fields or shadow docs. |
| TermScorer / ScoredTermBatch | Existing immutable-weight literal scored batches and 2048B shared scratch. No rank/heap ownership. |
| SegmentPostings | Existing decoded-block reconciliation and first-unread consumption. No extra format or cursor exposure. |
| Bm25Weight | Existing score_batch scalar-expression arithmetic. No leaf or normalization change. |

From accepted06 remove the old multi-essential minimum/match/advance scans; keep advance only if another actual use remains. Root reverts rejected08 separately; no ExactLeafWindow, 128-row matrix/bitmap or matrix observer is retained in the new source. Port 08's valuable semantic cases into heap ownership fixtures, not its production mechanism. Do not keep scalar, matrix, heap and gap-dispatch fallback branches simultaneously.

This has one query scratch allocation and one scoped replay owner. The API hides heap repair, decoded-block copies, cursor/leaf separation and refill timing while the caller retains its existing optional/publication semantics. There is no phase adapter that forwards the same arguments, no separately exposed heap protocol and no stored callback/threshold.

## Payload and runtime costs, with no speed claim

Planned 64-bit host layout with plain usize indices: each buffer has docs512B + leaves512B + counters16B =1040B; Head likely16B, fixed heap512B. One shared ScoredTermBatch remains2048B. Existing ClauseState/prefix/row record payload is expected 36n+8 bytes (n32:1160B). Thus the combined array/record payload is **1076n+2568B**, n3 **5796B**, n32 **37,000B**. This excludes Vec headers, scoped-region references/scalars/padding, scorer storage, allocator overhead and call frames. Expected query scratch is buffers Vec header24B + fixed heap512B; region has two references, two DocIds and heap_len. Actual host size/alignment must be measured by root's oracle, not frozen to these expectations.

Allocations: existing three Vec allocations plus one lazy buffer Vec when a multi-essential region first occurs; no per-region/per-candidate allocation. Buffer Vec has n initialized slots allocated once, even when only e2 ever become essential. A later region may have a larger suffix and reuse them. That bounded waste removes resize/mapping state. All-single-essential queries allocate no merger buffer Vec, though compiler frame placement and optional scratch storage still need actual inspection. Reset only counters, copy only emitted prefixes; don't memset 1KiB per slot per region.

08's current n32 matrix foundation is 1160+2048+16384+1024+16 =20,632B, smaller than this heap's planned 37,000B. Root's port source has pooled4096-address aggregate/accumulator scratch (about80.5KiB arrays) with different ownership/collector behavior; neither raw count predicts RSS or overall index efficiency. Retaining actual leaves costs memory, and this alternative is not a memory win over08. A packed u8 head/cursor variant can be considered only after the owner shape wins; it saves little and is not a separate candidate here.

Root's subsequent diagnostic profile covered all 103 OR queries, clean sequential runs, no lost samples and correct checksums. Reported cycle attribution was accepted06 caller 80.03%,08 caller 60.18% plus out-of-line ClauseState.fill_scored_batch 17.54%, block max about 4%, block load/decode about 9%; approximate total cycles 41.1B versus 45.6B. These root-provided diagnostic shares suggest investigating perphysical-window calls/copies, but are not a latency ruler or proof that the heap wins. Annotated locations were still pending at this package's finish. They do not relax fixed hi or establish large batches.

Causal hypothesis: batching each posting suffix once can cover arbitrarily wide sparse physical gaps; compact cached heap heads replace repeated essential scalar scans or08 per128-address gathering/scattering. Incoming row clear/fold remains O(n), optional probes and certified region setup remain unchanged. Copying docs+leaves adds8 bytes/posting into retained buffers before later reads. Dense overlap consumes e matching heads per candidate and pays O(e log e) repairs; bitmap scatter can be better. e2 may lose to two scalar compares; short clipped regions may hold only1–2 hits and fail to amortize setup/copies. No gain is asserted until balanced full-query measurements.

## Meaningful fixtures and runtime gates for root

All new observations are cfg(test) and semantic; no release observer/counter. Root alone implements/runs them and all heavy jobs.

1. **Independent entire callbacks**: BufferedUnionScorer+SumCombiner oracle, assert exact `(physical doc, score bits)` trace and strictly ascending unique callbacks. n3/n32, identical overlap, disjoint/interleaved lists, 127/128/129/255/256/257 postings, gaps1/127/128/2047/2048/2049 and huge legal jumps. Include e2 despite n>=3 and a term already exhausted at region start. A dense case prevents sparse-only correctness claims.
2. **Owner exhaustion before replay**: populate >=2 buffers; after fill every actual cursor is >=hi orTERMINATED while winning buffered heads remain. Observer records head versus authoritative doc and each replay candidate's matched slots. All pending callbacks must remain. Root tests both exhausted tails and full-block crossing; initial heap emptiness must not be inferred from actual cursors.
3. **Literal leaf and rank tie oracle**: incoming order differs from preference order; same doc matches several essential slots, including duplicate query clauses, disparate valid finite boosts and a valid positive-TF midpoint construction for legacy/native. Compare direct scalar leaf bits to retained bits, pruning known sequence in preference order, incoming f64 final cast, and expected next_down/equal/next_up strict thresholds. Prove the tested doc really used >=2 populated buffers; a passing all-single route is insufficient.
4. **Fresh thresholds inside pending replay**: Top1/Top10 rises after early callbacks, constant threshold limit0, deleted early/last physical candidates rejected with unchanged threshold, later stronger candidate. Compare entire traces against canonical callback oracle and actual indexed deletion integration fixture. Do not accept only final heap identities.
5. **Region reuse and clipped boundaries**: dirty shorter/longer suffix reuse; optional becomes essential and essential becomes optional only after old buffers drain; one term's actual cursor is beyondhi while another is still pending. A doc equalhi is never in the current buffer and appears correctly next region. Preserve stale shallow-selected full/tail floor reconciliation cases, optional seek coherence and existing global-certificate density fixtures.
6. **Progress and scratch evidence**: assert len<=128, next<=len, one head per nonempty owner, all returned docs<hi, zero fill implies doc>=hi, mirror coherence after every actual mutation. Observe owner refill only when its previous suffix is empty. Print actual buffer/head/scratch/region sizes, Vec len/capacity, foundation and total record payload alongside a fixture that exercises a multi-clause candidate; report the distinction between arrays and full RSS.
7. **Fallback isolation**: unsafe/signed/exceptional admitted-domain failures have no heap fill events and unchanged canonical traces. Keep accepted scored-postings, positions/SkipFreq and exact batch leaf tests; this design changes no serialization or public API.
8. **Root's acceptance pipeline**: chosen base named explicitly; targeted semantic fixtures then full existing library/integration/default plus both configured frozen 1227 gates before pilot. Complete balanced AB/BA and self-controls with no query exclusions; inspect sparse/disjoint and dense-overlap per-query regressions, historical policy IDs401/719/791/691/909/817 and full long-OR ratio against the unchanged actual port. Compare local delta and cumulative accepted baseline honestly. 08 rejection is already settled; no 09 implementation begins before root's end-to-end read and fresh judge.

## Alternatives rejected and synthesis input

* **SingleLead span dispatch / port2048 gap rule** is the other arena candidate, not folded here. It reduces ownership/scratch when one head dominates a span but needs a second multi route for close/interleaved heads. This package instead makes one posting merge protocol handle every multi-essential candidate regardless of physical span. Compare those whole shapes after reading both, not by inserting the gap threshold into this heap.
* **Changing 08's 128 physical window width** retains the address-indexed row matrix and repeated perwindow gather; it cannot consume long sparse posting suffixes in one owner buffer. It is not this alternative.
* **Heap of actual scalar cursors** removes minimum scans but retains scalar leaf/cursor calls, and adding buffered leaves later risks confusing first unread with replay head. The retained owner buffer is the domain model, not an incidental cache.
* **Aggregate only f64 known per candidate** loses original incoming leaf fold. Keeping leaves is required even when an aggregate looks numerically close.
* **Callback repartition while pending** requires a buffer-aware optional protocol and weakens isolation. It is excluded categorically, not patched with cursor rewinds.
* **Active-rank packing, shared score arena, delayed refill queue, generic heap abstraction** add mappings or hidden pending state for modest payload savings. n fixed buffers plus immediate root repair keeps ownership local and the caller small.
* **Batch all optional clauses or add a density selector** changes a second mechanism and obscures attribution. Existing optional probing and region policy stay intact.

This package is ready for root's end-to-end read and fresh judge, not an implementation approval or a benchmark verdict. The prior heap design's most useful invariant—actual first unread versus retained replay—is preserved; its obsolete attempt03 foundation contingency is removed because accepted06 already owns the needed APIs. The smaller scoped-region caller surface is the principal v2 maintainability change. The fresh judge may reasonably prefer SingleLead's smaller scratch or decline this candidate because dense heap work and short certified fills cannot amortize; this candidate's bounded37kB payload and perpost O(log e) cost are explicit tradeoffs.
