# Attempt 03: isolated single-essential exact block batching

Status: source implementation ready for root review and runtime gates; not accepted or measured. Worktree `search/tantivy-or-batch-oct07`, branch `perf/or-batch-oct07`, base `0e9329dfe782187a8feda0024102e4aabf517c28`. This checkout already existed clean at the requested base; I used it rather than duplicating the root-created worktree.

Only five existing source files changed. No window bitmap, local/global ordering change, density policy change, region widening, threshold repartition or two-term reroute. The extra fifth module is the crate-private ScoredTermBatch re-export in term_query/mod.rs; the four production implementation modules match the design.

## Actual flow/API

`or_maxscore` retains the same exact admission/fallback and region preparation. Only `num_terms - first_essential == 1` selects the batch branch. A single query-owned ScoredTermBatch lives outside the region loop. Per region, batch_len/index track buffered replay; the driver's mirrored doc remains the authoritative first unread decoded doc. The existing scalar optional pruning, incoming leaf vector, incoming-order f64 sum/one f32 cast and per-candidate callback threshold logic are shared by both branches.

The deep cursor operation is `SegmentPostings::read_postings_batch(floor, up_to, &mut [DocId;128], &mut [u32;128]) -> usize`. It reconciles an unloaded shallow selection directly through BlockSegmentPostings::seek at max(floor, decoded doc), copies at most one real decoded block suffix below exclusive up_to, then owns cur/advance. Full-block end advances/loads the next block; partial tail end points at its existing TERMINATED padding. NoFreq/SkipFreq outputs are explicitly filled with1 to match scalar frequency behavior without assuming freqs() slice length.

`Bm25Weight::score_batch(&FieldNormReader, docs, freqs, norm_scratch, scores)` first gathers the exact cached normalization component for actual doc IDs, then matches the scoring mode once outside the literal f32 arithmetic loop. Native remains `weight - weight / (1.0 + frequency * norm)`; legacy remains `weight * (frequency / (frequency + norm))`. No alternate algebra, approximation, clamp, reassociation or incoming score reordering.

`TermScorer::fill_scored_batch(floor, up_to, &mut ScoredTermBatch)` hides the owned postings advance and exact leaf calculation together. ClauseState's private fill method owns its mirror update once from actual scorer.doc(), with existing debug coherence checks. There is no ordinal inside the term batch and no mutable cur exposure.

## Actual bounded scratch

ScoredTermBatch contains only four arrays: docs512B + freqs512B + f32 leaves512B + cached normalization512B =2,048B, with no scalar/header fields, all4-byte alignment. Its Rust layout is consequently2,048B; the meaningful OR trace fixture prints actual sizeof as runtime receipt. The existing maximum array payload1,160B remains, total3,208B. Three existing Vec allocations/72B headers remain; no new allocation, pool, index bytes or per-region allocation. The batch is stack scratch, so frame/initialization overhead and generated code size remain performance risks.

## Meaningful tests added (not run here)

1. `postings::segment_postings::tests::batch_cursor_matches_literal_postings_across_blocks_and_tails`: literal independent docs/freqs, lengths1/127/128/129/255/256/257, dense/gapped spacing1/3/97, Basic and ReadFreq, clipped ends, first-unread doc/freq after every batch, mixed scalar advance, final termination.
2. `...::batch_then_scalar_reads_preserve_positions_and_skipfreq`: actual serialized300-doc positioned index, WithFreqsAndPositions and Basic/SkipFreq reads; position reads before and after clipping/fullblock/scalar seek/advance; cloned doc128 deliberately kept stale while shallow-selecting tail260, then batch260..270 reconciles it and verifies current positions.
3. `query::bm25::tests::batch_leaves_match_literal_scalar_bits_for_all_norm_ids`: all256 norm IDs verified roundtrip, frequencies1/7/128/16_777_217/u32::MAX, default and two configured BM25 profiles, native/legacy modes, boosts0/1/3.25/-2; compare raw leaf bits to scalar score.
4. `query::boolean_query::or_maxscore::tests::single_driver_batches_replay_every_dynamic_threshold_exactly`: independent BufferedUnionScorer complete callback raw-bit traces at constant and evolving Top1/Top10 thresholds; literal Top1 callback docs[0,132,960]; cfg(test) observer proves a128-hit batch actually executed. Collector-owned deletion callback model rejects0/132 without threshold change, oracle trace exact and literal callback docs[0,4,132,960]. This fixture also prints batch layout receipt.

The observer is thread-local and cfg(test), enabled only by that fixture, and taken before subsequent checks. It has no release symbols/counters. Existing10 raw-bit fixtures remain unchanged and continue exercising fallback/stale/tail/midpoint paths.

## Verification performed and unresolved risks

Only rustfmt (edition2021, skip_children=true on the five changed files), full actual diff review and git diff --check were run. No cargo build/test/clippy, profiling, timings or commit. Root owns these jobs and exact published-profile gates.

Unresolved until runtime gates: compilation and fixture assertions; all default/two configured frozenquery raw bits; optimized native/legacy code generation and whether the arithmetic loop vectorizes; stack initialization/code-size costs; how frequently the exact single-essential suffix appears. Region prefix construction and multi-essential/optional work are untouched, so this is not evidence of a2.3x total win. Tiny tail/clipped batches can lose to copying/setup; unchanged two-term path cannot improve the allOR ruler in this unit. A zero-length batch must mean no remaining doc<hi and must not hide buffered rows; the source resolves all buffered rows before refilling/returning.

Root should first inspect the whole diff, then run the four new fixture families/current OR fixtures and all required gates before any pilot. Measure against the frozen current baseline, preserve this standalone source/bin if rejected, and evaluate actual port only after the same correctness/controller/input guards.

## Frozen source SHA256 after formatting

- `src/postings/segment_postings.rs`: `dc884fe8227d9ce0f79e5b8a4c7118b32654b43174078f6ac5e6872c981740a1`
- `src/query/bm25.rs`: `64f8fc7922a31328c2410a4c7f8fea4ff599b5e31493829a0898a0ef660cd67c`
- `src/query/boolean_query/or_maxscore.rs`: `b4d093a2e728fa444efa07cb1a7cdd033aefaf7f5a8d7aefe5fe7d8bf018b881`
- `src/query/term_query/term_scorer.rs`: `dd1e18b5b1d273cf2c09959ea207997a1e93f854b0fc459fb0a1e76129b87277`
- `src/query/term_query/mod.rs`: `80bb5e8aae34e19e5272cea9ceff1550b419d9a1b0798ca34dd8b468d8e9de4c`
