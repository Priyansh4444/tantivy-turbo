# Current-build benchmark, 2026-10-30

The fork worker was built from commit `c2aabd490bc5c32ec2981811ffe26210694b8e3a`, using Rust 1.99.0, `-C target-cpu=native`, optimized release builds, fat LTO and one codegen unit. The machine was an Intel Core Ultra 7 255H laptop running Linux; AC power was disconnected when checked before the run. Background workloads remained active. The recorded one-minute load averages around the Rust timings ranged from 5.4 to 6.5; small differences should not be treated as conclusive.

## Rust comparisons

`port/` and `upstream/` contain summaries, per-query medians, raw samples, correctness dumps and input/binary identity receipts. Compressed files are ordinary gzip JSON. The same 1M-document Wikipedia corpus and 1,227 correctness cases were used, including 1,201 port-style and 20 Wikipedia timed queries. Each timing used persistent warm workers pinned to CPU 4, a controller pinned to CPU 6, 12 samples per query and both execution orders. Timed-result checksums were checked against verified dumps.

Turbo matched its exhaustive scorer and the frozen baseline on all 1,227 cases. Against lucene-rs, top-10 IDs, score bits and counts matched. All upstream Tantivy counts matched; its optimized and exhaustive results had the same top-10 IDs and ordering, but score bits differed on 69 queries, by at most 2 ULP. The independently built upstream index uses a different internal document-ID space, so these dumps do not prove cross-engine top-10 identity.

`controller/` contains the measurement scripts and worker source used for the Rust comparisons. Local machine paths and hostname have been replaced with placeholders in the published copies; the recorded controller hash refers to the original unredacted script. To rerun, edit the placeholders to point at a checkout and the retained corpus, indexes, query set, frozen dumps and baseline workers. `Cargo.lock`, build flags, input hashes and binary hashes document the run. Earlier result archives remain under their original paths.

## Java Lucene comparison

`java/` contains the four raw timing JSON files and the correctness report for Lucene 10.4, compressed with gzip. All 20 queries passed: counts and top-10 IDs matched, and the maximum relative score difference on shared top-100 results was 5.72e-8 (tolerance 2e-6). The two persistent processes shared CPU 4, with 40 seconds of warmup and 128 samples per query in each order. Unlike the Rust comparison, these timings include parsed requests and pipe overhead. The Java run used OpenJDK 21.0.13 and Lucene 10.4; the tool and input hashes are in the JSON provenance.

`java/memory-current.json` records a separate warmed-process RSS snapshot taken on October 31: 15.0 MiB for turbo and 554.4 MiB for Lucene with the default JVM heap. It includes mapped pages, uses 20 seconds of warmup per engine, and is not an indexing or universal peak-memory measurement. AC power was connected when checked after this separate snapshot; continuous power state was not recorded.

Machine-local paths and hostname were redacted in text artifacts; timing samples, scores, IDs and hashes were not changed. Storage figures from the earlier Lucene report and the existing demo video are historical and were not remeasured here.
