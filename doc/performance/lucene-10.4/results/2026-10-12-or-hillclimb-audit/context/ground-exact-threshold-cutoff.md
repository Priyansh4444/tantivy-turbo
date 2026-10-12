# Exact threshold cutoff for the optional bound predicate

Source-only grounding for a possible attempt22. The transformation is feasible within the existing admitted domain, with a rigorous exact predicate below. It can remove the repeated bit successor and f64-to-f32 conversion from the optional probe pipeline while preserving its original multiplication, factor, sum order and comparison result. This is neither implemented nor measured here. Root owns all runtime work.

Throughput checkpoint: n/a, source-only grounding.

Read accepted11's actual `SmallScoreSumUpperBound` and OR preparation/replay/callback code, complete decision trail through rejected20, actual tiny attempt21 diff, accepted11 long-OR profile receipt/root validation and the sampled instruction annotation. Read the installed official Rust Reference's float-narrowing rule at `/home/pronsh/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/share/doc/rust/html/reference/expressions/operator-expr.html`, lines2056–2077. It specifies closest f32, roundTiesToEven, and infinity on overflow; see [official float narrowing rule](https://doc.rust-lang.org/reference/expressions/operator-expr.html#r-expr.as.numeric.float-narrowing). No engine/source implementation, compiler, test, profile, benchmark, Python, browser or compression was run by this author.

## Observed opportunity and limits

Immutable accepted11 worker SHA `e11c4939984690d2c2d781cb6ace40d5e818e3a18a3f9f558d6cbcfd62546e5d` was profiled on all103 frozen long OR cases. Its receipt reports unchanged binary/inputs and matching checksums, while explicitly excluding latency acceptance. Annotation retained6954 samples in the BooleanWeight caller. The optional probe at0x29b3e2 adds known and prefix, then0x29b3e8 multiplies by the original factor,0x29b3ee moves f64 bits to an integer,0x29b3f3 increments them,0x29b3f6 moves bits back,0x29b3fb converts to f32, and0x29b3ff compares with the live f32 threshold. These instructions have substantial sampled period (including1,209,475,204 attributed at conversion and1,916,803,861 at comparison). Root's approximate17% local pipeline attribution is motivation, not causal cycle proof or a promised17% speedup: skid, dependency attribution, code placement and existing background load remain material.

This differs from rejected20's gap skipping and from attempt21's exact local-prefix accumulator. It preserves each candidate and optional cursor visit; only the predicate's representation changes. Attempt21 is currently unaccepted and must not be silently included. Build a separate candidate against whichever exact parent root explicitly selects after that verdict.

## Preserve the existing multiplication

The current helper computes, for admitted sum s:

```text
p = round_f64(s * factor)
q = f64::from_bits(p.to_bits() + 1)
old = (q as f32) <= T
```

Here s is finite, numerically nonnegative, at most32*f32::MAX; either signed zero is admitted. The original factor for n3..32 is finite and lies strictly between1 and2. Thus p is finite and nonnegative (possibly signed zero), and positive p is nowhere near f64 overflow. Keep **exactly this multiplication**, with the existing factor and floating evaluation. Do not divide a midpoint by the factor and compare s: rounded multiplication has its own boundary and a quotient shortcut would require a separate proof/search. Do not reassociate `known + prefix`, shorten the original n, change original-n error allowance, or publish any pruning sum.

For positive p, q is its immediate f64 successor. For +0, q is positive minimum f64 subnormal. For -0, the bit increment instead gives negative minimum f64 subnormal; narrowing returns -0. The latter special case is addressed explicitly below.

## Exact finite nonnegative cutoff

For finite nonnegative f32 T, normalize its bit pattern by clearing the sign bit (only -0 can have that sign in this branch). Let B be the normalized u32 bits. Define U as the next representable f32 converted exactly to f64, except that for f32::MAX use the hypothetical next finite endpoint2^128. Define:

```text
M = (f64(T) + U) * 0.5
L = f64::from_bits(M.to_bits() - 1 - u64(B & 1))
new = p <= L
```

Both adjacent f32 endpoints have at most24 significand bits; their sum/midpoint require at most25. The entire range, including subnormals and2^128, is normal and representable exactly in f64. The addition and halving therefore introduce no rounding error. M is strictly positive even when T is zero, so its bit predecessor(s) exist and the integer subtraction cannot underflow.

Round-to-nearest-even gives:

- If B is even, narrowing q <= T exactly when q <= M. Since q=successor(p), positive p passes exactly when p <= predecessor(M). Therefore subtract one from M's f64 bits.
- If B is odd, midpoint narrows upward to the even next endpoint, so q <= T exactly when q < M. On the discrete f64 grid this means q <= predecessor(M), hence p <= predecessor(predecessor(M)). Therefore subtract two.

For T=f32::MAX, its significand is odd. T=2^128−2^104, U=2^128, and M=2^128−2^103. At that midpoint round-to-nearest-even chooses the overflowing next endpoint, hence infinity. The midpoint is excluded, so the same odd rule applies. A naive `(T + T.next_up())/2` would yield infinity and is incorrect; this special U is mandatory.

Examples of boundary decisions, with p directly representable:

| T parity | p | old q | old/new decision |
|---|---|---|---|
| even | predecessor(M) | M | true |
| even | M | successor(M) | false |
| odd | predecessor(predecessor(M)) | predecessor(M) | true |
| odd | predecessor(M) | M | false |

T=+0 or -0 has even normalized bits, U=2^-149, M=2^-150 and L=predecessor(M). Positive p narrows q to zero exactly through that midpoint, matching the even rule. p=+0 and p=-0 both compare <= positive L. Old q narrows to the corresponding signed zero, and either signed f32 zero is <= either signed-zero threshold. Their predicate results therefore also agree. This proves signed-zero behavior without replacing the original q construction with a falsely signed next_up claim.

The largest subnormal/lowest normal boundary is the same adjacent-grid proof; B=0x007fffff is odd and B=0x00800000 is even. At every power-of-two transition, use the next upward neighbor's spacing, not a symmetric ULP assumption. The midpoint construction already does this exactly.

## All other thresholds

The old result can be only nonnegative finite f32, positive infinity, or signed zero. Map cold threshold construction as follows:

| Threshold T | Cached f64 L | Reason |
|---|---|---|
| negative finite, or negative infinity | negative infinity | old result <= T is always false; finite p, including -0, is never <= -infinity |
| positive infinity | positive infinity | every old result, including positive infinity, passes; all admitted p are finite |
| either signed NaN | f64 NaN | both old and new ordered <= comparisons are false |
| either signed zero or finite positive | finite cutoff above | exact midpoint/successor proof |

Use a classification order that does not accidentally treat -0 as negative: `T < 0.0` detects genuine negatives; `T == +infinity` handles the positive infinity; `T.is_nan()` can be checked before ordinary construction. Both NaN signs/payloads produce false; the predicate does not expose a NaN payload. Initial NaN already takes the unchanged generic fallback. A callback returning NaN violates the monotone callback contract and triggers the existing debug assertion; release code nevertheless assigns it. Refreshing L to NaN preserves the existing optional comparisons' false result, while leaving publication and later global/local partition predicates unchanged. Do not assume every callback obeys monotonicity merely to skip this cold classification.

## Minimal state and integration shape

One private cached f64 cutoff is sufficient; no hot enum, Boolean sign flags or approximation are needed. A private construction helper alongside `SmallScoreSumUpperBound` can compute L once at query entry. A method such as `score_is_at_most(sum, cutoff)` retains admitted-domain debug assertions and evaluates only `(sum * self.factor) <= cutoff`. Actual naming belongs to implementation, not this grounding.

Initialize L from the actual threshold after eligibility, and refresh it immediately after every callback's returned threshold is assigned, regardless of whether the f32 bits changed. Replace only the optional inner predicate `upper_bound.score(known + prefix[rank+1]) <= threshold`. Keep global certified-prefix and local partition calculations on the original helper for the first isolated trial; keep strict `score > threshold` publication unchanged. This bounds the surface and ensures no new stale-cutoff cache invalidation path is introduced by region transitions. Recomputing on unchanged callbacks may cost; optimize that only as a separately justified later mechanism if measurement warrants it.

Cold construction requires f32 next-neighbor bit work, exact widening/add/halving, parity and one/two predecessor adjustment. The hypothetical2^128 is a fixed exact f64 constant; a literal correctly representable as f64 or its verified bits is suitable. Threshold refresh does not allocate. Logical persistent state is one f64 (8B); compiled frame/register pressure/binary and RSS remain root measurements. The likely hot shape is f64 multiply + f64 compare instead of multiply + bitcast/inc/bitcast + f32 narrow + f32 compare. Only disassembly of the built candidate can establish generated code, and only frozen timing controls can establish a win.

## Independent tests before the trial

1. Preserve the old `score` helper as the expected predicate. For every n3..32, compare `old.score(sum) <= T` to the new cutoff predicate on identical s. Test all threshold classes: ±infinity, ±zero, negative smallest/ordinary/max finite, quiet/signaling NaN patterns of both signs, positive minimum/subnormal/normal/exponent transitions, odd/even significands, f32::MAX and its neighbors.
2. Independently test the post-multiply transformation over direct f64 p at M and several adjacent f64 predecessors/successors. Expected result must be the actual old bit-successor-and-f32-cast expression, not another midpoint implementation. Include the exact four even/odd rows above, zero midpoint, normal-power boundaries, largest-subnormal/min-normal, and MAX overflow midpoint. Require explicit even/odd midpoint witnesses so a sampled test cannot miss parity.
3. For actual s and each factor, derive candidate s near L/factor only as **test input generation**, then visit multiple f64 neighbors around that quotient. Compare against old helper, including signed zero, f64 subnormal/normal sums, every exponent region relevant to f32, near32*f32::MAX, and fixed-seed diverse valid rawbit sums/thresholds. The quotient is not the production cutoff. Retain both source and any failed original run before a correction.
4. Include original admitted bound-domain tests and structural checks that factor/n are unchanged. Non-admitted helpers need no new behavior. A test-only source-independent assertion in the optional loop can compare each new Boolean decision with the old helper under actual live thresholds.
5. Keep existing complete canonical callback doc/raw-score-bit traces, threshold-next_down/equal/next_up midpoint fixtures, fixed/Top1/Top10 schedules, deleted winners withholding a raise, native/default and safe configured profiles, and unsafe fallback. Add a callback rule that returns unchanged and then raised thresholds while an owned batch remains pending, and a release-compatible predicate unit check for callback NaN (without pretending the debug-contract panic is valid runtime behavior).
6. Root then runs the targeted exact predicate tests, existing OR traces, all three1227-query exact indexed gates, frozen pilot and, only if warranted, full AB/BA/self-controls and negative gates. Retain every query loss. No feature/API or format change is part of this proposal.

The proof is complete for the specified finite nonnegative admitted sum domain and original multiplication. Runtime equivalence, compiler code shape and speed remain unproven. The arithmetic is intricate enough to require the explicit parity/successor/overflow tests; a simplistic midpoint-only comparison or factor-divided cutoff should be rejected.
