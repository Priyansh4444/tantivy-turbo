# Independent attempt 12 source review

Judgment: **no correctness blocker found in the reviewed stable source**. Proceed
to root-owned targeted tests, exact gates and the frozen performance experiment.
This is a source review, not evidence that the candidate compiles, passes a test,
improves latency or matches all Lucene features/public APIs.

Reviewed identity:

- Worktree `/home/pronsh/Coding/playground/search/tantivy-or-region-widen-oct07`.
- HEAD/base `43f06833a325d2a18c03bd7e4fa65f2f27944292` (accepted 11).
- Sole dirty tracked source `src/query/boolean_query/or_maxscore.rs`.
- SHA256 `6c71f32394bac8d1c5998c511585904e09ac68757063b2f2bc96afb375258d1d`.
- Full actual 1114-line diff read in contiguous chunks. Resulting production,
  all new helpers/ten fixture families, and both adapted density fixtures read.
- Read original and adapted implementation reports, adapted static receipt,
  design synthesis, and the relevant actual TermScorer bound/batch owner code.
- Independently checked HEAD/source SHA, sole dirty tracked path, diff-check,
  and byte identity of the accepted 11 cache/density block and complete replay.

The workflow Investigation playbook guided this read-only review.
throughput checkpoint: n/a, read-only investigation

## Production findings

1. The 32-entry cold endpoint array adds 128 bytes of explicit source scratch;
   it does not expand ClauseState or allocate per region. Entries are reset on
   every preparation. Global and exhausted clauses have max_doc coverage. The
   original clipped exclusive endpoint expression is retained for full blocks,
   tails and the physical address space, including deletions.
2. Tentative zero cleanup is virtual. A real first unread doc outside tight_hi
   cannot contribute to that tight interval, but its raw selected bound survives
   for possible later contribution after expansion. A stale decoded head below
   lo is never used to justify absence. Empty tight certificates skip only their
   original endpoint and do not propose a wider interval.
3. A nonempty old essential suffix proposes exactly one endpoint minimum. Every
   crossed term with a real doc below final_hi receives its already-admitted
   global maximum before the full sequential prefix and final partition are
   recomputed. Terms whose selected endpoint covers final_hi retain their raw
   bound; real docs at/beyond final_hi get zero. This prevents both old-zero reuse
   and retaining a fine-block certificate over a later stronger block. Equality
   of selected_end and final_hi is sufficient for the half-open range.
4. The complete final partition is frozen before cursor reconciliation/replay.
   An empty final partition skips under the rebuilt wider proof. There is no
   endpoint feedback loop or repartition while an owned batch is pending.
5. Independent byte comparison confirms the accepted 11 persistent count/exact
   f64 sum, from-zero generic observer, all-certified early exit and density
   policy remain unchanged (3656 bytes; SHA
   `07136e553670fe6e6181cb9c52709a5856c83af4edcc00a8ae18073abc83302e`).
   Complete essential reconciliation through replay/callback/region retirement
   is also byte-identical to HEAD. My span excludes one trailing newline and is
   4233 bytes, SHA
   `63086168e3a7fbc768ca8d6529189aacc2d736d1c414c9d00b5796741462093e`;
   the adapted receipt includes that newline (4234 bytes), so its different hash
   is a span convention, not a source discrepancy.
6. Exceptional/signed/nonfinite admission still falls back before shallow
   metadata mutation. The existing optimized domain assumes callback thresholds
   do not decrease, with the existing debug assertion. The new preparation does
   not establish a broader callback/API contract. Literal incoming-order f64
   publication, strict final comparison and collector-owned threshold updates
   remain in the unchanged replay block.
7. A final sole essential is the final preference rank and was in the old
   nonempty suffix. Its selected endpoint therefore limits the proposal; it
   cannot consume multiple full blocks inside this one certificate. The corrected
   fixture claims match this geometry. A promoted old optional on the scalar
   multi-essential route can cross its own fine blocks under a global bound.

## Oracle and fixture findings

The enclosure oracle enumerates literal leaves from untouched cloned scalar
scorers, then compares each relevant unread leaf to the final observed bound.
It is independent of the endpoint/global replacement arithmetic, rather than
recomputing that arithmetic as its expected answer. Complete callback traces
are independently compared to BufferedUnionScorer/SumCombiner, including raw
score bits and dynamic thresholds. Mutation observations additionally check
seek/advance doc/TF/raw bits, contiguous batch docs/raw score bits, and the next
authoritative head/TF against those canonical postings.

The old-zero legacy witness requires doc300 inside expansion, actual global
replacement/promotion, a doc428 literal leaf exceeding the saved old block bound,
and actual scalar advance427 to428 with TF1000. It consequently cannot pass merely
because a nominally global bound was mislabeled as local. The real native DEFAULT
RAM witness likewise requires the crossed doc150 leaf to exceed beta's preserved
first-block bound and exact replacement by beta's global maximum, with promotion.
The native profile matrix checks enclosures/exact traces without asserting an
unobserved private transform policy; the nondefault legacy case explicitly checks
the existing global fallback. These are correctly distinguished.

The ten new families retain genuine n3/n32, overlap/disjoint streams, full/partial
batch lengths, the exclusive head382, stale full selection/reconciliation, tail
and EOF, strict midpoint neighbors/incoming orders, configured safe bounds,
simulated deleted hits, real alive-bitset rejections and public TopDocs. The real
public collector test requires actual observed widening, avoiding a merely
fallback-backed winner assertion. Zero/negative/infinite/NaN boost fallbacks must
produce no new preparation/cursor/skip evidence.

The two changed pre-existing density fixtures preserve the original behavioral
observations and canonical trace comparisons. Their finite sparse first blocks
force later preparations that legitimate widening of the former two-posting
tails would otherwise eliminate. The accepted 11 fixture grounds ranks/bounds
with the real scorers and requires all three states at floors0/2160/3891;
it does not relax the cache/density assertions. Their realizability still needs
the root-owned test run.

## Limits and performance risks

- No build, test, worker, profile, timing, browser or compression job was run by
  this reviewer. No source edit, child, commit or push was made. Only this report
  was written outside the Git source tree.
- Batch observations directly prove emitted doc/score bits and retained head/TF;
  they do not expose emitted batch frequencies or positions through a new API.
  The unchanged reader's existing batch/position/frequency fixtures own that
  evidence. The scalar mutation observations directly check TF.
- Literal enclosure observations cover the exercised unread ranges and admitted
  fixture domains, not a universal proof of all external scorers or policies.
- Looser global replacements can promote dense clauses and increase scalar
  candidates despite reducing region preparation. Each nonempty tight suffix
  scans endpoints; actual expansion rebuilds bounds/prefix/partition. There is
  no source-based speed prediction, RSS advantage or binary-size claim. Frozen
  paired latency/self-controls and per-query losses must decide acceptance;
  adverse query920 remains relevant.

No requested production correction or test-oracle blocker remains from this
static review. Keep this exact source identity tied to the runtime receipts.
