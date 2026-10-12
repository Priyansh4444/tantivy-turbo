# Attempt 11 independent source review

Verdict: no concrete correctness blocker found in the full diff or resulting source for the existing admitted, monotone-threshold OR domain. The change preserves the original global-prefix decisions while avoiding repeated accepted prefix additions. This is a source review, not a performance or acceptance verdict.

Throughput checkpoint: n/a, read-only review. No source edits, build, tests, profile, timing, compression, browser work, child delegation or commit was run by this reviewer.

## Artifact and scope

Reviewed the complete diff and current `search/tantivy-or-global-prefix-oct07/src/query/boolean_query/or_maxscore.rs` against parent `a674aaf2f6187f3010a135dc25fdacf9cab27e53`. Read `attempt-11-design.md`, `attempt-11-implementation.md` and `attempt-11-integration.json` end to end. Traced the unchanged generic/specialized bound helpers and current callback/cursor protocol.

Verified source SHA-256: `21153a76a7ae3091ac6400cc438f408b7ac367d9a534b1d290d30af5c3fb5901`. The sole tracked changed file is the OR source. The integrated root trial source is byte-identical to the isolated reviewed source and agrees with the integration receipt.

Direct source comparison proves eligibility, construction of immutable ClauseState maxima/costs, frozen preference sorting, original-n bound helper, prefix/contribution/batch setup and initial floor are unchanged. From the existing `observed_remaining_cost` declaration through the entire density/local-bound/prefix/candidate/optional/callback/cursor body to the test module, source is byte-identical to accepted06. Only global-prefix preparation and test-only observation/fixtures are changed.

## Exact cached-prefix proof

The production certificate consists of a count and its sequential f64 sum. The initial pair is `(0,+0.0)`. For count k, `certified_global_sum` is exactly the original left-to-right fold of `clauses[..k].global_max`; it is not an associative reduction, subtraction, reordered sum or score publication. The next operation adds the same f64-converted next copied maximum as the original from-zero loop.

Every append is accepted only after the unchanged `SmallScoreSumUpperBound(original num_terms)` compares that prospective sum to the current threshold. Count and sum then advance together. Rejected next additions are not saved into the certificate. Resuming later at this exact rounded sum reproduces the same future floating additions as recomputing the prefix from zero.

Global maxima are copied once from immutable query weights, and the preference order is sorted once before caching. Subsequent region code changes only doc and local_max; it never alters cached maxima, costs or ordering. Finite positive maxima and at most 32 original terms keep all cached/prospective f64 sums within the specialized helper's admitted domain. Valid indexed positive matching TF remains part of the existing literal-leaf safety domain; this change does not validate corrupt or impossible TF inputs.

With a numerically nondecreasing threshold, a previously accepted prefix remains accepted. Thus starting at its count cannot skip a newly failing rank, and extension finds the same first rejected rank as the original loop. Numerical equality, including threshold sign changes between -0 and +0, does not invalidate a positive-prefix certificate. Actual posting exhaustion also cannot invalidate the immutable global certificate; retaining an exhausted term's global maximum merely keeps the same conservative overestimate as the original computation.

## Density and early finish

The globally certified count and the region-effective optional count are separate values. `globally_optional = certified_global` is recreated each entered region; density rejection resets only that fresh local variable to zero. It cannot shrink or overwrite the certificate. Conversely the cached count does not lock in a past density decision.

Live remainder costs are recomputed each region from the same suffix and current authoritative `doc != TERMINATED` state. The existing cost maximum and peroptional density checks are untouched. Therefore actual high-cost exhaustion can switch the effective count from zero to the certified count, and a later larger certified prefix can cause density rejection again.

All-certified return still occurs before effective density selection, shallow selection or local cursor work. In production it differs only in deriving the same all-certified condition from persistent state. Test mode independently checks/records that final global state before returning; those extra operations have no release presence.

## Threshold and fallback scope

Initial NaN threshold and unsafe/signed/exceptional score domains still enter the unchanged canonical union before certificate state exists or any OR seek occurs. Negative starts, -infinity and both signed zeros use the same comparisons as before. +infinity accepts every finite original-prefix bound and terminates; a rounded bound that becomes +infinity is also <= a +infinity threshold in both helpers.

The proof relies on the existing optimized kernel's callback contract: thresholds remain ordered and numerically nondecreasing, as the unchanged `debug_assert!(next_threshold >= threshold)` enforces in debug execution. This review does not extend behavior to callbacks that lower thresholds or return NaN. Such callbacks are already outside the optimized fixed-partition pruning assumptions; the new cache adds no fallback/clamp for them. Public `Weight.for_each_pruning` prose does not explicitly spell out monotonicity, so this is a stated implementation-domain limitation rather than a claim of a newly documented universal public callback guarantee.

## Meaningful fixture evidence

The cfg(test) observer recomputes the original rank prefix from +0.0 using the old generic `ScoreSumUpperBound`, rather than deriving expectations from cached count/sum or the specialized helper. Every entered region checks exact count and raw f64 sum bits, including all-certified early finish. Append count equals the cached count and is bounded by original n. This compares the numerical certificate itself, while the independent BufferedUnionScorer compares every callback `(physical doc, raw f32 score bits)` and strict document order.

The scheduled-threshold fixture uses n4/n32, native/legacy valid finite boosts, forward/reversed incoming order, negative/infinite/zero starts, three successive generic prefix-bound equality neighborhoods, a larger finite jump and +infinity. Equal-cost boosts produce the intended weak preference prefix even under reversed incoming order. It explicitly requires observed counts 0,1,2,3,n, exact threshold-bit equality states for each prefix bound, enough regions to exercise repeated retention, and final all-certified termination. Neighbor thresholds are in the deterministic schedule; the per-region old-helper comparison catches any count/sum discrepancy under the actual visited states. It does not claim that all possible threshold bit patterns or configured native parameter profiles are covered by this one fixture.

The separate density fixture forces and asserts ordered observations `(certified1,effective0,cost200)`, then `(certified1,effective1,cost129)` after actual exhaustion, then `(certified3,effective0,cost129)` after the scheduled rise. It therefore exercises the certificate surviving density rejection and restoration, not only a final TopDocs identity. Existing stale full/tail cursor, scored-batch, current-threshold/deletion, duplicate/disparate-boost, midpoint/incoming-order, and unsafe fallback fixtures remain unchanged and relevant.

Root reports 13 targeted fixtures already green. I did not execute them. Native release build, frozen default/configured exact gates and timing acceptance were root-owned and pending when this review was requested. The report makes no fresh runtime claim beyond that attributed status.

## Limits and next acceptance evidence

No source correctness blocker remains within the stated domain. Production persistent payload is usize plus f64, 16B on the current 64-bit host, with no new allocation or changed index format. That is logical payload accounting, not measured register/stack placement, RSS or total efficiency. The first rejected next maximum is still retested each region, and density/local preparation remains; source inspection establishes neither how much this removes on the real workload nor a speed gain.

The frozen root pipeline must decide acceptance, including all query groups/perquery regressions, default exact equality and both configured own-oracle gates. This unit neither implements all Lucene features/public APIs nor establishes universal parity or superiority against Java Lucene or the fixed Rust port.
