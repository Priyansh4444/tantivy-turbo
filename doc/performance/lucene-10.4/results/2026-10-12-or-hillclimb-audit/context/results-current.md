# Search performance progress

The latest accepted source is `3dff75c5cfddee6dab50c144839b4cec645d9dca`
(attempt22). It improves longer OR execution against accepted11, while the
fixed `lucene-rs` build still wins the overall OR comparison. Full Lucene
feature and public API parity remains unfinished.

The current cycle completed 23 trials: 5 kept and 18 reverted. Attempt23's
reuse of the cutoff for global/local bounds passed 21 targeted tests and all
three exact profiles, but its longer-OR pilot was mixed (1.02930/0.98393);
it was reverted. No control-adjusted or intrinsic-loss claim follows from
that screening run. The next substantive direction is exact batched candidate
generation for multiple essential terms, grounded by a fresh phase diagnostic.

## What changed

Optional probes test whether a document can still beat the collector's minimum
competitive score. Previously, each probe multiplied its conservative bound,
advanced the result to the next f64 bit pattern, rounded to f32, and compared
with that minimum. Attempt22 computes the exact comparison edge when the
minimum changes. Each probe retains the same multiplication and compares
directly with the cached edge. Midpoint ties, overflow and exceptional thresholds
keep the original decisions.

The final document score still uses the original term order, f64 accumulation
and f32 conversion. The index files, posting cursors and collector behavior
retain their existing semantics. The change adds an 8-byte logical cache and
no allocation; the measured worker/text grows 448 bytes. No measured RSS win
or smaller compiled stack is claimed.

## Measured scope

This is a warm top10 execution comparison on the fixed 1M-document corpus.
The query suite has 1,201 port-style queries: 300 TERM, 300 AND, 301 OR and
300 PHRASE, plus 20 Wiki queries. The OR group includes 103 longer queries
and 198 two-term queries. All 1,227 correctness cases remain. Opening an
index, process startup and indexing throughput are outside this measurement.

Every query's median is computed from the retained samples. The headline
statistic divides arithmetic means of these per-query medians. AB and BA
reverse execution order; matched self-comparisons expose the observed noise.
All rounds, samples and individual losses remain in the packet.

| Attempt22 versus accepted11 | AB | BA |
| --- | ---: | ---: |
| Longer OR time | 260.06 / 267.46 µs | 259.49 / 267.89 µs |
| Longer OR candidate/parent | 0.97232 | 0.96866 |
| All OR candidate/parent | 0.97608 | 0.97672 |

Longer OR takes 2.77%/3.13% less time, beyond 0.003607 group control log
spread (about 0.36%). All OR also clears controls in both orders. 55 longer
OR queries win beyond their own controls, with zero controlled losses there.
The whole suite is mixed: all 60 controlled per-query losses remain, including
phrase queries and the two-term Wiki query `the american`. This is a partial
OR win, not a universal speed or efficiency claim.

## Current fixed-port gap

These are a separate contemporaneous comparison with the fixed actual
`lucene-rs` worker. Below 1 means our worker takes less time.

| Top10 group | Our time / port time AB | BA |
| --- | ---: | ---: |
| TERM | 0.67754 | 0.67550 |
| AND | 1.30636 | 1.31722 |
| Two-term OR | 0.97733 | 0.98442 |
| Longer OR | 1.70811 | 1.72373 |
| All OR | 1.47321 | 1.48661 |
| PHRASE | 0.89781 | 0.89811 |
| Entire port-style suite | 1.10265 | 1.10683 |

The frozen target requires controlled arithmetic wins in both orders for
all OR and longer OR. It is false. Changes between historical port ratios
and this table do not isolate a source speedup. The port collector also
tracks lower-bound hit totals with threshold1000; ours returns top10 without
totals. Benchmark runs preserve the user's background load and its limitations.

## Correctness and audit

Attempt22 passes 1,446 tests with 7 ignored, 13 targeted OR trace tests,
8 numeric bound tests, Clippy, both formatting checks and three exact-result
profiles of 1,227 cases each. The default profile matches both retained exact
references on counts, ordered document IDs and raw score bits. Two configured
profiles match their own exhaustive oracle; this establishes their correctness,
not configured Java or port performance.

The portable auditor recomputes retained arithmetic, protocols, schedules,
checksums and exact-result gates using Python's standard library. Engine binaries,
corpus and index payload are external. Its hashes prove consistency of recorded
bytes; they do not authenticate who ran the experiment or independently rebuild
the compiler output. Failed revisions, pilot rejections, partial wins and the
unavailable original attempt01 baseline profile stay explicit.

See [README.md](README.md) for the packet format and audit command, and
[search-architecture-current.html](search-architecture-current.html) for the
accepted search and OR execution flow.
