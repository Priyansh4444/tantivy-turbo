# Attempt 08 implementation: exact-leaf multi-essential bitmap

Status: source ready for root-owned runtime verification. No build, test, profile,
timing, compression, commit, or child agent ran in this implementation task.

Worktree: `/home/pronsh/Coding/playground/search/tantivy-or-multi-bitmap-oct07`
Branch: `perf/or-multi-bitmap-oct07`
Accepted parent / current HEAD: `05835b971452bd5bda75037cec2925b7ba6785bf`
Changed file: `src/query/boolean_query/or_maxscore.rs`
Receipt: `/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07/attempt-08-implementation.md`

Read grounding.md, design-multi-batch.md, design-multi-heap.md,
design-multi-judge.md, and design-multi-synthesis.md end to end. The chosen
bitmap mechanism includes the judge's complete refill ownership and pending
cursor exhaustion verification grafts; it adds no heap, selector, or policy.

## Production shape and invariants

`ExactLeafWindow` holds one lazily allocated query-owned incoming-order
`128 * num_terms` f32 matrix, 128 f64 essential-known values, and two u64
match words. The private complete `gather_window` operation owns minimum,
physical end, batch fill, first-touch zero, preference-ordered scatter, and
progress assertions. It can only gather into an empty bitmap. Empty shallow
reconciliations jump to the next authoritative minimum, never a sequence of
empty physical windows. Existing `ScoredTermBatch` scratch is reused.

Regions with >=2 essential suffix entries use that gather and ascending
trailing-zero bitmap replay. The existing exactly-one-essential batch branch
retains its fill/index behavior. The scalar optional loop and incoming-order
f64 fold / one f32 publication cast are shared. Callback threshold is read for
every candidate. Mirrors always hold actual first-unread cursors; pending
bitmap rows, rather than real cursor minima, govern replay completion. No
region expires or repartitions while rows remain. Copied numerical region
certificates remain in force after the batch loads another decoded block.

Safe-domain admission/fallback, global density admission, fixed preference
order, local region/prefix policy, two-term dispatch, max_doc physical-ID use,
strict publication threshold, and collector deletion ownership are preserved.
The obsolete scalar essential advance mirror method was removed because both
essential routes now use the owned batch method.

One additional matrix allocation occurs only at first multi-essential dispatch;
there is no production per-window or per-region allocation. Expected max-32
array payload remains 20,632 B including accepted batch scratch and existing
clause/prefix/contribution arrays. The semantic fixture prints actual host
container size, matrix len/capacity, foundation and payload; those are proposed
runtime receipts, not measurements made here. It avoids fixing a target ABI.

## Independent fixtures added

Eight bounded tests use the existing independent BufferedUnionScorer full raw
callback oracle, plus a cfg(test)-only window observer:

- all three actual cursors TERMINATED after gathering, with five pending winning
  rows at offsets 0/7/63/64/127 that must still replay in order;
- n3/n32, disjoint/dense overlap, 127/128/129/255/256/257 posting lengths,
  fixed and growing Top1/Top10 thresholds;
- first-touch clearing of dirty essential and optional leaves, including repeated
  offsets 0/7/63/64/127, with literal power-of-two incoming expected bits;
- dirty reuse as the fixed essential suffix grows and then shrinks across
  distinct certified regions, with observed ordinal sets;
- million-ID physical gaps and end clipping, asserting only occupied windows
  are visited;
- legacy/native boosted incoming-order leaf folds and f32 threshold neighbors,
  with explicit multi-essential activation at zero threshold;
- current within-window thresholds and simulated collector-owned early/deleted
  winner rejection, comparing complete callback traces;
- an actual RAM index with physical deletions, real alive bitset callback
  rejection, full independent callback traces, first-unread >=hi while pending,
  and the real generic TopDocs collector selecting the later live winner.

Existing unsafe-domain tests also assert no bitmap gather events. Existing
single-essential/foundation tests and full raw traces remain intact.

The actual indexed-delete fixture is distinct from the simulated callback
fixture. Neither is represented as executed evidence yet. Root owns their
execution and the existing integration/default/two-config frozen gates.

Fixed preference suffixes are nested. Simultaneously making an old essential
optional and an old optional essential at one boundary is impossible under the
retained policy. The fixture therefore proves sequential growth/shrink in
separate regions, as agreed with root, rather than introducing a role-swap
policy. No other architectural deviation was needed.

## Review and limits

Ran rustfmt on the changed Rust file and git diff --check, both successfully.
Read the resulting complete diff and the final literal-bit fixture edits.
Only one production source file is modified, uncommitted. This is a source
implementation handoff, with compilation, runtime correctness, full gates,
performance, controls, and acceptance all pending root-owned work. No speed,
RSS, index-size, or target-victory claim follows from this receipt.


## Discriminating fixture strengthening (2026-10-07)

Extended only two existing tests after the source handoff; all eight bitmap tests
and their existing cases remain. Production code and oracle helpers are unchanged
by this strengthening. No build, test, perf, compression, timing, commit, or child
agent ran in this follow-up. Root still owns execution and acceptance.

The first-touch test now also has a bounded sparse-optional case. Both essentials
match 0/7/128/135 with unchanged observed ordinal suffix [0, 2]; the weak optional
matches doc 7 only. Exactly two occupied windows must have actual bases 0/128,
ends 128/256, region hi 256, two filled essentials, and offsets 0/7. Both essential
columns overwrite row 7 in the second window, while the optional column is absent
there. The full independent raw callback oracle and literal bits require doc 7 =
0x40401000 (3 + 2^-10) and doc 135 = 0x40400000 (1 + 2). A stale optional leaf
would therefore change doc 135's publication even when both essentials overwrite.

The midpoint test retains the original n4/zero-threshold case and adds a separate
five-term corpus with boosts [.5, .5, 2^-24, 2^-53, 2^-53]. High-TF docs 127/255/256
put a substantial score in both full blocks and the tail. For both legacy and
native DEFAULT profiles, scalar scorer checks require each strong leaf at those
docs to exceed the low-TF expected.next_up threshold. Three incoming orders each
run expected.next_down/equal/next_up at limits 0/1/10 through observed_check, which
compares the complete raw callback trace with BufferedUnionScorer and requires
an actual base-0 bitmap gather containing both substantial essentials with at
least two fills. The low-TF doc-0 publication/tie and high-TF doc-127 callback are
also asserted. Legacy TF=1 leaves are checked against the literal boosts; native
expected values derive from the actual scalar leaves before the incoming f64
fold and one f32 cast.

All term frequencies in these added cases are positive (1 or 1000), matching the
posting domain they exercise; zero-TF behavior is not claimed. Native fixture
coverage uses Bm25Parameters::DEFAULT and does not substitute for the root-owned
configured-parameter/default/integration frozen gates. No execution, performance,
or configured-profile result is inferred from these source assertions.

Ran rustfmt and git diff --check successfully and read the complete resulting
diff. Source is stable for root-owned runtime checks. Actual changed-file SHA-256:
`40e9e0b194603b73907f7cc462650aa8b34ff19bb7e74c10fed1798d80869209`

This receipt was extended in place; no existing journal evidence or raw timings
were overwritten.

## Root-reported targeted compile failure and fixture correction (2026-10-07)

Root's first targeted attempt-08 test build reported E0689 at line 918,
`current.max(score)`, because the new simulated collector fixture declared the
threshold literal without a concrete float type. No runtime attempt-08 result
exists from that failed compilation. Root retains the failed build receipt.

At root's explicit request, the source reviewer made exactly one test-only edit
in the isolated bitmap worktree: line 914 now reads
`let mut current: Score = 0.1;`. The production implementation, all assertions,
and every other source byte are unchanged. A small read-only reconstruction check
removed that single annotation and reproduced the preceding SHA-256 exactly;
`git diff --check` passed. No build, test, profiling, timing, compression, commit,
or child agent ran for this correction. Existing review proof and strengthened
fixture findings remain unchanged.

Current stable changed-file SHA-256:
`c4f7402f22938906d111472b690285074a01ea2e3a224f5a0102773f1855abf0`.
Previous reviewed SHA remains above as historical evidence, not the current
source receipt. Root will graft the single declaration, refresh the source/build
guard even though production release code is unchanged, and rerun targeted,
default, and configured gates. Successful compilation and runtime acceptance
remain pending root-owned execution.
