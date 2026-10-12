# Portable hillclimb packet grounding — 2026-10-08

This is a design and source-grounding receipt, not a newly run audit, packet, benchmark, or publication. Root alone owns eventual packet authoring, execution, compression, and publication. The held attempt 13 implementation has not been changed. No raw timing files were copied. Attempts 13/14 are pending in this design snapshot; replace their descriptors only after Root records their actual decisions.

## Grounded inputs and scope

Local roots:

- `OUT=/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07`.
- Old tracked packet: `/home/pronsh/Coding/playground/search/tantivy-pr2937/doc/performance/lucene-10.4/results/2026-10-07-or-compact-state`.
- Frozen comparison references: `OUT.parent/lucene-rs-balanced-oct06`.

Read the old packet's full `verify-tracked-packet.py`, `recompute.py`, `pack.py`, manifest/origin schemas; current `run-contract.json`, `decision.tsv`, `baseline-provenance.json`, all four parent descriptors and acceptance receipts, all attempt 01..12 archive inventories, representative source/guard/gate/timing schemas, frozen `balanced.py`, `pilot.py`, `compare.py`, `stage.py`, `analyze.py`, `analyze_port.py`, `measure_audit.py`, `port_group_audit.py`, attempt 11 publication audit, attempt 01 profile correction, and failed-stage receipts/log signatures. Only metadata and representative raw rows were inspected; no old verifier or benchmark controller was executed. Throughput checkpoint: n/a, read-only publication investigation.

The existing baseline is commit `0e9329dfe782187a8feda0024102e4aabf517c28`, worker SHA256 `2efc80ae6cc57c4163cab798f50bfb35ceb085c039bc4dd4b8fe31d22d387bbf`. Accepted chain: baseline →01→03→06→11. Rejected 02 has parent 01; rejected 04/05 have parent 03; rejected 07/08/09/10 have parent 06; rejected 12 has parent 11. `run-contract.json` requires candidate/fixed-port arithmetic mean-query-median ratio below 1 in both orders for port-style OR 301 and OR3+ 103 beyond group self controls. Parent improvements do not establish that target. Accepted 11's actual-port OR3+ remains `1.805087/1.827905`; its target is false.

## Existing packet: what transfers and what cannot be reused unchanged

The old `packet-manifest.json` maps 162 relative files to stored `bytes`/`sha256` and records baseline/core/fixed-port commits. Its own bytes are not inside that self-referential map. `copy-origins.json` maps copied files to `original_path`, `original_bytes`, `original_sha256`; telemetry summaries instead have `derived_from_sha256` plus an explicit process-inventory omission. Old `pack.py` gzip-compresses files above 65,536 B except source/readable suffixes, with compression level 6 and mtime 0. Its `recompute.py` checks exact inventory, stored and decoded hashes, complete cells, timed count/top10 checksums, raw integer score bits, saved CSVs and summaries. Preserve those useful properties.

Concrete incompatibilities:

| Old source location | Actual limitation | Required change |
|---|---|---|
| `pack.py:5`, `recompute.py:64–67` | Worktree/source guard key hardcoded to `tantivy-or-refinement-oct07`; current stage snapshots use `tantivy-or-hillclimb-oct07`. | Each attempt/revision declares its actual snapshot key and content hashes. |
| `pack.py:32`, `recompute.py` four-file source loop | Only OR, boolean_weight, boolean_query/mod, query_pruning_correctness are copied/bound. Current 01 changes block_wand_union; 03 also changes segment_postings, bm25, term_query/mod and term_scorer; 06 changes score_combiner; 13 changes term_scorer. | Include the union of actual changed files and relevant fixture files, with explicit revision-to-source bindings; do not mistake a four-file sample for all compiled source. |
| `recompute.py:69–76` | Stages named `build-reviewed`/`tests-reviewed`; totals fixed 1,428 passed/7 ignored and layout 24 B/1,160 B. | Actual stage names/expected outcomes/test totals/scratch claims are per descriptor. Accepted 01/03/06/11 have 1,434/1,438/1,440/1,442 passed, each 7 ignored. Never synthesize a layout run for a trial without one. |
| `recompute.py:137–148` | Requires every affected round's arithmetic gain to exceed maximum observed round control. | Recompute all six rounds as descriptive evidence. Apply the recorded current aggregate arithmetic-group control rule separately. Accepted 11 has retained OR3+ round 0 BA ratio 1.005062, so universal per-round improvement is false. |
| `recompute.py:150–176` | Actual-port aggregates include port self geomeans but no independent recomputation of current arithmetic group controls/OR target. | Recompute `port_group_audit.py` arithmetic ratios/noise and target boolean from raw cells, including for accepted 01/03 where no such derived receipt is archived. |
| `recompute.py:179–184` | Assumes one valid baseline and candidate profile, 21 replies each, binds binaries but has no available/excluded/overwritten lifecycle. | Preserve explicit diagnostic statuses; see attempt 01 limitation below. |
| `recompute.py:191` | Fixed 1,160 B/776B/72B scratch accounting. | Keep measured layout, source-derived array payload, binary sections/file size and RSS as separate claims, each tied to evidence and trial. 2KiB batch scratch, 16 B persistent cache and 128 B endpoint scratch are different objects. |
| configured-gate loop in old `recompute.py` | Ordered zip against query-map values; current requirement is all IDs and profiles independently. | Enforce exact unique ID set and case equality by ID before score comparisons. |
| old `pack.py:25–28` | Publishes derived loadavg summaries while original process inventory remains local. | Retain raw full telemetry for this audit design, or explicitly label any Root-approved derived/redacted artifact and its limits. Do not call a telemetry summary the original. |
| old `verify-tracked-packet.py` | Requires Git and a checkout, uses `git archive` and tar extraction with `filter='data'`. | Keep as a distinct optional tracked-publication wrapper. Standalone audit must run from extracted packet with Python stdlib, no Git, Rust, Java, engines, absolute local paths, or third-party packages. |
| current `measure_audit.py`/`port_group_audit.py` | Group aggregators do not themselves validate all schedule keys/duplicates/checksums; `measure_audit.py` invokes external `size`. | New auditor first validates raw cells independently, then computes all aggregates; read retained section measurements instead of calling `size`. |
| old single-experiment manifest | No parent chain, rejected-trial policy, failed-revision statuses or pending descriptors. | Represent attempts and revisions explicitly, not by applying all old success assertions to every directory. |

`analyze.py` independently catches full main-lane duplicate/missing cells/checksums but fixes 3 pairs × 3 rounds in its expected-set comprehension; its summary's controls are geomeans. `measure_audit.py` supplies the current arithmetic control rule. `analyze_port.py` checks complete actual-port cells/checksums but its receipt alone is not the arithmetic OR target. The portable auditor must combine these behaviors without importing their absolute-path-bound controllers.

## Minimal standalone shape

Proposed top-level files: `README.md`, `packet-manifest.json`, `attempts.json`, `recompute.py`, and optional `verify-tracked-packet.py`. Put immutable common artifacts under `common/`, per-attempt artifacts under `attempts/XX/`, revision-specific failures under `attempts/XX/revisions/<label>/`, and changed source contents under a deduplicated `sources/<sha256>/<basename>`. This is a proposed layout, not an inventory of an already-created packet.

Use one small stdlib audit entrypoint that resolves its root from `__file__`, reads JSON/JSONL/CSV, streams stored hashes and raw timing rows, handles declared identity/gzip encoding, uses `statistics`/`math` for deterministic arithmetic and tolerances, and emits a structured result with every attempt's outcome, gate counts, lane counts, loss lists and target status. Do not execute copied controllers, subprocess engines, compile sources, or reconstruct profile measurements during audit. Controller copies explain provenance; they are not the portable reader.

Manifest schema should explicitly separate physical inventory and logical bindings:

```json
{
  "schema": "tantivy-or-hillclimb-audit-v1",
  "files": {
    "common/queries.jsonl": {
      "encoding": "identity", "stored_bytes": 92940, "stored_sha256": "...",
      "raw_bytes": 92940, "raw_sha256": "...", "role": "frozen_queries"
    }
  },
  "bindings": {"frozen_queries": "common/queries.jsonl"},
  "origins": {"common/queries.jsonl": {"original_path": "..."}}
}
```

Require exact inventory excluding only the manifest itself; include the auditor, descriptors, origin mappings and README in inventory. Reject absolute/traversal/duplicate logical paths and symlink inputs. For each file validate stored length/digest, then decoded length/digest. Content hashes are integrity bindings, not signatures proving who performed a historic run. Preserve original absolute strings as provenance only; auditor resolves all working references through packet-relative bindings. An optional Git wrapper checks exact tracked files and audits a clean committed archive, reporting HEAD and archive digest independently of the portable result.

Attempt descriptor fields: `id`, `outcome` (`accepted`, `rejected`, `pending`), `accepted_commit` if any, `parent_attempt`/baseline reference, `candidate_binary_sha256`, `configured_binary_sha256`, `compiled_snapshot`, `source_snapshot_key`, `source_bindings`, `patch`, `stages` with actual expected statuses, `exact_profiles`, `pilot_runs`, `parent_lane`, `actual_port_lane`, `diagnostics`, `decision_receipt`, `limitations`. Each failed revision has its own snapshot/source hashes, logs, command and expected nonzero status. Pending 13/14 have actual source/parent references only when known, and no invented binary/gate/timing claims.

Bindings should point to the full original descriptor bytes as well as relocated proof files. For example `selected_parent.descriptor_sha256` authenticates retained `parent-11.json` bytes; its original absolute `compiled_sources` string maps explicitly to packet-relative attempt 11 build snapshot. Compare the SHA in that descriptor to the relocated file bytes. Do not rewrite an old descriptor and then pretend its original digest still applies.

## Common frozen artifacts to retain once

1. `lucene-rs-balanced-oct06/queries.jsonl`, SHA256 `39a54b200392befbde83469841ce6eaefe7ec1850bb5b0e25c74a0eefa709a4e`,92,940B. Exactly 1,227 distinct IDs0..1226. Timed IDs0..1220; six correctness-only cases1221..1226. Retain full case fields (`id`,`kind`,`terms`,`tags`), not just terms. Wiki20 and port-style 1,201 differ; port-style TERM300/AND300/OR 301/PHRASE300, OR2=198/OR3+=103.
2. `lucene-rs-balanced-oct06/verify/dumps.json` (3,671,593B) contains fixed initial Tantivy and actual-port exact count/hits references. Preserve entire array and order/case metadata. Default 11,509 returned hit bit-values is independently derived and can be checked against this workload, not a general constant for configured profiles.
3. Full-payload `audit-port/stdout.log`, `receipt.json`, `index-after.json`, plus frozen Tantivy index file map/ruler reference. Audit stdout records documents1m, fielddocs917,578, terms1,642,896, postings126,703,012, positions294,827,020, payload SHA256 `893a75958e6d15c997d414d5fb5fc47c633aa18c68fcd33bc0f8e13abfa8dbb8`, physical SHA256 `6f7082e3ff479c7be7fe6f819857716630fc4b6b64acc2d6005f1107127029c7`, termlist SHA256 `dc15193a1ec1d93b778bf9cb0caedd1e71129180af0a1ea0e04b0f742c6c4d20`. Receipt returncode 0 and integrity pass matter. These are retained receipts and file maps; full index payload is outside the minimal packet.
4. Current `run-contract.json`, `decision.tsv`, `baseline-provenance.json`, `parent-01/03/06/11.json`. Baseline provenance records 600 compiled non-doc source matches/no differences. Copy its referenced frozen source-map evidence if claiming an independently checked baseline provenance; receipt-only evidence must remain labeled receipt-only.
5. Frozen controller source: stage/balanced/compare/configured_gate/pilot/port_measure/analyze/analyze_port/measure_audit/port_group_audit; any per-archive controller variants, test Cargo.lock, five harness files and the actual port worker source. Common deduplication must follow equality of bytes, not filenames. `baseline-provenance.json` binds initial rulers; guarded stage snapshots bind actual files used later.
6. Fixed actual-port identity: commit `e5d1f81bff42c87886b12770f3c79648bb1963ed`, worker SHA256 `b0923daad72d662a056496ab115a57519ec474216fe05b1522e6529ae2154a2a`. Preserve source snapshot/harness and index map references, not merely commit text.

## Stage/source and exact-gate proof surface

Actual build/test `before.json`/`after.json` snapshots use top-level `tantivy-or-hillclimb-oct07` (`head`,`status`,`files`), `lucene-rs-idf-fix-oct06`, and `balanced_harness` (five-file map). Source files map relative paths to SHA256. Final builds normally retain the accepted parent HEAD while the candidate is dirty. The candidate identity is the per-file content map plus patch/binary, not the HEAD string. Retain snapshots whole. Current `stage.py` omits exactly generated `tests/compat_tests_data/index_v11/`; preserve that explicit exclusion, do not ignore every compatibility fixture or all untracked files.

Receipts bind command, returncode, compiler 1.99.0, flags `-C target-cpu=native`, jobs 2, profile, source/index unchanged, stdout/stderr digests and wall time. Release profile is opt-level 3 / fat LTO / 1 codegen unit / panic abort; test/check profile differs. Recompute log digests from retained raw logs; check declared before==after and source maps across build/gates/timing. Check stage source-map equality only where the actual lane claims the same revision; do not conflate a failed pre-fix fixture with final source. Include complete changed source contents for each trial/revision referenced by claims. The patch headers show 01 block_wand_union+boolean_weight; 03 segment_postings+bm25+OR+term_query/mod+term_scorer; 06 OR+score_combiner; other 02..12 trial patches vary primarily OR, with04/05 block_wand_union. 13 adds term_scorer. Source blobs can be deduplicated by SHA256. A retained full source hash map proves more inventory than a copied four-file subset but cannot supply absent source contents to a reader.

Default `verify` guard contains source, indices, queries, binary map, harness, controller, payload-audit and optional comparison-controller/selected-parent bindings. Receipt: `queries`, `failures`, `unchanged`, `pass`, `wall_seconds`. Dump row: `case`, `tantivy`, `port`; each response has exact `count`, ordered `hits=[[docid,u32_scorebits],...]`, ID and independent `oracle`. In the parent lane `port` is preserved parent Tantivy, not lucene-rs. Default gate controller compares optimized candidate to own exhaustive oracle, frozen initial Tantivy, and frozen actual port. The auditor must independently perform those equality checks rather than trust empty `failures.json` or boolean pass.

For each final attempt 01..12 verify all 1,227 unique IDs and complete cases. Compare actual / own oracle exact count, ordered docs and unsigned 32-bit raw bits; default compare against both frozen references. Check hitlen=min(10,count), doc range[0,1m), bit range[0,2^32), unique docs and score/order behavior using retained worker protocol/validation contract. Do not compare only formatted floats. Bind all gate profiles to their actual binary/source snapshots and READY records. Default READY uses k1bits`3f99999a`, bbits`3f400000`, collector`top10-no-totals`, query cache disabled, doc_count917578/max_doc1m/num_docs1m/sumtotalTF294827020. Configured profiles `.9/.4` READY bits`3f666666`/`3ecccccd` and `2/1` bits`40000000`/`3f800000`; they compare optimized candidate to own independent exhaustive oracle, not default fixed-port scores. Preserve their request arguments/READY, receipts, full dumps, failures, protocols and before/after guards. Three profiles mean three exact gates, not three comparable performance lanes.

## All timing rows and controls

Main parent lane existing `measure/schedule.json`: warmup 10,16 samples/order,3 rounds,CPU4, seed 20261006,1,221 IDs, pairs candidate-parent/candidate-self/parent-self. JSONL row keys `(pair,round,order,id,mode,side)` plus engine/checksum/ns. Require the exact full Cartesian set: 3 × 3 × 2 × 1221 × 2 modes × 2 sides = 87,912 cells, 1,406,592 individual timings. Reject duplicates/extras/missing, wrong engine/ID/mode, nonpositive or non-integer sample values and samplecount≠16; avoid Python bool-as-int acceptance. For count checksum expected `(exact_count*16) % 2**64`; top10 checksum `(sum(doc ^ rawbits)*16) % 2**64`. Bind expectations to exact dump/references and lane identity. Verify retained protocol requests/responses and READY labels against schedule/binaries where included, including warmups; seed and stored order can be checked from complete raw row sequence in addition to Cartesian completeness.

Pool three rounds' 48 ns values separately for each pair/order/query/mode/side; compute query median. Primary ratio is Amedian/Bmedian. Compute per-query self ratios TT/PP each order and noise=max(abs(log(each of four controls))). A win beyond controls requires both primary log ratio<−noise; a loss requires both>noise. Preserve every query and both modes; publish complete recomputed CSV and every loss, not just worst 20. Groups use literal query metadata, not parsing joined strings; port-style OR3+ uses `len(terms)>=3`, retaining duplicates in a repeated-term query.

For each group and order current arithmetic primary/TT/PP ratio is sum(query mediansA)/sum(query mediansB), equivalently ratio of mean query medians. Group noise is maxabslog(TT/PP bothorders); controlled gain requires both primary log ratios<−noise. Recompute `attempt-XX-metrics.json` groups/rounds/regression lists and `summary.json` geomeans separately; never substitute geomean for the target statistic. Preserve six per-round arithmetic ratios and mixed rounds regardless of aggregate result. A descriptor should state the recorded acceptance condition; recomputing evidence need not pretend every accepted change improves every query/group/round.

Actual-port lane `port-measure/schedule.json`:2 pairs candidate-actualport/actualport-self,3 rounds,AB/BA,1,221 IDs,top10 only,16samples. Exactly 29,304 cells / 468,864 timings. Check same row/key/engine/checksum properties and candidate shared binary/source identity with parent lane; fixed port must bind actual port binary and port index, not parent Tantivy. Recompute `port-per-query.csv`, `port-summary.json` and arithmetic port group audit: primaryratio from pair0, selfratio from pair1, noise=maxabslog(portselfAB/BA). Target requires controlled gain for both OR 301 and OR3+ 103. Contemporaneous controls are fixed-port self only; candidate self lives in separate parent lane. Do not combine independent lane noise into a newly invented matched four-control schedule. Tantivy top10 collector omits totals; port maintains lower-bound totals 1000. Preserve this real collector-work difference.

Pilot lane: `pilot-final/samples.json` is an array, keys `(order,id,side)`; no pair/round. Frozen `pilot.py` chooses all timed OR, Wiki20, or IDs divisible by 17: 373 queries, four warmups,12 samples/order,AB/BA,seed 20261006,top10 only. Require 1,492 cells / 17,904 timings, matching engines/IDs/score checksum. There is no existing separate pilot scheduleJSON; add a descriptor explicitly recording these grounded parameters/ID set and bind the actual archived pilot controller digest. Recompute all ratios and receipt mean/median ratio groups, plus worst 20 sorted by min(AB,BA), while retaining all 373 rows. These pilots lack main-lane self controls and cannot prove a control-adjusted full-suite result. Attempt 10 retains `pilot-first-load-heavy` and `pilot-final`; audit both, with their original load context and mixed orders. Never drop the first as though it did not exist.

Keep full `telemetry.json`, RSS, protocols/stderr and raw schedules alongside raw samples; source/binary/index/query maps before and after must match. Full timing retention does not remove CPU-contention limits. Current indices are eight-file Tantivy map and eight-file Lucene map, excluding lockfiles by recorded policy. Index bytes remain external to the minimal packet, so guard hashes cannot re-audit payload files on another machine.

## Outcome inventory and failed evidence

| Trial | Outcome | Parent | Evidence lane limit |
|---|---|---|---|
|01|accepted|baseline|Full parent/actual-port lanes;1,434 tests; three exact gates; profile correction below.|
|02|rejected|01|11 targeted, three exact gates, pilot; full sampler/actual-port not run.|
|03|accepted|01|Full lanes;1,438 tests;2KiB fixed batch scratch; longer OR port target remains open.|
|04|rejected|03|10 targeted, three exact gates, mixed pilot; no reliable gain.|
|05|rejected|03|Three exact gates, pilot; no new tests; symbol shrink retained but reliable speed gain absent.|
|06|accepted|03|Full lanes;1,440 tests; no new allocation; binary−896B.|
|07|rejected|06|12 targeted, three exact gates, mixed pilot; initial/v2 design artifacts retained.|
|08|rejected|06|19 targeted final, three exact gates, pilot; failed fixture compilation and diagnostic profiles retained.|
|09|rejected|06|20 targeted final, three exact gates, pilot; pre-oracle-fix 17 pass / 3 fail retained.|
|10|rejected|06|17 targeted final, three exact gates, two pilots; failed collector fixture compilation retained.|
|11|accepted|06|Full lanes;1,442 tests;16 B logical global-prefix cache; mixed round and19 whole-suite per-query losses retained; actual-port target false.|
|12|rejected|11|23 targeted final, three exact gates, pilot; pre-witness-fix 21 pass / 2 fail retained; full sampler/broadtests/clippy/actual-port not run.|
|13|pending|11|Held source/receipt only in this task; no invented runtime results.|
|14|pending|11|Record actual Root source/decision later; no invented runtime results.|

Retain accepted/rejected patches and actual implementation/review/acceptance/rejection docs. Not every archive has every diagnostic controller:04/05 reuse frozen common controller references; do not fabricate per-archive files. Rejected trials have no full actual-port comparison; absent lane means `not_run`, not `pass` with zero samples. Overall portable audit can pass integrity while reporting rejected trials and expected failed revisions; it must distinguish an artifact-integrity pass from a trial-performance win.

Failed revision records:

- **08** `pre-fixture-fix/tests-targeted`: returncode 101, stderr `E0689` ambiguous float `.max`; corresponding build-final succeeded with returncode 0, so describe a fixture/test compilation failure, not a failed release build. `duplicate-unfixed-tests` is also retained separately.
- **09** `pre-oracle-fix/tests-targeted`: returncode 101, stdout 17 passed / 3 failed, with three heap-oracle test failures; final 20 pass follows corrected fixture semantics.
- **10** `pre-fixture-fix/tests-targeted`: returncode 101, stderr `E0277` TopDocs collector trait failure; corrected ordered collector fixture has final 17 pass.
- **12** `pre-witness-fix/tests-targeted`: returncode 101,21 passed / 2 failed; two named wider-region witness failures, final 23 pass. Bind original/adapted/corrected static/source receipts separately; only corrected source is the actual final pilot candidate.

For each classify expected status/result signatures and retain original before/after snapshots, logs, receipt digests and the correction explanation. Never require all archived returncodes 0, never exclude failures from manifest, and never count pre-fix source as final tested source. Diagnostic `.perf.data` is not necessary for recomputing latency math but is necessary if publishing complete retained raw profile evidence; either retain it under diagnostic role or disclose its absence and limit profile claims.

Attempt 01 `profile-evidence-correction.json` states a legacy post-profile call reused `baseline-two` output names and overwrote the initial raw baseline profile. The surviving bytes were candidate diagnostics renamed to `candidate-two-overlap-*`; they overlapped own clippy and are excluded from clean paired profile interpretation. The corrected serial diagnostic pair is `baseline-two-recreated-*` and `candidate-two-clean-*`, from preserved binaries under distinct labels and refusal-to-overwrite controller. The original first baseline raw data is unavailable; do not assign surviving candidate bytes that baseline role. Record original baseline `overwritten_unavailable`, overlap candidate `retained_excluded_own_clippy`, and clean baseline/candidate `retained_recreated`. The correction receipt says latency results unaffected: retain their separate unchanged raw timing lanes as evidence, without treating recreated profiles as the lost originals. Performance profiles are diagnostic; their instruction attribution does not independently prove a cycle cause or benchmark gain.

## Actual final candidate identity ledger

The following rows were extracted only from `attempt-XX/verify/before.json`, `build-final/after.json`, and `verify/receipt.json`; each source snapshot still uses its parent HEAD because candidate changes were uncommitted at build. Full hashes identify final candidate bytes. Accepted commits come from parent descriptors. Preserve all guards/receipts, not only this ledger.

```json
[
  {
    "attempt": "01",
    "outcome": "accepted",
    "accepted_commit": "22f4a270926d513b2641cd44c5445008ecf3b276",
    "parent_commit": "0e9329dfe782187a8feda0024102e4aabf517c28",
    "candidate_sha256": "fe03feaf8f0c822f5b500edab9330a41f7af9956def57d6aced6625802aff117",
    "compiled_snapshot_sha256": "0ea28b13e45a4883ece7bbba6ec66b0668f351c3981cff96bd4c24c42645e0a7",
    "build_snapshot_head": "0e9329dfe782187a8feda0024102e4aabf517c28",
    "or_source_sha256": "865da3ae47e283e23ca50116e8ed02f92dfd21d008dcc8df851cbc3fb5f90a93",
    "default_gate_receipt_sha256": "c5bb6bed7b6daae1e23e388d8b5fcfc07461738f17f4c7ceba7dae166abad21b"
  },
  {
    "attempt": "02",
    "outcome": "rejected",
    "accepted_commit": null,
    "parent_commit": "22f4a270926d513b2641cd44c5445008ecf3b276",
    "candidate_sha256": "c16cde6386dca0fad52a4909fa905b9f4204e0c8f54bcb31c922d595face0571",
    "compiled_snapshot_sha256": "761fdaa364af310e9d5748d9a8647efb092014f45f5f3d37d4060a8b74afa837",
    "build_snapshot_head": "22f4a270926d513b2641cd44c5445008ecf3b276",
    "or_source_sha256": "e1ab97b65eaf969e7bc7308964412bec1323ac9ec96ba44cb6395dee5d282b3a",
    "default_gate_receipt_sha256": "58e26c5d470e93848840e9ed8f88b3692a5bf2a311046cb43675d6de695cd9c2"
  },
  {
    "attempt": "03",
    "outcome": "accepted",
    "accepted_commit": "05835b971452bd5bda75037cec2925b7ba6785bf",
    "parent_commit": "22f4a270926d513b2641cd44c5445008ecf3b276",
    "candidate_sha256": "c4ab444bb759f0bc096d59eb0cfa693552840ef0a040581a1b36d9e92447b5e6",
    "compiled_snapshot_sha256": "5d12dad9bed1bcabc22eb3d4e51d0da7213e3492a4ee2c5238681882e97fc49f",
    "build_snapshot_head": "22f4a270926d513b2641cd44c5445008ecf3b276",
    "or_source_sha256": "b4d093a2e728fa444efa07cb1a7cdd033aefaf7f5a8d7aefe5fe7d8bf018b881",
    "default_gate_receipt_sha256": "098f0dce6d6cdf8a5169ba5e531335c1f5f90f33942f5bad59b8e7b57858f2ed"
  },
  {
    "attempt": "04",
    "outcome": "rejected",
    "accepted_commit": null,
    "parent_commit": "05835b971452bd5bda75037cec2925b7ba6785bf",
    "candidate_sha256": "99cbf834ac85138238a854c2944920b7580fc92d8e194739ecef6925aeb072dc",
    "compiled_snapshot_sha256": "a674ffa49d772a0e0c644594b18517d58a2ea59247b514db4b83d5ce856df4b4",
    "build_snapshot_head": "05835b971452bd5bda75037cec2925b7ba6785bf",
    "or_source_sha256": "b4d093a2e728fa444efa07cb1a7cdd033aefaf7f5a8d7aefe5fe7d8bf018b881",
    "default_gate_receipt_sha256": "ee36869786e86b918a32d51f4d66bb1c1f6a045bd2c8a46605a94864b2fcac5d"
  },
  {
    "attempt": "05",
    "outcome": "rejected",
    "accepted_commit": null,
    "parent_commit": "05835b971452bd5bda75037cec2925b7ba6785bf",
    "candidate_sha256": "a96cda9f3fb9ed5065e9792649df1c58f0658b3aa0f0e027060eee8db62b377c",
    "compiled_snapshot_sha256": "e5c18c9a94ce1451484735620c66b98ca10eb574ca2dd747d4bfe0d5500cff75",
    "build_snapshot_head": "05835b971452bd5bda75037cec2925b7ba6785bf",
    "or_source_sha256": "b4d093a2e728fa444efa07cb1a7cdd033aefaf7f5a8d7aefe5fe7d8bf018b881",
    "default_gate_receipt_sha256": "c87cdbd08021def5bd6d7eb3567c9fe489a793b4dc9501e819860b69395fb8d2"
  },
  {
    "attempt": "06",
    "outcome": "accepted",
    "accepted_commit": "a674aaf2f6187f3010a135dc25fdacf9cab27e53",
    "parent_commit": "05835b971452bd5bda75037cec2925b7ba6785bf",
    "candidate_sha256": "44356c9d2f849c17686dfae3b0fd3e7ebd1b986966d9a03ca292e1b64a72424d",
    "compiled_snapshot_sha256": "67e96e8cce1c65c7ba0017d65debf38a420af9800dfee1d51a396d61bc3ba425",
    "build_snapshot_head": "05835b971452bd5bda75037cec2925b7ba6785bf",
    "or_source_sha256": "5dd474782bd1ed076fee86d62b67fe231512dbcba168da73b3baec7781f5ebc9",
    "default_gate_receipt_sha256": "66323dbe88ea6f392d58581d745599e3c9605e0546cc74ceeb3cbebf671a4458"
  },
  {
    "attempt": "07",
    "outcome": "rejected",
    "accepted_commit": null,
    "parent_commit": "a674aaf2f6187f3010a135dc25fdacf9cab27e53",
    "candidate_sha256": "140f57666ef8f57dfd3f9c5e4e7b5d93530833b8bdcf492ce561a29705782d81",
    "compiled_snapshot_sha256": "513e90c3a8623108faaa5d943b023b9b8098392075072cf142f280671626cf55",
    "build_snapshot_head": "a674aaf2f6187f3010a135dc25fdacf9cab27e53",
    "or_source_sha256": "7d0077d946369751bfd24c15503a9c2b00ff7dc73364e9cab7cf39ee31ab0381",
    "default_gate_receipt_sha256": "88e30844950b4cd3c498b598294f19e6f09244dc7ea69769174a607fc0f906d5"
  },
  {
    "attempt": "08",
    "outcome": "rejected",
    "accepted_commit": null,
    "parent_commit": "a674aaf2f6187f3010a135dc25fdacf9cab27e53",
    "candidate_sha256": "84d1b697d823e770e95febb7132d59c9928b716e96615d430c4e0c1eac21a100",
    "compiled_snapshot_sha256": "b4ad00023b98b9bb787b2c77f502c34c2088918c3f28ed5d3d959ec1bd330c0a",
    "build_snapshot_head": "a674aaf2f6187f3010a135dc25fdacf9cab27e53",
    "or_source_sha256": "45ff507293072fafc15fb1c67de84fdb43ef80807d781b6be4953688bfd1c259",
    "default_gate_receipt_sha256": "86a6fe7eff74e064bf2bc6868ce0e38a5b21f92c7e09c0c2203abfe9e27d495a"
  },
  {
    "attempt": "09",
    "outcome": "rejected",
    "accepted_commit": null,
    "parent_commit": "a674aaf2f6187f3010a135dc25fdacf9cab27e53",
    "candidate_sha256": "31ffb022be8b7c590ddc6e218faf720020c728592af6d7504d38c3125c0cd3ec",
    "compiled_snapshot_sha256": "1e6b22ecec18d0618712f70f3da8b633e5342bbbeb734ac96ece200c60ea1eb8",
    "build_snapshot_head": "a674aaf2f6187f3010a135dc25fdacf9cab27e53",
    "or_source_sha256": "41065df1d96bb7c585e0593c68b0df09ee717b97366efc253bd036f062725f12",
    "default_gate_receipt_sha256": "b20d8a57ae827c5f17de96c50f7edfe33128f36cf4f814d3bc39f76f37d0dd6d"
  },
  {
    "attempt": "10",
    "outcome": "rejected",
    "accepted_commit": null,
    "parent_commit": "a674aaf2f6187f3010a135dc25fdacf9cab27e53",
    "candidate_sha256": "7d73eaa3a7a94168533fbdd39bfd7bc8612ddc3a41c087f41fa49c31ac9f98a0",
    "compiled_snapshot_sha256": "c73e7c4bcee970b4b0676932f9ff39b96fb72e0adbd8074c1ff2cf62116b08fa",
    "build_snapshot_head": "a674aaf2f6187f3010a135dc25fdacf9cab27e53",
    "or_source_sha256": "81cb888e53e467a03515f0224aeefbe3bd3b05a25eaf6cd294491eea578a9729",
    "default_gate_receipt_sha256": "142ab003da02525d7c2e170cb4c39b37d238a0bdeec64abbdb4de1af47fdde15"
  },
  {
    "attempt": "11",
    "outcome": "accepted",
    "accepted_commit": "43f06833a325d2a18c03bd7e4fa65f2f27944292",
    "parent_commit": "a674aaf2f6187f3010a135dc25fdacf9cab27e53",
    "candidate_sha256": "e11c4939984690d2c2d781cb6ace40d5e818e3a18a3f9f558d6cbcfd62546e5d",
    "compiled_snapshot_sha256": "9972db388f51e23da6de3f586e9a0f16073ce4fa675c0b354d3cf1b052a45e5f",
    "build_snapshot_head": "a674aaf2f6187f3010a135dc25fdacf9cab27e53",
    "or_source_sha256": "21153a76a7ae3091ac6400cc438f408b7ac367d9a534b1d290d30af5c3fb5901",
    "default_gate_receipt_sha256": "35c1fe105532a1d3fe8a699c1d088437b1873793f33b01510f952441ad98a3ed"
  },
  {
    "attempt": "12",
    "outcome": "rejected",
    "accepted_commit": null,
    "parent_commit": "43f06833a325d2a18c03bd7e4fa65f2f27944292",
    "candidate_sha256": "b112ee3f12b8b5d9acf3bae5293f282c11996e51916b9f19b131c9f10babeab9",
    "compiled_snapshot_sha256": "c5aa25741ef0b97a8c4a917b51aaf1a0558d407a0288bd7c4752f4907185e60a",
    "build_snapshot_head": "43f06833a325d2a18c03bd7e4fa65f2f27944292",
    "or_source_sha256": "a53374a75b5dbedb3e8f148b94f8da92a6d4383142e69cc8b50a92330c2f96d1",
    "default_gate_receipt_sha256": "2a137947c16fb7be0fc38dcae033a3dc7370dc62fe02942092641140759632a1"
  }
]
```

## Acceptance and publication limits

The standalone audit should finish with explicit results for integrity, all final three-profile exact gates, available stage/failure records, complete accepted timing lanes, all rejected pilots, arithmetic group targets, all loss rows, and pending attempts. It may verify claimed known numeric results against retained evidence; it must not use a speed-win assertion as a substitute for evaluating evidence. Accepted 11's OR3+ parent arithmetic ratios `.983542/.976535` are gains against its parent; actual-port ratios above 1 remain losses. A correct packet can report both.

Timed checksums are modulo-64-bit aggregates of counts or doc-XOR-scorebit sums; they are useful consistency checks but can collide and cannot replace the full exact-gate comparisons of ordered documents and bits. The observed self-control spread is an empirical threshold for these runs, not a statistical confidence interval or a guarantee about unseen workloads.

Minimal payload excludes engine binaries, target dirs, corpus and large index payload. It retains their hashes, source/config maps, payload audit and exact outputs, so users can audit arithmetic and recorded identities without rebuilding. It cannot independently execute historical binaries, prove their derivation from source, validate external index bytes, establish causal cycle attribution, guarantee noise-free latency, compare unmeasured configured performance, or establish full Lucene public API parity. Document those limits near the result. Root's tracked archive wrapper can additionally show which packet bytes were committed, not recreate missing profiles or unperformed gates.

Implementation priority for Root: freeze per-attempt descriptors with evidence states first; include common artifacts and all source/log/guard proofs second; author one packet-relative stdlib reader with strict schedule/checksum validation before summaries; retain full raw cells and unsuccessful records; finally run audit and tracked-archive wrapper on the actual candidate packet. This task performs none of those execution/publication steps. No speed claim is added by this design.
