# Attempt 05: outline only the fixed two-term region kernel

Worktree: `/home/pronsh/Coding/playground/search/tantivy-or-outline-oct07`.
Branch: `perf/or-outline-oct07`.
Base: `22f4a270926d513b2641cd44c5445008ecf3b276` (accepted fixed two-term local regions).
Changed source SHA256: `cc21dc468a3d8733af77bded589ca1572d84a3e81ac1456496997fbc1ef7a01f`.

## Actual diff

Only one production line is added in `src/query/boolean_query/block_wand_union.rs`:

```diff
 /// Local required/optional roles expire at that end; this remains an OR query.
+#[inline(never)]
 fn two_term_regions(
```

No other production file, scoring statement, numerical proof, cursor operation,
threshold policy, query routing, warmup, or existing test changes. No new tests.
Removing the attribute from the changed source reproduces the base file exactly;
`git diff --name-only` confirms this is the sole changed path.

## Grounded hypothesis, not a measured win

The caller is unchanged: `BooleanWeight<SumCombiner>::for_each_pruning` dispatches
the admitted two-term union to `two_term_or_maxscore`; that wrapper calls the
private `two_term_regions` once only after its existing 128-candidate warm phase
and only when neither global maximum selects an earlier global path.

The retained accepted attempt-01 worker at
`attempt-01/tantivy_worker` has SHA256
`fe03feaf8f0c822f5b500edab9330a41f7af9956def57d6aced6625802aff117`, matching its
`candidate-two-clean-profile-receipt.json`. Read-only `nm -C` inspection found the
BooleanWeight for_each_pruning symbol at `0x297760` and no standalone
`two_term_regions` symbol. This is consistent with the kernel being inlined under
the current optimized build; it does not itself prove instruction-cache stalls.
The retained five-case two-term profile assigns 44.15% of sampled self overhead
to that BooleanWeight symbol and has 407 samples / zero lost samples. Its limited
sample count and startup/warmup coverage do not quantify the region kernel alone.

Forcing the private kernel to remain out of line should reduce the code carried
inside the already-large BooleanWeight dispatch and may change its collateral
AND code layout. This is a code-layout experiment, not a new retrieval algorithm.
A once-per-query call cost and inhibited interprocedural optimization can instead
make the two-term fallback slower. Other routes may improve or regress due to
layout even though their source and arithmetic are unchanged. An AND improvement
without a two-term improvement is not proof the OR gap has closed.

## Root-owned verification

Existing independent exact callback/raw-bit fixtures and all default/configured
frozen result gates remain the useful correctness gate; a redundant test asserting
an inline attribute would add no retrieval evidence. Root should verify the new
release has an outlined kernel symbol/call, inspect BooleanWeight symbol size and
binary text-size changes against this exact accepted binary, then measure the
full paired ruler with both self-controls and retain all AND/non-OR controls.
No measurement exclusions or acceptance thresholds change.

Reject if the actual binary does not change as expected, if exact result gates
fail, or if speed changes are absent/regressive beyond the observed self-control
spread. Preserve source and binary identity before any subsequent trial. Do not
combine this attribute with attempt-02 local order, attempt-03 warm handoff,
attempt-04 batching, or another outline attribute before independent measurement.

## Checks actually run

- Read source caller/kernel and retained accepted profile/receipt.
- Read-only `nm -C` of the hash-matched accepted worker.
- Exact source restoration comparison and sole-changed-path check.
- `git diff --check` passed.

No builds, tests, profiling, benchmarks, commits, or child agents were run.
