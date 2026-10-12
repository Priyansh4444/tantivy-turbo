# Attempt 23: reuse the exact threshold cutoff for global and local bounds

Source-only, uncompiled and unmeasured. The isolated worktree is `search/tantivy-or-cutoff-all-bounds-oct12`, branch `perf/or-cutoff-all-bounds-oct12`, based on accepted22 commit `3dff75c5cfddee6dab50c144839b4cec645d9dca`. No commit was made and neither root nor production source was edited. Read the complete decision trail through accepted22, full all-bounds grounding and actual parent source/tests before editing. Root alone owns runtime work.

## Mechanism and exactness

The existing one-f64 cutoff is reused for the global certificate's ordered greater comparison and local partition's ordered at-most comparison. `optional_cutoff` becomes `threshold_cutoff` to describe its widened role. Initialization and the sole refresh immediately after every threshold assignment are otherwise identical. The optional predicate, canonical incoming-order score, strict callback publication, sequential prefix construction, density policy, certificate sum/count, original term count, region choice and cursor movement remain unchanged.

One inline `score_is_greater` method evaluates `(sum * self.factor) > cutoff.0`, with the same admitted-domain assertion as the existing method. It uses the original rounded multiplication. It never complements the at-most predicate. The cutoff constructor and existing at-most method remain unchanged, and there is no added cache, refresh, allocation, array, public API or index-format change.

Accepted22 established the exact edge L for which the old bit-successor-and-f32-narrowed bound is <=T exactly when the unchanged post-multiply product p is <=L. For non-NaN T, both old bound and p have total numeric ordering, so old>T exactly when p>L. Equality remains at-most, including the even/odd f32 midpoint and MAX overflow boundary. For a genuinely negative T the cutoff is -infinity, making greater true; +infinity makes it false, including old narrowing overflow to +infinity. Either signed-zero T retains accepted22's finite positive cutoff. A NaN T maps to NaN, so both ordered predicates are false. Negating at-most would incorrectly make greater true for NaN and is therefore excluded.

Global next_sum and every local prefix value retain their original expressions and original-n factor. The local replacement sits inside the existing `first_essential < num_terms &&` short circuit, so it cannot evaluate prefix[num_terms+1]. The cache is current at region transitions because accepted22 refreshes immediately after the kernel's sole live threshold assignment; no second refresh path was introduced. Callback NaN still violates the existing debug monotonicity contract; numeric tests verify release-compatible ordered predicates without claiming that schedule is valid under the contract.

## Independent checks in source

Both changed live sites have cfg(test) `assert_eq!` against the literal old helper using the current threshold. These assertions remain active in optimized test builds; ordinary release has no oracle instrumentation. The optional site's accepted22 debug equality assertion is unchanged except for the cache variable name. The independent generic from-zero certificate computation, exact sum-bit/count assertions, global_appends identity and all thirteen OR tests/helpers remain byte identical to parent22.

The shared numeric check now tests both `small.score(sum) <= threshold` and `small.score(sum) > threshold` as separate literal oracles. Thus all four accepted22 cutoff families now exercise both methods over all admitted term counts, every threshold class, signed zeros, subnormal/normal/max sums, every positive f32 exponent boundary, rounded products neighboring each cutoff and deterministic raw-bit samples. The four original generic/small-bound test families and their helpers remain byte identical.

The direct-product family independently checks ordered greater against the old literal successor/narrow expression for zero, -zero, pred2, pred1, midpoint and its next two f64 neighbors. Explicit greater witnesses at L and next_up(L) retain separate expected false/true results for every generated boundary, including MAX. Named NaN witnesses require both old and replacement ordered predicates to be false for either sign and quiet/signaling payload class. An old bound overflowing at the MAX midpoint is separately required to equal +infinity and to remain at-most, rather than greater, against a +infinity threshold. Expected greater is never derived by negating expected at-most.

The numeric and OR test names are unchanged: eight numeric tests and thirteen OR tests. Existing fixtures retain canonical callback doc/raw-score-bit traces, scheduled exact-bound neighbors, negative/zero/+infinity initials, density rejection/widening/rejection, full128 owned batches, unchanged deletion callbacks followed by raises and independent buffered-union oracles. No new observer or unrelated fixture was added.

## Static artifacts and remaining checks

`rustfmt --edition 2021` on the two edited files and `git diff --check` completed. A source-metadata script verified exact parent commit, exactly two edited tracked paths, no untracked files, identical OR test tail, identical original numeric helper/test tail, unchanged independent generic certificate oracle, unchanged test names/counts and unchanged term_scorer.rs. The static receipt records those hashes and the authentic `git diff --binary` patch hash.

Patch SHA256: `7f673aea60d7ce7248b06042b098e3c096a175d1d8f3b97ad066f8a4595f4ad2`.
OR source SHA256: `e8936f114df5bb4c09b27960afc126832f07fab73863b061ab9e3d55a6be23bb`.
Score-combiner source SHA256: `67a8070a9abad0d107eaebfc8ef731793b2d0f016e5216afc98d1b6ff3ac93b7`.
Receipt SHA256: `96e4e45cef8e94c39afc256e167b3a14738f8ce3423731ee2e52df9f9f1f101a`.

Root must review the full actual diff, run targeted OR/numeric tests, all three1227 exact indexed profiles, the frozen pilot and justified full controls/negative gates. No builds, tests, engines, profiling, disassembly, Python, compression or browser operations ran here. Failure histories and per-query losses must remain in the trail.

This extension removes successor/bit-transfer/f32 narrowing from two additional region-time decisions in source, while reusing the existing cold cache cost. Generated code and elapsed-time benefit remain unknown. Inlining, branch scheduling and register/code placement may harm queries; only the frozen built-candidate experiment can establish acceptance. No actual-port speed win, memory win or complete Lucene API parity is claimed.
