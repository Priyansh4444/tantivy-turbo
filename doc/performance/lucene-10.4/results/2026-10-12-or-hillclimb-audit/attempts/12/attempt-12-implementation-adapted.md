# Attempt 12 adaptation onto accepted 11

Stable source for root review and runtime gates; no compilation, tests, worker,
profile, timing, compression, browser, child task, commit or push was run here.
Static rustfmt (`--edition 2021 --check`) and `git diff --check` pass.

- Worktree: `/home/pronsh/Coding/playground/search/tantivy-or-region-widen-oct07`
- Branch: `perf/or-region-widen-oct07`
- Actual HEAD/base: accepted11 `43f06833a325d2a18c03bd7e4fa65f2f27944292`
- Sole changed tracked path: `src/query/boolean_query/or_maxscore.rs`
- Source SHA256: `6c71f32394bac8d1c5998c511585904e09ac68757063b2f2bc96afb375258d1d`
- Diff against11: 959 additions,18 deletions;1114 full diff lines.
- Original12 source SHA256: `00fad7ecc30941fa72e6fbd002db5aff85b5b4e7a76b89403a9624ba65d56559`.
- Preserved original source and patch were read; the original06 receipt remains
  distinct from this combined candidate's `attempt-12-static-receipt-adapted.json`.

The isolated original tracked OR path was restored only after SHA/backup checks,
then the isolated branch fast-forwarded to accepted11. Three-way source merging
used original12/base06/accepted11. Production merged without textual conflicts.
The sole conflict was adjacent inserted test-helper/fixture families after the
shared `check` helper: both complete families were retained with their proper
closing braces. No source from rejected09/10 or another alternative was added.

## Original12 to combined12 differences

Combined source adds accepted11's query-persistent certified count and exact
sequential f64 sum, lazy monotone extension, generic independent from-zero cfg(test)
observer, prefix-state thread local/type, scheduled-threshold helper and two
fixture families. Accepted11's full global cache/observer/effective-density block
is byte-identical to actual HEAD. Its all-certified early exit stays before local
preparation. The query-owned16B cache remains distinct from12's128B cold endpoint
array; neither count nor sum is repurposed by widening.

Original12's complete local/widening preparation is byte-identical, including
virtual tight cleanup preserving raw selected bounds, one old-essential suffix
endpoint proposal, crossed-clause global replacement, and complete final prefix
and partition rebuild before replay. No width loop, selector, lookahead, cache,
new scorer/range API, allocation or format change is introduced by adaptation.
All10 original12 fixture families and their evidence helpers are byte-identical.
The original12 prior density geometry adjustment is also byte-identical.

From essential floor reconciliation through both replay routes, optional probes,
literal incoming-order publication, callback, cursor retirement and region advance,
source is byte-identical to accepted11. Canonical cursor methods retain their
original actual operations, with12's cfg(test)-only evidence around them.
The original12 implementation document describes that mechanism and proof surface
in full; its06-specific provenance must not be used as this candidate's identity.

One accepted11 fixture needs new geometry because12 legitimately widens its first
region. The old tiny two-posting tail allowed that first widened region to reach
3891, after the scheduled threshold rise; there was no intervening cost129 density
preparation to observe the original required1/effective1 state. Removing or relaxing
that assertion would lose meaningful cache/density evidence.

Instead, tiny now has129 postings:128 at0,17,...,2159 plus its3999 tail, and boost0.1.
Weak300/high200/driver129 postings and boosts0.01/0.02/1 remain unchanged. Existing
legacy DEFAULT weights imply maxima approximately0.022/0.044/2.2/0.22 in incoming
order, and actual max/cost priority is asserted to be weak,high,tiny,driver. Actual
bounds assert that the first weak global maximum fits initial0.04 but weak+high do
not, and that weak+high+tiny fit scheduled0.4 while adding driver does not. Tiny's
actual first selected full block end2159 and bound>0.04 are also asserted; no fresh
scoring or normalization expression is used as the oracle.

At preparation0 the effective global prefix is rejected because weak cost300 is
below2*200. Old essential tiny/driver endpoints propose2160; crossed high receives
its global bound and becomes essential, so traversal really exhausts its200 docs
inside this certificate. Tiny leaves its3999 tail pending; driver leaves2165 next.
At preparation2160 the unchanged weak certificate is admitted because300>=2*129.
Tiny's actual head3999 is outside tight driver cap3891, so its local zero makes
driver the sole essential until3891. A callback at or after3800 raises threshold to
0.4. At preparation3891 the cache extends to weak+high+tiny, but effective density
is rejected because the prefix contains high cost200<2*129 (even though high is
actually exhausted). Driver tail3905 still publishes; tiny3999 does not exceed0.4.

All original three-state assertions survive:
`(certified1,effective0,cost200)` before `(certified1,effective1,cost129)` before
`(certified3,effective0,cost129)`, monotone certificate states, final append count3,
first callback0, last callback3905 and complete independent canonical callback/raw
score equality. New exact preparation floor assertions0/2160/3891 make this geometry
observable rather than relying only on state predicates. Runtime realizability is
still root's responsibility; none of these fixture assertions has been executed here.

## Static receipt and review limits

`attempt-12-static-receipt-adapted.json` records exact spans/hash proofs:
- accepted11 global cache/observer/density3656B;
- accepted11 unchanged replay4234B;
- original12 unchanged widening preparation5513B;
- original12 evidence helpers and10 new fixture families28519B;
- all15 existing accepted11 helper/test functions except the two explicitly
  documented density geometries remain byte-identical;
- original12's prior density geometry function remains byte-identical.

I read the full actual1114-line diff in three contiguous chunks, and re-read the
combined production/cache/replay, observer definitions and modified fixture source.
Root must read the actual adapted diff and run all targeted/full/default/configured
and frozen latency gates. This source adaptation claims no speed, RSS, binary-size,
Lucene feature/API parity or universal correctness result. Existing reader position
fixtures remain owned by unchanged reader code; no release positions API was added.
Source is now held unchanged unless root requests a specific correction.
