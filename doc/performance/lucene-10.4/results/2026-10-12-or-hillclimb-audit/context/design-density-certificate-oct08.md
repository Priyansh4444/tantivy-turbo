# Exact drained-boundary density certificate cache

Source-only grounding of a possible cheap next attempt against accepted11 `43f06833a325d2a18c03bd7e4fa65f2f27944292`. No implementation, source changes, compiler, tests, worker, timing, profile, compression, browser or child delegation was performed. Attempts13/14 are untouched. The actual immutable accepted11 OR Git object was read through git show, rather than treating the concurrent dirty attempt14 file as an accepted baseline.

## Grounded mechanism

Accepted11 currently recomputes, at each drained region with a nonzero certified global prefix, the maximum fixed cost of live clauses in the remaining suffix and then asks whether every prefix cost is at least twice that maximum. Cache those two exact integer facts, not the density outcome. Density can still switch 0→1→0 when real exhaustion changes the remaining maximum and later prefix growth introduces a cheaper prefix clause.

The OR source creates each ClauseState once from incoming ordinal, immutable max_score, size_hint and actual doc. It sorts the pruning preference once using max/cost and ordinal. Neither cost, global_max nor ordinal is subsequently assigned, and the vector is not subsequently sorted, removed from or resized. Only local_max and doc are mutable. TermScorer.size_hint delegates to SegmentPostings.size_hint, whose HasLen uses fixed block_cursor.doc_freq, so the copied cost is total physical posting count, including deleted documents, rather than decreasing remaining_docs. The proof uses the already copied cost regardless of future changes in scorer size-hint policy.

Actual doc mutations are owned by ClauseState seek, advance and fill_scored_batch, with coherence checks before and after. They retain authoritative first-unread positions. Shallow selection leaves actual doc stale; it does not turn a live clause into EOF for the density policy. TERMINATED is permanent in ordinary forward posting traversal. The certificate must test that real mirrored doc only, never metadata remaining_docs or last_doc_in_block. No per-seek/advance/batch cache update is needed.

## Minimum state and algorithm shape

Keep all state local to or_maxscore, outside hot ClauseState and the cursor wrappers. No new scorer method, public API, helper layer, allocation or index format is needed.

Three locals are sufficient:

- `min_certified_cost: u32`, initialized to u32::MAX; valid when certified_global > 0.
- `max_live_rank: usize`, initialized to num_terms; num_terms is the explicit no-live-suffix sentinel, never indexed.
- `live_max_ready: bool`, initialized false; distinguishes an uncomputed suffix from a computed empty suffix.

This is 13 logical bytes on the 64-bit host before compiler placement/alignment; it is not an RSS or frame measurement. No cached maximum value is necessary: when rank < num_terms, its immutable ClauseState.cost is the exact maximum. An alternative encodes uncomputed as num_terms+1 and empty as num_terms in one rank local, but the explicit readiness bit is easier to audit. Avoid adding a helper abstraction merely to package three locals.

On each successful append to accepted11's certified global prefix, update min_certified_cost with that appended rank's fixed cost. Update only after the numerical append is accepted; never update for the rejected next maximum. The numerical sum/count arithmetic and its original-n bound factor remain unchanged.

At the existing nonzero-prefix density boundary, rebuild the maximum only when not ready, or when a stored live rank is now before certified_global, or when that rank's actual doc is TERMINATED. A rebuild scans exactly the current certified suffix, considers only actual non-EOF clauses, selects a live maximum and records readiness even if none exists. Choose a deterministic tie rule, e.g. first live rank with the maximum; any maximum winner preserves the old scalar result. An empty cached suffix remains empty under future prefix growth/exhaustion. Read remaining_cost as u64::from(winner.cost), or zero for the empty sentinel. Replace the prefix all() with u64::from(min_certified_cost) >= 2 * remaining_cost. The effective globally_optional value is still recreated every region and reset to zero on density failure.

Do not initialize/rebuild the maximum while certified_global == 0: the old density path performs no cost scan there. Keep accepted11's all-certified early return before density work. No new update belongs in candidate completion or cursor mutation.

## Exact proof and invalidation

For certified count c, the prefix cost minimum is min(cost[0..c]). Count starts zero and only increases under the existing cached-prefix contract. Each successful append adds precisely one fixed cost, so taking the running minimum reproduces the from-zero minimum. Exhaustion does not remove a prefix member: the current all() predicate deliberately includes exhausted prefix clauses too. Filtering them would be a different policy and is not allowed.

For the remaining live set S(c) = {rank >= c whose actual doc != TERMINATED}, suppose cached rank w is a maximum. If w remains in S, then every later live suffix member belonged to the earlier S and retains the same cost. The live set can only shrink through prefix advancement or permanent EOF. Therefore no other member can overtake a still-live, still-in-suffix w. The cache remains exact even if a tied or cheaper member exhausts. When c passes w (w < c), or w reaches actual EOF, rescan the current set to obtain its new exact maximum. Equality w == c does not exclude the winner and must not trigger prefix invalidation.

If S is empty, prefix growth and forward exhaustion cannot make it nonempty again, so the zero maximum is stable. If live costs could be zero, choose a real zero-cost member during rebuild or the empty sentinel; either yields the old maximum zero. Comparing the minimum prefix cost to twice the maximum is exactly equivalent to all prefix comparisons. Both original costs are u32; conversion and doubling occur in u64, whose range safely contains 2*u32::MAX. No score or floating-point arithmetic changes.

Every rebuild caused by invalidation permanently removes the previous winner from consideration. Therefore at most n distinct live winners can be selected, plus one empty-set scan; repeated reuse needs only O(1) rank/doc/cost checks per active boundary. Worst-case rebuild work is O(n²) per query, bounded by n<=32; this is a conservative count bound, not a promise of amortized O(n) scanning or a latency estimate.

Existing eligibility executes before any cache state and retains the canonical union for signed/exceptional/unsafe score domains and NaN starts. Existing optimized threshold monotonicity is still required for certified_global; this cache does not establish a new public callback guarantee. If future code mutates costs/ranks, rewinds actual posting docs, or permits certified count to decrease, this proof fails. Today no such mutation occurs in the actual accepted11 kernel.

## Independent fixture/observer shape

Use a cfg(test)-only observer at the existing density boundary. Independently compute the old exact suffix maximum by a from-zero filter/map/max over actual clauses and the old all() over the entire certified prefix. Also independently compute prefix minimum for direct integer equality. Compare remaining maximum, prefix minimum and effective density choice to the cache before preparation. Verify any stored live winner is inside the suffix, non-EOF and equal in cost to the independently computed maximum. Verify the empty sentinel iff no live suffix member exists. Do not derive expected facts from the cached rank or cached minimum.

Record only test facts sufficient to assert initialization, reuse, winner-exhaustion invalidation, prefix-passes-winner invalidation and stable computed-empty reuse. The old oracle can choose a different tied rank: compare scalar maximum and the winner's legitimate membership, not unimportant tie identity. Full raw callback traces still compare to BufferedUnionScorer; the observed density states alone are not correctness evidence for publication.

Preserve existing accepted11 fixtures unchanged and add evidence around them or a small new fixture:

1. The existing `cached_certificate_survives_density_rejection_widening_and_rejection_again` gives meaningful rejection/restoration/rejection: certified1/effective0/cost200, then certified1/effective1/cost129 after actual high-cost exhaustion, then certified3/effective0/cost129 after threshold growth. At the final state, a newly included tiny-cost prefix member lowers the cached minimum while the remaining winner stays valid. Require the actual ordered observations and winner exhaustion reason; do not merely assert the final heap.
2. The existing n4/n32 cached-prefix equality/neighbor/large-jump fixture has equal costs, repeated regions and growing prefixes. With a first-rank tie winner, prefix growth necessarily passes cached winners; require those invalidations and nonzero reuse before them. The independent oracle remains from zero and all callback score bits remain canonical. Native/legacy, reversed incoming order, negative/infinite/zero starts retain coverage.
3. Add a small focused tie/exhaustion corpus if the old fixtures do not expose both causes separately: an equal-cost nonwinner reaches EOF while the winner survives (must reuse), then the winner reaches EOF (must rebuild); later c becomes equal to the winner rank (must reuse), then passes it (must rebuild). Ground real ranks/costs and numerical prefix membership from actual scorer maxima/size hints, rather than assume boosts alone determine preference. Add actual pre-exhausted initial clauses and an empty live suffix if realizable before the existing all-certified/lo termination. If stable-empty reuse is unreachable in this kernel, document that and test the exact zero result without inventing an impossible event.

The existing accepted11 density corpus is specifically the original one; the altered12/13 finite-block geometry fixtures are not the parent of this experiment. Do not import their widening changes or corrected corpus as bundled production behavior.

## Measured grounding and limits

The retained region diagnostic observed 103 long-OR kernel invocations and 95,315 entered regions. It records per-query original n and region count, but not certified-global count, active density scans, prefix short-circuit length, maximum-winner identity or winner invalidations. Parsing retained run/records.json gives sum(n*regions) = 922,219. This is a conservative upper bound on the old combined suffix-plus-prefix cost record visits across all those observed regions: when c>0 the suffix scans n-c records and all() examines at most c, and when c==0 density scans neither. It is not the actual cost-visit count, removed visits or accepted11 wall-clock work. The source of those diagnostic counters is an unaccepted instrumentation branch over the prior equivalent kernel, not a fresh compiled accepted11 measurement.

83 of the 103 queries have only three original terms; 13 have four, two have five, three have six, one has seven and one has 21. Query920 contributes 30,242 regions and its n*regions upper bound is 635,082, roughly 68.9% of the aggregate upper bound. Thus any larger opportunity may be concentrated in that high-rank query, while the common n3 path's original scan is already small. The diagnostic also reports 907,568 finite selected-block visits, 1,754,338 optional-bound checks and 564,664 scalar candidates. None is latency-weighted or directly comparable in cycles, and global/local optional classifications do not reveal how often the density cache would be active. Do not convert the 89.05% optional-only cap statistic into density-cache coverage.

This is a plausible cheap pilot, with exact integer/domain proof and no hot cursor maintenance, but likely a smaller gain than a replay/metadata design change. It trades a bounded scan for readiness/winner-validity branches, an indirect winning-record read and prefix-append updates. Small n or frequent prefix/winner invalidation can erase the benefit; added live state can affect register pressure. No speed, memory or efficiency gain is inferred from source. Implement only as its own isolated attempt against the actual accepted parent after the current trial decision, then use the frozen root gates and controls to keep or reject it.
