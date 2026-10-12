# Attempt12 two test-witness corrections

Root retained the first targeted run and exact source at
`attempt-12-pre-witness-fix`:23 tests compiled,21 passed,2 failed. I read its
actual stdout/stderr and receipt. Failures were the new required partial-batch
witness and the new midpoint fixture's unconditional widening witness. The shared
independent full callback trace and literal certificate/cursor oracle had already
passed before those explicit witness assertions; this is not a performance result.

Only these two new fixture functions change. Production, cfg(test) observers,
accepted11 tests, both previously documented density geometries, and all other
new12 fixtures are byte-identical to the failed source. Replacing exactly the two
changed function bodies with their previous bytes reconstructs the full failed
source and SHA256 `6c71f32394bac8d1c5998c511585904e09ac68757063b2f2bc96afb375258d1d`.

## Competitive partial final blocks

The old257-posting corpus put TF20 at driver index128/doc382, then onlyTF1 in its
last one-row tail. Top1 raises its threshold on382, so the kernel can correctly
certify that later weak tail as noncompetitive before filling it. Requiring a
partial batch from that corpus confuses correct pruning with a missing witness.

For every non-multiple-of128 corpus length, the last driver posting now hasTF30.
This is stronger than the earlierTF20 posting under the existing literal legacy
DEFAULT scorer. For129 postings the last posting is382 itself and thereforeTF30;
for255/257 the later partial final block retains a competitive stronger leaf. The
127-row single tail also retains a competitive last leaf. No norm, bound helper
or scoring formula changes. Lengths128/256/768 retain their existing full-only
expectations. The required nonempty partial-batch assertion remains unchanged in
strength and now prints the failing length. For every length>=129 the actual
128-row batch retaining exclusive unread head382/up_to382 and later382 callback
assertions remain. Complete independent canonical trace/raw bits, literal leaf
bound enclosure and actual cursor/batch-progress checks remain.

## Midpoint strictness while widening really occurs

The old driver had onlyTF1 leaves. At a threshold equal to or above doc0's maximum
midpoint publication, the initial full-block certificate can correctly skip all
regions. Demanding widening and an empty trace simultaneously was not realizable.

The first full driver block now has one laterTF2 leaf at index127/doc381; every
other driver leaf, including doc0, remainsTF1. Tiny leaves and all three incoming
orders remain unchanged. The existing actual stored bound must now cover a leaf
stronger than doc0 in that same full block, so initial equal/neighbor thresholds
can still produce a genuine wider first certificate. Doc0's raw midpoint leaf1
and the sequential incoming-order expected score computation remain unchanged.

For all three orders, threshold-down/equal/up, and fixed/Top1/Top10, the fixture
still demands observed widening and now explicitly demands the later competitive
381 callback. When initial threshold is below doc0's expected publication, the
first callback remains exactly `(0,expected_raw_score_bits)`. At equal/above it,
explicit doc0 absence replaces the old all-empty assertion, because381 is now a
purposefully stronger matching document. Complete canonical trace equality and
all literal certificate/progress assertions remain the independent score oracle.

## Stable artifact and limits

- Base: accepted11 `43f06833a325d2a18c03bd7e4fa65f2f27944292`.
- Source SHA256: `a53374a75b5dbedb3e8f148b94f8da92a6d4383142e69cc8b50a92330c2f96d1`.
- Sole dirty path: `src/query/boolean_query/or_maxscore.rs` in the isolated region
  widening worktree; root integration and production trees untouched.
- `attempt-12-witness-fix.patch`: entire old→corrected diff, read end to end.
- `attempt-12-witness-fix-full.patch`: entire candidate diff against accepted11.
- `attempt-12-source-witness-fixed.rs`: stable corrected full source.
- `attempt-12-witness-fix-receipt.json`: exact reverse reconstruction and unchanged
  production/observer/all-other-tests byte proofs.

Rustfmt `--edition2021 --check` and `git diff --check` pass. No compiler, test,
worker, profile, timing, compression, browser, child task, commit or push ran here.
Root must rerun the actual targeted fixtures and all later gates. No new runtime
success, score-bound formula, normalization policy or performance claim is made.
Source is held unchanged until root requests a specific correction.
