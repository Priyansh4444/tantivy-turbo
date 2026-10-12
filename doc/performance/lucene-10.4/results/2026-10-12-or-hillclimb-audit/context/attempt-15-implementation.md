# Attempt 15: exact density certificate cache

Stable isolated source SHA-256: `602867a5085c86c6923e15234a6ce381519b2834af6c627ef910db393e6d3477`.
Parent: accepted11 `43f06833a325d2a18c03bd7e4fa65f2f27944292`.
Worktree: `/home/pronsh/Coding/playground/search/tantivy-or-density-certificate-oct08`.
Only changed tracked source: `src/query/boolean_query/or_maxscore.rs`.
Full diff SHA-256: `b63ad6c388c15b64b80f7fbc4e7d34f22fe9710560c52713a1e7ac57cf04c3c6`; 362 lines.

Preserved artifacts: `attempt-15-source-or.rs`, `attempt-15-full.patch`, and `attempt-15-static-receipt.json`. Source is held unchanged for root review/gates. This is an unaccepted candidate; no speed or runtime correctness result is claimed.

## One production mechanism

The three local fields are the grounded design's `min_certified_cost: u32`, `max_live_rank: usize`, and `live_max_ready: bool`. The prefix minimum updates only after a numerical global certificate append succeeds, before advancing its count. All certified global sums/counts/bound factors are unchanged. Prefix minimum includes every admitted prefix clause's fixed cost even when that clause is already exhausted.

At a nonzero certified prefix, the cached suffix maximum is rebuilt only for uninitialized state, a cached winner rank strictly below the certified count, or actual mirrored winner doc==TERMINATED. Equality between winner rank and count retains the winner. Rebuild considers all and only actual non-EOF suffix clauses and takes the first maximum-cost tie. The sentinel num_terms denotes no live suffix and is never indexed. Readiness distinguishes that valid empty result from an uncomputed cache. Costs remain immutable, so storing the rank alone suffices; the maximum value is read from its record. The density predicate compares u64 prefix minimum to twice the u64 live maximum; effective globally_optional is still a fresh decision each region.

A still-live, still-in-suffix winner remains maximal because fixed rank costs do not change and the real live set can only shrink through increasing certified count or permanent actual EOF. No metadata endpoint/remaining-count inference can invalidate the winner. No state update was added to seek, advance, fill_scored_batch or any candidate path. There is no new helper/API/allocation/cache policy/index format or other candidate13/14 mechanism.

The exact empty result cannot be reused at a later active boundary in this kernel: a nonempty globally certified prefix is admissible against the current threshold; if all remaining suffix clauses are real EOF, their bounds are zero, the prefix keeps max_doc coverage, and the tight all-optional skip reaches max_doc and ends the query. The focused fixture asserts that one realizable zero result rather than fabricating a later empty reuse event.

## Preservation and numerical/domain limits

Static guards verify all 13 original accepted11 fixture bodies byte-identical from the first cached-prefix fixture through EOF. ClauseState declaration and complete cursor wrapper impl are byte-identical. Eligibility, score-bound construction, immutable preference sorting, prefix/contribution allocation and initial floor are byte-identical. Every byte from `let mut hi = max_doc` through region preparation, all skip/seek/candidate/replay/optional/publication/callback/retirement logic remains identical: 6999 bytes, SHA `b93646efb4641fbfa4bc1a0ec00c42a2f28c9af2306be3e938669b3741168421`.

Only integer density facts are changed. u32 cost conversion/doubling occur in u64, safely covering 2*u32::MAX. The existing admitted positive-TF/nonnegative finite literal score domain, exceptional fallback and monotone callback threshold contract remain unchanged. This does not establish universal public callback behavior or full Lucene feature/API parity.

The cache adds 13 logical bytes on this 64-bit host before compiler placement/alignment. Register/stack/frame/binary/RSS effects remain unmeasured. Reused winner checks replace scans but add branches/record reads and prefix-append minimum updates; small n or frequent invalidation can erase a benefit.

## Independent test observer

The cfg(test) global-prefix block recomputes a from-zero minimum over all certified prefix costs, including all-certified early finish, and asserts exact equality. At every nonzero-prefix density boundary the observer independently reproduces accepted11's old filter/map/max and old prefix all() predicate, never deriving expectations from the cached rank/minimum. It asserts exact maximum and decision equality, real winner membership/non-EOF/equal maximal cost, or the precise empty sentinel. A test-only record captures physical floor, certified/effective counts, minimum/maximum, winner rank/incoming ordinal, independently distinguishable initialization/prefix exclusion/EOF reasons and actual terminated-ordinal bits. No observer/counter or observer allocation exists in release.

## Four new fixture families

1. `density_cache_observes_exhaustion_and_rejection_restoration_rejection` invokes the unchanged existing complete-callback density fixture while recording cache evidence. It requires actual ordered (certified1,effective0,cost200), (certified1,effective1,cost129), (certified3,effective0,cost129), prefix minimum300→2, EOF-caused restoration, and reuse when the newly cheaper prefix rejects density again. Existing full canonical raw callback comparison remains part of that invoked fixture.
2. `density_cache_prefix_exclusion_preserves_native_legacy_n4_n32_neighbors` invokes the unchanged existing native/legacy DEFAULT n4/n32, reversed order, signed-zero/negative/infinite starts, equality-neighbor and threshold-jump fixture. Equal fixed physical costs2048 provide actual reuse and prefix-passes-winner invalidation, with exact minimum/maximum and first-suffix winner assertions. The global-prefix observer separately checks minimum even at all-certified return.
3. `density_cache_tied_nonwinner_eof_equal_rank_and_prefix_pass_are_real` grounds a new n5 corpus's actual costs [800,400,400,257,129], preference order and independent generic global-prefix threshold neighborhoods. Actual physical finite block endpoints force the relevant preparations. The lower-ranked tied cost400 clause exhausts while cost400 winner remains live (reuse); winner EOF yields cost257 rebuild at floor2883; threshold growth to count3 equals the cached rank and reuses it at floor3430; subsequent count4 passes that still-live winner and rebuilds to cost129 at floor3779. Actual terminated bits distinguish tied nonwinner EOF from winner EOF and prefix exclusion from accidental EOF. The entire scheduled callback trace is compared to untouched BufferedUnionScorer.
4. `density_cache_initial_exhausted_suffix_returns_exact_zero_once` starts two real scorers at actual EOF while a weak certified clause remains live. It asserts one nonzero-prefix/zero-maximum/empty-sentinel initialization, precise terminated bits and no canonical callbacks. It states the unavoidable terminal skip that makes later empty reuse unreachable.

All new fixture runtime path assertions remain unexecuted by this agent. Root should run them and all 13 existing fixtures first; do not infer their observed outcome from this receipt.

## Work performed and root handoff

Read the full grounded design and actual accepted11 source. Drafted only the assigned OR file, then read the complete actual resulting diff. Rustfmt (edition2021/check) and git diff --check pass. Receipt guards verified base/scope, unchanged original fixtures and unchanged cursor/preparation/replay/admission spans. No compiler, tests, workers, profiling, timing, compression, browser, child agent, commit or push was run. Root owns targeted tests, native build, exact default/two configured gates, frozen pilot and any later full paired controls.
