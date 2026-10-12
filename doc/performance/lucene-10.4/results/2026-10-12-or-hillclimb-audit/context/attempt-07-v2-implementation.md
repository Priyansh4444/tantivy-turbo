# Attempt 07 v2: scalar-only threshold partition on accepted 03

Prepared from accepted parent `05835b971452bd5bda75037cec2925b7ba6785bf`
in isolated worktree `search/tantivy-or-threshold-partition-v2-oct07`, branch
`perf/or-threshold-partition-v2-oct07`. The source change is uncommitted and
changes only `src/query/boolean_query/or_maxscore.rs`. Main and the original
attempt-07 worktree were not edited.

## Scope adaptation from original 07

The original attempt used scalar candidate traversal throughout a certified
region. Accepted 03 can prefetch a single-essential batch: the scorer cursor
then points to the first unread posting while pending batch rows still own the
candidate documents and scores. Repartitioning that phase could reinterpret
the batch's owner or terminate while pending rows remain. The original 07
mechanism is therefore deliberately narrowed to regions that START with more
than one essential scorer (`!single_essential`).

The existing region-entry `single_essential` flag stays fixed. In an initial
single-essential region, batch selection, replay, scoring, threshold updates,
and batch-index advancement are unchanged. There is no repartition scan on
that route. If an initial multi-essential scalar region later shrinks to one
essential, its candidate traversal remains scalar until the region ends.

## Ownership and update order

The region's half-open `[lo, hi)`, pruning order, local maxima, inclusive prefix,
and existing ScoreSumUpperBound remain unchanged. A strictly growing callback
threshold in a scalar region computes only a prospective boundary from the
existing prefix. The CURRENT essential suffix continues to own all score and
matching-document advance obligations for the processed candidate. Every
matching old essential advances before installing the prospective boundary.
Thus demoted matching scorers do not retain the processed decoded document.

If all clauses become optional under that unchanged certificate, the loop
ends the region only after the old suffix's advances. The outer loop discards
the certificate and resumes at the same existing `hi`. Remaining essentials
were already reconciled at region entry; demoted optionals stay alive, and
existing subsequent-region floor reconciliation handles later essential roles.

No bound construction, region cap, density policy, fixed order, admission,
leaf scoring, incoming-order f64 publication, batch API, or heuristic changed.
There are no new release allocations or scratch arrays. The observer is
`cfg(test)` only.

## Fixture changes prepared for root validation

The original independent BufferedUnionScorer full doc/raw-score-bit callback
fixture is preserved. In both incoming orders it asserts the within-[0,128)
0->2 transition, old matching dense scorers advanced to doc+1, weak-prefix
equality, deletion-rejected doc0 leaving the threshold unchanged, and subsequent
2->3 full-prefix equality at doc10 after the old driver advances to doc300.
Its fixed-floor variant retains later sparse doc300 after dense optional
expiration. Signed fallback retains the canonical raw trace and no events.

The scalar fixture also observes existing BATCH_LENGTHS at initial-region
callbacks: no prefetch may occur after shrinking from multiple essentials to
one. This catches accidental recomputation of the region-entry execution flag.

The existing `single_driver_batches_replay_every_dynamic_threshold_exactly`
fixture is extended, rather than duplicated. Its full raw callback oracle and
existing full-128-row batch assertion remain. It now asserts no repartition
events under its rising Top-1 threshold and repeats the same raw trace in
reversed incoming order, again with no repartition events. Existing deletion,
Top-K, later essential-role reconciliation, boundary, numeric and randomized
fixtures remain available unchanged apart from this extension.

## Eventual attempt 08 stacking hazard

Attempt 08 is not merged here. Attempt 07 v2 is disabled whenever the region
uses accepted 03's prefetched single-essential route. A later bitmap/prefetch
attempt 08 must independently preserve this ownership boundary: an initially
multi-essential region could acquire prefetched rows under 08 while the present
`!single_essential` gate still identifies it as scalar. Its pending-row owner,
callback replay and repartition rules must be validated on the actual combined
implementation. Passing this candidate alone would not establish that stacking
07 and 08 is safe, and no such claim is made.

## Static verification only

Read the actual final source diff against accepted 03. Ran `rustfmt --edition
2021` on the changed Rust file, `rustfmt --check --edition 2021` on that file,
and `git diff --check`; all completed successfully. Only that source file is
modified in the isolated worktree. No build, test, profile, benchmark, timing,
compression, commit, or child-agent task was run. Runtime correctness and
performance acceptance remain owned by root. No runtime or speed claim is made.
