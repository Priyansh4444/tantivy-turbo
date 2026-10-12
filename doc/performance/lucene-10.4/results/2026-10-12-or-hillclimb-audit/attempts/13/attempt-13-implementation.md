# Attempt13: metadata range certificates for the same wider OR geometry

Stable isolated candidate for root's source review and runtime gates. No compiler,
test, worker, timing, profile, compression, browser, child task, commit or push was
run by this agent. Root alone owns those jobs. Rustfmt (`--edition 2021 --check`)
and `git diff --check` pass; this is not a runtime correctness or speed result.

## Identity and scope

- Worktree: `/home/pronsh/Coding/playground/search/tantivy-or-metadata-range-oct08`
- Branch: `perf/or-metadata-range-oct08`
- Actual accepted parent11: `43f06833a325d2a18c03bd7e4fa65f2f27944292`
- Only dirty tracked source paths: `src/query/boolean_query/or_maxscore.rs` and
  `src/query/term_query/term_scorer.rs`.
- OR source SHA256: `1a1ce59b330b4a207d353385bf90cac96333cd4c6da49ed52a7c40cd5bb30804`
- Term scorer source SHA256: `21b05a2b6513223a81613ee0f40c4a31f6990a427bb0d4b60324154a55753cdb`
- Full diff SHA256: `589305ea0face0e33545393ef474edf177c0af9803b8a9526113f37c6f6bfb4a`; 1699 lines.

Root supplied corrected rejected12 OR source as the unaccepted geometry. I read
its actual source, implementation/adaptation/witness-fix reports and runtime
rejection, plus my full range grounding. Attempt12's longer-OR pilot ratio was
1.112893 AB and1.025007 BA, with very large sparse-query losses retained. That
supports trying tighter crossed certificates as a distinct mechanism, but does
not isolate causal cycles or establish that a metadata scan will win.

`attempt-13-full.patch`, `attempt-13-source-or.rs`, `attempt-13-source-term.rs`,
and `attempt-13-static-receipt.json` preserve the final actual source/diff/proofs.
The complete draft1652-line diff was read in contiguous1–550/551–1100/1101–1652
chunks; all later final changes were reread in actual source and a complete
old-diff→final-diff comparison. I rechecked scope, actual source bytes, numeric
policy/metadata ownership and independent fixture assertions before this receipt.

## One production mechanism

The caller keeps12's cold `[DocId;32]` selected-end array, original physical caps,
pre-cleanup bound preservation, virtual tight cleanup, old-essential suffix
endpoint proposal exactly once, final prefix/partition rebuild before replay,
and fixed final certificate throughout retained/scalar replay. A crossed clause
now uses `max(saved_raw_bound, scorer.range_max_score(lo,hi))` in place of A's
query-global replacement. No other production caller operation changed relative
to the corrected12 source. Globally optional clauses keep max_doc coverage and
never enter the crossed helper; actual doc>=hi still proves zero.

The new crate-private `TermScorer::range_max_score` is 3199 bytes
of source, owns trusted policy inside the actual query scorer, and clones only
its borrowed `SkipReader`. Neither real skip selection, decoder, postings cursor,
positions, fieldnorm/score weight, nor the real one-block score cache is read as
a writable range owner. No generic/public trait or release observer was added.

For a nonempty half-open range the helper starts at max(floor, real decoded doc).
A noninitial selected reader whose previous-last>=start may already be too late,
so it returns global. Initial identity is remaining_docs==doc_freq, preserving
legal initial doc0 despite the initialized previous-last0. The guard precedes
empty-EOF inference. A cloned reader seeks forward to start and includes each
whole partial/full block until its inclusive last doc covers up_to-1. Half-open
validation precedes subtraction; no sentinel increment occurs.

Missing pairs, actually Basic metadata, nonempty VInt tails, saturated selected
TF ceilings, unsupported/failed query policy, unusable numeric outputs and too-
late metadata return the existing immutable global maximum. Already-loaded tails
are deliberately not reduced. Empty relevant ranges/real EOF return zero. A
maximum that reaches global ends the scan conservatively. There is no entry
budget, new width selector, policy admission, lookahead/result cache, single-block
cache reuse, coarse metadata, format change or per-region allocation.

StoredDefault reuses `SkipReader::block_max_score` under the existing validated
scoring-selection/average binding. Eligible PendingNative resolves through the
existing `native_input_envelope` constructor into immutable Arc envelope or Global;
NativeTransform reuses `input_upper` and `score_from_native_input_upper` with the
existing finite validation. This permitted policy initialization does not move
actual postings or change scalar arithmetic. No custom rescoring of a stored
DEFAULT pair, norm/TF-extrema assumption, reordered score fold or fresh enclosure
formula was introduced.

The metadata scan's maximum covers each still-relevant literal positive-TF leaf
in the requested interval because each trusted full-block certificate covers a
superset, while unresolved nonempty coverage uses global. Keeping max(saved raw,
range) retains the old selected proof too. The helper is restricted to drained
boundaries; it cannot certify retained leaves behind the real first-unread cursor.

## Exact preservation proofs

Static reverse reconstruction replaces only the one new caller expression/comment
and one explicitly adapted native fixture with their corrected12 bytes. It exactly
reconstructs source SHA256
`a53374a75b5dbedb3e8f148b94f8da92a6d4383142e69cc8b50a92330c2f96d1`.
Thus all12 common literal certificate/canonical cursor/batch oracles, strict raw
callback traces, ten original fixture families except the native certificate
assumption, and both documented density corpus corrections remain byte-identical.

Removing the new range helper/test fragment and reversing the test-only Postings
trait import exactly reconstructs accepted11 TermScorer SHA256
`dd1e18b5b1d273cf2c09959ea207997a1e93f854b0fc459fb0a1e76129b87277`.
Every prior scorer implementation/API/fixture is therefore unchanged.

Against actual accepted11, the full4233-byte essential reconciliation/replay/
optional completion/publication/callback/retirement span is byte-identical:
`63086168e3a7fbc768ca8d6529189aacc2d736d1c414c9d00b5796741462093e`.
The3656-byte certified-global-cache/observer/density span is byte-identical:
`07136e553670fe6e6181cb9c52709a5856c83af4edcc00a8ae18073abc83302e`.
ClauseState's complete declaration is byte-identical; no hot selected-end field
or24→32B expansion was imported. Extra release scratch remains128 logical bytes;
actual compiled frame/binary/RSS differences still require root measurements.

## Five new scorer fixture families, plus retained caller proofs

1. `range_max_score_encloses_real_legacy_native_and_partial_endpoints`: existing
   trusted legacy serialized lists and real native indexed metadata, first doc0,
   actual independently literal-scored later TF20 leaf, and a required first-bound
   <crossing-bound<global witness. Requests straddle0/17/127/128/129/255/256/257/384/
   385/TERMINATED; actual cursors inside/beyond requests and reversed empty ranges
   are checked. Every literal unread leaf is independently enumerated. Real
   metadata/load/decoder snapshots stay identical after every helper call. A
   sparse bitpacked corpus spans physical addresses through1,024,000. Nonempty
   loaded tail remains global; real EOF returns0.
2. `range_max_score_native_custom_and_untrusted_provenance_fallbacks`: actual
   native0.9/0.4 Pending→NativeTransform with strictly sub-global range enclosure,
   actual2/1 Pending→Global under existing inverse-cache rejection, both opposite
   scoring-selection directions, shifted DEFAULT average, untrusted public
   construction and k1=0 global fallback. Bound policy changes are allowed;
   actual cursor/metadata/decoder snapshots are unchanged.
3. `range_max_score_missing_saturation_tail_and_legacy_custom_use_global`: an
   actually Basic-indexed field explicitly observes absent metadata, not merely a
   Basic request over TEXT. Native DEFAULT saturation explicitly observes the
   second full block's decoded selected-TF ceiling u32::MAX, while its unsaturated
   first block must give a usable sub-global certificate. Legacy0.9/0.4 constructed
   through public statistics retains validated Global policy. Independent literal
   enclosure remains checked for every returned fallback.
4. `range_max_score_preserves_actual_positions_and_current_block_cache`: real
   WithFreqsAndPositions native DEFAULT and0.9/0.4 scorers at nonzero loaded doc17,
   warmed positions, overlapping ranges/partial blocks/tails and real current-
   block bound calls interleaved with range lookahead. The real selected-block
   certificate's bits must remain unchanged; complete remaining doc/TF/positions/
   streamed-offset positions/raw-score-bit traces match an untouched scorer clone.
5. `range_max_score_later_shallow_start_falls_back_and_valid_shallow_state_stays_unloaded`:
   actual metadata shallow-selected to256 while real decoded doc17 remains stale,
   deliberately too-early requested start returns global, and valid later floor256
   obtains a strictly sub-global certificate without loading or changing anything.
   Independent scalar leaves enclose the latter; subsequent actual seek and full
   remaining positions/scoring trace match a separately built untouched scorer.

These are assertions in unexecuted source, not successful runtime outcomes.
Snapshots cover doc/TF/raw bits when loaded, real skip previous/current endpoints,
remaining count, byte/position offsets, BlockInfo, loaded flag and both decoder
arrays. Private SegmentPostings cur/position-prefix and PositionReader internal
buffers are not directly byte-snapshotted; no new release API exposes them. Their
semantic trace and existing reader ownership fixtures must establish that boundary.
Root should include the existing batch/positions/clone regression fixtures too.

The shared OR oracle still enumerates every relevant literal leaf against every
observed widened certificate and every real seek/advance/batch row/head against
untouched canonical postings. Full strict incoming-order callback bits remain
compared to independent BufferedUnionScorer. Existing n3/n32 overlap/disjoint,
full/partial suffix, exact exclusive head382, across-region refill, midpoint/
threshold-neighbor, stale full/tail/EOF, simulated deletes, exceptional fallback,
real1536/1534 indexed alive-bitset and public TopDocs assertions remain.

## Explicit A-specific native witness correction

The native DEFAULT crossed beta fixture retains the same real indexed TF1 first
block and TF20 doc150 in the second full block. A chose a midpoint toward query-
global and demanded equality with global; B can legitimately keep beta optional
at that old midpoint because its bound is tighter. Retaining that A assertion
would reject the intended certificate improvement.

B's threshold is now a midpoint between the independently computed first selected
alpha+beta bounds and `selected_alpha + literal_beta_leaf_at150`. Neither the new
range helper nor its observed output chooses the threshold. The fixture still
requires actual first-region widening, crossed coverage, old optional→new essential
promotion, later literal beta leaf stronger than saved current-block bound, and
canonical complete traces. It additionally requires final beta bound>=later leaf
AND strictly less than beta's query-global maximum, plus actual scalar advance
127→128. This rules out a global-only stub and proves promoted optional full-block
crossing under a trusted unsaturated native range.

The legacy TF1000 caller remains byte-identical and explicitly expects global;
that crossed block's selected TF saturation is a conservative B fallback. Its
real scalar427→428/raw leaf/callback witness is retained. It is not mislabeled as
a useful unsaturated metadata-range certificate.

The old-essential minimum endpoint rule remains unchanged. Any final sole essential
is the last rank and was old essential too, so its own selected block end caps
that certificate. The existing test-only invariant remains; full/partial retained
suffixes, exact head and refills across regions verify the realizable single route.
Promoted crossed old optionals exercise within-certificate scalar full-block
crossing. No impossible single-driver multi-full-block fixture is asserted.

## Costs and root handoff

The helper performs linear skip-entry parsing over crossed full blocks and real
Arc clone/drop work. Overlapping proposals can parse entries again. Native policy
initialization can allocate the existing cold256-entry inverse cache/envelope once;
no new result allocation exists. Every expansion still rebuilds O(n) final prefix
and partition; fewer preparations can lose to clone/arithmetic/scan/candidate work.
None of these costs has been timed or assigned cycles here. Format/index bytes,
scored batch size, hot record and replay owner remain unchanged by source proof.

Root must review actual source/diff, run the five range fixtures plus all existing
OR/reader fixtures first, then native build/three exact1227 gates, unchanged frozen
pilot and conditional broader/full paired controls. Runtime realizability remains
open, especially changed bound quality's effect on explicit path witnesses. No
performance, RSS, full Lucene feature/API parity or universal correctness claim is
made. Source is held fixed until root requests a specific correction.

Actual diffstat:

```text
src/query/boolean_query/or_maxscore.rs | 1015 +++++++++++++++++++++++++++++++-
 src/query/term_query/term_scorer.rs    |  522 +++++++++++++++-
 2 files changed, 1518 insertions(+), 19 deletions(-)
```
