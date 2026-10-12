# Attempt 14: candidate-local contribution mask

Stable source SHA-256: `6db826184ecfc6e745940b8049da6fd3755ad622085429ffca8fbb794bcb5010`.
Base: accepted11 `43f06833a325d2a18c03bd7e4fa65f2f27944292`.
Worktree: `/home/pronsh/Coding/playground/search/tantivy-or-contribution-mask-oct08`.
Only tracked source changed: `src/query/boolean_query/or_maxscore.rs`. Source is now held unchanged for root review and execution.

## Mechanism and data shape

Each candidate starts a new local u32 `matched = 0`. The incoming-order contribution Vec is unchanged, including its allocation and initial contents. All three existing write sites (single buffered essential, scalar essentials, matching optional probe) overwrite that leaf slot before marking `1u32 << ordinal`. Ordinals are already bounded by the admitted 3..=32 original-term count, so ordinal 31 is valid and the full mask is u32::MAX. No shift by original count, no sentinel bit, no second publication path exists.

Publication visits only set ordinals in ascending incoming order with trailing_zeros and `matched &= matched - 1`. It starts f64 +0 and rounds once to f32. Known/pruning sums, original-n factor, global/local certificates, density choice, cursor mirrors, batch heads, current callback threshold, admission/fallback and index format are unchanged. There is no helper layer, selector, allocation change or dense fast path.

The mask has candidate scope inside the replay loop, after validating `doc < hi`. Thus every scalar candidate, pending buffered candidate, candidate following early optional pruning, and candidate after a callback threshold update gets a fresh mask. Slots from older docs can survive in the Vec but cannot be read unless this candidate just overwrote that slot and set its bit. The publication loop consumes the local mask only after all candidate pruning has finished; no later logic depends on its contents.

## Numerical proof and limits

The existing pre-seek admission requires a safe literal BM25 domain and finite positive query maxima. Its documented invariant is finite nonnegative f32 matching leaves for valid positive indexed TF. Original n<=32 therefore bounds every f64 partial sum by 32*f32::MAX, far below f64 overflow. The unchanged score collector SumCombiner starts +0 and adds matching f32 leaves converted exactly to f64 in incoming order, then casts once to f32.

The old contribution publication inserted absent +0 entries between those same matching leaves. Adding +0 to an already nonnegative finite f64 partial sum is exact and leaves its bits unchanged. The new publication removes only those absent identity additions, preserving every matching leaf's position relative to other matching leaves. In particular matching +0 leaves remain marked and visited. Starting explicit +0 also agrees with the canonical SumCombiner; any floating iterator-sum negative-zero identity distinction cannot change the admitted nonnegative leaf result. No reassociation, subtraction, f32 intermediate reduction or preference-order fold is introduced.

Signed/exceptional/NaN-weight or unsafe normalization domains still enter the byte-unchanged canonical union before mask state exists. Existing monotone callback threshold assumptions are unchanged. This does not extend public callback guarantees or prove full Lucene API parity. Corrupt zero indexed TF is outside the existing literal score-bound domain, not newly admitted here.

## New independent fixtures

1. `contribution_masks_reset_across_scalar_regions_at_32_terms`: physical docs alternate full 32 matches, incoming ordinal 31 alone, ordinal 0 alone and odd ordinals. A cfg(test)-only candidate observer verifies masks against an independent postings predicate, actual full-mask/bit-31 occurrence, all-scalar replay and multiple region floors. Full callback raw score bits also match both BufferedUnionScorer and an independent incoming-order fold of exact literal leaves. Forward/reversed order deliberately distinguishes midpoint rounding; predecessor/equal/successor thresholds and fixed/growing collector limits compare complete callback traces.
2. `contribution_masks_reset_for_optional_probes_and_pending_single_batches`: n3, legacy/native DEFAULT literals, two dense weak optionals and a sparse driver. Matching sets cycle through driver-only, either optional and both optionals. The observer requires actual single-essential replay, a 128-entry buffer, changed regions and all four masks under a fixed threshold. A growing threshold run includes optional early pruning and a later high-TF winner; every observed partial mask must be a subset of independently known actual matches, and all callback raw bits match canonical union.
3. `matched_underflow_zero_is_marked_without_admitting_a_zero_weight`: legacy positive minimum-subnormal boost, finite positive maximum and safe positive normalization. A long physical field norm makes an actual positive-TF leaf underflow to +0 while the query weight stays admitted. The fixture asserts that raw zero and still requires its matching bit at doc 0, followed by changed touched sets at docs 1 and 4, with complete canonical callback equality. This uses no zero weight, negative/exceptional score, invalid TF or admission bypass.

The existing 13 accepted11 fixture bodies from the first cached-prefix fixture through EOF compare byte-for-byte identical to HEAD. The added observer is wholly cfg(test), records doc/region-floor/driver-kind/competitive/mask, and has no release counter, API or allocation.

## Verification performed here

Read the actual kernel, literal scalar/batch BM25 score expressions, bound admission, canonical SumCombiner, accepted11 independent review and workflow hillclimb. Read the full resulting diff after edits. Rustfmt (edition 2021) and git diff --check pass. A static receipt guard checked base SHA, sole changed tracked file, all three mask assignments and exact existing-fixture preservation.

No cargo/build/test/worker/profile/timing/compression/browser/child delegation/commit/push was run by this agent. The new runtime fixture assertions are not yet executed. Root owns targeted tests, exact default/two configured gates and the frozen pilot/full controls. No speed or memory win is claimed from source. The local mask is 4 logical bytes; compiler placement and RSS are unmeasured. Dense matches add bit handling and a bit loop, so the actual workload can reject this hypothesis.
