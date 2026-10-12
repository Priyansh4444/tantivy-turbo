# Attempt 19: capped proposals with exact tight-partition admission

Source-only, unmeasured, unaccepted. Held tree `search/tantivy-or-capped-partition-oct11`, branch `perf/or-capped-partition-oct11`, parent accepted11 `43f06833a325d2a18c03bd7e4fa65f2f27944292`. Attempt18's held tree was not edited. No build, test, worker, benchmark, Python, profiling, compression, browser, process alteration, commit, push or child task ran. Root owns all runtime work and keep/revert decisions.

Read workflow hillclimb and the actual decision trail including completed rejection16; read both safe-widening designs, actual complete17 OR source, actual13/17 TermScorer delta and all range fixture families,17 implementation/receipt, actual16 additive cap fixtures and retained16 geometry-only fixture correction. Parent reports17's mixed per-query pilot regressions despite aggregate improvement; those observations motivate a combined mechanism, not a causal timing conclusion.

## Cheap-cost decision and one combined mechanism

The combination is coherent and cheap relative to the existing17 candidate: cap the proposed endpoint **before**17's speculative optional loop, then retain that loop and exact optional-prefix admission/rollback verbatim. The sole release delta from17 is the compression constant import and

```rust
selected_ends[first_essential..num_terms]
    .iter().copied().min().unwrap_or(max_doc)
    .min(lo.saturating_add(COMPRESSION_BLOCK_SIZE as DocId).min(max_doc))
    .max(tight_hi)
```

No arbitrary arity, query ID, density threshold or width tuning is added. This is the explicitly requested A+B trial;19 does not stack18's independent minimum specialization or15's density cache. It retains13's metadata helper and17's 256 logical bytes of cold arrays. The physical cap adds no array, hot-record field, allocation, signature, module or cache.

Every attempted expansion has `lo < tight_hi < proposed_hi <= max_doc` and `proposed_hi-lo <=128`; an existing wide tight interval is preserved. At the actual prepared caller, a crossed clone starts at a covering block, so a128-address lookahead intersects at most two complete128-posting blocks, or uses the existing conservative fallback. This is not a bound on all general helper requests or the whole query. Many repeated short proposals, commit passes, clone/drop and policy initialization can still cost more than they save.

17's proof survives unchanged: proposed optional bounds never decrease their tight effective bounds; folding every computed optional prefix with the existing original-n transform proves all ranks before the original first essential remain optional. The next old essential still exceeds the threshold and is covered by its selected block, proving exact equality of the partition. Early overflow rejects without changing live prefix/certificate/endpoint/real cursors. PendingNative policy initialization remains allowed; whole scorer object byte identity after rejection is not asserted. The cap cannot weaken complete coverage.

The production speculative loop, entire shared replay/retirement, hot record, immutable global certificate/live density choice, canonical callback oracle, scalar leaf enumeration, cursor progress oracle and indexed original corpus/scorer helpers compare byte-for-byte against frozen17 via retained shell-extracted spans. The entire TermScorer source compares identical against17, including provenance/fallback/cache/positions fixtures. All original17 corpus data, boosts, threshold/heap/deletion schedules and semantic oracles are retained. Only cap-infeasible geometry assertions change.

## Six additive fixture families

These are source assertions, not executed outcomes:

1. n3/n32, both incoming orders, Top0/1/10: real initial seek100, dense tight128, future132..731 with TF1000@200, stride4 driver600. Proposal228 admits future boost.001 and rejects boost.02; require actual original last essential rank, restoration or exact rejected zero/prefix bits, admitted32-row batch/head228 and later canonical200. Always-admit and always-reject stubs fail opposite cases.
2. Four terms and both orders/Top0/1/10: tight128, proposed228, future132 high TF and later optional133. Require actual rejection after precisely two computed optional bounds, leaving the later optional uncomputed, exact chosen tight prefix/zero bits and later canonical200. Continuing scans past known overflow fails the witness.
3. n3/n32 wide tight509 from stride4 postings: do not shrink to128; require full128 retained batch/head512. Same family also requires dense full refills at0/128/256, and a competitive one-row tail384 retiring to actual EOF.
4. n3/n32 initial100, tight128, old essential endpoint192 before cap228: require actual admission to192,92-row batch ending191, untouched first unread winning192/TF2 and its ownership in the next refill/callback.
5. Actual native index: alpha/beta dense with stronger beta TF20@138, gamma2-of3, initial17. Literal selected-alpha/beta versus later-beta midpoint derives threshold independently. Require actual attempted145/rejected128, partition2, strict sub-global computed beta enclosure, selected raw bits/prefix rollback and later canonical138. This replaces the removed first-lo0 unbounded crossed-promotion witness without using the helper to derive expected scores.
6. Independent u64 address-ceiling arithmetic near TERMINATED plus actual max_doc0 exhausted-scorer empty traversal. Ordinary tails/refills supply real cursor semantics without an enormous norm allocation.

The observer records max_doc only under cfg(test), and uses independent u64 endpoint arithmetic for every proposal and chosen widened interval. Existing generic-from-zero partition/raw-prefix/certificate checks still run for every proposal. Existing literal chosen-leaf enclosure and complete first-unread/TF/raw-score cursor progress comparisons are unchanged. The release implementation does not contain observer allocations.

## Exhaustive17 fixture adaptation matrix

All26 old fixture corpora and complete callback/cursor/literal checks remain. The old unbounded-specific names are retained to preserve provenance.

| Existing17 fixture | Geometry adaptation | Preserved semantic evidence |
|---|---|---|
| `partition_expansion_admits_zero_restoration_without_promoting_a_driver` | First lo0/tight128/proposal128 has no attempted restoration; chosen future300 stays positive-zero, zero proposal computations;32-row/head128 replaces full128/head512 | Original rank/density/size facts, all orders/heap schedules and canonical428 |
| `partition_expansion_n3_n32_admission_and_overflow_are_both_realized` | Both original future300 variants first stop at128 without speculation; later inputs unchanged; additive family1 realizes both decisions under capped geometry | All original n3/n32/order/heap/callback428 cases |
| `partition_expansion_stops_at_first_overflow_before_later_optional_bounds` | Original future300/301 lie past cap128; zero speculative bounds, tight zeros/prefix retained; additive family2 realizes early overflow within228 | Original rank/global facts, callback428, complete chosen-certificate/cursor checks |
| `wider_region_restores_old_zero_and_promotes_future_high_tf_block` | First0→128 does not restore future300 or compute range; raw selected bound remains recorded; additive family1 supplies real capped restoration/rejection | Literal stronger428 comparison, actual seek/advance reach428, full canonical bits/progress |
| `wider_single_driver_retains_full_partial_batches_and_exact_end_heads` | Record actual preparation rather than promise widening; winning382 must be an owned batch row with real head after it rather than impossible exclusive382/full128. Family3 proves full128 ownership; family4 proves exclusive192 | All seven populations/TF20/30 and canonical382/partial batch checks |
| `wider_certificates_cover_n3_n32_overlap_disjoint_and_tail_leafs` | Require actual preparations under the general cap oracle rather than unconditional widening | All populations/orders/limits and literal leaf/cursor/canonical comparisons |
| `wider_stale_full_selection_tail_exhaustion_and_initial_skips_are_exact` | Actual preparation replaces unconditional widening witness | Exact skipped intervals, stale seek0→target255→doc256, callbacks300/1398 |
| `wider_midpoint_publication_keeps_literal_incoming_order_and_strict_neighbors` | Actual preparation replaces unconditional widening witness | Literal incoming orders, all adjacent thresholds/limits, strict doc0 and381 |
| `wider_native_indexed_default_and_safe_custom_profiles_enclose_every_leaf` | Actual preparation replaces unconditional widening witness | Original real DEFAULT/custom profiles, n3/n32 and all limit/leaf/callback cases |
| `wider_native_default_crossed_pair_is_replaced_before_stronger_leaf` | First0→128 cap makes crossing to150 infeasible; zero proposal computations and raw selected beta bound chosen. Additive family5 proves genuine sub-global crossed proposal rejection | Original index TF20@150, independent midpoint, literal stronger-beta comparison, callback/seek/advance reach150 |
| `wider_legacy_custom_parameters_reuse_existing_global_bound_policy` | Actual preparation replaces unconditional widening witness | Existing custom/global-policy proof and all exact traces |
| `wider_real_indexed_deletions_keep_threshold_and_public_topdocs_exact` | Require direct/public actual preparations; cap no longer promises expansion in the old density geometry | Actual1536/1534 alive bitset, deleted127 callback/current threshold bits, later live winner, exact public Top1 |
| `wider_simulated_deleted_winners_and_unsafe_fallback_observers_are_exact` | Actual preparation replaces unconditional widening witness | Deleted0/381/384 threshold ownership, canonical600; unsafe domains still have empty observers |
| `cached_global_prefix_preserves_equality_neighbors_and_large_threshold_jumps` | None | Body unchanged |
| `cached_certificate_survives_density_rejection_widening_and_rejection_again` | Original density restoration's first floor must be >=200 and <2160 rather than unbounded2160; original0 and3891 witnesses retained | Entire original corpus, generic sums/rank facts, scheduled callback trace and all0→1→0 density decisions |
| `single_driver_batches_replay_every_dynamic_threshold_exactly` | None | Body unchanged |
| `exact_traces_at_full_block_and_tail_boundaries` | None | Body unchanged |
| `deferred_optional_clause_becomes_essential_after_shallow_full_blocks` | None | Body unchanged |
| `globally_optional_dense_terms_cover_sparse_essential_gaps_and_tails` | None | Body unchanged |
| `threshold_growth_promotes_local_optionals_to_global_certificates` | None | Body unchanged |
| `similar_density_terms_keep_local_certificates_until_both_are_optional` | None | Body unchanged |
| `exhausted_high_cost_remainder_changes_live_density_choice` | None | Body unchanged |
| `exact_midpoint_leaves_preserve_incoming_order_and_strict_ties` | None | Body unchanged |
| `duplicates_disparate_boosts_exhaustion_and_sparse_regions` | None | Body unchanged |
| `exact_growing_threshold_traces_for_mixed_dense_and_sparse_terms` | None | Body unchanged |
| `unsafe_domains_use_exhaustive_canonical_sum_before_any_seek` | None | Body unchanged |

17 inherited13's finite density corpora, which already differ from two accepted11 fixture setups.19 keeps them unchanged and does not claim every accepted11 fixture is byte-identical. Changing geometry witnesses does not establish runtime feasibility; root must archive any failure before a justified fixture correction, retaining all semantic oracles.

## Frozen artifacts and limits

`attempt-19-static-guards.sh` executed successfully once with fail-on-existing output guards; `rustfmt --edition 2021` and `git diff --check` completed. All actual source changes and complete17→19 delta were read. Source is held unchanged, unstaged.

- OR SHA256 `4c56911f9cabe4fd06f8d1fe2f999564267c8fa3196600f29fe1b42ba1e848f7`.
- Term SHA256 `21b05a2b6513223a81613ee0f40c4a31f6990a427bb0d4b60324154a55753cdb`.
- `attempt-19-full.patch` SHA256 `770473eb3c55982717b7e813bc84e867004df986fb120595d020c5ff926f2ea0`.
- `attempt-19-vs-17.patch` SHA256 `cee3d861910107e97bc447b4ed4c59e235aebdf60edcbc917e9ae3cd12ef459b`.
- Shared replay-span SHA256 `4d126326d89b421cf75f0f068644558534550267e8fe48d476518b402192bd6f`.
- Scope: two tracked files, OR1715inserted/18removed, Term521inserted/1removed;32 OR tests in source, six new fixture families versus17.

No runtime correctness pass, compiled layout, binary delta, memory efficiency, storage improvement or speed win is claimed. Complete Lucene API/feature parity remains outside these source changes. Root should independently review the authentic full parent-relative patch and17 delta, run all32 OR plus range/posting ownership fixtures and three1227 exact profiles, then the unchanged untrimmed pilot and conditional controls.
