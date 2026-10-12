# Required optional clause: source-only grounding

Status: hypothesis only. No source changes, compilation, tests, diagnostic execution, or timing. Grounded on accepted parent 11, not on any unaccepted bitmap/widening candidate.

throughput checkpoint: n/a, read-only investigation

Investigation steps: trace source/configuration/runtime path; preserve the read-only checkpoint; produce an evidence-backed recommendation; apply a plain-writing pass. Completed by source inspection. Runtime verification belongs to root.

## Recommendation

A small conditional mechanism merits a reach diagnostic only if it can cheaply reuse an existing diagnostic build: in an existing **single-essential** local region, lazily certify the **first optional clause already probed by the current reverse-order loop** as required for any competitive document. A miss can skip its posting gap through pending driver batch entries. **Do not add refill-floor or driver-seek machinery:** accepted 11 already limits this route to one driver decoded block suffix per region.

The potential saving is smaller than the port's compacted batch route suggests: driver scoring is already paid, and the next lower prefix test already rejects these individual misses. The gain would be avoiding repeated candidate-array clearing and bound/cursor checks across a gap. This is narrower than copying the port's full required suffix. It adds no reordering, repartitioning, heap, score matrix, metadata range scan, or new cursor API. It does not establish that this optimization is frequent or faster on the benchmark. Essential global bounds may be too loose for the certificate to pass at all.

## What the port actually does

`lucene-rs/src/search/bulk.rs:396-409` only computes `first_required` when there is one essential clause. It extends a required suffix while the sum excluding the next optional clause is below the collector's threshold. `bulk.rs:340-358` then distinguishes required and ordinary optional probes. `src/search/util.rs:112-127` compacts the accumulated candidate array to documents matching the required clause. `apply_optional_clause` merely adds scores on matches.

This is a **local competitive-score requirement**, not a Boolean MUST clause. The port still queries an OR. Candidate documents that miss a required-for-this-window term cannot reach the competitive threshold using the remaining terms.

The port's required-suffix calculation includes direct f32 casts (`bulk.rs:404`). Its arithmetic is not a sufficient proof for our incoming-order sum contract. Our certificate must use the existing original-number-of-leaves conservative bound, and must build the excluded-clause sum independently rather than subtracting from the full sum.

## Accepted kernel ownership and the missing opportunity

Our kernel builds and fixes local bounds and the optional/essential partition per region (`or_maxscore.rs:218-281`). The one-essential route owns a decoded/scored batch (`292-315`). Its scorer cursor has already moved to the **first unread** document; the batch's documents and literal scores independently own pending candidates.

Each candidate currently clears its incoming-order contribution array, loads the exact essential leaf, probes optionals in descending rank, and finally folds the original f32 leaves in incoming order through f64 (`326-369`). The optional probe can reject a single candidate using its exact known contribution plus the remaining prefix (`344-348`). It does not currently use an optional miss to skip future driver candidates.

The frozen rank order is global-max/cost priority, not a descending local-bound order (`or_maxscore.rs:116-127`). A required clause cannot be inferred from rank alone. Even if one rank is required, adjacent ranks need not be required.

## One auditable mechanism

Scope: single-essential regions only. Derive `required_rank = first_essential - 1`, the first rank the existing reverse optional loop would probe. Under this scope and the existing 3..=32-term admission, it is a valid optional rank. Keep the existing multi-essential path unchanged.

1. Keep a region-local optional cache for one conservative `other_bound`. Initialize it as absent. Do no extra clause pass until the existing first optional probe sees a missing document after its ordinary competitiveness check.
2. At that first miss, form a new sequential f64 sum **from zero**, over every current `clause.local_max` except `required_rank`. Do not use `total - selected`, do not change `prefix`, and do not use the number of remaining terms as the error count. Convert through the existing `SmallScoreSumUpperBound::new(original_num_terms)` instance.
3. Cache that immutable region-local bound. At later first-optional misses, compare the cached bound to the current threshold. A rising callback threshold can make a previously false certificate become true. A true certificate remains true within the region because local bounds are fixed and thresholds do not decrease.
4. If `other_bound <= threshold`, any document in this region that misses the selected optional term is noncompetitive. Let `target = min(required_clause.doc, hi)` after the ordinary forward seek to the current candidate.
5. Skip pending batch entries whose document is **strictly less than** `target`, using their sorted `batch.docs` slice. Preserve every surviving entry's index-aligned literal score; do not compact scores or recompute leaves. Restart candidate processing at the first retained entry; bypass the ordinary one-entry increment after this jump so a surviving match is not skipped twice. At a certified miss, `target > current_candidate`, so progress is strict.
6. If `target >= hi`, discard the remaining pending batch and end this region. A required term's document exactly at `hi` belongs to the next region; it must not cause a callback here.
7. Leave the real driver cursor and existing refill floor unchanged. Only the pending `batch_index` advances; `ClauseState::fill_scored_batch` remains the exclusive existing batch owner. Do not add a driver seek, target-floor cache, or new refill path.
8. Discard the cached certificate at region exit. Rebuild it from fresh bounds in the next region. A term optional here can be essential there.

A first implementation should avoid certifying multiple optional ranks. It should also avoid checking the certificate before every candidate that already matches the selected optional clause: the existing top optional probe is the natural location for the miss-triggered check.

### Why this is safe

For any document `d` in `[lo, hi)` missing rank `r`, all actual nonnegative leaves are bounded by the existing local maxima for the other clauses. The independent excluded-clause sum and the original-n conservative helper bound the published incoming-order f64 sum and final f32 cast. Therefore `other_bound <= threshold` implies `score(d) <= threshold`, while callbacks require strict `score > threshold`. No callback is lost.

After `seek(current_candidate)`, an absent required clause's authoritative real `doc` is the first matching document at or beyond the candidate. All driver entries before that document miss it. Skipping them is a consequence of the score certificate, not of a density heuristic.

This proof also covers documents matching only optional clauses: their total is already bounded by the established optional prefix. Skipping their traversal does not add a new assumption.

The current next-lower-rank probe already rejects each such missing candidate: with `r = first_essential - 1`, no higher optional leaf has been added, so `known` is only the exact driver leaf. The independent excluded sum has the same lower-prefix addition order as `prefix[r]`, followed by the driver local maximum. Monotonic nonnegative addition gives `known + prefix[r] <= other_sum`, and the bound helper is monotone. Thus certification implies the existing next-lower test is also at or below the threshold. There is at least one lower optional rank in this single-essential 3..32-term route. The proposed optimization bypasses repeated visits to that cheap rejection; it does not rescue a missing logical prune or avoid later optional scoring that the old route would actually perform.

No document score uses this conservative bound. Survivors must still clear contributions, retain each literal f32 leaf, and perform the same incoming-ordinal f64 fold and single f32 cast. Do not publish the rank-order `known` sum.

### One-block limit and pending batch ownership

`TermScorer::fill_scored_batch` delegates cursor consumption to `SegmentPostings::read_postings_batch` (`term_scorer.rs:51-66`). That owner reads at most one decoded block suffix and leaves the real cursor at its first unread document (`segment_postings.rs:26-68`). A batch can therefore still contain candidates smaller than the driver's real cursor.

- Never set `ClauseState.doc` to a pending batch doc or to `target` without a real owner operation.
- Never seek the driver backward to a retained pending entry. Keep that entry's already-owned score.
- The driver is not globally optional: region setup always shallow-selects it and sets `hi <= selected_last_doc + 1` for a finite full block (`or_maxscore.rs:229-253`). A tail has at most 127 decoded postings. In the one-essential route, one `fill_scored_batch(lo, hi)` consumes every driver posting below this region's end. The existing next refill is empty. A fixture claiming multiple driver decoded blocks or 128-entry refills inside this unchanged region is not realizable.
- Consequently a certified gap inside this parent only skips **already scored pending batch entries**. It cannot save driver BM25 calls or postings decoding within the region. The batch owner's real cursor is already on the first document at or beyond `hi` (or TERMINATED); leave that mirror unchanged while skipping pending entries.
- Seeking an optional term beyond `hi` is already possible in the current probe loop. Its retained real cursor then justifies a zero local bound in subsequent region cleanup where applicable. Keep its mirror coherent through `ClauseState.seek`.

Deletion handling stays in the unchanged TopDocs callback. `collector/sort_key/sort_by_score.rs:44-53` returns the previous threshold for a deleted doc. Skips must not synthesize callbacks or raise the threshold from deleted scores. The certificate only uses the real current threshold.

## Why this is distinct from rejected attempts

The decision ledger rejects 02's local priority sort, 07's scalar same-region repartition, 09's posting heap/batches, and 10's unique-lead spans. This mechanism preserves rank order and partition, uses the already existing one-essential batch, introduces no heap or multi-term storage, and derives a gap only from a required-for-competitiveness proof. It does not select a new lead based on another essential's cursor distance.

It also does not widen regions (12/13), change density scanning (15), gather a bitmap/matrix (08), or change publication masks (14). A future accepted candidate may alter the replay route; if so, re-ground this proposal against that actual parent rather than silently grafting the parent-11 ownership assumptions.

## Cost and risks

| Point | Benefit or cost |
|---|---|
| Lazy independent excluded-clause pass | O(n), at most once in a qualifying region, only after an existing first-optional miss |
| Repeated certificate use | One cached-bound comparison at those misses; no bound recomputation after threshold growth |
| Skipping a pending batch gap | Avoids repeated contribution clearing, exact essential-leaf loads and bound/doc checks; driver BM25 was already paid, and missing candidates already fail the next lower prefix test |
| Real driver cursor | Unchanged; no decoder/BM25/refill saving is available within accepted 11's one-block region |
| Existing large optional-prefix regions | May offer amortization; static source does not establish frequency |
| Loose essential global bound | Can prevent all certification, leaving only added work |
| Dense required clause | Few missing candidates or short gaps can make setup overhead exceed savings |
| First optional only | Intentionally misses opportunities at lower ranks; avoids O(n²) certification/reordering complexity |
| Storage | One local optional bound cache and control state; no new heap allocation or per-region vector. Exact struct/frame size requires compiled inspection |
| Index/API | No format, persisted metadata, public API, or collector semantics change |

## Independent fixture shapes before acceptance

Use the existing exhaustive buffered-union callback trace and literal score-bit oracle, not the proposed certificate to generate expected output. Path evidence must witness the actual route as well as equal final results. Suggested families:

1. **Achievable required optional, sparse gaps:** actual serialized postings/BM25 bounds with a one-essential region, first optional bound large enough that missing it cannot compete, several driver entries before its next real doc, and at least two surviving matches. Derive the threshold independently between actual missing-term upper scores and actual all-term scores; assert rank and certificate witnesses rather than assuming boosts impose them.
2. **Already filled 128-entry batch:** a gap begins inside an owned full batch; retain a match later in that same batch even though the real driver cursor has advanced past it. Verify every callback score bit and strict increasing doc order.
3. **Partial owned suffix:** a gap discards the last pending entries in a shortened batch whose real first-unread driver document is exactly `hi` or beyond it. Verify mirror coherence and unchanged next-region callbacks. Do not invent an in-region multiple-refill witness: the accepted kernel's driver cap makes it unreachable.
4. **Exclusive region end:** next required doc at `hi`, beyond `hi`, and TERMINATED. End only this region; verify later-region callbacks when the previously required term's local requirement disappears or its partition changes.
5. **Threshold grows:** the cached excluded bound initially fails; a real later callback raises the threshold so it passes. Exact equality must certify under our strict callback rule. Test one representable threshold below the bound to ensure a candidate is not incorrectly skipped.
6. **Rounding/cancellation trap:** incoming-rank permutations and adversarial nonnegative f32 leaves near a final f32 rounding boundary. Compare the independent generic original-n bound and literal incoming-order score. A full-total-minus-selected shortcut should not be used by either implementation or oracle.
7. **No certificate:** high essential global bound, short required-clause gaps, and selected optional local bound zero. Verify unchanged callback trace and absence of skip witnesses. Include 3 and 32 clauses with rank order different from incoming order.
8. **Ownership and deletions:** partial batch, last-document tail, stale shallow-selected block needing actual reconciliation, native/custom similarity bounds where already eligible, a deleted high-score match that must not raise threshold, and a next-region transition where the selected optional becomes essential.
9. **Existing exceptional fallback:** signed/unsafe/nonfinite weights and NaN threshold remain on the unchanged canonical path. Positive underflow-to-zero leaves retain literal score bits.

The abstract local-bound example `[0.2, 0.8, 0.3]`, threshold `1.1`, with the final `0.3` clause essential, illustrates the requirement: omitting the `0.8` optional clause leaves only `0.5`. It is not a claim that these exact bounds arise from a serialized fixture or this benchmark. Actual fixture rank/bounds must be observed, independently checked, and retained.

## Root diagnostic and performance gate

Before spending full performance validation, root should establish reach on the frozen 103 long OR cases using diagnostics run separately from latency. Count region distribution, single-essential regions, first-optional missing probes, independent excluded-bound certificate passes, pending entries safely skipped, gaps reaching the region end, batch lengths, selected driver block/tail caps, and original incoming callback/score equivalence. Retain zero-opportunity queries and denominator counts. Report local-vs-global bound provenance if available without altering scoring.

If the diagnostic shows only one or two skipped entries per successful certificate, stop: an O(n) independent pass plus added branches is unlikely to beat the existing cheap next-prefix rejection. This is a screening judgment, not a measured speed conclusion. Many skipped entries are necessary to justify an isolated attempt, but not sufficient to prove a win.

Those counts establish opportunity only. They do not prove cycles saved, explain measured losses, or support a speed percentage. Compare an isolated implementation with its actual accepted parent using the existing three exact profiles and frozen AB/BA latency/control harness; preserve all query losses. No query-text/ID selection, new benchmark metric, or percentile substitution.

## Source identity and references

All paths below are absolute local files; cited line ranges refer to the inspected files.

- Accepted Tantivy commit: `43f06833a325d2a18c03bd7e4fa65f2f27944292`.
- `/home/pronsh/Coding/playground/search/tantivy-pr2937/src/query/boolean_query/or_maxscore.rs`; SHA256 `21153a76a7ae3091ac6400cc438f408b7ac367d9a534b1d290d30af5c3fb5901`.
- `/home/pronsh/Coding/playground/search/tantivy-pr2937/src/query/term_query/term_scorer.rs:13-66,249-269` (batch owns leaves; owner consumption; forward-only real seeks).
- `/home/pronsh/Coding/playground/search/tantivy-pr2937/src/postings/segment_postings.rs:26-68` (batch reconciles shallow selection and publishes first unread).
- `/home/pronsh/Coding/playground/search/tantivy-pr2937/src/query/score_combiner.rs:86-158` (generic/small original-n conservative sum).
- `/home/pronsh/Coding/playground/search/tantivy-pr2937/src/query/boolean_query/boolean_weight.rs:677-708` (pure 3..32-term f64-summing OR route).
- `/home/pronsh/Coding/playground/search/tantivy-pr2937/src/collector/sort_key/sort_by_score.rs:28-60` (exact top-k threshold and deleted-doc callback behavior).
- Fixed port commit: `e5d1f81bff42c87886b12770f3c79648bb1963ed`.
- `/home/pronsh/Coding/playground/search/lucene-rs-idf-fix-oct06/src/search/bulk.rs:340-411`; SHA256 `051f12a2f54d15f47f2eed50d2bf104c291b487f3675ff56a8e66696a7735c1a`.
- `/home/pronsh/Coding/playground/search/lucene-rs-idf-fix-oct06/src/search/util.rs:89-141`; SHA256 `c46f4da44826cebf3de57ab99d00f82d26caeebf5e2bc895e5e6083680dcdaba`.
- `/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07/decision.tsv` inspected through rejected 15.

No performance, frequency, compactness advantage, public API parity, or correctness test result is claimed by this source-only report.
