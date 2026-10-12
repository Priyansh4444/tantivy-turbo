# Reuse the exact cutoff for global and local bound comparisons

Source-only grounding/design for a possible attempt23. Reusing the same cached cutoff is mathematically exact for both remaining production bound predicates in the admitted OR kernel. It is a cheap, untried extension worth screening if root selects a suitable parent after attempt22's verdict; there is no speed guarantee. This document neither implements the extension nor assumes attempt22 is accepted.

Throughput checkpoint: n/a, source-only investigation.

## Reviewed evidence and parent discipline

Read the complete decision.tsv through rejected21; ground-exact-threshold-cutoff.md; attempt22's full authentic patch, source-only report/static receipt and independent review; and the actual held or_maxscore.rs and score_combiner.rs, including all numeric and OR tests. The held worktree HEAD is accepted11 `43f06833a325d2a18c03bd7e4fa65f2f27944292`; its two-file diff contains attempt22 only. Its final patch SHA recorded by the static receipt and independent review is `4594ba2e4096973b495e1fa019b637bdd1b833b08f540ad62ce9969d140c40e9`.

The reviewed source artifacts correctly identify themselves as uncompiled/unmeasured by their source authors. Root separately reports targeted checks, three exact gates and a promising pilot, with full22 still running at delegation time. Those root results were not rerun or independently inspected here and do not establish acceptance. No implementation, build, test, Python, engine, profile, browser or compression work ran here. Only source/document reads and Git metadata reads occurred.

Preferred isolated shape if root accepts22: extend its existing cache to global and local predicates, leaving its optional kernel unchanged. If22 is rejected, do not silently stack it or describe its mechanism as accepted. Root must explicitly select a standalone accepted11 trial, with the optional old predicate retained and the necessary constructor/cache introduced only for the chosen global/local extension, or skip this candidate. The expected cold cost in that standalone variant differs from the reuse case below. Rejected21's local-prefix accumulator must not enter either variant.

## Remaining call sites and state

The source-wide search finds SmallScoreSumUpperBound production use only in or_maxscore.rs. The ordinary score method is also used by numeric tests and test-only expected predicates. The three production decision sites in the held source are:

| Site | Current decision | Required decision after extension | Lifetime/work |
|---|---|---|---|
| Global certificate, lines144–150 | `score(next_sum) > threshold` | Explicit ordered greater predicate on `next_sum` | At most original n successful appends over the query; one failed test at every region until the certificate completes |
| Local partition, lines270–275 | `score(prefix[first_essential+1]) <= threshold` | Existing ordered at-most predicate on that unchanged prefix value | Up to n bound tests per region, retaining the same first_essential and loop short circuit |
| Optional probe, lines345–352 | Attempt22's cached at-most predicate on `known + prefix[rank+1]` | Unchanged | Per candidate/optional rank; already covered by attempt22 |

The generic ScoreSumUpperBound call in the cfg(test) naive global certificate computation, lines157–169, is deliberately independent and must remain old arithmetic. Do not replace that expected computation with the cutoff. Generic helpers outside this admitted kernel remain untouched.

The only optimized-kernel assignment of the live threshold is lines372–375, immediately after the callback. Attempt22 initializes the cutoff after numeric admission and refreshes it unconditionally immediately after that assignment. Global and local decisions at the next region thus see the current cache without a new refresh path. Region transitions, density choice, shallow seek, prefix construction and batch fill do not assign threshold. Callback publication remains strict `score > threshold`, using the canonical incoming-order f64 fold followed by f32 conversion.

## Exact extension to ordered greater

Retain the exact original multiplication `p = round_f64(sum * original_factor)`. Existing admission and sums certify finite numerically nonnegative sum, including either signed zero, at most `32 * f32::MAX`. The factor is the unchanged original-n factor for n3..32, finite and strictly between1 and2. Consequently p is finite, numerically nonnegative (possibly -0), and far below f64 overflow. No quotient, reciprocal, reassociation, shortened count, approximate allowance or new leaf arithmetic is allowed.

Let old denote `f64::from_bits(p.to_bits()+1) as f32`. It is nonnegative finite f32, positive infinity from narrowing overflow, or signed zero; it cannot be NaN. For finite nonnegative threshold T, the existing cutoff constructor gives L at the exact upper edge where old is <=T: even f32 significands use the first f64 predecessor of the exact upward midpoint; odd significands use the second predecessor. It already accounts for the old integer bit successor. At f32::MAX the hypothetical finite endpoint2^128 supplies the exact overflow midpoint, with the odd tie excluded. This proof and its signed-zero/subnormal/exponent/overflow witnesses are unchanged from the reviewed grounding.

For this non-NaN branch, the already proved equivalence `old <= T` iff `p <= L` and total numeric ordering imply `old > T` iff `p > L`. Equality matters: when p is exactly L the old value is <=T and greater must be false. At the first representable product above L, greater must be true, including the MAX overflow boundary. Either signed-zero p is below the positive zero-threshold cutoff, so old's signed zero is never greater than either signed-zero T.

All exceptional classes preserve BOTH ordered predicates:

| T | L | Old/new <= | Old/new > |
|---|---|---|---|
| Genuine negative finite or -infinity | -infinity | Always false | Always true, including either signed-zero product |
| +infinity | +infinity | Always true, including old narrowing to +infinity | Always false; old infinity is equal to T |
| Either sign/payload of NaN | NaN | Always false | Always false |
| Either signed zero or finite positive | Existing finite cutoff | Exact midpoint rule | Exact ordered greater rule |

The greater implementation MUST evaluate ordered `p > L` directly. It MUST NOT be `!score_is_at_most(...)`: that negation is true for NaN, while the old global `old > NaN` and direct new `p > NaN` are both false. Initial NaN takes the unchanged fallback; a callback NaN violates the existing monotonic debug contract, but release's assigned threshold still needs identical predicate behavior. Do not justify a negation through the callback contract or introduce special hot branches for NaN. The existing constructor's NaN cutoff handles it without a hot classification branch.

Global next_sum remains the same sequential addition of immutable maxima to the cached global prefix. Local prefix remains the same sequential from-zero array construction, including zeroed bounds for cursors beyond hi. Optional sum retains its existing disjoint known-plus-prefix expression. All carry the original num_terms allowance. Admitted leaf/max values do not exceed f32::MAX; monotone rounded nonnegative addition cannot exceed the exactly representable coarse upper endpoint32*f32::MAX. No new exceptional-sum support should be inferred outside that admitted domain.

## Minimal interface and integration

Add one inline method alongside the existing at-most method, conceptually `SmallScoreSumUpperBound::score_is_greater(sum: f64, cutoff: &ScoreSumThresholdCutoff) -> bool`. It uses the identical admitted-domain debug assertion and the literal expression `(sum * self.factor) > cutoff.0`. Keep the existing score method as the expected arithmetic and leave the constructor and at-most implementation unchanged. The private cutoff wrapper remains one f64 with the same original factor owner; no enum, classification flag, allocation or additional persistent state is necessary.

Reuse the same initialized/refreshed cutoff for the global break and local while condition. Renaming optional_cutoff to threshold_cutoff is a mechanical clarity change, not a new mechanism; root may retain the name to minimize diff noise. Do not change the optional predicate, callback assignment/refresh, canonical publication, prefix construction, global cached sum/count, density policy, loop ordering or cursor updates. For local instrumentation, evaluate the same prefix slot inside the short-circuited while condition (or an exactly equivalent single loop body); do not accidentally read prefix[num_terms+1]. Only the existing first_essential < num_terms branch may evaluate its bound.

Under cfg(test), compare every global replacement Boolean directly against `upper_bound.score(next_sum) > threshold`, and every evaluated local replacement Boolean against `upper_bound.score(local_sum) <= threshold`, using the current live threshold. This checks actual input, comparator direction and refresh state at both changed sites. Prefer an assertion that remains active in cfg(test) even with release test optimization; no observer/counter belongs in ordinary release. Preserve attempt22's existing optional old-helper equality assertion as authored. Preserve the independent generic naive global computation, exact sum bits/count equality, global_appends identity and original monotonic threshold assertion.

## Meaningful test plan

Strengthen attempt22's shared numeric check_threshold_cutoff helper to assert the new greater method against literal `small.score(sum) > threshold` on the same sum and T, retaining its existing <= assertion. The four existing additive families will then exercise every n3..32, signed zero/subnormal/normal/max sums, negative thresholds, both infinities, both signs/payloads of quiet/signaling NaNs, actual rounded products around every exponent boundary and deterministic valid raw-bit sums. Keep all four original generic/small numeric test families unchanged.

Extend the direct-product family independently: for each already generated zero, -zero, pred2, pred1, M, next1 and next2 product, compare `product > cutoff` with the old literal bit-successor/narrow expression's `> threshold`. Keep the explicit even/odd at-most witnesses and add their corresponding greater witnesses at L and its next_up. The direct comparison must not derive its expected value by negating <=. Add an explicit NaN witness proving both predicates false, and a MAX/infinity witness proving the old overflowed bound is not greater than a +infinity threshold. Shared helper tests cover those classes, but the named witnesses prevent accidental complement regressions from hiding in broad diversity.

The existing thirteen OR fixtures supply live coverage with the proposed cfg(test) comparisons. In particular:

- cached_global_prefix_preserves_equality_neighbors_and_large_threshold_jumps exercises native and legacy fixtures, n4/32, incoming reversal, negative/±zero initials, exact bound next_down/equal/next_up callback schedules, large jumps to +infinity, count0/1/2/3/n, and original generic from-zero certificate state at every region.
- cached_certificate_survives_density_rejection_widening_and_rejection_again retains the immutable certificate while the density policy rejects/selects/rejects its effective use. This rules out confusing certificate count with local partition.
- single_driver_batches_replay_every_dynamic_threshold_exactly forces a full128 owned batch, fixed/Top1/Top10 thresholds, unchanged deletion callbacks and later raises. At the following region the same refreshed cache must feed global and local comparisons. Keep its canonical callback doc/raw-score-bit trace and observed128 batch witness.
- exact_midpoint_leaves_preserve_incoming_order_and_strict_ties exercises f32 boundary neighbors, incoming fold order and fixed/Top1/Top10 strict publication; other full-block/tail, optional promotion, sparse/dense, duplicate/max-n and deterministic corpora compare complete canonical callback traces.
- unsafe_domains_use_exhaustive_canonical_sum_before_any_seek continues to test pre-movement fallback, including an initial NaN. Callback NaN is not a valid debug-runtime schedule; numeric predicate tests cover its release-compatible mapping without removing the contract assertion.

No new trace observer is necessary solely to prove this two-site Boolean substitution: active live comparisons plus unchanged independent generic certificate state and full canonical callback traces cover it. If root wants stronger path evidence, additive cfg(test)-only counts for successful/failed global/local decisions may prove that the existing fixtures execute both directions; they must not replace those independent oracles or enter release. Do not copy a shortened n into expected bound construction.

Root owns targeted numeric/OR tests, three1227 exact indexed profiles, fresh frozen pilot, and justified full AB/BA/self-controls/negative gates. Retain original failures and every per-query loss. None ran here. No complete public Lucene API parity or actual-port win follows from this design.

## Performance assessment and risk

If22 is accepted, the expensive cold cutoff construction and callback refresh already occur; these two substitutions add no further refresh and reuse the same one-f64 state. They remove old successor/bit transfer/f32 narrowing from additional region-time decisions while retaining one original f64 multiplication and ordered comparison each. Global successful appends are bounded by n for the whole query, so most possible global benefit is the failed test repeated per region. Local partition scans are more numerous in dense/high-region cases, but have no existing measured attribution sufficient to predict a gain.

There is no extra data-dependent release branch required by the new greater method; it replaces an existing branch predicate. Extra inlining/code placement, f64 compare scheduling and register live-range effects can still harm other queries. Profile samples originally motivated optional narrowing, not a measured proportion for global/local work. Sample skid and background load remain reasons not to claim an instruction percentage as removable elapsed time. Rejected21 is direct evidence that an apparently cheaper prefix operation can fail full controls/negative gates; the extension must face the same ruler.

This is a low-source-complexity next screen because the intricate cutoff construction and proof already exist, the additional comparator is literal ordered greater, and the same numeric families can verify both directions. It is not a reason to relax either timing controls or the material-regression gate, and if the selected parent lacks22's cache, the added cold cost must be judged afresh.
