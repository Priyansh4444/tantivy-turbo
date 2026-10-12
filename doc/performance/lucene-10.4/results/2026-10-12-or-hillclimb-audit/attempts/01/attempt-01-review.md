# Attempt 01: actual implementation review

Read-only review of the complete current diff in block_wand_union.rs and
boolean_weight.rs, all six new raw-bit fixture bodies and their independent
oracle, plus the relevant postings seek/shallow, bound, and score-sum APIs.
No production edits or builds/tests/profiles/timings were performed by this
reviewer. Test execution and acceptance remain root-owned.

**No blocking correctness finding from source review.** The implementation
matches the fallback-only synthesis: the production caller passes physical
max_doc; unsafe numerical domains take canonical union before movement; the
existing warm/global routes remain; the new post-warm region kernel adds no
batch API, allocation, local sort, or threshold-driven repartition. There is
no new intersection hand-off from shallow region state.

## Counterexample checks

* **Shallow cursor versus bound:** a selected finite block caps hi before its
  maximum is used. Actual decoded docs may remain stale below lo, but newly
  essential cursors seek to lo before scoring and optionals seek to each
  candidate. A shallow block can move only when its prior selected end is below
  the target, so stale decoded next-doc shortcuts cannot bypass loading when
  reconciliation crosses that end. Tail selection explicitly reconciles at lo
  and saves the selected cap. doc>=hi cleanup cannot erase a possible earlier
  hit: actual decoded cursors are monotone.
* **Global optional cap omission:** it occurs only when exactly one global
  maximum is <= threshold, uses that immutable maximum, and keeps the other
  stream's finite cap. Rising thresholds cannot invalidate that certificate.
  Both-global-optional mode uses both local caps rather than transferring stale
  cursor state. Region sum rejection remains conservative.
* **Zero/exhausted and negative thresholds:** terminated clauses contribute
  zero. At negative thresholds zero maxima can still select Union, which is
  necessary to retain zero-score matching hits. TERMINATED docs never become
  candidates. If an essential stream is exhausted while the remaining global
  optional cannot win alone, the region sum safely rejects. max_doc-1 is reached
  only inside lo<max_doc, so there is no empty-range underflow.
* **Required roles expire:** the required-membership flag is rebuilt each
  region. It is used only when both individual maxima cannot beat the threshold,
  and is not converted into a persistent query intersection. Keeping a prior
  mode while threshold rises can add work but cannot remove a winner.
* **Published leaves:** incoming scorer slots remain fixed. Every new-kernel
  score uses its original two f32 leaves, f64 addition, then one f32 cast.
  Bounds use forward sums and the existing two-leaf helper. Warm/global f32
  two-operand sums were retained; they do not introduce multi-leaf order drift.

## Do the fixtures reach the changed branch?

Yes. The first two fixtures observe direct region choices and assert both
identity reversal and required-to-single-essential expiry. Their wrapper runs
include limit=0, fixed threshold 0.15, and two positive global maxima above that
threshold; 512-doc lists therefore pass the 128-candidate warm phase without
switching to the old global routes and enter the changed fallback. The expiry
fixture also has growing-threshold wrapper runs that preserve both global
maxima above threshold at fallback entry.

The complete-block/tail fixture exercises stale skipped full blocks followed
by an actual tail seek and local required-membership rejection. The midpoint
fixture checks literal leaf bits and exact threshold neighbors. Warm boundary
cases 127/128/129 protect the old hand-offs. Exceptional-domain runs use only
the wrapper and independently compare canonical full callback bits. Collector
rejection checks the complete callback trace, including rejected early and
post-warm high-score docs, without raising the threshold. Both incoming orders,
fixed and growing thresholds, and direct/wrapper paths are included where valid.

## Residual limits

All six new helper-built fixtures use legacy BM25 weights and stored legacy
selection. Native/default/configured transform behavior needs root's complete
frozen gates and existing native bound tests; these fixtures alone do not prove
that path. Deletion is simulated through the collector callback; no new real
deleted-index integration fixture was added. The unchanged production deletion
callback and physical reader.max_doc boundary reduce that risk, but the simulated
test should not be described as a real-index deletion test.

The observer records executed modes, not rejected regions or a precise cap-
omission transition. Cap omission has a straightforward immutable global proof,
but direct observed coverage of that policy is weaker than the two reversal
checks. These are evidence limitations, not a found correctness defect. No
speed or test-pass claim follows from this source review.
