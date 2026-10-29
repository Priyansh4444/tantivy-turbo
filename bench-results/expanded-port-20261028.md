# Expanded lucene-rs comparison, October 28

This is a warm, single-core, top-10 comparison on the existing aligned indexes
and 1,221 timed query cases: 1,201 port-style queries and 20 Wikipedia queries.
The benchmark uses prebuilt queries, 10 warmups, six shuffled rounds, both run
orders and 32 timing samples per query per order in each round. It also runs
Tantivy-versus-Tantivy and lucene-rs-versus-lucene-rs controls. The 2,813,184
timing samples are repetitions, not distinct queries.

| Workload | Cases | Tantivy-turbo / lucene-rs, AB | Tantivy-turbo / lucene-rs, BA |
| --- | ---: | ---: | ---: |
| Port-style, all | 1,201 | 0.808 | 0.808 |
| Port-style, OR with 3+ terms | 103 | 0.967 | 0.968 |
| Wikipedia, all | 20 | 0.848 | 0.846 |

Each entry is the ratio of the two engines' **mean per-query medians**; lower
means less time for Tantivy-turbo. For all 1,201 port-style searches, the
averages of the two order-specific mean query medians were 136.2 µs for
Tantivy-turbo and 168.5 µs for lucene-rs (rounded to one decimal). On the longer OR cases, 30 of 103 were
slower in both orders. The target of 1.7× faster would require a ratio of
about 0.588. The small long-OR aggregate advantage is not a firm conclusion:
the candidate self-control's geometric-mean A/B ratios were 0.986 and 0.967
for the same cases. Controls ran in separate periods, so they do not directly
correct the comparison.

The run validated all 87,912 timing cells and their result checksums against
the retained exact-result gate; inputs and binaries were unchanged during the
run. The candidate worker was built at commit
`1494b21ccf1be2f3665d29170d457d6cf3500b49`; 376 engine source dependencies
were byte-identical to the current checkout. The port worker and query set
were the previously audited artifacts. The two APIs have a total-hit
threshold difference, and these results do not establish parity for every
search feature or for indexing, memory, cold start or other machines.

The machine remained in use. The sampled one-minute load average ranged from
4.89 to 13.50 and other CPU-intensive processes were present. The workers
were pinned to one CPU, but that does not isolate the machine. Raw samples,
checksums, telemetry, provenance, and the analysis are retained locally in
`search/bench/lucene-rs-or-hillclimb-oct07/attempt-36/extended-port-20261028/`.
The preceding interrupted run is preserved separately and excluded.

Separately, the release-mode differential test in
`tests/pruning_exactness.rs` passed 14,400 query/scoring-configuration checks
on eight generated indexes, comparing top-1/10/100 document addresses and
score bits plus counts with an exhaustive scan. Generated queries may repeat;
this is not a count of unique queries.
