# Attempt 12: one wider OR region with global replacement certificates

Source is stable for root integration review. This is an uncompiled candidate,
not an accepted performance change. Root alone owns all builds, tests, worker
runs, profiles, timings, compression and browser jobs. No such job, child agent,
commit or rebase was performed here. Rustfmt (`--edition 2021`) and
`git diff --check` pass. The complete actual final diff was read end to end in
three contiguous chunks, and the resulting production/test sections were read.

## Exact provenance and scope

- Worktree: `/home/pronsh/Coding/playground/search/tantivy-or-region-widen-oct07`
- Branch: `perf/or-region-widen-oct07`
- Base: accepted06 `a674aaf2f6187f3010a135dc25fdacf9cab27e53`
- Only changed tracked source: `src/query/boolean_query/or_maxscore.rs`
- Final source SHA256: `00fad7ecc30941fa72e6fbd002db5aff85b5b4e7a76b89403a9624ba65d56559`
- Diff: 923 additions, 15 deletions; 1055 full-diff lines.
- Static comparison receipt: `attempt-12-static-receipt.json` in OUT.
- Attempt11 is intentionally not copied or rebased into this source. Root reports
  its subsequent acceptance at `43f06833a325d2a18c03bd7e4fa65f2f27944292`; root will
  preserve this original patch, explicitly adapt onto that parent, and review the
  actual combined source before running attempt12. None of09/10/11 is imported.

The production change implements the agreed A preparation only. One query-owned
`[DocId;32]` records selected exclusive ends (128 bytes); the hot ClauseState shape
is unchanged. No release observer, per-region allocation, metadata lookahead,
SkipReader clone, scorer API, format, width/density selector, threshold policy,
priority reorder, or pending repartition is added.

## Production invariants

The existing global optional prefix and its density policy are byte-identical
against06. Metadata selection, tail reconciliation, conservative current-block
bound validation and physical cap clipping retain their original operations.
Every new endpoint entry starts at max_doc; actual finite selection overwrites
it with the original safe `last_doc.min(max_doc-1)+1` expression. Globally optional
and exhausted clauses keep max_doc coverage. Sentinel tails use that same clipped
expression without incrementing TERMINATED.

Tentative prefix construction now applies doc>=tight_hi cleanup virtually, leaving
ClauseState.local_max as the trusted pre-cleanup selected bound. An empty tentative
suffix skips exactly tight_hi and makes no widening proposal. Otherwise one
minimum over the old essential suffix chooses final_hi. If it expands, every
final bound is selected afresh from the preserved bound/global maximum/real-doc
zero cases, and the complete prefix and essential partition are rebuilt before
replay. No subsequent partition can propose another width. If no expansion occurs,
the original effective zeros are written into records while reusing the tentative
prefix/partition. An empty final suffix may skip only under its final wider proof.

A selected bound whose endpoint covers final_hi conservatively covers all unread
leaves through the range. An uncovered clause uses its existing admitted global
maximum. An actual doc>=final_hi proves absence; a stale actual doc below lo does
not. The original positive-TF admitted score domain and original-count inflated
sum helper are retained. Original exceptional/signed canonical fallback remains
before metadata mutation. The final certificate/partition stays fixed through
candidate replay, including callback threshold increases and deleted-hit rejection.

The entire block from essential floor reconciliation through single/scalar
candidate selection, literal known-score accumulation, optional probing,
incoming-order publication, callback, cursor retirement and region end is
**byte-identical to06**. Its SHA256 is
`63086168e3a7fbc768ca8d6529189aacc2d736d1c414c9d00b5796741462093e`.
The unchanged global/density block SHA256 is
`bd950e3f104916faf5a8581bbcbdf826a68e6f66a3a47c0c863776a4b9870391`.
These were checked by exact substring equality, not whitespace-normalized matching.

Actual ClauseState cursor methods add cfg(test)-only observations after their
unchanged real operations. These observers never exist in release. An enabled
observer records post-mutation doc/TF/raw-score bits and emitted batch rows/next
real head. Conditional test scoring reads the same literal scorer; it adds no
cursor or position mutation.

## Test-only proof surface and ten new fixture families

`WIDENED_REGIONS` records only actual expansions: lo/tight_hi/final_hi, old and
final partition, rank-to-ordinal mapping, authoritative docs, saved original
selected bounds, selected ends and final bounds. `CURSOR_PROGRESS` records actual
seek/advance/fill operations; `TIGHT_SKIPS` records the original empty-certificate
skip. All are cfg(test) thread-local opt-in vectors, absent from release.

Each observed expanded certificate is checked against independent untouched
scalar scorers, enumerating every relevant positive-TF literal leaf in its
half-open range and comparing to the final bound. The oracle also verifies every
observed actual seek/advance `(doc,TF,scorebits)` and each contiguous emitted batch
row/retained next head against the untouched canonical postings. Complete strict
callback traces/raw bits remain compared to independent BufferedUnionScorer;
these are additional witnesses, not replacements for exact gates.

1. `wider_region_restores_old_zero_and_promotes_future_high_tf_block`: the future
   term starts at300, beyond the original tight endpoint but inside the expanded
   certificate. Its first128 TF1 postings end at427; doc428 has TF1000. Observe
   old optional role, restored nonzero global bound, new essential role, later
   literal leaf greater than the saved current-block bound, and actual scalar
   advance427→428 with TF1000. A later428 callback must survive. Reverse incoming
   order retains exact publication. These fixtures use the existing trusted
   legacy DEFAULT serializer/scorer helper.
2. `wider_single_driver_retains_full_partial_batches_and_exact_end_heads`: lengths
   127/128/129/255/256/257/768, comparable costs forcing the unchanged global
   density test to reject exemptions, real expansion, full and partial suffixes,
   growing Top1 threshold, and authoritative head382 exactly at exclusive382
   after a128-row fill. Later382 remains a winning callback.
3. `wider_certificates_cover_n3_n32_overlap_disjoint_and_tail_leafs`: genuine n3
   and n32, overlap versus fully odd/even-disjoint streams, changing norms/TFs,
   reversed order and fixed/Top1/Top10 thresholds. Every observed bound encloses
   untouched leaves. A layout receipt prints array/ClauseState host size when
   root actually runs it.
4. `wider_stale_full_selection_tail_exhaustion_and_initial_skips_are_exact`:
   initial skipped certificates, explicitly exhausted clause, stale actual head0
   while a full block is shallow-selected, actual seek255→256, later winners300
   and1398, sentinel tail/EOF reconciliation and fixed/growing/infinite thresholds.
5. `wider_midpoint_publication_keeps_literal_incoming_order_and_strict_neighbors`:
   actual expansion with literal leaves1,2^-24,2^-53,2^-53, three incoming orders,
   exact publication threshold and both adjacent f32 neighbors, fixed/Top1/Top10.
6. `wider_native_indexed_default_and_safe_custom_profiles_enclose_every_leaf`:
   real RAM index with native serialized selection, n3/n32, DEFAULT,0.9/0.4,2/1
   and0/0.75 profiles, actual expansions plus leaf enclosure/full traces. Existing
   scorer policy/enclosure/global fallback is reused; this observer does not
   separately assert the private transform enum or invent normalization arithmetic.
7. `wider_native_default_crossed_pair_is_replaced_before_stronger_leaf`: real
   native DEFAULT first beta block contains only TF1; crossed doc150 has TF20.
   A threshold derived from existing selected/global maxima makes beta originally
   optional and finally essential. Observe actual first-region expansion, a later
   literal leaf greater than its preserved current-block bound, and a final bound
   exactly equal to its query-global maximum. The stronger-than-current-bound
   assertion rules out a mislabeled all-global witness.
8. `wider_legacy_custom_parameters_reuse_existing_global_bound_policy`: existing
   public custom statistics/`Bm25Weight::for_terms` construction, valid0.9/0.4,2/1
   and0/0.75, actual expansion and exact traces. Existing nondefault legacy full
   block global fallback is explicitly observed; this case does not claim a
   trusted custom stored-pair upgrade.
9. `wider_real_indexed_deletions_keep_threshold_and_public_topdocs_exact`: physical
   1536/live1534 RAM index, deleted early hits0/127, stronger later live hit,
   collector-owned alive-bitset rejection without threshold growth, complete
   canonical callback traces, actual widening in both the direct kernel and
   public BooleanQuery+TopDocs, and exact public winner/raw score.
10. `wider_simulated_deleted_winners_and_unsafe_fallback_observers_are_exact`:
    early rejected winning docs0/381/384 leave current threshold unchanged,
    later600 callback survives, real expansion, and zero/negative/infinite/NaN
    boosts enter canonical fallback with no widening/cursor/skip observation.

All test result statements above describe assertions and intended witnesses;
none has been compiled or run by this agent. Root must validate their realizability
and the release candidate. Existing unsafe normalization/NaN-threshold tests are
retained; no arbitrary zero-TF or new scoring-policy admission is created.

## Agreed geometry corrections

Grounding corrected the original design's proposed single-driver multiple-full-
block span. A nonempty old suffix always contains the last preference rank; a
final sole suffix contains that same rank. The chosen minimum cannot exceed its
own selected end. The test-only evidence checks this invariant for every observed
widened sole suffix. Full/partial retained batches, current threshold, exclusive
head and across-region refill witnesses replace an unrealizable same-certificate
multi-full-block assertion. A **promoted crossed optional** can cross its own fine
blocks on the scalar multi-essential route because its final bound is global;
fixture1 proves that actual within-region route. Root agreed this correction, and
`design-global-region-widen.md` now has a clearly labeled design-only correction.
No production shape changed; B's runtime alternative was left untouched.

One existing density observer fixture required an explicit corpus geometry change,
not weaker assertions. Its old two-posting tail can now legitimately widen to
max_doc and finish in one preparation, removing the fixture's required later
re-evaluation. It now has128 early sparse postings plus a3500 tail hit. It still
asserts global certificate count1 throughout, effective choice0 at live cost200,
then effective choice1 at cost129 **in a later preparation after high-cost actual
exhaustion**, complete canonical callback/raw-score equality and exact sparse-hit
list. Every other existing test is byte-identical; the static equality SHA256 for
those untouched existing tests is
`bfe3a5216534ee4986f70f1b5baa708f741ffb570df0b2f46f833326d664db52`.

## Runtime and evidence limits

Fixed additional release scratch is128 bytes; ClauseState and2048-byte batch stay
unchanged. Each prepared region writes ends; each nonempty tentative suffix scans
its ends; each actual expansion adds one final bound/prefix pass and partition.
No binary-size, RSS, speed or universal correctness claim follows from source.
Loose globals can promote dense clauses and massively increase scalar candidates;
query920 remains an explicit adverse case. Frozen latency gates decide keep/reject.

The OR caller exposes doc/frequency/score but no positions API. This patch adds
no release interface for positions. Existing reader fixtures
`batch_cursor_matches_literal_postings_across_blocks_and_tails`,
`batch_then_scalar_reads_preserve_positions_and_skipfreq`, and
`position_prefix_tracks_repeated_reads_seeks_blocks_and_clones` cover the unchanged
reader owner; root can reuse/run them and the broad suite. This new caller observer
proves canonical doc/TF/raw-score progress, not a new positions-state comparison.
All final source proofs/gates must be repeated against root's actual accepted11
adaptation before timing; this06 receipt must not be mistaken for its SHA.
