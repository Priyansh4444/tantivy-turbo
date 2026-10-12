# Attempt 21: carry the regional bound prefix in a scalar

Source-only draft, uncompiled and unmeasured. Parent `43f06833a325d2a18c03bd7e4fa65f2f27944292`. Decision trail read through rejected attempt 20 before editing.

## Mechanism

Each region already clamps out-of-range clause local maxima to zero and builds the sequential f64 prefix in preference order. The old statement reloads `prefix[rank]` to add the next f32 local maximum. The new region-local `local_sum` starts at positive zero, adds the same converted local maximum in the same rank order, and writes the same prefix slot. The existing full prefix array and all consumers remain.

Accepted-11 instruction samples supplied by root singled out the regional prefix store at 0x29a664 (6.74% of routine-local sampled period); root also observed the prior-prefix reload at 0x29a65d. This only grounds a hypothesis about a dependent load/store chain. Sample location is not proof of a memory bottleneck, intrinsic latency reduction, or actual candidate code generation.

## Exactness proof

`prefix` is initialized with positive f64 zero; slot zero is never assigned thereafter. At each region the new scalar is also positive f64 zero. For rank 0, old and new evaluate `0.0f64 + f64::from(clause.local_max)` after the identical clamp. Inductively, after rank k-1 the scalar has exactly the value/bit pattern stored in old `prefix[k]`; both rank-k expressions evaluate the same f64 addition with the same converted f32 operand and store it in `prefix[k+1]`. There is no reordered addition, subtraction, FMA, reciprocal, or changed cast. Every region overwrites all nonzero prefix slots, and resets the scalar. The admitted original term count, upper-bound inflation/successor/cast, strict threshold/equality behavior, probe order, scores, cursor updates, and callbacks are untouched.

There are no cursor/callback mutations inside the prefix pass, so the scalar introduces no stale cross-region state. Valid admitted local maxima remain finite and nonnegative as before; the unchanged clamp and bound fallback retain the old safety domain.

## Scope and existing verification

Authentic Git patch: `attempt-21-full.patch` SHA256 `b0c6cbd13fe4369270aa743a12489f05b61605049dd173f82f716b3d5b444c45`. Retained full source: `attempt-21-source-or.rs` SHA256 `4bb058a4dd125ee819776b50f4506b01419bf42bba8cb2d6a0994a30cceae136`. Sole changed tracked file: `src/query/boolean_query/or_maxscore.rs`. Term scorer source byte identical to parent. All 13 OR test bodies/helpers byte identical; full test names in `attempt-21-static-receipt.json`. Existing canonical BufferedUnionScorer comparison fixtures cover full/tail regions, moving thresholds, exact midpoint leaves, strict ties and disparate boosts, 3/4/20/32 clause counts, reversed incoming order, exhaustion, and unsafe-domain fallback. No test duplicating this assignment was added. Root must run the frozen targeted and three-profile correctness gates and timing controller; no tests/builds/disassembly/profile/benchmarks were run here.

## Risks

LLVM may already derive the register recurrence, or may still spill the scalar under register pressure. Local source simplification does not prove different code generation or better latency. Code layout can change unaffected query families. One additional source-level f64 local exists; actual register/frame size and final binary size remain unmeasured. No scratch allocation, persistent cache, ABI, index format, or public API is added. No performance win is claimed.
