# Attempt 08 independent source review

Reviewed against accepted parent `05835b971452bd5bda75037cec2925b7ba6785bf` in
`/home/pronsh/Coding/playground/search/tantivy-or-multi-bitmap-oct07`.
Current changed-file SHA-256: `c4f7402f22938906d111472b690285074a01ea2e3a224f5a0102773f1855abf0`.
Initial reviewed SHA-256: `40e9e0b194603b73907f7cc462650aa8b34ff19bb7e74c10fed1798d80869209`.
The complete actual diff contains only `src/query/boolean_query/or_maxscore.rs`
(676 additions, 36 deletions). The initial review was read-only; the subsequent
root-authorized one-line fixture correction is recorded below.

throughput checkpoint: n/a, read-only investigation

## Overview

No production source-level blocker found. The change follows the selected bitmap
design, preserves exact incoming-order score publication and current-threshold
callback behavior, and separates authoritative first-unread cursors from retained
replay rows. The two strengthened fixtures address the previous discriminating
coverage gaps. This is a source judgment, not runtime or performance acceptance.
No build, test, profiling, timing, compression, commit, or child agent ran here.

Root subsequently reported a targeted-test compile failure, E0689 at line 918:
`current.max(score)` could not resolve the float type of `let mut current = 0.1`
in the new simulated collector fixture. At root's explicit request, this reviewer
changed only that fixture declaration at line 914 to
`let mut current: Score = 0.1;`. Reconstructing the previous file by removing that
single annotation reproduces the initial reviewed SHA exactly. `git diff --check`
passes. No production code changed, and the proof/coverage findings below remain
unchanged. The failed build receipt remains root-owned historical evidence; no
successful compile or executed attempt-08 result is inferred from this correction.

Read the complete actual diff, `design-multi-synthesis.md`, and
`attempt-08-implementation.md`; inspected the accepted parent and the unchanged
owned batch, posting cursor, scalar/batch BM25, upper-bound, independent union,
and pruning callback implementations. The workflow principle **Prove It Works**
shaped this review: inspect the real diff and fixture assertions, and leave
execution evidence explicitly pending rather than treating source intent as proof.

## Key concepts and proof

- **Complete gather before replay.** `ExactLeafWindow::gather_window` owns the
  minimum, clipped physical end, all essential fills, and scatter (OR source
  lines 76–147). It is called only when the bitmap is empty (403–416). Each
  clause fills repeatedly until its first-unread doc is at least `end` (95–124),
  which matters because the existing posting batch consumes only one decoded
  suffix per call (`src/postings/segment_postings.rs:29–69`). There is no
  publication of a partially assembled row. The progress assertion covers both
  emitted batches and empty shallow reconciliation.
- **Cursor ownership and exhaustion.** The wrapper updates `ClauseState.doc`
  immediately after every batch fill (OR 32–43). The existing batch reconciles
  shallow selection through its owning block cursor before extracting docs;
  its full-block advance loads the next block, and partial tails leave the first
  padding sentinel as the unread position (posting source 37–67;
  `src/postings/block_segment_postings.rs:359–374,500–505`). Pending bitmap rows
  govern replay at OR 403–416, even when every actual scorer is already beyond
  `hi` or exhausted. Region transition occurs only after drain (450–457).
- **Exact leaves and publication.** The existing batch BM25 uses the same f32
  expressions, frequency conversion, and cached normalization as scalar scoring
  (`src/query/bm25.rs:569–615`; `term_scorer.rs:51–68,278–282`). Each emitted
  leaf is stored at its original ordinal, and the pruning-known f64 sum is added
  in the unchanged fixed preference order (OR 94–120). The separate final fold
  visits the incoming-order row, sums f64, and casts once to f32 (436–440),
  matching the accepted publication convention. No pre-summed score replaces
  that fold.
- **Dirty-row isolation.** A newly visible bit zeroes the entire row, including
  former optional columns, and resets its known sum before any leaf is added
  (112–120). The row cannot be touched again by a new window while pending rows
  remain: gathering is empty-only, and replay clears bits in strictly increasing
  offset order (149–162). Untouched dirty storage remains invisible.
- **Current threshold and certificates.** Every replayed row uses the current
  threshold in the unchanged optional bound/probe loop and strict publication
  check (418–445). Decoder advance cannot alter the already copied region
  maxima/prefix, `hi`, or suffix partition. No threshold-driven repartition or
  pending-window expiry was introduced. The existing `ScoreSumUpperBound`
  remains responsible for preference/grouping order error during pruning;
  score publication does not use that allowance.
- **Bounds, gaps, and allocation.** `end=min(hi,saturating_add(base,128))`
  limits offsets to 0–127 (86–91,105–107), including near the DocId limit.
  Empty reconciliation recalculates the next actual minimum instead of scanning
  physical empty windows. One lazy matrix allocation occurs at first multi
  dispatch (60–67,245,403), with no production per-window allocation. All
  observer allocation is cfg(test). Existing single-essential batching remains
  at 375–401,447–449; eligibility/fallback and region policy are unchanged in
  the actual diff. Removing the now-unused scalar essential advance wrapper is
  consistent with both essential paths owning movement through batching.

## Fixture review

`check` records every callback's doc and score bits and compares against the
independent `BufferedUnionScorer` path (473–520). That union iterates scorers in
incoming order, scores scalar leaves into `SumCombiner`, retains ascending doc
replay, and applies the same strict `score > threshold` callback contract
(`src/query/union/buffered_union.rs:48–79,126–150`;
`src/query/score_combiner.rs:64–91`; `src/query/weight.rs:49–63`).

The strengthened sparse-optional case is discriminating (OR 671–706). Its
observer asserts actual bases 0/128, the same suffix [0,2], and row offsets 0/7.
Both essential columns overwrite offset 7 in the second window, while optional
ordinal 1 matches only doc 7. Therefore doc 135's literal `0x40400000` cannot be
rescued by optional overwrite if zero-on-first-touch is removed: the stale
optional would publish `0x40401000`. The independent full trace and all four
literal publications are asserted. The preceding case separately exposes stale
essential columns when their next-window match is absent (615–669).

The strengthened five-term midpoint case is also discriminating (807–892).
Legacy TF=1 scalar leaves are explicitly checked against [.5,.5,2^-24,2^-53,
2^-53]. Each strong scalar leaf at docs 127/255/256 must exceed the highest
tested low-TF threshold, and every profile/order/threshold/limit run asserts an
actual base-0 bitmap gather containing both strong ordinals with at least two
fills. Thus a single-essential route cannot satisfy the near-midpoint test by
itself. Different incoming orders exercise the low-TF rounding boundary; the
expected publication/tie at doc 0, later high-TF callback, and complete independent
trace are checked. Native expectations use actual scalar native leaves, which
appropriately tests batch-versus-scalar preservation but does not independently
certify the native BM25 formula. The original n4 zero-threshold bitmap case and
neighbor checks remain (770–805).

Other useful adversaries are present: all actual cursors exhausted with pending
offsets spanning both bitmap words (545–565); n3/n32 disjoint/overlap and full
block/tail lengths (590–613); suffix growth/shrink with observed actual ordinal
sets (710–735); million-ID gaps and clipping (738–767); current thresholds and
collector-owned rejection (897–933); real indexed deletes, alive-bitset rejection,
pending `unread>=hi`, and real TopDocs dispatch/score bits (936–1037). Unsafe
domains additionally require zero bitmap events (1358–1387). These are source
assertions only at this review boundary.

## Gotchas and acceptance limits

No further mandatory source revision requested after the fixture annotation correction.
Root must execute the strengthened tests,
existing raw callback/default/configured gates, actual collector integration,
and performance/control gates before accepting attempt 08. In particular,
`filled` counts nonempty batch calls, not distinct clauses; the strengthened
midpoint test avoids treating that count alone as proof by also asserting both
strong essential ordinals and independently checking their real high-TF leaves.

The new native midpoint corpus uses DEFAULT parameters; configured-parameter
coverage still belongs to the existing root-owned frozen gates. The printed
layout payload is array accounting, not total allocator usage, query stack size,
or RSS. Sparse/disjoint workload speed and effects on other dispatch paths must
be measured; this review makes no speed or memory victory claim.

This worktree remains based on the named accepted parent and is not integrated.
If grafted onto accepted experiment 06, preserve that experiment's conservative
bound import/constructor, inspect the actual merged diff, and re-run the exact
semantic gates. The reviewed SHA certifies only the current prepared source.
