# Attempt 07 integrated scalar repartition review

Verdict: no correctness or API blocker found by source review. Runtime correctness,
the new fixture's literal expectations, and performance remain unproven here;
root owns all build/test and frozen-harness gates.

Reviewed worktree: `/home/pronsh/Coding/playground/search/tantivy-or-hillclimb-oct07`.
Accepted parent: `a674aaf2f6187f3010a135dc25fdacf9cab27e53` (attempt 06).
I read the complete actual diff against that parent, the complete production OR
function and changed fixtures, `attempt-07-v2-implementation.md`, and
`attempt-07-integration.json`. Only `src/query/boolean_query/or_maxscore.rs` is a
modified source file. The untracked compatibility fixture directory remains a
separate artifact, not part of this source change.

Current source SHA256:

| Source | SHA256 |
| --- | --- |
| `src/query/boolean_query/or_maxscore.rs` | `7d0077d946369751bfd24c15503a9c2b00ff7dc73364e9cab7cf39ee31ab0381` |
| `src/query/score_combiner.rs` | `9392f9d0c0f537252e27bdb68eccc2623d43607980e12d45e45458026aab6eb0` |

The integration manifest names the actual accepted-06 parent and matches the
current OR hash. The implementation note's accepted-03 base describes its
isolated preparation. In the actual integration, the accepted-06
`SmallScoreSumUpperBound` import/construction is preserved; the new fixture alone
qualifies its generic `ScoreSumUpperBound` oracle. Line references below refer
to the current OR source identified above.

## Correctness argument

The region's local maxima and inclusive prefix remain fixed for `[lo,hi)`
(`or_maxscore.rs:191-251`). Each prefix is a conservative certificate for its
original clauses within that interval. The new scan (`343-349`) runs only when
a callback strictly raises the threshold, starts at the current boundary, and
demotes a prefix only while its rounded bound is <= the new threshold. A
document matching no remaining essential can then contribute only from that
certified prefix and cannot exceed the threshold. Equality is deliberately
safe because score publication still requires strict `score > threshold`
(`335-351`). Scanning uses the original constructor and original `num_terms`
allowance (`107-111`); it does not narrow the factor to the surviving suffix or
change any bound arithmetic.

The prospective boundary is a separate local (`298-299`). Candidate scoring
and optional probing use the old partition (`300-339`), and every matching old
scalar essential advances through `ClauseState::advance` before installation
(`353-386`). Thus demotion cannot abandon a just-processed decoded posting.
The next scalar candidate is the minimum of the new suffix and is strictly
beyond the processed document. Existing optional clauses may retain older
positions, as before, and seek when probed. Remaining essentials form a subset
of the region-entry suffix, so all were already reconciled at entry (`253-258`).
No newly promoted essential exists within the region.

The execution mode is fixed once at entry (`260-264`). Initial single-essential
regions never enter the repartition scan; their batch owner, pending rows,
batch-index update, and fixed boundary remain intact (`268-287,343,353-354`).
An initial multi-essential region remains scalar even if its suffix shrinks to
one. Hence no pending prefetched row is reinterpreted as another owner's score
or discarded by a repartition decision.

If the scan makes every clause optional, the old suffix still advances first,
then the loop ends (`356-389`). This only skips the rest of the currently
certified interval. The outer loop invalidates that certificate and resumes
at its existing `hi` (`392-397`), allowing all still-live optional clauses to
become essential under the next region's ordinary reconciliation and bounds.
Neither the physical address cap nor the shallow-block ownership changes.

The accepted-06 admission remains unchanged (`92-105`), including canonical
fallback before any cursor movement. Its finite nonnegative matching-leaf
proof still assumes valid indexed matching TF > 0, including native k1=-0
with -infinity inverse and b=1/norm0 with +infinity inverse. This attempt does
not broaden that numeric domain or certify synthetic zero-TF/invalid postings.
The callback's existing nondecreasing-threshold contract is also retained;
arbitrary decreasing or NaN callback returns are not newly certified.

## Fixture expectations and blast radius

The new fixture (`544-660`) compares every callback's document and raw score
bits against exhaustive `BufferedUnionScorer`/`SumCombiner` replay, using the
same callback behavior in both executions. Both incoming orders are exercised.
Its expectations target the mechanism rather than merely a final top hit:

- The first interval is asserted as `[0,128)` with a 0->2 boundary change
  (`616-621`). The deletion variant must delay that change from doc0 to doc1,
  and fixed-floor/held-threshold cases compare raw threshold bits with the
  weak-prefix certificate (`626-628`).
- The observed minimum across the entire *old* suffix must be doc+1 after
  the first event (`622-625`). In this dense fixture, a demoted matching
  scorer left on the processed doc would make that minimum fail.
- Ordinary score-growth variants assert the second transition 2->3 at doc10,
  raw threshold equality with the whole certificate, advancement of the old
  remaining driver to doc300, and no later accepted hit (`635-646`).
- The fixed-floor variant requires later sparse doc300 after the dense clauses
  expire, with callbacks `[0,10,300]` and exactly one event (`629-634`). It
  therefore checks that within-region demotion does not permanently lose a
  later region's surviving clause.
- Batch observations at callbacks below doc128 must remain empty even after
  the scalar suffix becomes one (`570-583`), catching an accidental switch
  to prefetch after the first transition.
- Signed admission compares canonical raw traces and requires no repartition
  event (`649-659`). Existing single-driver batch coverage now also requires
  zero repartition events for rising thresholds in both incoming orders,
  while retaining its full-128-row batch assertion (`484-504`).

These are meaningful literal assertions whose satisfaction still requires the
root's test run. The certificate calculations use the generic helper as an
oracle, with equivalence to accepted 06 established separately; callback
expectations use the independent exhaustive union. The new fixture uses
classic BM25 weights. It does not by itself prove every native/custom-parameter
configuration or every possible clause count. Existing exact/raw-bit gates,
midpoint/order fixtures, numeric-domain fixtures, and broader regression tests
remain necessary and unchanged.

The sole modified release mechanism is scalar-region partition shrinking.
There is no public API change, new release allocation, scratch array, score
format, scoring arithmetic, combiner, document publication order, global
density decision, batch API, or index/compression-format change. The observer
struct/thread-local and event allocations are `cfg(test)` only (`66-81,366-383`).
The claim is exact callbacks/scores with fewer potential scalar candidate
drivers; it is not identical internal cursor trajectories. Cursor advancement
obligations and subsequent-region reconciliation are preserved as explained
above.

The implementation note correctly scopes the change to initially scalar
regions and explicitly warns about stacking another multi-essential prefetch
mechanism. That warning matters: a future attempt that prefetches within an
initial multi-essential region must replace or strengthen this mode gate.
Passing this candidate would not certify such a combined implementation.

This delegated workflow hillclimb review follows boundary discipline by tracing
the unchanged certificate and row ownership, and prove-it-works by separating
source evidence from pending runtime measurements. I ran no build, test,
profile, benchmark, browser, or compression job; made no source edits; and
spawned no children. Root's build was running when review was requested. No
runtime-pass claim or performance verdict is made in this artifact.
