# Attempt 07: threshold-driven partition within a certified OR region

Isolated base `22f4a270926d513b2641cd44c5445008ecf3b276`, worktree
`search/tantivy-or-threshold-partition-oct07`, branch
`perf/or-threshold-partition-oct07`. This unaccepted trial changes only private
3+ OR execution, with no new bounds, ordering, global density decision, window
widening, batching, index format or public API.

## Owner/state sketch before implementation

The region owns immutable half-open `[lo, hi)`, fixed preference order, existing
local maxima and inclusive `prefix` certificates. The current `first_essential`
owns the candidate merge/scoring/advance slice for one entire candidate.

On a strictly growing callback threshold, compute a prospective
`next_first_essential` by extending the already-certified optional prefix with
the existing ScoreSumUpperBound. Do NOT replace the current partition yet.
Advance every matching scorer in the OLD essential suffix at the processed doc,
using existing cursor-owned mirror updates. Only then assign the prospective
partition and choose the next candidate. This prevents a demoted matching
essential from retaining the just-processed decoded doc through a missing
advance, and prevents a newly smaller candidate suffix from skipping old
advance obligations.

Within one region, essential roles can only disappear as the threshold grows;
no optional becomes essential and no new essential cursor needs reconciliation.
Every remaining essential was already reconciled at region entry. After old
matching essentials advance, each remaining essential doc is strictly greater
than the processed candidate (or TERMINATED). A demoted scorer remains an alive
optional and can be actual-seeked by later probes. At the next region, roles may
reverse; unchanged region-entry floor seeks reconcile newly essential cursors.

If the prospective prefix includes every term, the same immutable certificate
proves the entire remaining region cannot beat the threshold. Finish old
candidate advances, then end this region; the outer loop discards all local
certificates and continues at unchanged hi. Published leaves retain original
incoming ordinal f64 accumulation and final f32 cast.

## Implementation ready

Only `src/query/boolean_query/or_maxscore.rs` changed. The release mechanism adds
one prospective boundary local per candidate, scans the fixed inclusive prefix
ONLY after a callback strictly increases threshold, and assigns the boundary
only after unchanged old-essential matching-doc advances. If every term becomes
optional, it ends the current region after those advances. No region cap/bound
construction, fixed ordering, global density policy, scoring or admission code
changed. No new allocations or scratch arrays were added in release.

One bounded new fixture compares full callback doc/raw-bit traces to independent
canonical BufferedUnionScorer traversal in both incoming orders. An opt-in
cfg(test) observer proves a 0->2 boundary transition inside unchanged[0,128),
then 2->3 on the later stronger hit, including full-prefix equality and safe
current-region termination. It records the old suffix's post-advance minimum;
the first event must show the demoted dense matches moved to doc+1, which would
fail if the new boundary were applied before their advances.

The same fixture rejects deleted doc0 without raising threshold and proves the
first transition occurs at doc1, with the exact weak-prefix equality. A fixed
weak-prefix callback floor proves equality demotes the prefix and later sparse
doc300 remains visible across subsequent regions. Signed fallback produces the
same full canonical trace and no region event. Existing configured, numeric,
midpoint, tail and randomized raw-bit fixture gates remain untouched.

The observer and its event storage are absent from release. Runtime overhead
is an extra prospective boundary register and final branch per candidate, plus
prefix checks on strictly rising callbacks. It can lose if most thresholds
grow without actually extending the prefix, or regions end before enough
candidate work is avoided. No mode histogram or timing evidence is fabricated;
root must measure the trial against accepted parent before accepting it.

`git diff --check` passed. No formatting/build/test/profile/benchmark/commit or
child task was run. Source edits are stable; root owns all formatting, actual
diff review and runtime gates. No speed claim is made.
