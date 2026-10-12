# Attempt 18: exactly-two-essential head minimum

Source-only candidate held in `/home/pronsh/Coding/playground/search/tantivy-or-two-essential-min-oct11`, branch `perf/or-two-essential-min-oct11`, parent `43f06833a325d2a18c03bd7e4fa65f2f27944292` (accepted 11).

Frozen source: `src/query/boolean_query/or_maxscore.rs` SHA-256 `252c08f4e6401bd5b1c497fefe6e3885c2a10bf233ecd87085ddcca7ef514819`.

Frozen full parent-relative patch: `attempt-18-source.patch` SHA-256 `fbb5f88a6b6fea9c321e9efa9e3518db7269ffa7047d3af1b96cbafb106704e4`. Scope is one tracked file, 121 inserted lines, zero removed lines. Every accepted-11 line, fixture, corpus, and oracle remains unchanged. No other tracked or untracked files are in the candidate tree.

## One production mechanism

The certified partition is already fixed throughout the candidate loop. Compute `two_essential = num_terms - first_essential == 2` once per region beside the existing single-essential predicate. In that case choose `clauses[first_essential].doc.min(clauses[first_essential + 1].doc)` directly; otherwise retain the complete original variable-length iterator minimum. The single-essential owned-batch route is untouched.

The exactly-two predicate certifies both indices exist: `first_essential == num_terms` was handled earlier, and equality with two means `first_essential + 1 < num_terms`. Mirrored docs retain their existing first-unread meaning, including `TERMINATED`. Choosing the minimum is identical for equal heads, distinct heads, and either/both exhausted heads. The existing `doc >= hi` break still owns interval termination.

There is no new hot record, allocation, sorting, heap, score unrolling, helper layer, prefix arithmetic, threshold policy, cursor update, or retirement rule. Only one region-local boolean and local doc operands are added in release. Actual essential score extraction, optional probing, incoming-order f64 publication/final f32 cast, callback, scalar advancement, and coherence checks are byte-identical accepted-11 text. All geometry and metadata certification remain accepted 11; other attempts 12–17 are not stacked.

This differs from rejected 07 (repartition), 08 (bitmap/exact-leaf matrix), 09 (per-essential batches/heap), and 10 (unique-lead span/replay). It removes only the two-element case's variable-length minimum; its usefulness and generated machine-code effect are unknown until root's frozen runtime experiment.

## New independently grounded fixture family

One new test, `two_essential_min_preserves_equal_disjoint_exhausted_and_dynamic_traces`, uses real serialized scorer fixtures. At n=3 and n=32 it creates a dense weak prefix, a 257-posting even-address stream with a later high-TF leaf, and an odd-address stream with equal heads at 0/128/256 plus a high-TF tail at 700. Both incoming orders are exercised.

A cfg(test)-only observer is entered only in the actual exactly-two branch. Fixed-threshold assertions require observed equal and disjoint live heads, a finite candidate at 700 with one exhausted head, and the both-exhausted terminal selection. An independently sorted two-element array checks each observed candidate minimum. Full callback documents and score bits are compared against `BufferedUnionScorer` with canonical `SumCombiner`, independently of the new minimum route. Later documents 512 and 700 must remain returned in the fixed traversal.

The same family compares Top-1/Top-10 growing traces; derives a strict tie at document 2 from original-order literal leaf scores and tests predecessor/equality/successor thresholds; and compares collector-owned deletion behavior by withholding threshold growth for documents 0 and 128. The deletion comparison preserves every callback and score bit, requires later equal-head 256 and tail 700, and checks strict document order. The test observer, alias, and TLS allocation do not exist in release.

## Static verification and limits

Read the full accepted-11 OR implementation, actual BooleanWeight dispatch, serialized TermScorer fixture construction, the complete resulting source (including a separate read of the middle range omitted by the output cap), and full final delta. Read workflow hillclimb and decision.tsv before the attempt. `rustfmt --edition 2021` and `git diff --check` completed; strict static script checked parent and sole changed path before exporting the patch. Source remains held unchanged.

No builds, tests, Python, benchmark, profiling, compression, browser work, commits, or child tasks were run. No runtime correctness, speed, binary-size, or memory-layout claim is made. Root owns targeted tests, exact score gates, latency controls, and any keep/revert decision.

If attempt 16 becomes the accepted parent, graft only these additive production/test/observer hunks into its retained shared replay. Do not replace its entire OR file with this accepted-11-based file. The two-head production hunk depends only on the unchanged fixed-partition replay contract, not on attempt 16's geometry or TermScorer range helper. Freeze and re-review the actual rebased source before measuring it.
