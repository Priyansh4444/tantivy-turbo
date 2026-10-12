# Attempt 22 independent source review

No concrete production correctness blocker or invalid additive-test domain was found in the final source. This is source review only; root's numeric/OR tests, three indexed exact profiles and controlled timing remain required. No correctness runtime, code-generation or speed result is inferred from this review.

Throughput checkpoint: n/a, source-only independent review.

## Reviewed artifact and scope

Read the complete actual two-file diff and final authentic `attempt-22-full.patch`, SHA-256 `4594ba2e4096973b495e1fa019b637bdd1b833b08f540ad62ce9969d140c40e9`, against accepted11 `43f06833a325d2a18c03bd7e4fa65f2f27944292`. Final OR SHA is `590f6a2b0f493fe2c0b6ab7c17a41ab610c95e50a76c8f4a6ff42f44e544806e`; score combiner SHA is `2679f2aab4434a4f4b75456185fee01d03506b5d40fb95f519e135664068997e`. Read final static receipt/report, actual bound helper and replay/callback sites, all four new numeric tests, and existing owned-batch/deletion threshold fixture. The exact-grounding proof and installed official Rust float-narrowing reference were read during grounding. Git HEAD is accepted11 and `git diff --check` passed. No author/root source files were edited. No compiler, test, Python, benchmark, profile, browser, compression or engine execution ran here.

The actual diff contains no attempt21 accumulator and no OR-test hunk. All13 original OR fixtures/helpers therefore remain byte-identical in the final patch. The four original numeric tests/helper tail is also unchanged; only the numeric test import and an additive prefix of four families change. There are13 OR and8 numeric tests proposed, not reported passing tests.

## Exact cold constructor

`threshold < 0.0` catches every genuinely negative finite threshold and negative infinity, while leaving -0 for normalized finite construction. Positive infinity returns positive f64 infinity. Either sign/payload of NaN fails both prior comparisons and returns a NaN cutoff, preserving ordered `<=` as false. Negative finite/infinite thresholds map to negative infinity because all admitted products are finite and nonnegative, possibly -0; old narrowed bounds are nonnegative/infinite or signed zero and cannot be <= a genuine negative threshold. Positive infinity accepts every old bound, including overflowed f32 infinity, just as the finite product accepts a +infinity cutoff.

For nonnegative finite thresholds, clearing the sign bit normalizes -0 without changing numerical comparisons. The successor f32 bits are valid and finite except MAX, whose special f64 endpoint bits `0x47f0000000000000` are exactly2^128. The midpoint has at most25 significant binary digits and is exactly represented in f64, even for zero/minimum f32 subnormal, subnormal/normal transitions, exponent boundaries and MAX. Its magnitude is at least2^-150 and at most2^128−2^103, so the one/two f64 bit predecessor adjustment cannot underflow or encounter f64 subnormals/overflow.

The subtraction `midpoint.bits - 1 - (threshold.bits & 1)` accounts for both the old helper's f64 bit successor and the f32 tie parity. Even thresholds admit q=M and require p<=pred(M); odd thresholds exclude q=M and require p<=pred(pred(M)). MAX is odd and its midpoint2^128−2^103 narrows to infinity, so the special endpoint and two-predecessor rule are necessary and correct. Either signed-zero product becomes a signed minimum f64 subnormal after the old bit increment and narrows to corresponding signed f32 zero; both old and new predicates pass at either signed-zero threshold's same positive cutoff.

`SmallScoreSumUpperBound::new`, its original-n factor construction, and old `score` are unchanged. The new hot method uses the identical admitted-domain assertion and evaluates the same `sum * self.factor` once, then compares that rounded f64 product against the cached cutoff. It introduces no factor division, reciprocal, reassociation, shortened leaf count or approximate boundary. The wrapper's private f64 field prevents accidentally passing a raw widened f32 threshold through the new method's signature.

## Integration and cache validity

The cache is initialized from actual threshold immediately after numeric eligibility and original-n bound construction. The only mutation of the live threshold in the optimized kernel is `threshold = next_threshold` after the callback; the next statement refreshes the cutoff unconditionally, including unchanged threshold bits. Region transitions do not alter threshold. The only replaced predicate is optional replay's old `score(known + prefix[rank+1]) <= threshold`; its f64 sum operation/order is preserved by the local variable. Global prefix and local partition predicates remain old-helper calls.

No cursor, candidate, region, array, local/global bound or partition changes. Strict callback publication and original incoming-ordinal f64 sum followed by f32 conversion are untouched. A callback NaN still violates the unchanged debug monotonicity contract; in release, construction maps it to a NaN cutoff, preserving the old optional predicate's false result. Other threshold/publication/global behavior is unchanged rather than relying on callback validity to ignore this case.

Each cfg(test) optional predicate has a debug assertion against the untouched old helper and live threshold. This catches stale-cutoff errors under root's debug test profile and existing full128 owned-batch fixture with callback raises and deletion callbacks withholding raises. It is not a claim of instrumentation in release exact-profile workers: `debug_assert_eq!` disappears when debug assertions are disabled, and cfg(test) disappears from ordinary release builds.

## Numeric tests are independent and within domain

The expected Boolean is the actual unchanged `small.score(sum) <= threshold`, not a second cutoff formula. Direct-product tests also execute the old literal bit-successor/narrow expression, independently of the factor transform. Their midpoint input construction uses upward spacing (MAX adds2^103), differing from production's endpoint-average calculation. Explicit even/odd predecessor/midpoint rows and actual narrowed midpoint bits distinguish off-by-one errors and overflow tie handling; structural cutoff-bit assertions supplement rather than replace this old-expression oracle.

Threshold generation covers positive finite neighbors at each exponent boundary1..254, zero/minimum subnormals, largest-subnormal/minimum-normal, MAX and neighboring significands. Exception patterns include both signed zeros, genuine negative finite/infinity, +infinity, and both signs of quiet/signaling NaNs. Every n3..32 is used, with original generic factor-bit equality asserted.

The quotient L/factor appears only in test input generation. Its centers and eight neighboring f64 sums remain positive and far below32*f32::MAX throughout the generated finite threshold range, including zero's2^-150 midpoint and MAX's cutoff. Both Boolean outcomes must actually be observed around each center. This does not smuggle a production quotient approximation into the expected result.

The deterministic rawbit generator uses `state % (max_sum.to_bits()+1)` as positive f64 bits. Positive finite f64 bit ordering is monotone, the maximum is finite with exponent far below the NaN/infinity range, and the integer upper bound does not overflow u64. Every generated sum is therefore in `[+0, 32*f32::MAX]`; an explicit assertion enforces that domain. Rawbit thresholds may intentionally be negative, infinite or NaN, and old-score neighbors include zero/subnormal/MAX/infinity transitions. Fixed-seed diversity is not exhaustive floating-point enumeration; the proof and explicit transition witnesses carry that scope.

## Remaining limits

No source blocker requires another fixture or mechanism before root's existing targeted checks. Retain all failed originals before any correction. Only compiled disassembly can establish that the intended hot bit transfer/increment/f32 conversion disappears; source exactness does not imply a speedup. Cold construction on every callback, register/frame pressure and code placement may offset savings. The logical added cache is one f64, with no new allocation or format/API change; binary/frame/RSS still require measurement. Full actual-port wins and Lucene API parity remain outside this review's claim.
