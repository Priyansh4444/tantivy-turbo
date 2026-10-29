# Search performance gate

On each pull request into `main`, the workflow builds the same base-branch
harness against the base revision and the proposed merge commit. It creates
one deterministic 80,000-document index with the base binary and reopens it
with both binaries. Seven paired measurements alternate execution order. Each
sample warms up and measures each workload for at least 150 ms.

The gate compares exact counts or top-10 document addresses and score bits,
and fails if they differ. It also fails any workload whose median paired
candidate/base latency exceeds 1.25, or whose median absolute deviation
exceeds 15% of the median. An unstable result is inconclusive and should be
rerun and investigated. The log records every ratio. This conservative gate
is intended to catch large regressions; it does not prove small changes are safe.

The synthetic suite covers warm term, AND, OR and phrase top-10 searches and
AND/OR counts. It does not cover cold starts, index construction, memory,
storage or all query shapes. Use the larger documented benchmarks when
reviewing search optimizations. The correctness gate is separate and should
still run: matching fingerprints on these few queries is not general proof of
correctness. The harness lives on the base branch so the same workload is used
for both sides; changes to the gate itself require review and take effect on
subsequent PRs.

Run the comparator tests with `python3 ci/search-perf/test_compare.py`. Run the
full comparison with `python3 ci/search-perf/compare.py BASE_DIR CANDIDATE_DIR`.
Both directories must be checked out and usable as path dependencies. It needs
Rust stable and builds both binaries in temporary directories; use an otherwise
idle machine for meaningful numbers.
