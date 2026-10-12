# Attempt 15 independent source review

Verdict: **no source correctness blocker found; proceed to root-owned runtime gates.** This is a static review, not an acceptance or performance result.

Reviewed the full 362-line `attempt-15-full.patch`, full `attempt-15-implementation.md`, static receipt, and complete actual isolated OR source. Parent HEAD independently verified as `43f06833a325d2a18c03bd7e4fa65f2f27944292`. Actual held source SHA-256 independently verified as `602867a5085c86c6923e15234a6ce381519b2834af6c627ef910db393e6d3477`. Tracked status contains only the assigned OR source modification.

Throughput checkpoint: n/a, read-only investigation.

## Cache validity

The numerical `certified_global` count is distinct from the per-region effective `globally_optional` count. Only the latter becomes zero when density rejects widening. The persistent prefix minimum and suffix winner are consistently keyed to the former; a density rejection cannot reset or silently broaden the cached suffix. The numerical prefix only extends under the existing monotone-threshold contract, and its minimum is updated precisely after successful numerical admission, including admitted clauses already at EOF.

Suffix costs and preference ranks are immutable. Actual cursor positions can advance or become permanently TERMINATED, and the numerical suffix can only lose ranks as the prefix grows. Consequently a retained live winner remains a maximum until it either reaches actual EOF or its rank becomes strictly less than the numerical prefix count. The two invalidation conditions cover these events. Equality between winner rank and prefix count is valid membership and correctly retains the winner. A tied nonwinner reaching EOF cannot invalidate the retained maximum. Rebuild picks the first live maximum tie, which agrees with the test witness and does not alter the old value-only density predicate.

`live_max_ready` distinguishes uninitialized state from a valid empty suffix. `num_terms` is guarded before every clause indexing operation; the test-only `.then` closure is lazy. An empty suffix cannot gain a live member or expand again under these cursor/prefix invariants. Its zero maximum yields the old density result. The documented immediate terminal skip is consistent with unchanged local preparation: the certified prefix has global coverage, EOF suffix bounds are zero, and the full sum fits the current threshold. No second empty-cache witness is required for an unreachable active boundary.

The prefix is nonempty whenever its minimum is consumed, so `u32::MAX` initialization is a correct min identity. Converting to u64 before doubling the u32 maximum prevents overflow. The minimum comparison is equivalent to the old all() density test, including exhausted prefix costs. No float arithmetic, literal score, callback threshold, or pruning-bound construction is changed by this mechanism.

## Cursor and replay preservation

No mutation is added to scorer ownership or seek/advance/fill_scored_batch wrappers. Metadata selection cannot mark a winner EOF; validity depends solely on the mirrored actual cursor. The next drained boundary observes EOF from any unchanged real cursor mutation.

An independent Git-object comparison verifies that the entire span from `let mut hi = max_doc` through region preparation, cursor movement, optional probes, incoming-order f64 score replay, callback publication, and region retirement is byte-identical to accepted11: 6,999 bytes, SHA-256 `b93646efb4641fbfa4bc1a0ec00c42a2f28c9af2306be3e938669b3741168421`. All original fixture source bodies through EOF are also independently byte-identical. Source-only scope therefore does not stack metadata-range or contribution-mask mechanisms.

## Test independence and limits

The test observer independently recomputes the old filter/map/max suffix value and old prefix all() density predicate from actual clause records at every nonzero numerical-prefix boundary. Expected values do not come from the cached minimum, rank, or effective choice. The separate from-zero global-prefix observer independently checks the prefix minimum, including the all-certified early return. Recording actual ordinal EOF bits lets fixtures distinguish tied nonwinner EOF, winner EOF, and exclusion of a still-live winner.

Two fixture families reuse unchanged full canonical callback-trace tests. The new n5 fixture grounds real costs and preference order, verifies independent threshold neighborhoods, compares its entire scheduled trace with BufferedUnionScorer, and asserts distinct physical witnesses for tie retirement, winner retirement, equality, and exclusion. The initial exhausted-suffix fixture compares the canonical trace and validates the precise empty sentinel and one terminal observation. Test-only TLS observation is absent from release.

Path assertions remain unexecuted in this review. Root must establish that the four added fixtures realize their claimed geometry and run the existing fixtures, native build, exact default/configured gates, and frozen latency controls. Static equivalence does not establish a speed or memory gain: the cached winner branches and prefix-min updates can cost more than short scans, and the 13 logical bytes do not determine stack layout or RSS. No universal Lucene API parity is claimed.

No edits to source, builds, tests, workers, benchmarks, profiling, compression, browser activity, commits, or pushes were performed. Only this review report was written.
