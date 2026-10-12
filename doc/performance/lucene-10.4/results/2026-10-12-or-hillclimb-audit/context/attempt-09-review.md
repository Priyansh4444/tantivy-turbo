# Attempt 09 independent source review

Verdict: no remaining concrete production correctness blocker found in the complete source diff. Runtime acceptance and performance remain root-owned and pending. One independent oracle defect was identified by root's targeted execution and corrected narrowly during this review; it was not a heap execution defect.

Throughput checkpoint: n/a, source review and one authorized test-oracle correction.

## Reviewed artifact and scope

Reviewed the complete isolated `search/tantivy-or-posting-heap-oct07` diff against accepted parent `a674aaf2f6187f3010a135dc25fdacf9cab27e53`, the complete resulting source, `attempt-09-implementation.md`, `design-posting-heap-v2.md`, and `design-sparse-merge-synthesis.md`. Also traced the existing `TermScorer.fill_scored_batch`, `SegmentPostings.read_postings_batch`, scalar and batch BM25 expressions, and admitted `SmallScoreSumUpperBound` domain.

Original source SHA-256: `4e97064413426debda620071e274215f64e1e4bddfdde5624e5a3dd86d56cad5`.

Corrected stable source SHA-256: `41065df1d96bb7c585e0593c68b0df09ee717b97366efc253bd036f062725f12`.

Only tracked source difference from the parent remains `src/query/boolean_query/or_maxscore.rs`. No compiler, tests, profiling, benchmark, browser, compression, child delegation or commit was run by this reviewer. `git diff --check` passed after the correction. The implementation receipt still names the original source SHA; this report binds the later oracle-only correction explicitly.

## Exact score and bound proof

Eligibility, frozen preference sorting, global/density policy, shallow preparation, physical `hi`, local fallback bounds, inclusive prefix construction and essential-floor reconciliation are text-identical to accepted06 apart from declaring the optional heap scratch. The specialized helper remains constructed with original `num_terms`; neither its arithmetic nor its proof domain changes. That domain includes valid indexed positive matching TF, not arbitrary corrupt or synthetic zero-TF inputs. The inherited comment saying numeric admission alone certifies leaves is shorthand for this existing structural invariant, not a new validation promise.

`Head.precedes` orders `(doc, slot)`. A replacement head has strictly greater doc because each owner's postings are strictly increasing. Hence every equal-doc owner is popped in ascending fixed suffix slot order even when a popped owner is synchronously refilled. `known += f64::from(leaf)` therefore has exactly the former preference-scan accumulation order, including duplicate query clauses. The essential row scatters each actual f32 leaf by original ordinal. `row.fill(0.0)` removes prior essential and optional contributions. Publication continues the original incoming-order f64 iterator sum followed by one f32 cast; it does not publish pruning-known.

The shared optional/completion body is text-identical to accepted06 after substituting the disjoint `optionals[rank]` view for `clauses[rank]`. It still checks `upper_bound(known + prefix[rank+1])` against the current callback threshold before each reverse optional probe, scores only matching actual cursors, publishes only strict `score > threshold`, and immediately stores the callback's returned threshold. Heap replay owns no cached threshold and never repartitions a pending suffix.

## Pending ownership, boundaries and heap repair

Each slot owns a copied docs/leaves prefix plus next/len counters. `ClauseState.doc` continues to mirror the authoritative first unread scorer cursor after every batch fill. The scratch buffer is not substituted for that cursor. Copying a buffer before the shared scratch is reused protects retained leaves from subsequent fills and optional scoring.

The existing batch owner reconciles unloaded shallow selections and stale floors, consumes at most one decoded block suffix below `hi`, and leaves the cursor on the first unread actual doc. Region construction and refill use `floor=max(lo, previous_doc)`, never seek backwards, and do not widen `hi`. Selected-block advancement during batching cannot invalidate the local numerical certificate held for already certified docs; optionals are disjoint scorer ordinals and retain their scalar protocol.

Bottom-up heap construction is standard. Replacement/removal affects the root only, and sift-down restores the lexicographic min-heap. Each owner is refilled only after its retained suffix drains. A refill is repaired into the heap before `next_candidate` returns; an absent owner head thus means no unread posting below `hi`. Candidates are selected from the heap until empty, independently of whether all authoritative cursors already exceed `hi` or are TERMINATED. No early exit uses authoritative minima.

The sole caller exhausts heap replay before `drop(replay)` and before changing `lo`, local certificates, roles or suffix ranks. Drop only asserts drainage in debug builds and skips that assertion during panic unwinding; it neither mutates cursors nor performs hidden cleanup. Buffer slots outside a shrunken suffix remain drained from their prior use, and construction resets counters before slot reuse. The exhaustive `EssentialReplay` variant keeps the accepted single-essential batch branch mutually exclusive with the scoped heap metadata borrow. Its original refill, row assignment, index retirement, optional completion and threshold semantics remain equivalent; new enum/frame placement may affect speed and must be measured.

## Fixtures and execution evidence

Nine new fixture families inspect actual heap events in addition to complete canonical callback traces: n3/n32 overlap/disjoint/tails; pending actual exhaustion; exact scalar leaf bits and preference-known bits; dirty row and grow/shrink role reuse; large sparse gaps; direct 127/128/129 boundary/refill ties and retained future heads; native/legacy valid positive-TF threshold neighbors; callback-owned simulated deletion rejection; and a real indexed delete bitset plus public BooleanQuery TopDocs.

The direct boundary fixture deliberately supplies a valid global-max certificate to the private owner, allowing a refill across a 128-posting edge that today's tighter production certificate often clips. It proves owner behavior, not the frequency or profitability of that behavior in production. The midpoint witness separately asserts two substantial essential owners at doc0 under next_down/equal/next_up thresholds, so it cannot pass merely by routing to the accepted single branch. The actual-index fixture requires max_doc160/live158, observes deleted physical candidates without raising their threshold, requires pending heads after actual cursors exceed the interval, and checks the TopDocs winner's physical doc127 and raw score.

Root reported and retained the initial 20-test execution with 17 passing and three failures. `tests-targeted/stderr.log` confirms all three failed `TermScorer.seek`'s forward-only assertion. The candidate observer cloned a term whose initial doc could already be greater than another term's merged candidate and unconditionally sought that lower candidate. This affected disjoint, sparse and runner-boundary observations.

Per root's explicit follow-up, changed only the candidate oracle loop:

```rust
if scorer.doc() < candidate.doc {
    scorer.seek(candidate.doc);
}
if scorer.doc() == candidate.doc {
    // Existing expected leaf/order calculation and advance.
}
```

An initially ahead term now contributes zero and retains its initial next head, as the independent posting oracle should. Reversing only this exact guard reconstructs the original source SHA above; the complete production prefix before `mod tests` is byte-identical. I read back the actual corrected loop. Corrected runtime execution remains root-owned; this review does not claim that it passes.

## Remaining limits

No source correctness blocker remains, but successful targeted fixtures, frozen default/configured exact gates, and prescribed acceptance checks are still required. Source review cannot establish generated-code equivalence or latency. The planned n32 array/record payload is 37,000B, and root's failed initial targeted run already printed that host layout; this excludes allocation overhead, headers, frames, scorer storage and RSS. One lazy allocation, copied 8B per posting, O(log e) repair per matched owner, dense overlap and short 1–2-hit regions remain explicit possible losses. No index-format, RSS, universal speed, Lucene API parity or acceptance claim is made.
