# Attempt 10 implementation receipt

Stable source is ready for root integration and runtime checks.

- Parent: `a674aaf2f6187f3010a135dc25fdacf9cab27e53`.
- Worktree: `/home/pronsh/Coding/playground/search/tantivy-or-single-lead-oct07`; branch `perf/or-single-lead-oct07`.
- Sole changed source: `src/query/boolean_query/or_maxscore.rs`.
- SHA256: `1e57a16764161506a8dc1e62a909e13f5b21604c161b6992eea8d53ebc4bc7dd`.
- Root read and agreed the whole revised design before source mutation.
- No source-shape deviations. No attempt09 heap or attempt08 matrix carry.
- Architect Ground/Sketch/Agree and implementation/static review complete;
  root compiler, test, exact gate and latency verdict pending.
- No builds, tests, profiles, timing, compression, browser, child or commit jobs
  were run by this implementer. Rustfmt and git diff --check passed.

## Actual source review and invariants

The real changed production loop and all new fixture bodies were read. A script
proved admission/region preparation and the one optional/current-threshold/
incoming-publication body byte-identical to accepted06. No new allocation,
reader/scorer API, index format, preference order, numerical bound, or certificate
change. Only candidate dispatch/replay and meaningful test-only evidence changed.

`choose_essential_dispatch` returns no-cutoff true-single spans. Otherwise it
records both minima in one pass, counting ties twice, then tests only fixed
`min(hi, second_doc)-first_doc >=128` after safe unsigned ordering. Rejected span
selection carries that same pass's minimum into scalar replay; there is no repeated
minimum scan. Scalar matches/known accumulation and post-callback advances retain
accepted preference order. Pending rank/cap/len/next live in one exhaustive
ReplaySource variant. A nonempty batch drains after actual cursor exhaustion or
hi crossing; refill retains the same rank/cap, and no fresh dispatch or region
expiry occurs until rows retire. Zero fill asserts authoritative doc>=cap.

True-single literal scoring and authoritative reader movement are unchanged;
the owner now avoids a final no-op zero fill once the cursor is already at cap.
That fill in accepted06 would return zero without mutation, so no posting or
score behavior changes. Callback completes before row retirement or scalar
matching advance. Old region setup/bounds/threshold certificate remain fixed.

## Distinguishing fixtures prepared

Six families add independent raw callback comparisons and test-only activation
checks: capped127/128/129 classification, ties/sentinel/no-cutoff true-single;
explicit runner/hi boundary with127/128/129 real lead postings and future docs;
pending exhaustion with later winner and rejected early collector winner at n3
and n32; lead->scalar tie->different lead with dirty optional contributions;
positive-TF literal midpoint leaves in native and legacy profiles with equality
and adjacent thresholds and actual temporary multi-essential fill; real RAM index
deletes/alive bitset and public TopDocs, including a selected128-row temporary
multi-essential span containing deleted physical docs. Accepted existing role-
change, unsafe-domain, tail and threshold fixtures are preserved unchanged.

The fixture evidence oracle refuses backward seeks when the original scorer
starts after a candidate. Static review caught an invalid empty-postings fixture
constructor and replaced it with valid positive-TF scorers explicitly advanced
to TERMINATED before finalization. No failure was hidden; no runtime was run.
Native midpoint calibration uses DEFAULT weight2.2 times power-of-two boosts,
and the fixture asserts actual scalar leaves equal the target powers before
calling the union oracle; root runtime must validate this claim.

## Limits

No speed/activation/RSS claim. Physical128-wide spans can have one posting; extra
second-min and enum dispatch work can hurt dense/short regions. Bounds, selected
block caps, optional work and per-candidate incoming clearing remain. Actual
SingleLeadSpan/ReplaySource host bytes are emitted by a fixture only when root
runs it. No new heap payload or per-region allocation. Root must review full diff
and run the frozen parent06 gates/pilot, accepting only a measured win.
