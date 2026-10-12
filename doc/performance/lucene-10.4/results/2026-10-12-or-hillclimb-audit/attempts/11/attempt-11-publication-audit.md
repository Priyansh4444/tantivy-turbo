# Attempt 11 archive and publication audit

Verdict: retained local archive, descriptor, committed source and reported arithmetic agree. No material inconsistency found. Accepted11 is a small measured improvement over accepted06, while the actual-port OR target remains unmet. The archive is complete for local evidence review; it is not yet a self-contained portable Git publication packet.

Throughput checkpoint: n/a, artifact review and lightweight retained-data arithmetic. No worker, build, test, benchmark, profile, compression or browser job was run. No source edits or commits were made. This report is the only new audit artifact.

## Provenance and integrity

Read actual acceptance/design/implementation/integration/source-review artifacts, immutable parent descriptor, stage receipts and guards, raw schedule/sample data, perquery tables, both summaries, arithmetic metrics and controls, archived sampler/analysis/parent-selection/publication controllers. Read the controller arithmetic rather than treating its summary as proof. Independently recomputed all group arithmetic from retained samples without invoking controller entrypoints or workers.

`parent-11.json` binds commit `43f06833a325d2a18c03bd7e4fa65f2f27944292`, archived candidate binary, compiled-source snapshot and default exact gate. All three descriptor SHA bindings match actual files. A fresh read-only `git ls-remote fork refs/heads/main` returned that exact commit during this audit.

The committed OR source SHA is `21153a76a7ae3091ac6400cc438f408b7ac367d9a534b1d290d30af5c3fb5901`, matching the measured snapshot, source review and integration receipt. Compared all archived source-file maps against accepted06: only this OR file hash changes, and the file-map key set is unchanged. `accepted.patch` equals the actual commit's binary diff bytes. The original patch has identical index line and hunks; its diff path prefixes are i/w rather than a/b, an ordinary header-only difference, not a source adaptation.

Key actual SHA-256 values:

| Artifact within attempt-11 | SHA-256 |
| --- | --- |
| `tantivy_worker` | `e11c4939984690d2c2d781cb6ace40d5e818e3a18a3f9f558d6cbcfd62546e5d` |
| `configured_worker` | `229a230244424f8abe060d5ff76a66bcf30605003228374498ba63413fd621e7` |
| `build-final/after.json` | `9972db388f51e23da6de3f586e9a0f16073ce4fa675c0b354d3cf1b052a45e5f` |
| `verify/receipt.json` | `35c1fe105532a1d3fe8a699c1d088437b1873793f33b01510f952441ad98a3ed` |
| `measure/samples.jsonl` | `8a781633d8f5e1257d39d88e925bdf2692f239fc80d4a02808fe4fde7377640b` |
| `port-measure/samples.jsonl` | `a3c1030b5fa3cd6eaee8817e8f661666f6f7d4858f03785569535a0d1686d5c7` |
| `summary.json` | `7c864e38f75c1cb96b51f634c502665e2bd8cd665f91609d7607eb32a6b29591` |
| `attempt-11-metrics.json` | `f5ac51c357828b81abd321e128147348ddcacb159ecf7156c574904fdfc4aa25` |
| `port-summary.json` | `4b555a9f00abfab098f195f8b67b09c384add6bcb5ea8e0458a72919810efa0f` |
| `port-group-audit.json` | `f4bbc1ef4273a2149b29ec5db5aae6eae3fd00e86ba9ee82dd93879f8c44c688` |

Every archived before/after pair matches for build, targeted/full tests, Clippy, formatting, default verification, both configured gates, pilot and both measurement lanes. Stage log hashes match receipts. Parent-comparison guard binds preserved accepted06 binary `44356c9d2f849c17686dfae3b0fd3e7ebd1b986966d9a03ca292e1b64a72424d`; actual-port guard separately binds fixed port binary `b0923daad72d662a056496ab115a57519ec474216fe05b1522e6529ae2154a2a`. Configured gates bind the actual configured binary. Archived balanced/compare controller hashes match guards, selected parent06 descriptor hash matches, all ten copied controllers match their current frozen originals, and the port arithmetic audit's controller hash matches its archived copy.

Before/after equality verifies the recorded source/index/binary identities remained fixed during those stages. This audit checked their retained maps and source/binary bindings; it did not rehash the complete large on-disk indices or rerun the original payload scan.

## Correctness and schedule completeness

Summed the actual full-test stdout result lines: **1,442 passed, 7 ignored**, agreeing with acceptance. No stage receipt reports a failed return code or failed gate. Each of the default and two configured gate dump files has exactly 1,227 queries and an empty failures list. Configured profiles remain own-exhaustive-oracle correctness checks; this does not assert fresh configured Java/port comparison performance.

Replayed both raw schedules and all checksums independently:

* Parent lane: **87,912 unique cells, 1,406,592 positive integer samples**, all three pairs, three rounds, both orders, count/top10, all 1,221 timed queries. No missing, duplicate or extra cell; labels match scheduled engines; every checksum matches the corresponding exact dump count or raw hit doc/score-bit checksum.
* Actual-port lane: **29,304 unique cells, 468,864 positive integer samples**, two pairs, three rounds, both orders, top10, all 1,221 timed queries. Same completeness/engine/sample/checksum checks pass against the retained fixed-port proof.

Raw samples, query schedule, telemetry and perquery CSV hashes match their summary bindings after resolving original pre-archive paths into attempt-11. The fixed query file hash remains `39a54b200392befbde83469841ce6eaefe7ec1850bb5b0e25c74a0eefa709a4e`. Both frozen schedules retain 10 warmups, 16 samples per order and worker CPU 4; parent controller selects CPU 6. Samples were not trimmed.

## Parent improvement and retained limitations

Independent raw arithmetic reproduces every group primary ratio, arithmetic TT/PP control, group log spread and every perround primary ratio in `attempt-11-metrics.json` within 1e-12. It also reproduces actual-port group ratios and arithmetic port self-controls within 1e-12. The acceptance decision therefore uses arithmetic means of perquery medians and matching arithmetic group controls. `summary.json` additionally reports geometric controls for descriptive views; those are not substituted for the arithmetic acceptance controls.

| Group versus accepted06 | AB candidate/parent | BA candidate/parent | Group max absolute control log |
| --- | ---: | ---: | ---: |
| 103 OR3+ | 0.983542001982 | 0.976535254819 | 0.011551427828 |
| 301 OR | 0.987340246901 | 0.982623912953 | 0.010177436227 |

Both group pairs lie beyond their contemporaneous arithmetic self-control spreads. This supports **1.65–2.35% less OR3+ time** and **1.27–1.74% less all-OR time** than accepted06 under this run. The overall 1201 primary ratios 0.9967465/0.9909166 do not clear overall controls in both orders; no overall causal improvement is established by this unit.

The mixed longer-OR round is preserved: round 0 BA primary 1.005062082737, TT 0.983286577436, PP 1.000014406963. Thus it is incorrect to claim every round was faster. The perquery table retains **19 top10 losses beyond each query's own observed controls**, none among OR3+; all four OR losses are two-term queries. Longer OR has 16 perquery wins beyond controls, 63 raw both-order wins and 7 raw both-order losses; zero controlled losses does not mean zero raw losses. These are parent-comparison classifications, not claims about the actual port.

Candidate and parent binary file sizes are both 4,765,928B; text 2,598,898B and rodata 458,816B are identical in the recorded section accounting, while their hashes differ. Source has no index-format change or new allocation; persistent logical state is 16B. Recorded RSS values do not establish a memory advantage. Equal binary size is not equal generated code or a measured register/stack claim.

## Actual port remains ahead on OR

The fresh actual-port arithmetic comparison is distinct from the accepted06 ruler:

| Actual-port group | AB candidate/port | BA candidate/port |
| --- | ---: | ---: |
| 103 OR3+ | 1.805087210412 | 1.827904992647 |
| 301 OR | 1.551770627591 | 1.573342530403 |
| Overall 1201 | 1.096892504876 | 1.098580754531 |
| 198 two-term OR | 0.995356477371 | 1.012267745367 |

`actual_port_target_met` is false. These numbers support a small parent win with a remaining actual-port OR loss, not port parity/superiority. Two-term OR is mixed/near parity. Term and phrase groups are ahead in this cumulative accepted stack's actual-port run; those cross-engine group positions do not attribute their advantage to this narrow global-prefix unit.

Port arithmetic controls are contemporaneous port/port comparisons only; candidate/candidate controls are in the separate parent lane. The port collector also tracks lower-bound hit counts, while the Tantivy worker's collector reports top10 without totals. This extra collector work prevents claiming identical collector workloads or a universal engine result. The accepted implementation remains within the existing valid indexed-TF, monotone ordered-threshold optimized domain; the public callback API prose does not establish an expanded guarantee.

## Archive completeness versus portable publication

The archive contains both workers, original and committed patches, design/implementation/integration/review/acceptance, full source guards and compiler/build logs, targeted/full tests, Clippy/format checks, all three exact dump/protocol/failure sets, pilot, both complete measurements including protocols/raw samples/schedules/telemetry/RSS, both perquery tables, summaries, arithmetic audits and controller copies. Parent-11 descriptor and prior accepted artifacts remain outside it with SHA bindings.

The moved receipts intentionally retain original absolute paths and precommit dirty-source HEAD provenance. Committed source equivalence was verified above. Archived controllers retain their original OUT/relative lookup assumptions; directly executing them inside `attempt-11/controllers` would not provide a portable standalone replay. External query input, harness sources/lockfile, payload proof, fixed-port exact proof, parent descriptors/artifacts and indices remain in central locations rather than duplicated into this attempt archive.

For the final tracked publication packet, preserve those dependencies and a relocated standalone recomputation entrypoint, or explicitly publish the dependency manifest with content hashes. No final tracked-inventory/git-archive replay validation was done here. This is a packaging limit, not evidence that the completed local attempt is missing its raw timing/proof files.
