# Candidate B: cap only newly expanded OR certificates at one physical block width

## Caller usage first

The existing OR caller prepares the original tight region and chooses its old
essential suffix exactly as source13 does. It then proposes one endpoint from
that suffix, but admits an expansion only inside the next
`COMPRESSION_BLOCK_SIZE` physical document addresses. If the cap is at or before
the tight endpoint, retain the tight endpoint and its partition. Do not subdivide
an existing sparse tight region merely because it already exceeds 128 addresses.

For an admitted expansion, source13's complete certificate replacement and final
partition rebuild still run before any essential reconciliation or retained
batch exists. Crossed clauses still call the same query-owned
`TermScorer::range_max_score(lo, hi)` capability. There is no partition-equality
condition: promotion is legal, and its new essential suffix owns candidate
generation for the entire final interval.

```rust
// Within the existing drained region-preparation block, not a new API.
let tight_hi = hi;
let old_essential_end = selected_ends[first_essential..num_terms]
    .iter().copied().min().unwrap_or(max_doc);
let physical_end = lo
    .saturating_add(COMPRESSION_BLOCK_SIZE as DocId)
    .min(max_doc);
let proposed_hi = old_essential_end.min(physical_end);

// The cap constrains an expansion, never shrinks the original tight interval.
if proposed_hi > tight_hi {
    hi = proposed_hi;
    // Existing source13 operation: full final coverage, then prefix/partition.
    rebuild_final_certificates_and_partition_inline(); // pseudocode only
} else {
    hi = tight_hi;
    // Existing source13 no-expansion cleanup; retain the tight prefix/partition.
    publish_original_tight_cleanup_inline();           // pseudocode only
}
// Existing accepted11 reconciliation/replay/publication/retirement, unchanged.
```

The two named pseudocode operations above are not proposed methods or modules.
Their existing inline source bodies remain in `or_maxscore`; introducing a
pass-through abstraction here would obscure the drained-boundary ownership.

Equivalently, final `hi = max(tight_hi, min(old_essential_end, physical_end))`.
The explicit branch avoids assigning a smaller speculative endpoint and prevents
misstating the invariant as `hi - lo <= 128` for every region. That invariant is
true only for newly admitted expansions. Root confirmed this interpretation of
the brief; unconditional shrinking is outside this candidate.

## Grounding and status

Accepted parent is `43f06833a325d2a18c03bd7e4fa65f2f27944292` (accepted11).
I read its actual Git OR production object, including admission, frozen preference
order, exact global-prefix cache, live density choice, current local preparation,
essential reconciliation, both replay routes, optional probes, incoming-order
publication and retirement. I read the entire actual 1,699-line source13 diff in
contiguous chunks, plus its implementation report, independent review and static
receipt. The held working source hashes match the receipts:

- OR: `1a1ce59b330b4a207d353385bf90cac96333cd4c6da49ed52a7c40cd5bb30804`.
- TermScorer: `21b05a2b6513223a81613ee0f40c4a31f6990a427bb0d4b60324154a55753cdb`.
- Full retained diff: `589305ea0face0e33545393ef474edf177c0af9803b8a9526113f37c6f6bfb4a`.

I additionally read the actual SkipReader representation, selected-pair policy,
constructor and block metadata parsing, compression constant, DocId definition,
TERMINATED definition and TermScorer policy owner. `DocId` is u32, while
`TERMINATED` is `i32::MAX as u32`; do not assume the sentinel is u32::MAX.
The compression width comes from `BitPacker4x::BLOCK_LEN`, currently 128. Full
blocks hold 128 distinct sorted postings; physical span can be far larger.

This is a source-only design. I ran no compiler, tests, workers, timing, profiling,
browser, compression, child tasks, commits or pushes, and edited no worktree.
Root's brief reports source13 exact/targeted gates and its unchanged pilot:
about 8.5% aggregate lower OR3 mean-query-median latency, but median per-query
ratios above one and retained sparse losses around 9–15 times. Those observations
do not prove which cycles caused the losses. Source inspection identifies one
plausible mechanism: a long expansion can increase optional certificates, promote
old optionals, and require dense candidate traversal over a large physical span.

## Minimal state, signatures and module map

No new type, enum, persistent cache or public signature is needed.

```rust
// Existing query-local cold storage from source13, unchanged.
selected_ends: [DocId; 32]

// Existing local facts, unchanged except the capped endpoint expression.
tight_hi: DocId
old_essential_end: DocId  // scalar temporary; may be folded into proposed_hi
physical_end: DocId      // scalar temporary; may be folded into proposed_hi
proposed_hi: DocId

// Existing deep capability: implementation and policy remain unchanged.
impl TermScorer {
    pub(crate) fn range_max_score(&mut self, floor: DocId, up_to: DocId) -> Score;
}
```

`src/query/boolean_query/or_maxscore.rs` imports the existing compression constant,
changes only the one-shot endpoint proposal, and adds independent fixture
coverage for admission/rejection geometry. The existing test-only region observer
must report the actual chosen endpoint and use an independent cap oracle instead
of source13's old equality to the unbounded essential endpoint.

`src/query/term_query/term_scorer.rs` retains source13's actual range helper and all
five provenance/positions/cache fixtures byte-identically. No changes to postings,
skip serialization, scoring arithmetic, collectors, reader APIs or index format.
ClauseState remains its original five-field hot record; source13's logical 128-byte
selected-end scratch remains. The cap adds no array or per-region allocation.

## Exact preparation pseudocode

1. Retain accepted11 admission and immutable original term count for
   SmallScoreSumUpperBound. Retain exact cached global sum/count and fresh live
   density decision; do not bundle attempt15's independent density cache.
2. Retain source13 selected-end preparation. Globally optional/endless clauses
   use max_doc coverage. Every other live clause shallow-selects at
   `max(lo, clause.doc)`. Tail reconciliation and mirror updates remain unchanged.
   Save each raw local certificate before virtual tight zero cleanup.
3. Compute the tight prefix with virtual zero when real doc >= tight_hi. Choose
   the tight first_essential using the original-n helper and current threshold.
   If all optional, skip ONLY the original tight interval; do not propose wider.
4. Compute the old essential suffix endpoint once. Cap it at
   `min(max_doc, lo.saturating_add(COMPRESSION_BLOCK_SIZE as DocId))`.
   If the capped endpoint <= tight_hi, keep tight_hi and the already chosen tight
   partition, applying source13's original no-expansion zero cleanup.
5. Otherwise set hi to the capped endpoint and execute the entire source13 final
   loop: real doc >= hi -> zero; selected_end < hi ->
   `max(saved_raw_bound, range_max_score(lo, hi))`; otherwise saved_raw_bound.
   Rebuild every prefix cell sequentially, then select final first_essential from
   zero before replay. No partially rebuilt prefix may enter candidate generation.
6. If final all optional, skip the completely certified final interval. Otherwise
   retain accepted11 essential reconciliation, fixed final partition/endpoint,
   single-batch or scalar replay, optional reverse probes, exact publication,
   strict callback and retirement. The callback's monotone threshold may rise
   during replay; no final partition is speculatively revised inside that phase.

## Safety and ownership arguments

**Endpoint/progress.** The old essential suffix is nonempty and every selected
end is >= tight_hi, so old_essential_end >= tight_hi. The cap either yields an
actual expansion, `lo < tight_hi < hi <= max_doc`, or leaves the original positive
interval intact. Saturating addition prevents wrap; min(max_doc) prevents an
endpoint beyond the physical ceiling (TERMINATED is legal as an exclusive end).
No sentinel increment is introduced. Existing
selected ends still use `last_doc.min(max_doc - 1) + 1` inside `lo < max_doc`, so
max_doc=0 never enters that subtraction. Near TERMINATED, min(max_doc) owns the
physical ceiling regardless of u32 arithmetic headroom.

**Coverage.** An uncrossed selected certificate covers the entire final interval
because its end >= hi. A crossed range helper covers unread leaves from
`max(lo, real_doc)` to exclusive hi; saved_raw is retained too. A real first-unread
doc >= hi proves zero. A real stale doc below lo never proves zero. Thus each
clause has complete final coverage before prefix/partition rebuilding. Shortening
the proposed expansion cannot invalidate the same trusted range enclosure.

**Virtual-zero restoration.** A clause whose doc was beyond tight_hi but below
final hi must regain its saved raw certificate or crossed range certificate. The
original virtual cleanup leaves that raw value intact. If no expansion is
admitted, the original zero cleanup is published exactly as source13 does. No
rollback cache or undoing of actual shallow selection is needed.

**Policy/numerics.** The range helper remains the actual query scorer's owner.
StoredDefault, validated native transform and all unsupported/missing/saturated/
nonempty-tail/too-late metadata fallbacks stay unchanged. No new provenance
inference is made from a stored pair. Maximum over whole trusted blocks encloses
their leaves; unresolved nonempty coverage uses the immutable global maximum.
The final sequential f64 prefix and original-n bound factor are unchanged.

**Cursor, positions and cache.** Preparation remains drained. No retained batch
has leaves behind the real first-unread cursor when range lookahead runs. Only
the helper's SkipReader clone moves; live decoder, positions, real skip offsets
and selected-block numeric cache are not speculative range owners. The permitted
PendingNative transition remains query owned. The caller's mirrored doc is updated
only by existing actual seek/advance/batch operations.

**Replay and strict scoring.** Final partition changes, including promotions, are
fully settled before essential reconciliation. The sole essential remains the
last rank and was old essential; hi cannot exceed its selected end. Full and
partial retained rows keep their scores even when the real cursor advances ahead.
Scalar multi-essential traversal may cross a promoted optional's full-block end;
its final range certificate covers that case. Publication remains original incoming
order, each f32 leaf converted to f64, one final f32 cast, and strict score > current
threshold. Deletes remain collector owned and cannot raise the threshold merely
because a deleted callback occurs. No new callback API guarantee is invented.

**Tails/EOF.** Nonempty tails still use global bounds, already-loaded tails included.
Real EOF and empty relevant ranges return zero only under source13's existing
guards. Exhausted clauses remain coherent and cannot be revived by a cap. A
too-late shallow reader still falls back before empty metadata inference.

## Costs and honest limits

Every admitted expansion has at most 128 physical addresses, hence at most 128
distinct union candidate docs and at most 128 postings from any one clause within
that interval. For valid sorted unique postings, such a window can intersect at
most two complete 128-posting blocks: intersecting three would require one posting
from the first, all 128 from the middle, and one from the third, exceeding 128
physical addresses. Whole boundary-block over-coverage may still include a strong
leaf outside the window and promote a clause.

At this caller, real preparation already selected the block containing
`max(lo, actual_doc)`. The cloned reader therefore starts at a covering block; its
seek need not parse an arbitrary prefix to that start. An admitted crossed call
needs at most two trusted full-block pair evaluations and one block advance, or
an earlier conservative fallback (including a nonempty tail). This bound relies
on the actual caller preparation and valid postings, not on a new global budget
inside the general range helper. The helper's larger external private requests
remain linear and unchanged.

The cap adds constant arithmetic and an endpoint comparison to each proposed
region. Existing O(n) selection, virtual-prefix construction and conditional final
prefix/partition rebuilding remain. Successful expansion can still combine several
nearby short preparations. When tight_hi-lo >=128, no expansion is admitted and no
crossed lookahead is introduced; the original sparse tight interval is retained.

This limits the extent of newly promoted candidate work per expansion, but does
not prove a per-query performance floor. Many small expansions can repeat clone/
drop, score-bound arithmetic and partition rebuilding. Admitting a promotion may
still make a query slower; failing to widen can surrender source13's aggregate
gain. Existing cold native envelope allocation, Arc work, binary/frame size and
RSS remain real costs. No claimed measured memory or storage improvement follows
from unchanged logical state and index bytes.

## Realizable independent fixtures

Keep the existing independent scalar leaf enumeration, untouched canonical
postings progress oracle and full BufferedUnionScorer raw-bit callback oracle.
The region observer is evidence; no expected score/bound is computed using the
candidate's range helper. Compute expected cap arithmetic in widened u64 from
lo plus the documented compression width, clamp to max_doc, then compare the
chosen interval against the original tight and essential endpoints.

1. **Admitted clipped expansion with real crossed promotion.** Reuse the native
   indexed alpha/beta/gamma geometry from source13, move the independently stronger
   beta leaf from doc150 to doc138, and initially actual-seek each scorer to17
   before invoking the kernel. Dense alpha/beta select end128; gamma's first full
   block ends later (source13's 2-of-3 doc geometry reaches doc190). The original
   tight endpoint is128 and the proposed essential endpoint exceeds145, so the
   capped first expansion is [17,145). Compute the threshold from independently
   selected current alpha/beta bounds and literal beta score at138, as source13
   already does, then assert the actual preference/density facts before relying
   on them. Require old-optional beta promotion, a strictly sub-global final beta
   bound enclosing138, scalar beta127->128 progress, exact callbacks and leaf
   coverage. This tests legal promotion; B must not accidentally become A's gate.
   Runtime feasibility of actual bound/rank facts still needs root's execution.
2. **No expansion at equality.** The original native first-region lo0/tight128
   geometry gives cap128 even if old-essential end191. Require no widened row
   at lo0 and exact canonical callbacks, including later beta150. This rejects an
   unbounded implementation without deleting the original correctness corpus.
3. **No shrinking of a sparse tight region.** Use n3/n32 scorers with full posting
   blocks at docs `4*i`, initially at0, so the minimum first selected endpoint is
   509 (>128). Arrange admitted positive weights and independently verify the
   old essential suffix. Require the original tight endpoint509, no widened row
   for that preparation, and canonical complete progress/callback traces. There
   must be no artificial 128 endpoint, even though the physical cap is smaller.
4. **Restored virtual zero within a clipped expansion.** Initially seek the
   weak dense clause and sparse driver to100; give another clause first unread
   doc132. Dense selected end128 makes the doc132 clause's tight contribution
   zero, while the sparse driver's selected end is beyond228. Require the actual initial lo100, old
   tight128 and admitted cap228. The previously zero clause at132 must regain a
   positive certificate and its literal contribution must appear in canonical
   callbacks. Choose boosts/threshold from independent actual bounds and assert
   rank/density facts, rather than assuming boost order. This geometry restores
   a zero without requiring an impossible first-block endpoint below128 at lo0.
5. **Exact exclusive head and batches.** Use dense/driver populations where the
   admitted region has an actual winning posting exactly at hi and another just
   below. Require no batch row at hi, authoritative first unread head at hi, later
   callback inclusion under its new region, and exact retained score bits. Exercise
   partial batch sizes and refills across regions. Existing full128-row cases can
   occur in retained tight regions; do not demand a full128-row clipped expansion
   when its floor is inside a driver's block.
6. **Final end selection rather than cap.** Construct a tight endpoint below both
   proposed ends but an old-essential end before lo+128. Require actual hi equal
   to that essential end, not always lo+128. n3/n32 overlap/disjoint and duplicate
   clauses preserve original-n arithmetic. Derive physical endpoints from the
   actual recorded selected blocks and literal fixture postings.
7. **Near-address ceiling and empty segment.** Check the pure arithmetic expression
   with lo=TERMINATED-100, max_doc=TERMINATED and an essential endpoint at the ceiling;
   it must clamp to the ceiling and stay positive. Include max_doc0 with exhausted
   scorers to prove the kernel never enters endpoint subtraction. Do not allocate
   a multi-gigabyte fieldnorm fixture merely to mirror arithmetic. End-to-end
   ordinary tail/EOF and head-at-max_doc tests supply cursor semantics.
8. **Strict/raw-bit/deletes/policy matrix.** Retain independent incoming-order
   midpoint and adjacent-f32 thresholds in several clause orders, simulated and
   actual alive-bitset deletions, Top1/Top10 callback growth, unsafe canonical
   fallback, native DEFAULT/custom enclosure and existing helper position/cache
   tests. Geometry-specific source13 witness equalities must be revised only to
   assert the selected cap policy; complete raw-bit/canonical oracles stay intact.

Source13's old tests that require first hi >300 from lo0, a batch exclusive
endpoint382 inside that same first widened region, or fixed preparation floors
0->2160->3891 are policy-specific witnesses. Under B those can correctly become
unrealizable. Preserve their independent callback and progress oracles; replace
only the geometry witness with the cap-admitted examples above. Do not relax all
evidence to merely nonempty results or remove the original sparse documents.

## Alternatives rejected and rationale

- Unconditional `min(oldessentialend, lo+128)` as final hi: can shrink an existing
  tight region, introducing a separate subdivision/partition policy and misusing
  the source13 no-expansion branch. Preserve-tight is an essential boundary.
- Partition-equality admission: belongs to distinct candidate A. B instead bounds
  the extent while retaining full certificate replacement and legal promotions.
- Width families, sparse-density thresholds or corpus-specific exclusions: create
  tuning mechanisms and change the experiment. This candidate uses the existing
  compression width once; no new numeric selector.
- New provenance enum or range cache: source13 already owns trust and coverage
  inside TermScorer. A cache would add invalidation/ownership questions and a
  separate mechanism; the cap needs none.
- Early exit after a metadata-entry budget: not equivalent to full range coverage
  unless it falls back globally, and would alter the helper beyond the caller's
  endpoint policy. The address cap provides a source-grounded natural bound here.
- Hot selected-end fields, generic trait APIs, intermediate pass-through methods,
  live cursor speculative seeks or tail decoding: all add shape/ownership changes
  unrelated to bounding expansion. Existing private capability is sufficient.

## Synthesis contribution and root acceptance contract

B offers the smallest new state: one capped proposal and no rollback buffer. Its
cost bound directly limits how far a newly promoted clause can drive candidates,
while retaining plausible reduction of clustered short preparations. It accepts
the risk of repeated preparations and legal promotion regressions. A may avoid
promotions more directly but requires proving speculative certificate rollback;
root should compare the complete alternatives against the stated rubric.

If root selects B, graft only its endpoint mechanism over held source13, with
accepted11 as the explicit parent and source13 still unaccepted. Root reviews the
actual full resulting diff, executes realizable geometry/positions fixtures and
the three 1,227 exact gates, then uses the unchanged untrimmed pilot and conditional
full paired/self controls. All per-query losses remain visible. No design result
changes the actual-port win predicate or establishes full Lucene API parity.
