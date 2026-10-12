# Attempt 22: exact optional-bound threshold cutoff

Source-only, uncompiled and unmeasured. Built against accepted11 commit `43f06833a325d2a18c03bd7e4fa65f2f27944292`, without attempt21. Read the complete decision trail through attempt20 and the full exact-cutoff grounding before editing. Root owns runtime checks.

## Mechanism and scope

A `ScoreSumThresholdCutoff` with one private f64 field prevents supplying a widened f32 threshold by mistake. Initialize from the actual threshold after numeric eligibility; refresh immediately after every callback assignment, including unchanged thresholds. Only the optional inner bound predicate changes. Its sum and original-n factor multiplication remain exactly the same. The old score helper, global certificate, local partition, final score arithmetic, strict publication, candidate/cursor order and callback logic remain.

The cold constructor classifies genuine negative thresholds to -infinity, +infinity to +infinity and either-sign/payload NaNs to NaN. Signed zeros use the finite nonnegative construction. Production never divides by the factor. No allocation, array, persistent cross-query cache, ABI, index format or public API changes. The source-level extra state is one f64 wrapper; compiled placement/size is unmeasured.

## Exactness argument

For the admitted finite numerically nonnegative sum, including either signed zero, preserve `p = sum * factor` with the unchanged original-n factor. Positive finite products make the old helper's bit increment an immediate f64 successor q, then it narrows q to f32 and compares against the live threshold T.

For finite nonnegative T, normalize its bits B only to clear a signed-zero sign. Let U be the exact next f32 endpoint widened to f64; for MAX, substitute exactly 2^128 (`0x47f0000000000000` in f64). The adjacent endpoints have at most 24 significand bits, so their sum and midpoint M need at most 25; `(f64(T)+U)*0.5` is exact throughout the f32 range, including zero/subnormal and overflow transitions.

When B is even, round-to-nearest ties-to-even narrows q to a value <=T exactly when q<=M. Accounting for q=successor(p) gives p<=pred(M). When B is odd, a midpoint narrows upward, so q<M and p<=pred(pred(M)). Thus `L = f64::from_bits(M.bits - 1 - (B & 1))` and the replacement is exactly `p <= L`. For MAX, the odd midpoint chooses overflowing 2^128 and becomes infinity; the two-predecessor rule excludes that tie correctly.

At +0 and -0 products, the old integer increment yields the corresponding signed least f64 subnormal; its f32 narrowing is signed zero. Both signed zeros compare <= the same positive finite L, exactly as old signed-zero comparisons. All admitted products are finite and numerically nonnegative. Their old narrowed scores are nonnegative finite, positive infinity or signed zero: hence negative thresholds always reject the <= predicate, +infinity always passes it, and NaNs make either ordered comparison false. Mapping to the classified f64 cutoffs is therefore exact for every threshold class.

The constructor is refreshed immediately after the only live threshold assignment. A cfg(test) debug equality assertion compares each replacement decision against the untouched old helper and current threshold, so region/owned-batch callback changes exercise cache consistency directly. Callback NaN still violates the unchanged debug monotonicity contract; release-compatible predicate tests cover its ordered-comparison mapping without claiming that contract violation is valid runtime behavior.

## Meaningful source tests

Four additive numeric tests keep `SmallScoreSumUpperBound::score` as the expected predicate. The all-class test covers every n3..32, original-factor bits, either signed zero, negative smallest/ordinary/max finite and -infinity, +infinity, both signs of signaling/quiet NaN payload patterns, valid f64 subnormal/normal sums and the admitted maximum.

The direct-product test independently constructs midpoint via upward spacing and compares p<=L against the actual old bit-successor-and-f32-cast expression. Every positive f32 exponent boundary and its neighbors, zero/minimum subnormal, largest subnormal/minimum normal and MAX are included. Explicit pred2/pred1/M/next1/next2 witnesses prove the even/odd tie distinction; both parity classes must be observed. The production 2^128 literal is separately checked against powi(128).

The rounded-product test covers every n3..32 and all those threshold transitions. Division by the unchanged factor only generates candidate test inputs, then eight neighbors on either side of L/factor are compared against the old helper; both Boolean outcomes must occur at each boundary. The fixed-seed test adds 2048 diverse valid nonnegative raw-bit f64 sums per n, raw-bit thresholds and old-score neighboring thresholds. This is deterministic diversity, not exhaustive enumeration of the f64 domain; the proof supplies domain coverage.

All 13 original OR tests/helpers remain byte identical. The existing single_driver_batches_replay_every_dynamic_threshold_exactly fixture already forces a full128 owned batch and has unchanged deletion callbacks followed by raises, with independent canonical callback doc/raw-score-bit traces. No duplicative OR fixture was added. All four original numeric helpers/tests remain byte identical after their existing start marker.

## Required next checks and risks

Only rustfmt and git diff --check ran here. Root must run all eight numeric tests, 13 OR tests, all three1227 exact indexed profiles, the frozen pilot and any justified full controls/negative gate. Root must retain failures before corrections. No compiler-derived shape or speed claim is made.

The cutoff construction may add callback overhead, register pressure or code-layout changes elsewhere. Instruction sampling motivated avoiding repeated f64 bit successor and f32 narrowing, but sampled period does not quantify removable work. Only built-candidate disassembly can establish generated code, and frozen balanced timing can establish a win.
