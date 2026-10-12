# Attempt 03: independent actual-diff review

Reviewed the complete five-file diff in the current hillclimb checkout, the
implementation note, relevant block loading/seek/advance and position-prefix
code, the existing scalar BM25 expressions, and all four added fixture families.
Read-only: no production edits, heavy jobs, child tasks, or commits.

**No blocking correctness finding from source review.** Runtime gates remain
root-owned; this review makes no test-pass or performance claim.

* **Cursor ownership:** SegmentPostings owns cur and moves at most one real
  decoded suffix. Shallow unloaded selection reconciles through the actual
  BlockSegmentPostings owner. Full-block consumption advances and loads the
  next block; clipped consumption retains its first unread slot; tail end
  points at existing TERMINATED padding. No sentinel enters a scoring batch.
  ClauseState updates its mirror from the actual scorer exactly once afterward.
* **Positions/frequencies:** no cursor or position offset is exported. The
  existing position-prefix cache uses block offset and cur; a new block resets
  its prefix, while forward movement within a block adds the same skipped
  frequencies as scalar reads. The positioned-index fixture checks both cases
  plus a shallow-selected tail. Basic/SkipFreq fills literal frequency 1,
  matching the scalar decoder convention rather than copying an empty slice.
* **Leaf arithmetic:** the batch gathers the existing cache entry by actual
  fieldnorm ID and uses the identical native/legacy f32 expressions and u32-to-
  f32 conversion. No alternate algebra, FMA request, approximation, saturation,
  or changed cache computation appears. The all-256-norm-ID raw-bit fixture
  covers both modes, all three parameter profiles, signed/zero boosts, and
  frequencies around the integer-to-float precision boundary and u32 maximum.
* **Buffered replay:** the driver cursor means first unread, while retained
  arrays own the current replay doc/leaf. The region partition and maxima remain
  fixed until every buffered row is processed. Candidate publication and
  optional probing share the old scalar logic, read the current threshold after
  each callback, and fold contributions in incoming order. No actual driver
  score is read from its advanced cursor and no second advance occurs per row.
  A rising threshold merely permits additional conservative rejection; it does
  not rebuild the region or erase already-consumed buffered hits.
* **Region boundary:** copied docs are strictly below hi. Advancing/loading the
  driver past hi cannot invalidate already retained exact leaves or optional
  maxima, since the metadata certificate is immutable during replay. After
  replay ends, the next ordinary region rebuild sees the actual first unread
  doc. Empty clipped/exhausted batches exit without hiding buffered rows.
* **Scope/readability:** only an exactly one-clause essential suffix uses the
  new path. Two-term routing, global density policy, local ordering, region
  widths, and threshold repartition are unchanged. The extra helper layer owns
  mirror movement rather than forwarding unrelated parameters. Shared optional
  pruning/publication avoids maintaining two numerical kernels. Four bounded
  arrays add 2,048 bytes and no allocation; unused-path initialization, frame
  size, copying, and generated code growth remain measurement questions.

## Coverage and limits

The new OR fixture observes an actual 128-hit batch, compares the complete
callback raw-bit trace against canonical BufferedUnionScorer, and asserts
literal winner docs while thresholds rise inside the batch and across later
blocks/tail. Its incoming order differs from pruning order. The collector
rejection variant verifies that rejected early high-score hits do not change
the threshold for later buffered candidates. Literal posting/frequency tests
cover clipped reads, gaps, full blocks/tails, mixed scalar moves, and terminal
empty reads; positioned reads cover real serialized ownership invariants.

The OR fixture itself uses legacy weights and simulated collector rejection.
Native/configured end-to-end evidence still comes from root's frozen gates;
real deletion behavior relies on the unchanged collector and broader existing
coverage. Exceptional float boosts are covered by existing pre-movement OR
fallback tests rather than the new batch leaf fixture. The batch helper assumes
its caller's floor agrees with any prior forward shallow selection; the actual
OR caller satisfies that contract. Future reuse must retain it.

The implementation note describes the author's original separate checkout and
base, whereas this review is of the applied diff atop the current accepted
two-term commit. Preserve that distinction in trial provenance; do not present
the note's original worktree/base as the measured current build identity.
