# Attempt 10 independent source review

Reviewed stable source SHA256:
`81cb888e53e467a03515f0224aeefbe3bd3b05a25eaf6cd294491eea578a9729`.
Worktree: `search/tantivy-or-single-lead-oct07`; accepted06 parent
`a674aaf2f6187f3010a135dc25fdacf9cab27e53`.

Read end-to-end: the full actual diff (749 additions/52 deletions), all1448 lines
of the resulting OR source in bounded chunks, design-single-lead-128.md and
attempt-10-implementation.md. Also traced unchanged fill_scored_batch through
SegmentPostings::read_postings_batch and test scorer construction. Root confirmed
this stable SHA after the fixture owner correction. Independently reversing ONLY
`TopDocs::with_limit(1).order_by_score()` reproduces original source SHA
`1e57a16764161506a8dc1e62a909e13f5b21604c161b6992eea8d53ebc4bc7dd`.
The retained initial compile failure belongs to that earlier fixture call; no
production repair or runtime result is attributed to this review.

## Findings

No production correctness blocker found within the existing admitted positive,
valid-postings and monotone callback-threshold domain. This is a static review;
root-owned compiler, exact gates and timing decide the runtime verdict.

1. Classifier (lines80–117) takes both minima in one pass. An equal first doc
   occupies second, so tied matches are scalar. For unique leads, cap=min(hi,
   second) is fixed and exclusive. first>=hi is handled before subtraction;
   subtraction is guarded by first<cap. TERMINATED is never incremented or used
   as an emitted doc. Scalar fallback carries the same first minimum, avoiding
   a hidden second min scan. A one-clause suffix skips the128 cutoff.
2. Ownership (lines343–425): fixed rank/cap/len/next persists while retained
   rows exist; real doc>=cap/TERMINATED is tested only after next==len. Therefore
   a batch which advances past hi still emits every retained candidate first.
   Successful fills create finite rows, retirement increments next, exhausted
   spans become Drained, and zero fills assert authoritative doc>=cap before
   redispatch. The unchanged postings owner explains zero-fill progress: it
   reconciles shallow-selected metadata when needed, then returns zero only
   when first-unread is outside the cap.
3. Cursor movement: ClauseState fill updates the authoritative mirror. Only
   selected lead advances during fill; every other essential is >=fixed cap
   throughout that span, and optional probes cannot mutate an essential. Rank
   indexes metadata, ordinal indexes incoming scorer/contribution storage.
   Scalar retirement advances all matching essential cursors in preference
   order. Region prep still reconciles essential docs below lo before dispatch.
4. True-single equivalence: same floor, hi cap, score batch and reader movement
   for nonempty fills, same contribution/known arithmetic and row retirement.
   The new code omits the final no-op fill when actual doc already>=cap. This
   emits no doc/leaf and does not change relevant cursor work; later ordinary
   region preparation/first actual fill still owns shallow reconciliation.
5. Arithmetic/publication: single lead has exactly one essential leaf below its
   cap, so f64(leaf) equals the old preference-order known sum including absent
   nonnegative zero contributions. Scalar accumulation preserves old order.
   A light source-comparison script proved admission and region preparation,
   common optional/current-threshold/original-incoming-fold/callback body, and
   the entire accepted fixture suffix are byte-identical to accepted06. No
   same-region repartition, new bound, format mutation or allocation was added.

## Meaningful fixtures and limits

Six new families distinguish classifier127/128/129 and sentinel/ties; actual
posting batches127/128/129 at runner and hi boundaries with future heads;
retained127-row exhaustion plus later winner at n3/n32; lead->scalar tie->other
lead with dirty optional rows; native/legacy midpoint and neighboring strict
thresholds with actual temporary multi-essential fills; simulated collector
rejection and a real RAM index deletion/alive-bitset/public TopDocs check.

The observer independently recreates original scalar leaves, refusing backward
seek when original doc is already after the candidate. It verifies incoming
essential contribution bits and preference-order known sum before optionals.
Pending row checks compare against original scorer snapshots, not advanced real
cursors. Midpoint fixture uses positive indexed TF, checks calibrated native2.2
power-of-two leaves before oracle comparisons, and asserts >=2-essential lead
fill contains doc0. The real delete fixture asserts400 physical/398 live docs,
retains physical deleted docs0/63 in a selected128-row multi-essential span,
and checks live doc127 raw winning bits through public score-ordered TopDocs.
Native midpoint calibration and these branch activation assertions still require
root runtime checks; source reasoning alone does not establish them.

No speed, activation-rate or RSS conclusion. The extra second-min/enum dispatch
can hurt dense or short capped regions; a128-address span may contain one posting.
Region setup, tight optional block caps, clearing and publication costs remain.
The existing2048-byte batch is reused; extra O(1) owner metadata is stack/register
payload, not an efficiency claim. No reviewer builds, tests, profiles, timings,
compression, browser, children, commits or source edits were run.
