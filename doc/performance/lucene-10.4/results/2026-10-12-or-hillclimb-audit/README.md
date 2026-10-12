# Portable hillclimb evidence auditor

This packet contains assembled retained hillclimb evidence. Assembly did not execute a benchmark, engine, correctness test or the standalone audit. Run the retained auditor to verify its bytes and recorded arithmetic; rejected trials and pending source-only work remain explicit.

The finished packet is read from the directory containing `recompute.py`. Python's standard library is sufficient. It runs no subprocess, Git, Rust, Java, copied controller, or worker. A successful exit means the retained evidence is internally consistent; an actual-port target of `false`, rejected experiments, and expected failed revisions can all coexist with that exit. No engine binary, corpus, or large index payload is required or implied to be present.

## Running a populated packet

Run `python3 /path/to/extracted-packet/recompute.py` and redirect stdout to a file **outside the packet** if desired. The auditor rejects extra files inside the packet, including an audit-output file created before inventory checking. It emits complete JSON with every query, group, round and loss, not just selected headline results. Errors emit `integrity_pass: false` on stderr and exit with status 1. The source-only example fragments are not a runnable packet.

The manifest excludes only `packet-manifest.json` itself. It includes `recompute.py`, `attempts.json`, `README.md`, every proof/source/controller/log/diagnostic artifact, and any optional publication wrapper. Root creates the manifest last, after all descriptor and source bytes are final. Do not rewrite old retained receipts to relocate their absolute provenance strings. Every working reference in the new descriptor is packet-relative.

## Manifest

`packet-manifest.json` has exactly `schema`, `files`, `origins`.

- `schema`: `tantivy-or-hillclimb-audit-v1`.
- `files`: map from canonical POSIX relative path to `encoding`, `stored_bytes`, `stored_sha256`, `raw_bytes`, `raw_sha256`, `role`.
- `encoding`: `identity` or `gzip`. Lengths are nonnegative JSON integers; digests are lowercase 64-character SHA256 strings. Stored hashes bind compressed bytes; raw hashes bind the decoded original bytes. Gzip artifacts must have a zero header mtime, matching Root's deterministic packer.
- `origins`: a subset of the file map, mapping copied artifacts to `original_path`, `original_bytes`, `original_sha256`. The latter two must match the raw file entry. Newly authored descriptors/docs/auditor have no copied origin. Absolute origin strings are never opened.

Duplicate JSON keys, missing/extra physical files, symlinks and special files, absolute/traversal/backslash/colon paths, noncanonical paths, nonfinite JSON numbers and unsupported encodings fail. Stored and decoded hashes are streamed; protocol JSONL is streamed. Expected byte lengths bound decoding. Neither checksums nor digests are signatures identifying who ran an experiment.

## Reference rules

All artifact strings are packet-relative unless explicitly listed below as attempt-local. References to JSON values have `artifact` and optional `selector`; selectors are arrays of object-key strings, not array offsets. A hash binding has `selector`, `artifact` and compares the selected value with the artifact's **raw** SHA256. A value check has `selector`, `expected`. Selectors cannot silently select a missing field. Duplicate selectors in one binding list fail.

Attempt-local fields are `candidate.snapshot`, each revision's `snapshot`, stage `folder`, gate `folder`/`protocol`, pilot `folder` and each pilot protocol path, lane `folder`/`csv`/`summary`/`metrics`/`port_group_audit`, and lane pair protocol paths. These are joined to the attempt's `prefix`. Source bindings, candidate/revision `patch`, copied controller paths, hash-binding artifacts, parent proof paths, diagnostic controller/cases/trace artifacts, and decision artifact paths are packet-relative. Diagnostic receipt/replies/report/stderr/correction paths are attempt-local.

## Top-level descriptor

`attempts.json` has `schema`, `common`, `attempts`, `required_attempt_ids`.

- `schema`: `tantivy-or-hillclimb-attempts-v1`.
- `attempts`: array in the original decision trail's completion order, with unique IDs. Accepted parents must precede children; cycles fail. Completion order need not be numerical:14 completed before13 in the retained trail.
- `required_attempt_ids`: exact array of those IDs. Historic attempts `01` through `12` cannot be omitted. Root must append every actual subsequent completed/pending trial to the final inventory. This field is not a mechanism for dropping an inconvenient trial.
- `common`: fields listed below. Root fills these from actual retained bytes.

| Common field | Shape and binding |
|---|---|
| `queries` | Full frozen query JSONL, exact SHA `39a54b200392befbde83469841ce6eaefe7ec1850bb5b0e25c74a0eefa709a4e`. All 1,227 full cases remain. |
| `frozen_gate` | `dumps`, `receipt`, `failures`, `before`, `after`, `protocols`. Protocols have `tantivy`, `port` paths. These are original exact-output references, not the later hillclimb parent gate. |
| `baseline` | `binary_sha256`, `source_commit`, `compiled_snapshot`, `exact_receipt`, `provenance`, `provenance_bindings`. The hillclimb baseline is `0e9329df…`, binary `2efc80ae…`. |
| `fixed_port` | `binary_sha256`, `source_commit`, `source_snapshot` (JSON-value reference), `source_bindings`, `binary_proof` (JSON-value reference), `harness`. Actual port is commit `e5d1f81b…`, binary `b0923daa…`. Source bindings map actual repository file names to copied source artifacts. Harness maps every original six-file harness key to copied bytes, including `port_worker.rs`. |
| `indices` | `tantivy`, `actual_port`, each a JSON-value reference to the complete recorded eight-file map. Lockfiles remain excluded by the original guard policy. Index payload bytes are external. |
| `harness` | Five hillclimb harness keys mapped to copied artifacts: Cargo.toml, Cargo.lock, lib.rs, tantivy_worker.rs, configured_worker.rs. These differ from the original six-file port harness. |
| `guard_proofs` | Hash-binding list. Must include the `payload_audit` field bound to retained audit stdout. |
| `payload_audit` | `stdout`, `receipt`, `index_after` paths. Actual payload counts, physical/payload/term-list hashes, receipt status/log hash, and recorded index map are checked. |
| `run_contract` | Entire unchanged frozen run-contract JSON. |
| `decision_trail` | Original complete decision TSV. Completed IDs/outcomes must match the descriptor. |
| `pilot_controller_sha256` | Frozen `pilot.py` SHA `28e34a3482ccbe0dbbad943c7bdcc13c93e56ff8b7ed063609cf20d7a8a6446e`. |
| `comparison_controller` | Copied original `compare.py` bytes, used by parent-selection guard binding. |

The original frozen Tantivy exact reference uses binary `49aa8196…7cd1`. It is **different** from the hillclimb baseline `2efc80ae…bbf`. The auditor checks their separate identities and compares exact default outputs. It does not claim the two binaries are the same.

## Attempt and source revisions

Each attempt has `id`, `prefix`, `outcome`, `parent`, `candidate`, `revisions`, `guard_proofs`, `stages`, `exact_gates`, `pilots`, `parent_lane`, `actual_port_lane`, `diagnostics`, `decision`, `limitations`; accepted attempts additionally have `accepted_commit`.

`outcome` is `accepted`, `rejected` or `pending`. Pending trials claim no stages/gates/pilots/diagnostics/revisions/decision/accepted commit. Their lanes explicitly have `status: pending`, `reason`. `candidate` may be null for an as-yet-unprepared pending trial. A pending source-only candidate may instead bind actual source/snapshot bytes with null binary hashes. Never invent runtime results to fill fields.

Final and revision candidate objects have:

- `snapshot`: attempt-local retained full stage source snapshot.
- `snapshot_key`: actual worktree key, e.g. `tantivy-or-hillclimb-oct07`.
- `source_bindings`: repository relative source file to packet-relative copied bytes. Every patched source requires a binding. Extra relevant fixture/source bindings are allowed and checked.
- `patch`: packet-relative original patch bytes.
- `changed_paths`: ordered unique repository file paths, matching actual `diff --git` headers. The reader supports same-name unquoted `a/b` and Git mnemonic `i/w` prefix pairs; failed08's original patch uses `i/w`. Original patch bytes are never normalized. Renames/deletions or quoted path names require an explicit future schema extension, not silent parsing.
- `binary_sha256`, `configured_binary_sha256`: required actual hashes for final completed trials; null allowed for source-only/failed revisions where no such binary identity is known.

Every copied source hash must match the selected full snapshot file map. Full harness hash maps are checked as well. A snapshot's parent HEAD plus dirty status is preserved; it is not relabeled with the later accepted commit. File maps/patches/copied blobs/recorded binary digests are integrity evidence, not proof of compiler derivation.

`revisions` maps non-final revision labels to their actual candidate descriptors. Stage references resolve by label. Each listed revision must have a retained stage. An original static design without a run can remain inventoried as documentation rather than masquerading as a failed execution revision.

`parent` has `kind`, `binary_sha256`, `source_commit`, `descriptor`, `compiled_snapshot`, `exact_receipt`; an accepted parent also has `attempt_id`. `kind: baseline` has null descriptor and no attempt ID. `kind: accepted` points to unchanged original parent descriptor bytes and the relocated exact/build proofs with their original digests. These must match the accepted parent's own candidate and default gate. No original absolute parent path is resolved.

`guard_proofs` binds at least the guarded `controller` to copied balanced-controller bytes. Parent-selection `comparison_controller_sha256` is checked separately only in parent lanes. Actual-port lanes deliberately have no parent-selection label.

## Stages and failures

A stage descriptor has `role`, `folder`, `expected_returncode`, `expected_receipt`, `tests`, `revision`, `required_log_fragments`. Roles: `build`, `targeted`, `tests`, `clippy`, `formatting`, `layout`. Revision is `final` or a key in `revisions`.

The conventional folder contains unchanged full `before.json`, `after.json`, `command.json`, `receipt.json`, raw `stdout.log`, raw `stderr.log`, and `compiler.txt`. The full compiler text is checked against frozen rustc1.99.0 commit `b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`, host and LLVM23.1.1. Expected receipt must explicitly supply compiler, rustflags, jobs, profile and exact command (including original provenance path strings). Logs are checked against receipt digests. Root must record real totals rather than extrapolate counts. Test totals, when available, have `passed`, `failed`, `ignored`, `measured`, `filtered_out`; the reader sums all retained Rust test-result records. Compilation failures have null totals when no test-result line exists.

Final stages require status 0. Accepted trials require build, broad tests, Clippy and formatting. Expected failure revisions require their real nonzero status plus retained log signatures. Historic08's two unfixed E0689 runs,09's17 passed/3 failed,10's E0277, and12's21 passed/2 failed cannot be omitted.16's original28 passed/1 failed and19's original30 passed/2 failed fixture-setup runs also remain, with their separate corrections and final gated sources. Both10 pilots remain. Record correction explanations/source bytes as separate artifacts; never treat pre-fix source as the final gated candidate.

No generic assertion says every archived command must pass. Artifact integrity may pass while explicitly retaining these failed revisions.

## Exact gates

Each completed trial has exactly three gate descriptors with `profile`, `folder`, `protocol`:

1. `default`: optimized candidate equals own exhaustive oracle and **both** frozen exact references for all1,227 IDs, counts, ordered documents and unsigned raw32-bit score bits.
2. `configured-k0.9-b0.4`: own oracle only; READY bits `3f666666`/`3ecccccd`.
3. `configured-k2-b1`: own oracle only; READY bits `40000000`/`3f800000`.

Each folder contains receipt, dumps, failures, before/after guards, candidate protocol/stderr. READY binds actual field/corpus statistics, engine, collector/cache and parameter bits. Configured guards bind the actual configured worker hash; their port-labelled guard entry is still the preserved parent binary. Three exact profiles are not three measured performance comparisons. Default `port` dump label in this lane means preserved Tantivy reference.

Counts are bounded by the actual1,000,000 physical documents for every profile, including each independent oracle. Hit documents and score bits are range checked, IDs are unique and complete, and ordered hits are checked by score then document ID.

## Timing lanes

A non-retained lane is `{status: not_run, reason: ...}` (`pending` for a pending trial). Do not represent an absent lane as a successful zero-sample run. Completed accepted trials need both full lanes; rejected trials may retain a full parent lane with no actual-port lane.

A retained lane has `status: retained`, `folder`, `csv`, `summary`, `protocols`, `summary_hash_bindings`, `metrics`, `metrics_format`, `metrics_metadata`, `metric_hash_bindings`, `port_group_audit`, `port_group_controller`.

`protocols` maps string pair IDs to `{A: attempt-local-path, B: attempt-local-path}`. Retain schedule, before/after guards, receipt, all raw sample JSONL, protocols/stderr, complete raw telemetry and RSS. Schedule cell order and seeded query shuffle are independently checked against the **full raw row sequence**, alongside unique-key completeness. Every ns is a positive integer; JSON booleans are rejected as numbers. Protocol READY/warmups/requests/responses must match raw rows. Timed checksums are independently recomputed from exact outputs.

| Lane | Exact surface and statistic |
|---|---|
| Parent |3 pairs×3 rounds×AB/BA×1,221 queries×count/top10×2 sides=87,912 cells and1,406,592 ns values. Parent label `port/P` means preserved Tantivy. Pooled48-sample per-query medians; arithmetic group ratios and four matched TT/PP self-controls; all six rounds and every individual loss retained. |
| Actual port |2 pairs×3 rounds×AB/BA×1,221 top10 queries×2 sides=29,304 cells and468,864 ns values. Actual engine READY is `lucene-rs`, collector `top10-totals-threshold1000`; index/binary are actual port. Contemporary controls are port-self only. Target is controlled arithmetic win in both orders for OR301 and OR3+103. |

Saved complete CSV and saved summary are compared independently with recomputed values. Geometric summaries are descriptive; arithmetic ratios of mean query medians are the target. No assertion requires every round or every query to win.

Parent `metrics_format` is `legacy-arithmetic` for01 (its stored groups lack newer control-result fields), `group-controls` for later receipts. Both recompute the same complete arithmetic groups/rounds/losses. `metrics_metadata` may have `efficiency` and `profile_overwrite_guard`; retain original values and label them receipt-only. Binary sizes/sections and logical scratch are separate from RSS. `metric_hash_bindings` checks any corresponding receipt digest fields.

Parent `summary_hash_bindings` maps the five original summary hash-key strings to relocated samples, CSV, schedule, telemetry, queries. Actual-port `summary_hash_bindings` is empty. Actual-port metrics fields are null/empty; `port_group_audit` and copied controller may be null if no historical derived audit exists, in which case groups/target are still independently recomputed. Parent port-group fields are null.

## Pilots

A pilot has `folder`, `protocols` (`A`, `B`), `controller` (packet-relative actual archived pilot source), `schedule`, `limitations`. There was no historical separate pilot schedule file, so the descriptor explicitly records the actual frozen parameters:373 selected full cases, warmup4, samples12, AB/BA, seed20261006, top10, CPU4. All1492 cells/17,904 timings, checksums, protocol records, receipt means/median ratios, worst20 and all373 query results are checked. Pilot controls are absent; these results cannot establish a control-adjusted full-suite win. Root supplies the full actual query-ID array in schedule, not a shorthand string.

## Diagnostics and profile history

`receipt-only` diagnostic: `kind`, `artifact`, `bindings`, `limitations`. These are retained identity/provenance claims, not newly executed measurements.

Unavailable original01 profile: `kind: profile`, `status: overwritten_unavailable`, `role: original-baseline`, `reason`, `correction` (attempt-local original correction receipt). There is no artifact containing lost original raw bytes.

Retained profile: `kind: profile`, `status`, `role` (`baseline`/`candidate`), `receipt`, `receipt_format` (`legacy`/`guarded`), `replies`, `controller`, `cases`, `binary_sha256`, `trace`, `report`, `stderr`, `limitations`. Cases/controller are packet-relative. Trace is `{status: retained, artifact: ...}` or explicit `{status: omitted, reason: ...}`. Any trace omission limits a complete-raw-profile claim. Cases, READY, all warmup/reply checksums and guarded receipt hashes are checked; profiles remain diagnostic.01 cases use `{id, terms: joined-string}`;08 uses full frozen `{id, kind, terms: array, tags}`. The latter is compared with the complete frozen case, and its retained query hash/input guards/diagnostic-only flag are checked.08's original `profile-or3-all.json` is retained and matches guarded SHA `b27fe4740a1b2e155f248bf8bcb98dc8ac9a1b8d2712cf843db3b63e69369d90`; it does not need reconstruction.

01 must retain one `overwritten_unavailable`, one `retained_excluded_own_clippy`, two `retained_recreated` profile states. The overlap record uses the legacy format, whose original receipt lacks binary-unchanged/script/case hash fields. The auditor must not invent those fields. The recreated clean pair uses guarded format. Surviving renamed candidate bytes cannot be assigned the original baseline role; recreated profiles do not recover the lost original. Other profile states may be `retained_diagnostic`.

## Decisions and result bindings

A completed decision has packet-relative `artifact`, `recorded_verdict`, `checks`, `result_bindings`, `acceptance_rule`. Verdict is `kept`, `kept_partial_win`, `reverted`, `rejected`, `reverted_no_reliable_win`, `reverted_material_per_query_regressions`, or `reverted_pilot_regression_risk`, matching the recorded outcome and original receipt. Checks bind fixed receipt values. Result bindings have `receipt_selector`, `result_selector` and compare receipt claims to computed output. An accepted rule has `{kind: parent-group-controls, groups: [...]}` listing the actual recorded aggregate groups; a rejected rule has `{kind: not-accepted, groups: []}`. This does not relax the frozen actual-port target.

The current inherited archive includes13's1,457 passing broad tests/7 ignored and14's1,445/7. Both have three exact1,227-query gates and full parent timing, but no actual-port lane.13 was rejected despite a controlled aggregate OR improvement because severe individual regressions remained;14's raw aggregate improvement failed the controls. These are completed rejected trials, not pending placeholders. The actual completed decision trail now also retains15's mixed pilot,16's controlled aggregate gain with material dense losses, and17's sparse pilot regressions.18 failed to win past controls in both orders;19's combined expansion pilot and20's required-gap pilot were rejected.20's pilot observed increased external CPU load and establishes no intrinsic slowdown. 21's explicit prefix accumulator passed1,442 tests and three exact profiles but its−0.75%/−2.25% raw gain failed2.69% group self-control spread; it was reverted. Later trial status comes from `attempts.json` and the retained decision trail. No pending trial is assigned a completed result.

22 is a completed partial win: 1,446 passing tests/7 ignored, three exact profiles, and longer OR arithmetic ratios 0.97232/0.96866 against accepted11, past 0.003607 control log spread. All OR also clears controls. All 60 full-suite controlled per-query losses remain, none in longer OR; ALL and PHRASE are mixed. Binary/text grows 448 bytes, with one 8-byte logical cutoff cache and no allocation/index change. Its actual-port longer OR ratios 1.70811/1.72373 and all OR ratios 1.47321/1.48661 leave the target false. Historical ratio changes between separate runs do not measure a source speedup.

23 passed21 targeted tests and three exact profiles but its longer-OR pilot was mixed (1.02930/0.98393, median query ratios above1 in both orders). It was reverted without broad tests, full timing or actual-port measurement. This is a conservative screen rejection, not a controlled intrinsic slowdown claim. The completed trail contains23 attempts,5 accepted and18 rejected.

Bind accepted commit, actual source/binary hashes, stage totals, gate counts, and numeric group ratios/target to their proof/result values where those fields occur in the historical receipt. Decision binary identities bind for accepted and rejected trials, including the legacy `binary_sha256` spelling. Scalar `source_sha256` claims bind the copied final OR source; mapped claims bind their named copied final sources. These are byte consistency checks. Root must derive actual descriptors from archive evidence; the fragments file illustrates field shapes and accepted11 source identity without fabricating13/15 results.

## Limits and Root's remaining verification

This authoring pass performed static reads only. Root must inspect every resulting source line and populate descriptors against every actual artifact before executing. Root should exercise malformed inventory/symlink/traversal/duplicate JSON, missing/duplicate/extra/reordered schedule cells, wrong ns/checksum/rawbit/READY/source/parent binds and expected-failure behavior against an actual packet. A clean committed-archive wrapper is a separate optional publication check, not part of this standalone reader.

The auditor does not independently rebuild engines, prove source-to-binary authenticity, validate absent index/corpus payload, re-run baseline600-source provenance, infer causal cycles, remove CPU contention, establish configured Java/port performance, guarantee universal wins, or establish full Lucene feature/public-API parity. Its target result is specific to the retained frozen workload and controls.

Assembly note: large artifacts keep their original logical filenames while manifest.encoding records gzip; do not infer encoding from filename. Read through the auditor or gzip.open as declared. packing-receipt.json explicitly distinguishes original copies, Git-object sources, and any hash-matched reconstruction. Outputs from verification belong outside this exact-inventory directory.
