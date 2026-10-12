# First-optional required gap: minimal parent-11 production proposal

Status: source-only design, no production candidate yet. No compilation, tests,
benchmark, profile, disassembly, compression, browser, or engine execution was
performed by this author. Root owns runtime verification. Parent is accepted11
`43f06833a325d2a18c03bd7e4fa65f2f27944292`; only
`src/query/boolean_query/or_maxscore.rs` would change.

The ground-required-optionals.md and required-reach-report.md were read, along
with actual accepted production kernel, independent callback fixtures, score
bound implementation, and actual diagnostic miss/gap observer. Decision.tsv
was read through rejected18. This does not repeat region expansion/partition,
mask, heap, priority-sort, density-cache, or two-head attempts.

## Data and control shape

Keep `ClauseState`, `ScoredTermBatch`, all arrays, signatures, admission, bounds,
rank, partition, contribution arithmetic, and callback unchanged. The only new
release state is `let mut required_other_bound: Option<Score> = None;` inside an
existing local region, immediately after `single_essential`. Logical payload is
one f32 plus an Option discriminant (normally eight bytes); compiled stack/frame
and actual binary sizes remain root's measurements, not source claims.

Label the existing candidate loop `'candidates`. After the existing first
optional competitiveness test and ordinary seek, in its miss arm, execute the
new path only when `single_essential && rank + 1 == first_essential`. Copy its
real `clause.doc` before releasing the mutable clause borrow. The matching arm
retains exactly its existing scoring and `known` update.

Build the cached bound only on this actual first-optional miss. Start an
independent f64 accumulator at zero, visit clauses in fixed rank order, skip
exactly the current optional rank, add every other immutable `local_max`, then
call the existing original-num-terms `upper_bound.score`. No subtraction, f32
sum, shortened error count, prefix mutation, reassociation, or full-sum cache is
introduced. `get_or_insert_with` makes this pass at most once per region.

Compare cached bound against the live current threshold at every qualifying
miss. Do not cache a yes/no decision: callback threshold growth can turn an old
failure into a certificate. If `other_bound <= threshold`, let
`target = optional_real_doc.min(hi)`. Search only the owned pending slice:

```rust
let skipped = batch.docs[batch_index..batch_len]
    .partition_point(|&pending| pending < target);
debug_assert!(skipped > 0);
batch_index += skipped;
continue 'candidates;
```

The ordinary final `batch_index += 1` is bypassed by the labelled continue. The
first retained matching entry keeps its original index-aligned literal score;
contributions are cleared by the unchanged next iteration. If every pending
entry is skipped, the unchanged next iteration performs the ordinary empty
refill and region exit. Avoid a new direct region-break fast path initially:
keeping the existing owner refill/exit logic minimizes semantic surface and
preserves existing empty batch observations. Parent11 has at most one nonempty
owned driver block suffix per region, so that refill cannot expose further
nonempty candidates in the current region. This is verified by root's reach
observer, not assumed as a decoding opportunity.

Only `batch_index` changes. Never advance or seek the driver, rewrite its doc
mirror, compact arrays, regenerate scores, or fabricate callbacks. Optional
real seek remains the existing `ClauseState.seek`. Cache is discarded on the
unchanged region exit. Multi-essential paths and all ordinary misses whose
certificate fails run the exact parent path.

## Proof obligations

1. Numeric admission yields nonnegative finite f32 leaves, each bounded by its
   immutable local maximum. Excluding one missing optional rank gives an
   independent sequential f64 upper sum from zero. The original-n helper bounds
   differing addition order and the final f32 publication, so `bound <= live
   threshold` certifies no strict callback for any missing document.
2. The first optional rank in a single-essential 3..32-term region is
   `num_terms - 2 >= 1`. No earlier optional contribution was added at the miss.
   Its real post-seek doc is strictly larger than the current candidate; all
   pending docs strictly below it miss that term. `min(doc, hi)` preserves the
   exclusive end. This makes progress strict and prevents skipping the matching
   target itself.
3. Parent's next-lower prefix rejection already rejects each skipped miss:
   exact driver score plus the lower prefix is no greater than the excluded
   local-bound sum, in the same lower-prefix-then-driver order. Therefore parent
   would neither callback nor mutate lower optional cursors on skipped entries.
   The proposed shortcut consequently preserves all live real cursor states,
   not merely final TopDocs.
4. Driver cursor means first unread after batch ownership. It may exceed the
   retained pending entry. Skipping owned indices requires no cursor operation;
   surviving score bits stay original. Region boundaries, TERMINATED and tail
   handling use the unchanged owner/exit path.
5. Cache depends only on immutable local bounds and selected rank. Threshold
   comparison is live; it uses strict callback semantics, including equality.
   Deleted callback winners cannot change threshold because unchanged collector
   code retains it. New code makes no callback and cannot raise it.
6. Survivors still perform the unchanged original incoming-ordinal f64 fold and
   one final f32 cast. No pruning sum is published. Generic numeric fallback and
   optional-to-essential transitions in later regions remain unchanged.

## Independent additive verification before acceptance

Existing 13 OR tests remain untouched. Add a test-only observer with compact
records of region end, original n/rank, trigger/target, old/new batch index,
real driver doc, cache bound bits and live threshold bits. This is path evidence
only, excluded from release. Callback expected values must still come from the
existing exhaustive BufferedUnionScorer and canonical SumCombiner, never from
this certificate. Add a test-only independent generic `ScoreSumUpperBound`
from-zero check of each built excluded sum and endpoint invariants.

Serialized postings fixtures should cover n3/n32 with incoming order reversed
relative to rank, full128 batch and partial suffix, retained matches while real
driver cursor is ahead, target exactly at exclusive hi/beyond hi/TERMINATED,
and next-region term becoming essential. Use actual BM25 maxima to derive
thresholds, retain corpus TFs/positions, and demand skip witnesses only from
achievable geometry. Top1/10/fixed thresholds, equality/next-down/next-up and
monotone schedules must compare complete callback doc/score-bit traces. A cache
that initially fails and passes after a real callback must be witnessed.
Include deleted early high-score callbacks that deliberately withhold a raise,
no-certificate and zero-bound cases, native and custom safe bounds, stale
shallow reconciliation, and final document tails. Every skip record must show
strict progress, sorted pending slice, identical real driver before/after, and
no lower optional mutation/callback across the skipped interval.

Do not require a repeated nonempty driver refill or driver decoding/BM25 saving:
those paths are absent from parent11. The trigger already pays contribution
clear/driver leaf/first optional seek. Only later candidate clearing, leaf load,
and cheap rejection/probe checks can be saved. Existing reach histograms decide
whether that opportunity warrants implementation. Long gaps are necessary for
amortization, not a measured win; dense matches or no certificate add overhead.
No benchmark IDs/text/arity-specific selection is allowed.

Root should retain all103 reach queries/counter identities and exact bits. Only
if gaps justify an isolated attempt20 should implementation proceed. After root
review, run the existing three exact profiles, independent new callback/cursor
fixtures, and unchanged parent AB/BA/control timing protocol; retain all query
losses and index bytes. No speed, memory, compactness advantage, full feature
parity, or correctness runtime result is currently claimed.
