# Fresh cross-judge: multi-essential merging

Recommendation: use **128-doc exact-leaf bitmap windows** as the first isolated implementation base. Graft the heap candidate's stronger encapsulation of pending replay versus authoritative first-unread cursors and its explicit exhausted-cursor replay adversary. Keep the posting-buffer heap as a distinct sparse/disjoint alternative rather than combining the mechanisms or adding a selector now.

This is a source/design judgment, not a measured speedup. I read grounding.md and both candidate packages end to end, then inspected the actual attempt03 OR loop, ClauseState mirror mutation methods, TermScorer batch ownership, SegmentPostings batch consumption, Bm25Weight batch arithmetic, and the independent callback-trace fixture. I also read the retained attempt-01/port-summary.json after root clarified that grounding's fixed-port figures predate accepted01. No builds, tests, compression, profiles, timings, production edits, commits, or child agents were run.

throughput checkpoint: n/a, read-only investigation

## Criterion scores

Scale: 1 = inadequate; 3 = credible with substantial limitations; 5 = strong design coverage. These scores assess the design contract; neither candidate has earned runtime acceptance.

| Criterion | Bitmap | Heap | Evidence and judgment |
| --- | ---: | ---: | --- |
| 1. Exact published bits, callback order, cursor/certificate proof | 5 | 5 | Both retain literal f32 essential leaves, fixed preference order for pruning-known additions, incoming ordinal order for f64 publication, current threshold for every candidate, physical deletion ownership, and cursor-owned batch mutation. Bitmap proof is in design-multi-batch.md:64–102; heap proof is in design-multi-heap.md:115–230. |
| 2. Causal potential to remove repeated essential scans rather than another small bookkeeping tweak | 4 | 4 | Bitmap replaces three per-candidate suffix scans with per-window scans and per-hit scatter. Heap replaces linear candidate selection with compact O(log e) head maintenance and posting-bounded batch scoring. Both change the expensive multi-essential mechanism. Neither removes per-region prefix work, O(n) row clearing/publication, optional probing, or two-term work; there is no occupancy evidence establishing a halving. Bitmap has the better dense-overlap mechanism; heap has the better sparse/disjoint mechanism. |
| 3. Compact bounded scratch; no format change or per-window allocation | 5 | 4 | Both allocate one lazy query-owned Vec and reuse fixed batch scratch. Bitmap's stated maximum payload is 20,632 B; heap's expected payload is 36,360 B. Heap also copies retained doc and score prefixes and reserves slots for every clause even when only two are essential. Host layouts and compiler frame costs remain unverified. |
| 4. Ownership hidden behind small readable interfaces | 4 | 5 | Bitmap exposes scatter/pop primitives and leaves the full refill protocol in the caller; its returned row borrow prevents mutation during replay of that row, but alone does not prevent discarding other pending rows between calls. Heap presents complete begin_region/next_candidate operations and makes replay-head ownership explicit. Both retain cursor ownership at the existing deep postings boundary. |
| 5. Meaningful independent verification, including old-policy regressions | 5 | 5 | Both demand complete raw-bit traces against BufferedUnionScorer, activation evidence, threshold/deletion adversaries, dirty storage reuse, clipping/tails, batch boundary lengths, n3/n32, and preservation of 401/719/791/691/909/817 plus complete frozen gates and balanced measurements. These are proposed gates, not executed results. |
| Total | **23/25** | **23/25** | Tie broken by bounded scratch, simpler dense union enumeration, and absence of a per-matching-hit heap protocol. |

The tie is intentional: the heap package has a deeper top-level interface, while the bitmap package has a smaller and more favorable dense-overlap data shape. The preferred base can absorb the interface improvement without inheriting heap maintenance or doc buffers. No measured workload density is available to make a universal performance claim.

## What the current source establishes

The actual multi-essential loop does a suffix minimum scan at or_maxscore.rs:267, a matching/scoring scan at :283, and a matching/advance scan at :325. This is the exact repeated work both candidates attack. Row clearing (:276), reverse optional probing (:294), incoming-order publication (:312), and the region cleanup/prefix pass (:207) remain. Grounding's prefix samples are region work, so attributing those samples to a candidate replay win would be incorrect.

ClauseState::fill_scored_batch (:32) updates its mirror from actual scorer.doc after the owned call. TermScorer::fill_scored_batch (term_scorer.rs:51) does not retain an OR ordinal and evaluates returned postings through score_batch. SegmentPostings::read_postings_batch (segment_postings.rs:29) reconciles shallow selection, copies only docs below up_to, and leaves the cursor at first unread. Its full-block branch advances the decoder before replay. That last fact makes pending-hit completion the most important implementation invariant.

Bm25Weight::score_batch (bm25.rs:575) preserves each native/legacy scalar scoring expression. Both designs reuse the same literal leaf foundation. They must not publish their pruning-known sum or read scorer.score after prefetching; scorer state then refers to first unread, not the retained hit.

The independent trace helper (or_maxscore.rs:353–400) records every callback and raw score bits, enforces strictly ascending docs, and compares to BufferedUnionScorer using SumCombiner. It is a suitable oracle. New tests must demonstrate multi-essential dispatch; the existing single-driver BATCH_LENGTHS witness does not prove a new multi-essential branch ran.

## Grafts into the bitmap base

1. **Complete refill ownership from heap.** Prefer one private operation that hides the entire minimum/end/fill/scatter phase, with an entry assertion that no rows are pending and completion assertions that all essential authoritative mirrors are at or beyond window_end. This is the complete gather_window operation already allowed by the bitmap package, not a new cross-module abstraction or a wrapper around each bit operation. Keep optional/replay arithmetic shared with the current loop. Its contract must explicitly forbid refilling or changing region partition while rows remain.
2. **Prefetched-but-exhausted adversary from heap.** After filling essentials, make their actual mirrors all >=hi or TERMINATED while several winning rows remain. Then prove every retained row is replayed in order. This directly catches accidentally reusing the scalar cursor-minimum termination condition. Also retain heap's region role-swap adversary: an old essential becomes optional and a deferred optional becomes essential only after complete drain, with dirty storage lengths/offsets changing.
3. **Explicit two-domain naming from heap.** State in comments and observer receipts that ClauseState.doc is authoritative first unread, whereas the bitmap's next set offset is a replay candidate. Do not mirror a buffered doc into ClauseState merely to reuse scalar control flow.

These grafts improve interface depth and verification without changing the bitmap mechanism. Keep its zero-on-first-touch rows, no offset list, 128-doc size, and preference-ordered gather.

## Rejections and blockers

Reject grafting per-term doc buffers, the heap itself, sparse/dense selectors, callback-driven repartition, adaptive window sizes, early all-row pruning, optional batching, or modified global-density admission into the first bitmap attempt. The first two erase the chosen compact shape; the rest add mechanisms whose correctness or causal effects are not isolated. Repartition is particularly dangerous: a newly optional clause's real cursor may already have consumed pending essential hits.

**No fatal correctness blocker was found in either written contract.** Both are feasible against the current owned batch API. Acceptance is blocked until root implements and independently verifies the new protocol and measures it against an explicitly frozen accepted base. The scores above do not establish that the designs compile, pass, or improve performance.

Implementation hazards to make categorical:

- Gather only into an empty bitmap. The mutable row borrow protects the current row, not all remaining rows after that borrow ends. A drain-before-refill assertion belongs at the complete gather boundary.
- Iterate essential clauses in fixed preference order; offset order does not establish known-sum order. Incoming ordinal remains a separate index.
- Zero every contribution in a row on its first new-window essential touch, before optional replay can see it. Popped match bits are reusable only after the prior window fully drains.
- Use retained numerical region certificates after essential prefetch. Never recompute hi or local bounds from newly loaded decoder blocks during replay.
- A guarded fill with a mirror below end must either emit hits or reconcile to at least end/TERMINATED. Assert progress rather than permitting a zero-result spin.
- Empty physical gaps must jump to the new minimum essential doc. Bitmap does not inherently enumerate every empty 128-address interval; its sparse disadvantage is repeated clause scans and small fills at occupied windows.

The heap's structural downside is not a correctness blocker: overlapping essentials require heap repair for every matching posting, potentially O(e log e) work per union doc in place of O(e) scatter, plus larger copied buffers. The bitmap's structural downside is also real: sparse or tightly clipped windows may amortize none of its gather/setup cost. Neither downside warrants an unmeasured selector in this unit.

## Target and verification limits

The ruler is **the ratio of arithmetic mean query median durations**, equivalently the ratio of summed query medians on the same group. It is not the mean of per-query ratios. The retained accepted01 fixed-port report has two-term ratios 0.992816 AB / 1.012051 BA, three-plus ratios 2.291402 / 2.228355, and allOR ratios 1.862226 / 1.835791. Grounding's two-term 2.69 figures are the earlier preattempt01 baseline and must not be used for the next candidate's target.

Neither candidate changes the 198 two-term rows, which accepted01 brought to approximate parity. For each balanced pair, allOR success requires `103 * candidate_3plus_mean + 198 * candidate_2_mean < 103 * port_3plus_mean + 198 * port_2_mean`, using that pair's measured median-duration means. Thus a strong three-plus improvement can close the allOR gap; no obsolete count-weighted mean-ratio floor blocks it. The unchanged BA two-term mean being slightly above port requires a correspondingly small margin below parity from three-plus, or a separately measured two-term improvement. Given controls, parity alone is not a durable lead for two-term.

Root reports latest03-versus-parent three-plus group ratios 0.96519 / 0.92983 with 2.64% group control log spread, 1438 library tests and three 1227-query gate sets green. These are parent-provided completed local receipts, not new judge measurements. The fresh03 fixed-port report was still pending when this judgment was written. Its completed values should supersede accepted01's target estimates when available.

Before acceptance, root should run the bitmap-specific raw-bit, cursor, dirty-row, dynamic-threshold, deleted-ID, and scratch/activation fixtures; retain the attempt03 batch foundation checks and all historical policy gates; then run complete balanced timing and self-controls against the frozen accepted03 base if03 is separately accepted. Report local multi-essential delta separately from cumulative progress against accepted22f4a and fixed port. Inspect sparse/short-window regressions and unchanged single-essential/two-term/nonOR controls. A pilot or final Top-K equality alone is insufficient.

Fresh judge verdict: **bitmap base, heap ownership/verification grafts, heap algorithm retained as a separate alternative.** This is a warranted next mechanism trial and does not yet substantiate the user's <1 target.
