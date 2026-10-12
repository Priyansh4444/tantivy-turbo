# Alternative: posting-bounded essential batches merged by a small heap

Design only. I read design-multi-batch.md in full, the actual attempt03
SegmentPostings/TermScorer/ClauseState batch APIs, its production OR dispatch and
implementation receipt, and the accepted scalar region/optional/publication
loop. No production edits, heavy commands, children or commits were run.

## Caller first and isolated scope

The caller is the admitted3..32 OR engine after its unchanged certified region
and essential floor reconciliation. Dispatch only regions with at least two
essentials to this merge. Exactly-one-essential regions keep their measured
base's current route; two-term OR stays unchanged. This candidate does not
include callback-driven repartition, region widening, mutable ordering,
optional batching, global-density adjustments, or a dense/sparse heuristic.

```text
prepare existing region [lo, hi), fixed order/prefix/first_essential
reconcile essential floors exactly as current scalar code
if number of essentials >= 2:
    lazily acquire query-owned BufferedEssentialMerge(num_terms)
    begin_region(first_essential, lo, hi, clauses, scorers, shared_batch)
    while next_candidate(..., incoming_contributions) returns (doc, known):
        run existing reverse scalar optional probes and current threshold
        publish unchanged incoming-order f64 fold / one f32 cast
        callback iff exact score > current threshold
else:
    existing single-essential route of the named measured base
```

`next_candidate` fills the EXISTING incoming-order contribution slice with
literal essential f32 leaves, then releases all mutable cursor/scratch borrows
before today's optional loop executes. Its returned known sum is pruning-only.
The replay/publication body remains shared. There is no scorer-owned ordinal,
new public query interface, or leaf-algebra change.

If attempt03 is rejected, include its private postings/score-batch foundation
as an explicit prerequisite in the new isolated candidate, with its full
independent verification. Do not include its rejected single-essential dispatch.
The measured candidate then owns both the foundation and this multi-essential
mechanism; report local delta and cumulative comparisons separately. If03 is
accepted, reuse its unchanged APIs and shared2048B scratch.

## The structurally different data shape

The bitmap design groups hits by128 PHYSICAL addresses and holds a128×n leaf
matrix. This design groups each essential by at most128 POSTINGS, holds one
per-term doc/leaf buffer, and merges its sorted heads. A buffer can span huge
doc gaps. No physical window, bitmap, per-row matrix, survivor list, or running
wrapper-order published score exists.

Private to or_maxscore.rs:

```rust
const POSTING_BATCH: usize = COMPRESSION_BLOCK_SIZE; // 128

struct EssentialLeafBuffer {
    docs: [DocId; POSTING_BATCH],
    leaves: [Score; POSTING_BATCH],
    next: u8, // 0..=len, at most128
    len: u8,  // at most128; zero means no buffered hits
}

#[derive(Copy, Clone)]
struct Head {
    doc: DocId,
    rank: u8, // FIXED preference rank, not original incoming ordinal
}

struct BufferedEssentialMerge {
    buffers: Vec<EssentialLeafBuffer>, // len=num_terms; stable rank slots
    heap: [Head; 32],
    heap_len: usize,
    first_essential: usize,
    floor: DocId,
    hi: DocId,
}

impl BufferedEssentialMerge {
    fn new(num_terms: usize) -> Self; // once, lazy first multi-essential region
    fn begin_region(&mut self, first: usize, lo: DocId, hi: DocId,
        clauses: &mut [ClauseState], scorers: &mut [TermScorer],
        shared: &mut ScoredTermBatch);
    fn next_candidate(&mut self, clauses: &mut [ClauseState],
        scorers: &mut [TermScorer], shared: &mut ScoredTermBatch,
        incoming_leaves: &mut [Score]) -> Option<(DocId, f64)>;
}
```

Type bodies are not implemented. Head comparison is lexicographic `(doc,rank)`.
Do not pack rank into a physical DocId or overload TERMINATED; physical IDs keep
their full domain. u8 cursors are valid because128 is representable and the
private batch API cannot return more than128; checked slice use and debug
invariants retain that fact. usize cursors are a readable alternative costing
about12 extra bytes per buffer; this is not a separate performance mechanism.

The class owns the complete merge protocol, not cursors. It does not retain
mutable references to clauses/scorers across calls: that would prevent caller
optional probing, since preference order is different from incoming scorer
storage and Rust cannot split those aliasing indices into two simple slices.
Short-lived method borrows end before scalar optional scoring. Do not expose
pop/refill/reset chores to the caller or require a forgettable finish_batch.

Module map:

* or_maxscore.rs owns heap/buffer lifetime, incoming ordinal scatter, essential
  known order, region dispatch and the existing optional/publication body.
* ClauseState::fill_scored_batch owns the mirror update from actual scorer.doc.
* TermScorer::fill_scored_batch owns exact leaf evaluation with immutable BM25.
* SegmentPostings::read_postings_batch owns cur/reconciliation/advance and
  first-unread positioning; Bm25Weight::score_batch preserves literal arithmetic.
* term_query/mod.rs retains only the private ScoredTermBatch re-export if the
  prerequisite is needed. No extra cross-module heap interface is justified.

## First-unread versus buffered head: the central ownership invariant

ClauseState.doc ALWAYS remains actual authoritative first-unread decoded doc.
After filling a term buffer it may be ahead of every buffered doc, perhaps
already >=hi orTERMINATED. The buffer's `docs[next]` is the replay head, and the
heap owns exactly one cached `(head_doc,rank)` for EVERY nonempty essential
buffer. NEVER overwrite ClauseState.doc with a buffered head merely to reuse
the scalar merge loop.

For an essential term, all unprocessed hits belowhi are partitioned into:

1. its sorted buffered suffix `[next,len)`, already consumed from the actual
   cursor and carrying retained exact leaves; and
2. actual postings at/after ClauseState.doc, not yet consumed by batching.

No hit is in both and none is in neither. When the buffer empties, refill ONLY
from that actual first-unread cursor, never from the last replay doc. A copied
leaf remains valid after mutable decoder data is overwritten. Caller optional
score() is invoked only for terms that are optional in this fixed region and
were not prefetched into essential buffers.

Heap emptiness, rather than any minimum actual cursor doc, determines whether
replay is complete. An early `all actual docs >=hi` exit would discard all
prefetched hits and is forbidden. begin_region asserts no pending heads from
the prior region; every active slot is reset then filled anew. Old array payload
beyondlen remains dirty but unread, so there is no full-buffer clearing per
region. Rank slots may change essential status at the next region, only after
the old region has fully drained.

## Begin, pop matches, and forward refill

At begin_region:

1. Save the already-certified physical floor/hi and fixed essential suffix.
2. Reset active buffer next/len, without zeroing their1024B payloads.
3. For each essential whose actual mirrored doc<hi, call the owned batch helper
   with floor=lo, up_to=hi and the ONE shared ScoredTermBatch. Copy only its
   returned doc/leaf prefixes into this term's buffer. Norms/freqs stay shared.
4. Add a cached heap head for each nonempty buffer; bottom-up heapify inO(e).
   No callback or optional seek has occurred yet.

At next_candidate:

```text
if heap empty: return None (every actual essential is >=hi/TERMINATED)
doc = heap minimum physical doc
clear existing incoming_leaves exactly as scalar loop does
known = 0f64
while heap minimum doc == doc:
    rank = pop minimum (ties prefer smaller FIXED preference rank)
    leaf = buffers[rank].leaves[next]
    incoming_leaves[clauses[rank].ordinal] = leaf
    known += f64(leaf)
    consume this buffer head
    if buffered suffix nonempty:
        reinsert its next head (strictly greater doc)
    else if actual ClauseState.doc <hi:
        forward refill this rank from its actual first-unread doc
        copy only doc/leaf prefixes; update mirror through owned API
        reinsert nonempty next head (strictly greater doc)
    else:
        omit this rank for remainder of region
return (doc, known)
```

The existing region caps each essential at its selected block end. Since the
owned API emits at most one decoded block suffix and current blocks have128
postings, normally one fill per essential suffices for this certified region;
fully consuming a selected complete block leaves first-unread>=hi, and a tail
has at most127 remaining postings. The guarded forward-refill path is still
sound and makes the abstraction independent of that incidental count fact; it
must never manufacture an empty refill when actual doc>=hi already proves no
remaining hit. A zero fill must yield actual doc>=hi/TERMINATED; otherwise it
violates the owned API's progress contract and must assert, not spin silently.

Refill before returning the current candidate is safe: no next buffered head
can equal the just-consumed term doc because postings are strictly increasing.
Other terms with the same candidate remain at the heap root ahead of greater
heads, with rank tie-breaking preserving their addition order. Current leaf is
copied into caller contribution storage BEFORE refill overwrites term arrays.

If root prefers postponing refill until the next next_candidate call, the heap
must retain an explicit exhausted-rank queue whose absent head cannot hide
smaller unseen docs. That extra hidden state is avoidable; immediate refill is
the favored simpler ownership shape.

## Exact known order, publication, threshold and deletions

For one candidate, heap ties pop matching essentials in their current FIXED
preference rank order. Thus known starts at0 and receives the identical f64
matching-leaf sequence as the scalar engine. Do not use arbitrary heap tie
order or a rank-unsorted matched list; conservative ScoreSumUpperBound would
still bound nonnegative reorder error with originaln, but could change internal
pruning decisions and add extra callbacks, weakening complete-trace identity.

Publication remains the incoming ordinal contribution vector's unchanged f64
fold followed by one f32 cast. Heap known is never published. Optional probes
retain inclusive prefix bounds using originaln, reverse fixed optional order,
actual seek coherence and literal optional leaves. Every candidate sees the
threshold returned by the immediately previous callback; batch-entry threshold
must not be captured and reused. Strict `>` retains tie behavior.

Gathering essential leaves early has no effect on scoring inputs, since weights
and norm readers are query/snapshot immutable. It only changes the timing of
cursor-owned consumption. Region partition remains fixed untilhi in this trial.
Do not graft attempt07 repartition while buffers are pending: a newly optional
term's actual cursor may already be past its pending buffered docs, so scalar
optional probes would otherwise lose contributions. That requires a different
buffer-aware optional protocol and is outside this isolated mechanism.

No deleted doc is removed during fills/merge. Physical ID bounds and original
max_doc remain; the generic callback/Top-K collector owns deletion rejection
and returns unchanged threshold. Unsafe/signed/exceptional domains fall back
BEFORE any batching, exactly as the current engine does. Do not early-discard
buffered hits after a threshold rise without a separately proved all-remaining
certificate; draining against current callbacks is this trial's contract.

## Scratch and allocation accounting

One shared ScoredTermBatch is2048B (docs/freqs/norm temp/exact scores), reused
sequentially for all fills. Per-term storage holds ONLY docs512B+leaves512B and
next/len. With u8 indices and4-byte alignment, expected EssentialLeafBuffer is
1028B; Head expected8B. These are planned host layouts, not runtime receipts.

Atn32: term buffers32×1028=32,896B; heap32×8=256B; shared batch2048B; existing
ClauseState/prefix/contributions max1160B: **36,360B array/record payload**,
plus ordinary merger scalars/alignment and its additional24B Vec header. The
three baseline allocations become four with one lazy buffer Vec; no per-region
allocations or pool. The heap itself stays fixed stack storage. At n3, expected
combined payload3×1028+256+2048+116=5504B. Exactly-single-only queries allocate
no buffer Vec, though stack/frame layout still needs measurement.

Unlike a physical matrix, active essential count can vary by region. Allocating
num_terms slots once gives stable fixed-rank indexing and bounded scratch; do
not resize/allocate per region to shave optional slots. A packed active-rank
mapping saves some payload but adds ownership/index conversion and is not part
of the first mechanism. A query that only ever has e2 can still pay n-buffer
allocation if n32; this is an explicit compactness tradeoff.

The bitmap proposal's maximum20,632B is smaller than this design's expected
36,360B; this design is not claimed to win memory. Both differ from the port's
80.5KiB window scratch and collector work; neither count establishes RSS or
index-efficiency superiority. Root should print actualsizeof, Vec len/capacity,
frame layout and full payload in a semantic oracle fixture before publishing.

## Causal hypothesis and honest comparison

Scalar multi-essential candidate work repeatedly performs O(e) minimum,
matching-score and advance scans touching large scorers. Here leaf scoring and
cursor reads are batched per term, heap metadata is compact, and candidate
selection is O(log e) per matching posting. Existing O(n) contribution clearing,
optional probes and incoming publication remain, as does region prefix work.

Likely advantages over bitmap: disjoint/sparse heads can cover arbitrarily large
physical gaps without allocating/enumerating windows or rechecking every clause
per128 addresses. One term fill consumes up to128 contiguous POSTINGS, rather
than often1 hit within a clipped physical window. Scoring arithmetic is shared
with the same batch foundation in either design.

Likely losses: dense overlap matching e terms on every doc costsO(e log e) heap
operations versus bitmap'sO(e) scatter or scalar'slinear scans; e2 can make heap
bookkeeping exceed two scalar comparisons; short/staggered certified regions
mean each buffer holds1-2 hits and copied arrays/setup are not amortized; larger
query scratch and buffer copy bandwidth; code growth/inlining pressure; scalar
optional seeks or per-region bound/prefix work remain dominant.

Thus bitmap is structurally favored for dense overlap and smaller payload,
heap for sparse/disjoint merges where physical windows are poorly occupied.
Do not force a universal winner or insert an unmeasured workload selector into
this first isolated candidate. Structural e>=2 dispatch provides a transparent
comparison, while self-controls and per-query regressions decide actual merit.

## Independent runtime gates

1. Full BufferedUnionScorer callback raw-bit traces for disjoint/interleaved
   essentials, dense identical sets, n3/n32, physical gaps from1 to huge jumps,
   and lengths127/128/129/255/256/257. Ascending callback order and duplicates
   must be checked; final Top-K alone cannot detect hidden/replayed heads.
2. Equal-doc matching heap ties in a preference order different from incoming
   ordinal order, midpoint/disparate-magnitude leaves and exact threshold
   neighbors. Literal expected f64 published bits and unchanged known sequence
   catch unordered heap accumulation or incoming ordinal confusion.
3. Essential actual cursor becomes>=hi orTERMINATED immediately after its fill
   while buffer still contains winning hits. Observer proves those hits are
   replayed and heap, not actual minima, governs region completion. A refill or
   region reset may not drop/overwrite another pending term head.
4. Dirty-buffer/rank reuse at next region: a term previously optional becomes
   essential, one old essential becomes optional, buffers have different new
   lengths, and old tail leaves must never enter publication. Existing stale
   shallow/tail floor and authoritative first-unread fixtures remain mandatory.
5. Early Top1/Top10 threshold rises during multi-buffer replay, collector rejects
   deleted early/last physical hits, later stronger hit retained. Constant
   threshold limit0 and growing callbacks compare whole traces. Signed and
   exceptional admission must produce no heap-fill events and canonical output.
6. Scratch/progress oracle: prove >=2 buffers were populated and an actual
   multi-clause candidate consumed; observe head doc versus actual first-unread
   cursor, empty tails and clipped hi; print actual layouts/bounds within this
   fixture. Retain prerequisite03 exact leaf/raw posting/positions/SkipFreq tests.
7. Root runs existing full library/integration/default/two configured frozen
   correctness gates before pilot, then complete balanced AB/BA and self-controls
   against the explicitly named base, with no query exclusions. Keep historical
   global-policy regression IDs401/719/791/691/909/817 visible; new heap does not
   authorize changing their region/global-density behavior.

Fresh fixed-port comparisons establish actual progress separately. No design
claim of full allOR victory, memory superiority, or a speedup follows from this
source-only sketch. The private ownership shape is feasible without mutable
cursor leakage, but the larger scratch and dense-overlap heap work are real
reasons a fresh judge may prefer the bitmap attempt.
