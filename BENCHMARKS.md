# tantivy-turbo: benchmarks and details

Back to the [README](README.md).

## Contents

- [At a glance](#at-a-glance)
- [Benchmarks](#benchmarks)
  - [How we measure](#how-we-measure)
  - [vs Tantivy 0.27](#vs-tantivy-027)
  - [vs lucene-rs](#vs-lucene-rs)
  - [vs Apache Lucene 10.4](#vs-apache-lucene-104)
- [How it works](#how-it-works)
- [Compatibility](#compatibility)
- [Correctness](#correctness)
- [Known limitations and roadmap](#known-limitations-and-roadmap)
- [Reproducing the benchmarks](#reproducing-the-benchmarks)
- [Credits and license](#credits-and-license)

<br>

## At a glance

| | vs **Tantivy 0.27** | vs **lucene-rs** (Lucene 10 port) | vs **Apache Lucene 10.4** |
| --- | :---: | :---: | :---: |
| TOP-10, typical query (geomean) | **1.19× faster** | **1.30× faster** | **1.82–2.09× faster** |
| TOP-10, total time (1,201 queries) | **1.65× faster** | **1.12× faster** | n/a¹ |
| TOP-10, OR queries | **2.3–3.2× faster** | 2 terms 1.02× faster, 3+ terms **1.04× faster** | n/a¹ |
| TOP-10, AND queries | 1.15× slower | 1.08× slower on total time, **1.12× faster** typical | faster on all 6¹ |
| COUNT, total time | **1.5× faster** | **6.4× faster** | **1.27–1.30× faster** |
| Results | identical match counts | identical top-10 IDs, scores and counts | identical raw BM25 scores |

<sub>¹ The Lucene 10.4 comparison uses the 20-query search-benchmark-game Wikipedia set (8 term, 6 AND, 4 OR, 2 phrase). The fork has the lower median latency on all 20 queries, for both TOP-10 and COUNT, in both process orders.</sub>

<br>

## Benchmarks

### How we measure

- **Corpus:** the first 1,000,000 English Wikipedia articles. Each engine builds a single-segment index from the same JSONL.
- **Queries:** 1,201 "port-style" queries: 300 single-term, 300 AND, 301 OR (103 with 3+ terms, 198 with 2) and 300 phrase. Also the 20 search-benchmark-game Wikipedia queries.
- **Warm, persistent workers.** Each engine runs as a long-lived process pinned to one core, with the query cache disabled.
- **Interleaved, both orders.** For every query, both engines run back-to-back in rounds of 12 iterations, first in A→B order and then B→A. Ratios use summed per-query medians. "Typical query" is the geometric mean of per-query ratios.
- **Results are checked during timing.** Every timed call's checksum is compared against a frozen correctness dump.
- **Machine:** Intel Core Ultra 7 255H laptop on AC power, Rust 1.99, `-C target-cpu=native`, fat LTO.

Absolute microseconds are only comparable within one table, because each table is a separate run. The current tables were measured partly on battery, and ratios between separate runs vary by a few percent (for example, AND against lucene-rs measured 1.02 and 1.08 in two runs of code that executes the same instructions). tantivy-turbo's index is the same 1M-document index re-encoded by a single-segment merge to add level-1 skip data; documents and doc IDs are unchanged.

<br>

### vs Tantivy 0.27

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/vs-tantivy-dark.svg">
    <img alt="tantivy-turbo vs Tantivy 0.27 by query type" src="assets/vs-tantivy-light.svg" width="860">
  </picture>
</p>

Both engines index the same documents. Each uses its own default text analysis and scoring, so match counts are identical on all 1,227 queries, but top-10 scores differ slightly. A time ratio below 1 means tantivy-turbo is faster.

| TOP-10 | Queries | turbo | Tantivy 0.27 | Time ratio (AB / BA) | Typical query | Queries faster |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| OR, 3+ terms | 103 | 279 µs | 885 µs | **0.32 / 0.31** | 0.41 | 100 / 101 |
| OR, 2 terms | 198 | 68.7 µs | 157 µs | **0.44 / 0.44** | 0.48 | 198 / 197 |
| Phrase | 300 | 280 µs | 394 µs | **0.72 / 0.69** | 0.90 | 134 / 147 |
| Single term | 300 | 15.1 µs | 14.1 µs | 1.07 / 1.07 | 1.05 | 104 / 109 |
| AND | 300 | 128 µs | 111 µs | 1.12 / 1.18 | 1.15 | 39 / 23 |
| **All 1,201** | 1,201 | 141 µs | 232 µs | **0.61 / 0.60** | 0.84 | 575 / 577 |
| Wikipedia 20 | 20 | 183 µs | 1,091 µs | **0.17 / 0.17** | 0.63 | 8 / 8 |

<details>
<summary><b>COUNT results</b></summary>

| COUNT | Queries | turbo | Tantivy 0.27 | Time ratio (AB / BA) | Typical query | Queries faster |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| OR, 3+ terms | 103 | 258 µs | 976 µs | **0.27 / 0.26** | 0.53 | 73 / 71 |
| OR, 2 terms | 198 | 63.9 µs | 94.5 µs | **0.68 / 0.68** | 1.26 | 40 / 38 |
| Phrase | 300 | 366 µs | 374 µs | 1.00 / 0.96 | 1.03 | 108 / 107 |
| Single term | 300 | 0.7 µs | 0.7 µs | 1.06 / 1.05 | 1.06 | 70 / 65 |
| AND | 300 | 101 µs | 125 µs | **0.81 / 0.81** | 0.98 | 126 / 136 |
| **All 1,201** | 1,201 | 150 µs | 224 µs | **0.68 / 0.66** | 1.00 | 417 / 417 |
| Wikipedia 20 | 20 | 456 µs | 1,454 µs | **0.32 / 0.31** | 0.59 | 13 / 13 |

</details>

- **The win is concentrated in OR.** OR queries with 3+ terms take under a third of the time, 2-term ORs under half. The Wikipedia-20 total drops 6×, but per query turbo is faster on only 8 of those 20.
- **AND is the remaining regression.** Conjunctions are about 1.15× slower than in 0.27 (down from 1.4× after exact cheaper bound checks in the block-max intersection).
- **Resources:** the index is 598 MB against 661 MB. tantivy-turbo's heap stays at about 1.7 MB; total RSS is dominated by memory-mapped index pages (roughly 190–240 MB for either engine, varying run to run with the page cache).

Data: [`bench-results/turbo-vs-tantivy-0.27`](bench-results/turbo-vs-tantivy-0.27).

<br>

### vs lucene-rs

[lucene-rs](https://github.com/pc-style/lucene-rs) is a Rust port of Apache Lucene 10, including its MaxScore and impacts machinery. It's the most direct "Lucene algorithms in Rust" baseline available. Both engines read the same corpus with the same Lucene-style analysis and scoring, and **return identical top-10 document IDs, f32 score bits and match counts on every query.** The comparison is therefore purely about speed.

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/vs-lucene-rs-dark.svg">
    <img alt="tantivy-turbo vs lucene-rs, TOP-10" src="assets/vs-lucene-rs-light.svg" width="860">
  </picture>
</p>

| TOP-10 | Queries | turbo | lucene-rs | Time ratio (AB / BA) | Typical query | Queries faster |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| OR, 3+ terms | 103 | 277 µs | 287 µs | 0.97 / 0.95 | 0.93 | 68 / 65 |
| OR, 2 terms | 198 | 66.3 µs | 67.5 µs | 0.97 / 0.99 | 0.73 | 142 / 139 |
| Phrase | 300 | 272 µs | 337 µs | **0.82 / 0.80** | 0.71 | 266 / 265 |
| Single term | 300 | 14.6 µs | 20.9 µs | **0.70 / 0.70** | 0.71 | 289 / 291 |
| AND | 300 | 130 µs | 120 µs | 1.09 / 1.07 | 0.90 | 208 / 202 |
| **All 1,201** | 1,201 | 139 µs | 155 µs | 0.90 / 0.89 | 0.77 | 973 / 962 |
| Wikipedia 20 | 20 | 190 µs | 224 µs | **0.85 / 0.84** | 0.69 | 15 / 15 |

**What this means:**
- **Typical query:** tantivy-turbo is **1.30× faster** and wins on 973 / 962 of 1,201 queries (by run order).
- **Total time:** tantivy-turbo is **1.12× faster**. lucene-rs keeps a small total-time lead only on AND, while tantivy-turbo is faster on the typical AND query.
- **Long-OR progress:** this release took 3+ term ORs from 1.47× slower than lucene-rs to **1.04× faster** (1.07× faster on the typical query, faster on 68 / 65 of 103). See [Bulk scoring](#bulk-scoring-for-long-or-queries) and [Level-1 skip data](#level-1-skip-data) below.

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/vs-lucene-rs-count-dark.svg">
    <img alt="tantivy-turbo vs lucene-rs, COUNT" src="assets/vs-lucene-rs-count-light.svg" width="860">
  </picture>
</p>

<details>
<summary><b>COUNT results</b></summary>

| COUNT | Queries | turbo | lucene-rs | Time ratio (AB / BA) | Typical query | Queries faster |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| OR, 3+ terms | 103 | 243 µs | 7,122 µs | **0.04 / 0.03** | 0.05 | 103 / 103 |
| OR, 2 terms | 198 | 62.4 µs | 763 µs | **0.08 / 0.08** | 0.15 | 193 / 194 |
| Phrase | 300 | 358 µs | 378 µs | 0.96 / 0.93 | 0.75 | 257 / 264 |
| Single term | 300 | 0.7 µs | 192 µs | **< 0.01 / < 0.01** | 0.02 | 300 / 300 |
| AND | 300 | 95.0 µs | 156 µs | **0.61 / 0.60** | 0.59 | 284 / 283 |
| **All 1,201** | 1,201 | 144 µs | 918 µs | **0.16 / 0.16** | 0.18 | 1,137 / 1,144 |
| Wikipedia 20 | 20 | 470 µs | 3,387 µs | **0.14 / 0.13** | 0.02 | 20 / 20 |

</details>

COUNT is lopsided because lucene-rs counts by visiting every match. tantivy-turbo answers single-term counts from document frequencies, and counts OR and AND matches with bitset windows and independent counting lanes. Both engines' resident memory is mostly memory-mapped index pages (about 190–240 MB here, varying with the page cache); tantivy-turbo's heap is about 1.7 MB. Data: [`bench-results/turbo-vs-lucene-rs`](bench-results/turbo-vs-lucene-rs).

<br>

### vs Apache Lucene 10.4

This comparison uses the search-benchmark-game Wikipedia 1M index and its 20 queries. **Both engines use native BM25 (k1 1.2, b 0.75) with their own collection statistics.** Full sorted term statistics, stored IDs, norms and raw top-10 scores were cross-checked as identical before timing.

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/lucene-per-query-dark.svg">
    <img alt="Per-query speedup over Apache Lucene 10.4" src="assets/lucene-per-query-light.svg" width="860">
  </picture>
</p>

| Fork / Lucene geometric mean | Fork process first | Lucene process first |
| --- | ---: | ---: |
| TOP-10 | **0.478 (2.09× faster)** | **0.549 (1.82× faster)** |
| COUNT | **0.769 (1.30× faster)** | **0.788 (1.27× faster)** |
| Queries with the lower median | 20 / 20 | 20 / 20 |

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/resources-dark.svg">
    <img alt="Memory and index size vs Apache Lucene 10.4" src="assets/resources-light.svg" width="860">
  </picture>
</p>

- **Memory:** a warmed tantivy-turbo query process is about **15 MiB resident, against 346 MiB for Lucene** with the default JVM heap (387 MiB peak). That includes mapped index pages. A JVM with a deliberately constrained heap would use less, and that wasn't measured.
- **When this was measured:** 2026-10-04, on the fork before the OR work in this release, so the current build should be at least this fast. The full report has correctness gates and raw samples: [native comparison](doc/performance/lucene-10.4/results/2026-10-04-native/README.md). See also [COUNT, storage and memory](doc/performance/lucene-10.4/results/2026-10-03-followup/README.md).

<br>

## How it works

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/regions-dark.svg">
    <img alt="Fine MaxScore regions vs adaptive coarse regions" src="assets/regions-light.svg" width="860">
  </picture>
</p>

### OR queries: MaxScore with adaptive regions

Top-k disjunctions use **MaxScore**. Terms are ordered by how much they can contribute. Low-impact *optional* terms are only checked for documents that the high-impact *essential* terms surface and that could still enter the top-k. The pieces:

- **MaxScore for 3–32 terms** with compact per-clause state, certified global prefixes, an exact cached threshold cutoff, and batched scoring when a single term drives the candidates.
- **Two-term OR** pruned by its essential term in fixed local score regions.
- **Adaptive coarse regions:**
  - **The problem:** MaxScore splits the doc-ID space into regions and re-derives each term's score bound per region. Tantivy ends a region at every term's 128-document block boundary. For a 21-term query that means 30,242 regions, each costing work for every term.
  - **The probe:** tantivy-turbo starts each query on that fine geometry and measures it. If region bookkeeping outweighs candidate work (fewer than 16 candidates per term per region), it tries wide regions, then keeps whichever geometry measures cheaper.
  - **The bounds:** wide regions are bounded by **cached range maxima over the skip data the index already stores**, so no format change is needed. Non-leading terms are capped at 256-block groups, and a region shrinks again when looser bounds would promote a term to essential.
  - **The result:** the 21-term query drops to 362 regions and runs **2× faster**. Long OR queries overall take **15% less time** than before this change, with identical results.

### Bulk scoring for long OR queries

New in this release, and all exact: results are bit-identical to the previous build on every gate.

- **Batch filtering:** when one term drives the candidates, its scored batch of 128 postings is filtered against an exact leaf cutoff. Each comparison is the same predicate as the first optional-term test, solved once per batch by bisection over f32 bits. Only survivors take the per-candidate path.
- **Windows for several essential terms:** like Lucene's `MaxScoreBulkScorer`, dense essential terms are batch-scored into 4,096-document windows (a bitmap plus f64 sums in clause order). Candidates are compacted without branching on an exact precomputed sum cutoff. Only candidates that could enter the top-k recover their leaves for the canonical, clause-ordered score.
- **Choosing the geometry by cost:** the switch between fine and coarse regions is re-measured once the threshold has warmed up. The query keeps whichever costs less per document; the cost counts region bookkeeping, candidates and covering-bound blocks.
- **Cheaper bounds and skipping:** galloping skip-directory lookups hinted by the previous region, direct leaf maxima for short ranges, and skips over many blocks that read only each block's last doc and size.
- **Dense blocks:** bitset blocks decode a byte at a time from a lookup table.
- **Repeated terms are scored once:** like Lucene's rewrite of repeated SHOULD clauses, `to be or not to be` scores 4 terms instead of 6. The published sum still adds each leaf at every original clause position.

### Level-1 skip data

Like Lucene's two-level skip lists and impacts, every 32 full blocks of a term get one fixed-width record after the per-block skip records: the group's byte and position offsets, and one max-score input pair selected over all the group's postings (with the same rule that selects a block's pair, so its bound is sound for the same reason).

- **Skipping:** long skips jump whole groups instead of reading every block's record.
- **Bounds:** score bounds over more than a few blocks use the whole groups covering them, without resolving each block.
- **Cost:** 0.18% larger postings for the benchmark index.
- **Compatibility:** readers detect the records from the skip data's length. Indexes without them (Tantivy 0.27, older tantivy-turbo) read as before, and older tantivy-turbo builds ignore the records.

Short OR queries that mix a stopword with rarer terms gained the most: `medal of honor` and `the news journal` execute 40–50% fewer instructions.

### Lucene-equivalent scoring

- **Arithmetic:** native Lucene 10 BM25, with f64 accumulation in clause order and a single f32 cast.
- **Statistics:** overlap-discounted norms, exact persisted field statistics (document count and token totals), and BM25 parameters configurable per searcher.
- **Phrases:** Lucene-correct sloppy-phrase traversal and frequency.
- **Verification:** raw score bits match Lucene 10.4 on the benchmark corpus and on a 330-query synthetic suite.

### COUNT and storage

- independent mask lanes for medium-density terms, with lane horizon checks amortized over four postings;
- streamed exact positions for phrase counts;
- PFOR exceptions for outlier term frequencies, which shrink frequency payloads by 31%.

<br>

## Compatibility

- **API:** the Tantivy 0.27 API, imported as `tantivy`. Additions include per-searcher BM25 parameters and exact field statistics.
- **Rust:** 1.87 or newer, the same as Tantivy 0.27.
- **Reading old indexes:** **tantivy-turbo reads indexes written by Tantivy 0.27.**
- **Not the other way round:** Tantivy 0.27 cannot read indexes written by tantivy-turbo, because they carry extra field statistics and block-bound metadata. Switching is easy; switching back means reindexing.
- **Level-1 skip data:** written for new and merged segments. Older segments work without it, at lower long-OR speed; a merge (for example `tantivy::indexer::merge_indices`) rewrites them.
- **Cargo features:** the same as Tantivy (`mmap`, `lz4-compression`, `zstd-compression`, `stemmer`, `quickwit`, ...).
- **Lucene parity:** scoring and the core query types (term, boolean, phrase, sloppy phrase, phrase prefix) match Lucene 10. Full Lucene feature and public-API parity is a longer-term goal and is not complete; see the [parity scope](doc/performance/lucene-10.4/parity).

<br>

## Correctness

Speed changes are only accepted if results are **bit-identical**:

- **Exact gates:** top-10 document IDs, f32 score bits and match counts are compared against the previous accepted build on three 1,227-query gates (default BM25, k1 0.9/b 0.4, and k1 2/b 1). They are also compared against an exhaustive scorer. The level-1 build passes them on the re-encoded index against results recorded on the original index.
- **Tests:** the full workspace suite passes, 2,158 tests with 9 ignored. That includes exhaustive-equality fixtures that force the coarse-region path, and forced hand-backs to fine regions, on every OR fixture, plus fixtures with repeated terms.
- **Upstream CI:** all of Tantivy's CI workflows (unit tests across feature sets, long-running tests, coverage) pass on this repository.
- **One frozen ruler:** every optimization attempt, kept or reverted, was judged by the same frozen benchmark ruler. 34 attempts and their raw samples are published under [`doc/performance/lucene-10.4/results`](doc/performance/lucene-10.4/results).

<br>

## Known limitations and roadmap

- **AND queries:** about 1.15× slower than Tantivy 0.27; within a few percent of lucene-rs on total time and 1.12× faster on the typical query.
- **Long OR queries:** 1.04× faster than lucene-rs on total time, but some 3-term queries are still 1.2–1.3× slower (`world bank president`, `us coast guard`).
- **Single-term TOP-10:** about 4–6% slower than Tantivy 0.27.
- **Index compatibility:** turbo-written indexes can't be opened by upstream Tantivy.
- **Lucene API parity:** not complete beyond scoring and the core query types.

<br>

## Reproducing the benchmarks

| Comparison | Driver | Data |
| --- | --- | --- |
| vs Tantivy 0.27 | [`compare.py`](bench-results/turbo-vs-tantivy-0.27/compare.py) plus the [upstream worker and indexer](bench-results/turbo-vs-tantivy-0.27/upstream-harness) | [`summary.json`](bench-results/turbo-vs-tantivy-0.27/summary.json) |
| vs lucene-rs | [`compare.py`](bench-results/turbo-vs-lucene-rs/compare.py) | [`summary.json`](bench-results/turbo-vs-lucene-rs/summary.json), [`per-query.json`](bench-results/turbo-vs-lucene-rs/per-query.json) |
| vs Lucene 10.4 | [Lucene 10.4 harness and instructions](doc/performance/lucene-10.4/README.md) | [2026-10-04 report](doc/performance/lucene-10.4/results/2026-10-04-native) |

What you need to know to rerun them:
- **Shared harness:** the drivers use the frozen worker protocol from the [OR hillclimb audit](doc/performance/lucene-10.4/results/2026-10-12-or-hillclimb-audit). Paths inside them point at the author's machine; adjust them to your corpus and index locations.
- **Corpus:** build the 1M-document corpus with the [configured Wikipedia replay](doc/performance/lucene-10.4/configured-wiki.md).
- **Charts:** regenerate every chart with `python3 bench-results/make_charts.py`.

<br>

## Demo video

The demo is a 32-second, six-scene video written in Rust with [fframes](https://github.com/dmtrKovalenko/fframes). It uses Bayer-dithered bars and every number in it comes from the tables above.

```bash
cd demo
cargo run --release -- preview                # real-time window
cargo run --release -- render -o demo.mkv     # 1080p VP9; then e.g. ffmpeg -i demo.mkv demo.mp4
```

See [`demo/README.md`](demo/README.md) for details.

<br>

## Credits and license

- **Origin:** tantivy-turbo is a fork of [Tantivy](https://github.com/quickwit-oss/tantivy), created by Paul Masurel and developed by the Quickwit team and contributors. Their history is preserved; see [AUTHORS](AUTHORS) and [README.tantivy.md](README.tantivy.md).
- **Baselines:** the lucene-rs baseline is [pc-style/lucene-rs](https://github.com/pc-style/lucene-rs), and Lucene results come from [Apache Lucene](https://lucene.apache.org/) 10.4 via the [search-benchmark-game](https://github.com/quickwit-oss/search-benchmark-game) engine.
- **License:** MIT, like Tantivy ([LICENSE](LICENSE)).
- **Affiliation:** not affiliated with or endorsed by the Tantivy, Quickwit, lucene-rs or Apache Lucene projects.
