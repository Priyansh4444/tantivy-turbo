<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/banner-dark.svg">
    <img alt="tantivy-turbo: a faster Tantivy. Same API, same results, Lucene-equivalent scoring." src="assets/banner-light.svg" width="860">
  </picture>
</p>

<p align="center">
  <a href="https://github.com/Priyansh4444/tantivy-turbo/actions/workflows/test.yml"><img alt="Unit tests" src="https://github.com/Priyansh4444/tantivy-turbo/actions/workflows/test.yml/badge.svg"></a>
  <a href="LICENSE"><img alt="MIT license" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
  <img alt="Rust 1.87+" src="https://img.shields.io/badge/rust-1.87%2B-orange.svg">
  <img alt="Fork of Tantivy 0.27" src="https://img.shields.io/badge/fork%20of-tantivy%200.27-555.svg">
</p>

<br>

**tantivy-turbo** is a performance fork of [Tantivy](https://github.com/quickwit-oss/tantivy), the Rust full-text search library. On the same index and the same queries, it:

- **Runs OR queries about 2× faster than Tantivy 0.27.** That covers 101 of 103 long OR queries and 195+ of 198 two-term OR queries.
- **Matches Apache Lucene 10's BM25 scoring exactly.** Raw score bits are identical to Lucene's on the benchmark corpus.
- **Returns the same top-10 as [lucene-rs](https://github.com/pc-style/lucene-rs), the Rust port of Lucene 10.** IDs, score bits and match counts are all identical, while turbo is faster on 907 of 1,201 queries.
- **Keeps Tantivy's API.** The crate is still imported as `tantivy`, so it's a one-line dependency change.

<br>

<p align="center">
  <a href="demo/tantivy-turbo-demo.mp4"><img alt="Watch the 32-second demo" src="demo/poster.png" width="760"></a>
  <br>
  <sub>▶ <a href="demo/tantivy-turbo-demo.mp4">32-second demo video</a>, rendered from Rust with <a href="https://github.com/dmtrKovalenko/fframes">fframes</a>; source in <a href="demo">demo/</a></sub>
</p>

<br>

## Contents

- [At a glance](#at-a-glance)
- [Quick start](#quick-start)
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
| TOP-10, typical query (geomean) | **1.10× faster** | **1.22× faster** | **1.82–2.09× faster** |
| TOP-10, total time (1,201 queries) | **1.36× faster** | about the same (1.03× slower) | n/a¹ |
| TOP-10, OR queries | **2.0–2.2× faster** | 2 terms 1.04× faster, 3+ terms 1.46× slower | n/a¹ |
| TOP-10, AND queries | 1.4× slower | 1.29× slower | faster on all 6¹ |
| COUNT, total time | **1.37× faster** | **5.9× faster** | **1.27–1.30× faster** |
| Results | identical match counts | identical top-10 IDs, scores and counts | identical raw BM25 scores |
| Memory | 186 vs 198 MB RSS | 178 vs 188 MB RSS | **15 vs 346 MiB RSS** |

<sub>¹ The Lucene 10.4 comparison uses the 20-query search-benchmark-game Wikipedia set (8 term, 6 AND, 4 OR, 2 phrase). The fork has the lower median latency on all 20 queries, for both TOP-10 and COUNT, in both process orders.</sub>

<br>

## Quick start

```toml
[dependencies]
tantivy = { package = "tantivy-turbo", git = "https://github.com/Priyansh4444/tantivy-turbo" }
```

Your code does not change: it keeps using the `tantivy` crate name and the Tantivy API.

```rust
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Schema, STORED, TEXT};
use tantivy::{doc, Index};

let mut schema = Schema::builder();
let title = schema.add_text_field("title", TEXT | STORED);
let index = Index::create_in_ram(schema.build());
let mut writer = index.writer(50_000_000)?;
writer.add_document(doc!(title => "The Old Man and the Sea"))?;
writer.commit()?;

let searcher = index.reader()?.searcher();
let query = QueryParser::for_index(&index, vec![title]).parse_query("old OR sea")?;
let top = searcher.search(&query, &TopDocs::with_limit(10).order_by_score())?;
```

For the full Tantivy documentation and features, see [README.tantivy.md](README.tantivy.md) and [docs.rs/tantivy](https://docs.rs/tantivy).

<br>

## Benchmarks

### How we measure

- **Corpus:** the first 1,000,000 English Wikipedia articles. Each engine builds a single-segment index from the same JSONL.
- **Queries:** 1,201 "port-style" queries: 300 single-term, 300 AND, 301 OR (103 with 3+ terms, 198 with 2) and 300 phrase. Also the 20 search-benchmark-game Wikipedia queries.
- **Warm, persistent workers.** Each engine runs as a long-lived process pinned to one core, with the query cache disabled.
- **Interleaved, both orders.** For every query, both engines run back-to-back in rounds of 12 iterations, first in A→B order and then B→A. Ratios use summed per-query medians. "Typical query" is the geometric mean of per-query ratios.
- **Results are checked during timing.** Every timed call's checksum is compared against a frozen correctness dump.
- **Machine:** Intel Core Ultra 7 255H laptop on AC power, Rust 1.99, `-C target-cpu=native`, fat LTO.

Absolute microseconds are only comparable within one table, because each table is a separate run.

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
| OR, 3+ terms | 103 | 271 µs | 550 µs | **0.49 / 0.50** | 0.56 | 101 / 101 |
| OR, 2 terms | 198 | 50.3 µs | 112 µs | **0.45 / 0.47** | 0.51 | 195 / 198 |
| Phrase | 300 | 251 µs | 315 µs | 0.80 / 0.77 | 0.95 | 127 / 114 |
| Single term | 300 | 10.0 µs | 9.2 µs | 1.08 / 1.04 | 1.04 | 108 / 108 |
| AND | 300 | 105 µs | 74.6 µs | 1.41 / 1.40 | 1.36 | 13 / 15 |
| **All 1,201** | 1,201 | 123 µs | 165 µs | **0.74 / 0.73** | 0.91 | 544 / 536 |
| Wikipedia 20 | 20 | 160 µs | 847 µs | 0.19 / 0.20 | 0.78 | 9 / 9 |

<details>
<summary><b>COUNT results</b></summary>

| COUNT | Queries | turbo | Tantivy 0.27 | Time ratio (AB / BA) | Typical query | Queries faster |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| OR, 3+ terms | 103 | 172 µs | 611 µs | **0.28 / 0.28** | 0.54 | 75 / 79 |
| OR, 2 terms | 198 | 36.8 µs | 59.2 µs | 0.62 / 0.66 | 1.05 | 90 / 63 |
| Phrase | 300 | 326 µs | 309 µs | 1.06 / 1.06 | 1.07 | 91 / 112 |
| Single term | 300 | 0.4 µs | 0.3 µs | 1.06 / 0.94 | 1.02 | 36 / 81 |
| AND | 300 | 73.0 µs | 83.1 µs | 0.88 / 0.85 | 1.04 | 100 / 124 |
| **All 1,201** | 1,201 | 121 µs | 160 µs | **0.75 / 0.71** | 0.99 | 392 / 459 |
| Wikipedia 20 | 20 | 501 µs | 1,063 µs | 0.47 / 0.33 | 0.59 | 11 / 14 |

</details>

- **The win is concentrated in OR.** OR queries take half the time. The Wikipedia-20 total drops more than 5×, but per query turbo is faster on only 9 of those 20.
- **AND is the regression.** Conjunctions are about 1.4× slower than in 0.27. Closing that is the next target (see [roadmap](#known-limitations-and-roadmap)).
- **Resources:** peak RSS is 186 MB against 198 MB after the full run, and the index is 598 MB against 661 MB.

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
| Single term | 300 | 7.7 µs | 11.3 µs | **0.68 / 0.69** | 0.70 | 290 / 283 |
| Phrase | 300 | 167 µs | 199 µs | **0.84 / 0.87** | 0.76 | 254 / 251 |
| OR, 2 terms | 198 | 38.3 µs | 40.2 µs | 0.95 / 0.97 | 0.72 | 149 / 143 |
| OR, 3+ terms | 103 | 234 µs | 161 µs | 1.45 / 1.48 | 1.18 | 33 / 30 |
| AND | 300 | 87.6 µs | 68.0 µs | 1.29 / 1.28 | 1.02 | 181 / 182 |
| **All 1,201** | 1,201 | 91.8 µs | 89.9 µs | 1.02 / 1.05 | **0.82** | **907 / 889** |
| Wikipedia 20 | 20 | 126 µs | 115 µs | 1.09 / 1.11 | 0.81 | 13 / 13 |

**What this means:**
- **Typical query:** tantivy-turbo is **1.22× faster** and wins on about three in four queries.
- **Total time:** about even, because the expensive queries are long ORs and ANDs, where lucene-rs still leads.
- **Long-OR progress:** this release cut the long-OR total-time gap from 1.71× to about 1.46×. The remaining gap is per-candidate execution cost, analyzed in the published [work-attribution report](doc/performance/lucene-10.4/results).

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
| Single term | 300 | 0.3 µs | 101 µs | < 0.01 / < 0.01 | 0.02 | 300 / 300 |
| OR, 3+ terms | 103 | 141 µs | 4,198 µs | 0.03 / 0.03 | 0.05 | 103 / 103 |
| OR, 2 terms | 198 | 30.2 µs | 412 µs | 0.07 / 0.07 | 0.13 | 197 / 197 |
| AND | 300 | 63.1 µs | 87.3 µs | 0.72 / 0.72 | 0.67 | 254 / 250 |
| Phrase | 300 | 230 µs | 220 µs | 1.05 / 1.03 | 0.80 | 236 / 236 |
| **All 1,201** | 1,201 | 90.5 µs | 530 µs | **0.17 / 0.17** | 0.17 | 1,090 / 1,086 |
| Wikipedia 20 | 20 | 263 µs | 1,866 µs | 0.14 / 0.14 | 0.02 | 20 / 20 |

</details>

COUNT is lopsided because lucene-rs counts by visiting every match. tantivy-turbo answers single-term counts from document frequencies, and counts OR and AND matches with bitset windows and independent counting lanes. Peak RSS after the run is 178 MB for tantivy-turbo and 188 MB for lucene-rs. Data: [`bench-results/turbo-vs-lucene-rs`](bench-results/turbo-vs-lucene-rs).

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
- **Adaptive coarse regions (new in this release):**
  - **The problem:** MaxScore splits the doc-ID space into regions and re-derives each term's score bound per region. Tantivy ends a region at every term's 128-document block boundary. For a 21-term query that means 30,242 regions, each costing work for every term.
  - **The probe:** tantivy-turbo starts each query on that fine geometry and measures it. If region bookkeeping outweighs candidate work (fewer candidates than regions × terms), it switches the rest of the query to wide regions.
  - **The bounds:** wide regions are bounded by **cached range maxima over the skip data the index already stores**, so no format change is needed. Non-leading terms are capped at 256-block groups, and a region shrinks again when looser bounds would promote a term to essential.
  - **The result:** the 21-term query drops to 362 regions and runs **2× faster**. Long OR queries overall take **15% less time** than before this change, with identical results.
  - **No cost to other queries:** the fine path keeps the exact previous code, and the coarse path lives in a separate out-of-line function. Queries that never switch therefore pay nothing measurable.

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
- **Cargo features:** the same as Tantivy (`mmap`, `lz4-compression`, `zstd-compression`, `stemmer`, `quickwit`, ...).
- **Lucene parity:** scoring and the core query types (term, boolean, phrase, sloppy phrase, phrase prefix) match Lucene 10. Full Lucene feature and public-API parity is a longer-term goal and is not complete; see the [parity scope](doc/performance/lucene-10.4/parity).

<br>

## Correctness

Speed changes are only accepted if results are **bit-identical**:

- **Exact gates:** top-10 document IDs, f32 score bits and match counts are compared against the previous accepted build on three 1,227-query gates (default BM25, k1 0.9/b 0.4, and k1 2/b 1). They are also compared against an exhaustive scorer.
- **Tests:** the full test suite passes, 1,448 tests with 7 ignored. That includes exhaustive-equality fixtures that force the new coarse-region path on every OR fixture, and a 12-term mixed-density test that runs both paths.
- **Upstream CI:** all of Tantivy's CI workflows (unit tests across feature sets, long-running tests, coverage) pass on this repository.
- **One frozen ruler:** every optimization attempt, kept or reverted, was judged by the same frozen benchmark ruler. 34 attempts and their raw samples are published under [`doc/performance/lucene-10.4/results`](doc/performance/lucene-10.4/results).

<br>

## Known limitations and roadmap

- **AND queries:** about 1.4× slower than Tantivy 0.27 and 1.29× slower than lucene-rs. This is the next optimization target.
- **Long OR queries:** still about 1.46× slower than lucene-rs on total time, though 1.18× on the typical query. The remaining cost is per-candidate execution: leaf scoring, field-norm lookups and seeks into optional terms. The largest lever left is persisting two-level impacts in the index, as Lucene does, which would be an index-format change.
- **Single-term TOP-10:** about 4–8% slower than Tantivy 0.27.
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
