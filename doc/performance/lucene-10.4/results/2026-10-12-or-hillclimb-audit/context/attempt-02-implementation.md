# Attempt 02 implementation: local order only

Worktree: `/home/pronsh/Coding/playground/search/tantivy-or-local-order-oct07`. Branch `perf/or-local-order-oct07`.
Base `0e9329dfe782187a8feda0024102e4aabf517c28`. Source SHA256 `e1ab97b65eaf969e7bc7308964412bec1323ac9ec96ba44cb6395dee5d282b3a`.

Only `src/query/boolean_query/or_maxscore.rs` changes. Diff stat:

```
src/query/boolean_query/or_maxscore.rs | 145 ++++++++++++++++++++++++++++++---
 1 file changed, 133 insertions(+), 12 deletions(-)
```

## Implemented mechanism

A query-local stack `[u8; 32]` stores ranks into the fixed global-preference
ClauseState vector. After the existing region selects its bounds and zeroes
actual decoded docs at or beyond hi, ranks reset to fixed global order. The
actual density-selected global optional prefix stays in place. Only the suffix
sorts by `f64(local_max) / f64(max(cost,1))`, with original ordinal tie break.
Existing inclusive prefix construction then uses this local permutation. Every
candidate-min, essential-score, reverse optional-probe, and matching-essential
advance uses the same permutation, but actual scorers and contributions retain
original incoming order.

The global certificate scan, density guard, shallow selection, tail reconciliation,
physical region boundaries, callback threshold timing, and published score replay
are unchanged. All actual cursor movement remains through ClauseState seek/advance.
Numerical eligibility and the pre-mutation canonical fallback are unchanged.
A debug assertion proves the partition retains all density-certified globals as
optional. No scorer API, bound arithmetic implementation, collector route, two-term
route, batch, window, heap, cached global prefix, or per-region allocation changes.

Scratch: three existing allocations stay; existing compact records/prefix/leaves
stay the same sizes. New local order is 32 stack bytes (plus any compiler stack
alignment/spills), not a Vec. No RSS or total stack-frame improvement is claimed.
One cost is that cleanup can no longer share the bound-prefix pass because local
sorting occurs between them; another is a local-rank indirection in scalar loops.
The port-like division comparator is intentionally the first isolated comparator,
not an unreported selection among cross-product/division alternatives.

## New independent oracle fixture

`local_priority_reversals_reconcile_promoted_stale_decoders` adds one fixture to
the existing ten. Two dense same-cost/same-global clauses reverse TF1/TF100 local
bounds at full block boundaries 128 and 256. A sparse high-value third clause has
hits at 0 and 383. The existing independent buffered union supplies EVERY callback
(doc, exact raw score bits), not only final Top-K results. The fixed-threshold run
asserts all 384 literal matching docs, and test-only partition evidence proves
optional A -> optional B -> optional A, correct physical boundaries, no global
certificate in these local-only regions, and a promoted decoder still at lo-1
before actual essential reconciliation. The observer records before the essential
floor seek and exists only under cfg(test).

The Top-1 run independently validates the whole trace again and asserts literal
accepted doc 0. Its rising threshold density-certifies both dense global clauses;
test-only evidence proves their original [0,1] prefix stays optional and the rare
tail widens the region from [128,256) to [128,512). Top-10 and reversed incoming-
order Top-1/Top-10 traces are also independently compared. Existing density,
exhaustion, midpoint, exceptional-domain, mixed-corpa and other fixtures are
untouched.

## Checks actually run

- Read the complete production diff after editing; only the module above changes.
- `rustfmt --edition 2021 --config skip_children=true` completed successfully.
- `git diff --check` completed successfully.

No compilation, tests, profiles, benchmarks, commits, or child agents were run by
this implementation agent. Runtime fixture assertions remain unverified until
root runs its guarded stages. This trial is NOT a claim of speedup and must not be
stacked into another mechanism before separate acceptance/rejection.
