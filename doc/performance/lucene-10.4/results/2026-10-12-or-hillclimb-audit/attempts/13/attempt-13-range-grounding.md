# Attempt13 grounding: clone-only metadata range bounds

Design-only addendum to `design-metadata-region-widen.md` and `design-region-synthesis.md`. Actual accepted parent11 is `43f06833a325d2a18c03bd7e4fa65f2f27944292`, in `/home/pronsh/Coding/playground/search/tantivy-or-hillclimb-oct07`. Cited source bytes are those of that commit; other inspected files were byte-checked against the worktree. Root began grafting attempt12 into the OR file during grounding, so its receipt uses the immutable accepted11 Git object, reread through git show, rather than the later dirty OR source. Attempt12 remains a separate experiment; root must decide from its runtime evidence whether to implement this alternative. This file authorizes no implementation and predicts no speed gain.

throughput checkpoint: n/a, read-only investigation

I read the complete original B design, synthesis, and `ground-region-caps.md`, then the actual scorer, skip cursor, bound policies/envelope, serializer, posting/position ownership, and existing ownership fixtures. Only this addendum was written. No source edits, build, test, worker, timing, profile, compression, browser, child agent, or commit was performed.

## Shape retained and corrections to original B

Keep the same endpoint rule and preparation as A: save the selected exclusive endpoints in a **cold `[DocId; 32]` array**, preserve pre-cleanup `ClauseState.local_max`, derive the old tight partition, and propose exactly once `final_hi = min(selected_end[old_first_essential..], max_doc)`. Keep the existing hot ClauseState fields unchanged. Its accepted layout is 24B on the benchmark host; this addition contributes 128B logical fixed scratch rather than the original B hot field that could expand every record. The actual compiled frame/layout still requires root measurement.

The only distinct B mechanism replaces A's crossed-clause query-global certificate with:

```text
if clause.doc >= final_hi: final bound = 0
else if rank < globally_optional: final bound = global_max
else if selected_end[rank] >= final_hi: final bound = preserved raw local_max
else: final bound = max(preserved raw local_max,
                        scorer.range_max_score(lo, final_hi))
```

Rebuild the entire final prefix and partition before reconciling drivers or creating retained leaves. Keep accepted11's certified global prefix/density logic and original-n numeric helper. Keep the final endpoint and partition fixed throughout replay. An initially all-optional tight certificate still skips its original endpoint without proposing widening. No new endpoint loop, width selector, decoder clone, buffer, pending repartition, cache, format, hierarchy, or allocation for range results belongs here.

Original B's fixture phrase “single accepted batching across 128 refill boundaries inside one expanded certificate” is not generally realizable under this endpoint rule. Any nonempty old essential suffix contains the last rank. A final sole essential is that last rank, so it was also old essential. The proposed endpoint is at most its selected block's exclusive end. Therefore this sole driver cannot span its own multiple full blocks within one widened certificate. Verify a retained full or partial suffix, the exact exclusive-end head, and refills **across later regions**. To exercise multiple real full-block crossings *inside* a widened certificate, use a crossed old optional promoted to an essential **scalar multi** driver. A helper-level range fixture separately scans multiple full blocks regardless of the caller's eventual driver count. Do not claim an impossible single-driver witness.

## Existing API mapping and minimum new surface

One crate-private scorer capability is sufficient:

```rust
// term_scorer.rs, no public Scorer/DocSet trait change
pub(crate) fn range_max_score(&mut self, floor: DocId, up_to: DocId) -> Score;
```

It covers still-relevant unread positive-TF leaves in `[floor, up_to)`, under the existing admitted nonnegative score domain, and is called only at drained query boundaries. The optional numerical chunk can remain private within this impl:

```rust
fn trusted_metadata_max(&mut self, reader: &SkipReader) -> Option<Score>;
```

Do not expose an independently supplied weight, fieldnorm reader, selection tag, envelope, or cursor. `TermScorer` already owns `postings`, immutable `similarity_weight`, and the validated `TermBlockBounds` enum (`term_scorer.rs:33–47,104–124`). Its `postings.block_cursor.skip_reader()` supplies a borrowed metadata reader (`block_segment_postings.rs:521–523`). `SkipReader: Clone` copies its independent current block, endpoints, counters, offsets, and sliced OwnedBytes owner (`skip.rs:127–139`). OwnedBytes clones share Arc-backed immutable file bytes (`ownedbytes/src/lib.rs:11–15`), so this does not copy postings or allocate a decoder.

The existing reader calls are enough: `remaining_docs()`, `last_doc_in_block()`, crate-visible `last_doc_in_previous_block`, `block_info()`, `selected_input_pair()`, `seek(start)`, and `advance()` (`skip.rs:175–196,276–294,388–445`). No SkipReader API/parser change is necessary. The real cursor's `seek_block`, `advance_skip_only`, `load_block`, `seek`, and its numeric cache are never used by lookahead. `seek_block` changes real metadata and unloaded status while leaving decoded doc state stale (`block_segment_postings.rs:415–420`); it is not a read-only lookahead primitive.

`TermScorer::block_max_score()` cannot be called on future cloned metadata: it reads the real cursor, can use its one-block cache, and may reduce loaded tails. The public weight-taking `BlockSegmentPostings::block_max_score()` intentionally returns global for full blocks (`:165–177`); it has no trusted selection provenance. The range chunk therefore evaluates the borrowed cloned reader directly while retaining policy ownership inside TermScorer. Reuse the existing arithmetic functions; do not move the validated enum or numeric policy into a generic cursor merely to share a call.

## Exact scan contract

```text
if floor >= up_to: return 0
if authoritative decoded doc is TERMINATED or >= up_to: return 0
    [these early zeros require coherent first-unread/caller ownership]
global = immutable weight.max_score()
if unsafe or Global policy: return global, without clone/scan
start = max(floor, authoritative decoded doc)

real_reader = borrowed actual SkipReader
initial = real_reader.remaining_docs() == postings.doc_freq()
if !initial && real_reader.last_doc_in_previous_block >= start:
    return global

reader = clone ONLY real_reader
reader.seek(start)
maximum = 0
loop:
    if reader.remaining_docs() == 0: return maximum
    if reader.block_info() is a nonempty VInt tail: return global
    pair = reader.selected_input_pair(), else return global
    if pair.selected_tf_ceiling == u32::MAX: return global
    local = trusted policy evaluation(reader, pair), else return global
    if local nonfinite or negative: return global
    maximum = max(maximum, local)
    if maximum >= global: return global
    if reader.last_doc_in_block() >= up_to - 1: return maximum
    reader.advance()      [independent metadata only]
```

The starting guard must precede an empty-stream inference from a possibly later shallow reader. A forward reader cannot recover earlier relevant postings. `remaining_docs == doc_freq` is the semantic initial-block identity: the previous-last field starts at 0 even when legal doc 0 exists. Byte offset 0 is not a block ordinal. For a noninitial reader, `previous_last >= start` means a relevant document might be in a skipped earlier block; fallback rather than assert or rewind. The guard can reject safe gaps too, conservatively. Because outer preparation shallow-selects at `max(lo, actual doc)`, normal drained calls should begin no later than their needed start; later-start fixtures must still demonstrate fallback.

Whole first and last full blocks intentionally over-cover partial endpoints. `seek(start)` skips only blocks whose last doc is below start. Stop after including a full block whose last doc covers `up_to - 1`; otherwise advance until a later block, tail, or EOF. Validate the half-open interval before subtraction; never increment TERMINATED. A nonempty VInt tail has no finite last-doc endpoint or stored pair, even if already loaded; this mechanism returns global and performs no tail arithmetic. Empty EOF contributes zero only after the starting coverage guard. Missing metadata never contributes zero.

This operation is not valid while retained scored leaves exist: those leaves can precede the real first-unread doc and metadata. The caller's drained-boundary structure is the primary ownership proof. Document that precondition instead of accepting a synthetic buffered head as a real cursor or expanding this experiment into a second lookahead owner.

## Reuse of trusted numeric policies

`TermWeight` obtains the actual reader's stored selection average/tag and binds them through `with_stored_block_max_selection` (`term_weight.rs:199–209`; `inverted_index_reader.rs:265–270`). Serializer selection chooses a DEFAULT **input maximizer pair**, not independent raw norm/TF extrema (`serializer.rs:534–565`). The pair's selected frequency ceiling bounds that selected posting; the comparator/enclosure proof covers other leaves.

* **StoredDefault:** the constructor validated matching legacy/native scoring-selection direction, exact DEFAULT parameters, safe nonnegative weight, and finite positive identical query/selection average (`bm25.rs:675–697`). For an unsaturated full pair, reuse `reader.block_max_score(&self.similarity_weight)` (`skip.rs:243–274`), then check usable finite nonnegative output. Do not substitute a different arithmetic expression or fieldnorm lookup.
* **PendingNative:** only native nondefault, actual native DEFAULT selection, k1>0, safe positive weight, and finite positive averages can issue this context (`bm25.rs:699–726`). On the first usable unsaturated full pair, reuse `native_input_envelope(context)` and the existing transition to `NativeTransform(Arc::new(envelope))` or `Global` (`term_scorer.rs:185–194`). A policy transition is permitted; actual metadata/postings/positions and the score weight are unchanged. The existing cold envelope construction/allocation is real work.
* **NativeTransform:** reuse `envelope.input_upper(pair.norm, pair.selected_tf_ceiling)` and then `weight.score_from_native_input_upper(upper)`, with the same finite-input validation as the existing current-block owner (`block_segment_postings.rs:239–255`). `input_upper` encloses the rounded DEFAULT input, query/selection inverse ratio, multiplication, and f32 cast (`bm25.rs:318–338,729–780`). Do not rescore the pair under arbitrary custom k1/b or reconstruct normalization arithmetic in the caller.
* **Global/failed enclosure:** return query-global immediately. This includes untrusted public construction, opposite selection tags, DEFAULT shifted-average mismatch, legacy custom, native k1=0, and failed inverse-cache validation (including b=1/norm0). Exceptional/signed OR domains retain the canonical route before any widening.
* **Saturation, missing pair, Basic, nonempty tail:** return global for this first range mechanism. Native DEFAULT's current-block path can support a saturated TF ceiling, but deliberately do not import that extra range behavior. This rule also avoids attempting a native transform from a saturated witness. Do not populate or invalidate the real single-block cache.

For each trusted full block, its existing selected-pair bound/enclosure bounds every literal matching f32 leaf. Whole-block supersets cover clipped ranges; maximum over all covered block certificates covers the interval. Any unresolved nonempty coverage has the valid global certificate. `max(preserved raw initial bound, range result)` preserves old coverage and prevents silently bundling a tighter-initial-bound experiment. Score fold order, strict callback semantics, original-n inflation, sequential f64 prefix arithmetic, and the final single f32 publication remain unchanged.

## Verification available without a release observer API

A test in `term_scorer.rs`'s own test module can access `scorer.postings` and its crate-visible block cursor. Existing accessors expose real selected end/previous end, remaining docs, byte/position offsets, BlockInfo, block-loaded status, doc decoder output, and frequency output (`block_segment_postings.rs:309–346,427–429,521–523`). Snapshot those fields before and after several overlap/widen requests, including unloaded shallow state. Preserve the test's actual doc mirror and independent scalar trace.

TermScorer does not implement Postings, but its own test module can invoke **existing** `Postings::positions`, `position_cursor`, and `append_positions_with_offset` on `scorer.postings`. Build a real TEXT RAM index and request WithFreqsAndPositions; create an actual native scorer with the reader's tag/selection average and valid query weight. At a nonzero loaded position, warm the positions path on candidate and untouched clone identically, perform range calls only on candidate, then compare every remaining `(doc, TF, positions, raw score bits)` plus repeated/nonzero-offset position reads and across-block seeks. This exercises the new helper rather than only rerunning old tests.

Reuse these existing semantic ownership fixtures as regression coverage:

* `segment_postings.rs:612`, `batch_then_scalar_reads_preserve_positions_and_skipfreq`: 300 actual indexed docs, WithFreqsAndPositions and Basic/SkipFreq reads, full/partial suffixes, stale shallow tail reconciliation, boundaries 127/128/256 and partial tail 280/290.
* `segment_postings.rs:731`, `position_prefix_tracks_repeated_reads_seeks_blocks_and_clones`: repeated positions/streamed positions with offset, clone independence, seeks and exact boundaries through 299.
* Existing TermScorer tail/public-cache tests (`term_scorer.rs`, names `test_public_block_max_does_not_reuse_another_weights_cache`, `test_tail_cache_public_weight_does_not_poison_fixed_scorer`, `test_tail_cache_public_reader_does_not_poison_fixed_scorer`) establish the existing cache identity separation; range calls should also be interleaved with real current-block bound requests to catch accidental range-cache pollution.

The private SegmentPostings `cur`, `position_prefix`, and PositionReader internal offsets/bytes/decoder are not directly inspectable from the sibling TermScorer module. Therefore **do not claim byte-for-byte snapshots of every private internal field**. Complete remaining public posting/position behavior plus source-local metadata/decoder snapshots and existing owner tests provide the minimum proof. No new release accessor is justified solely for a test. If root finds a material unproved mutation, use a narrowly scoped cfg(test) owner-local snapshot only after identifying that gap, rather than widening release API visibility.

## Minimum fixture families for an implementation decision

1. **Multi-block literal enclosure:** actual serialized dense and sparse bitpacked postings with independently scored positive-TF leaves; initial doc 0, inside-first/final block, exact exclusive ends 127/128/129/255/256/257, million-address gaps, physical max_doc, actual cursor inside/beyond the request, complete EOF and nonempty tail. Prove a returned non-global certificate is actually below global in at least one usable case; otherwise a global-only stub could satisfy enclosure tests.
2. **State/progress:** real WithFreqsAndPositions scorer/untouched clone as above; overlap ranges, first partial selected block, normal stale shallow selection, deliberately too-late selection, exact-end replay, retained full/partial batch head and across-region refill. Assert later-start fallback equals global and changes no actual state. Policy Pending→NativeTransform/Global changes are explicitly allowed.
3. **Policy matrix:** genuine legacy DEFAULT serializer provenance, genuine native DEFAULT indexed/serialized provenance, native .9/.4 and2/1 profiles, shifted selection/query averages, opposite tags, absent/Basic pair, saturated selected TF, legacy custom, k1=0 and b=1/norm0. Observe usable stored/native-transform paths rather than label a legacy `create_for_test` fixture as native stored selection. Assert fallback policy and every literal leaf enclosure; no synthetic zero-TF scorer witnesses.
4. **Caller bridge:** old locally optional cap, later high-TF crossed-block winner, actual first unread already after old_hi but before final_hi, preserved raw pre-cleanup bound, actual final partition rebuilt before replay. Compare complete raw callback trace/current thresholds with independent union, n3/n32 and actual native/legacy scoring. Witness multiple crossed full blocks via promoted optional scalar multi traversal; sole-essential suffix/end rule asserted independently.
5. **Public/deletion integration:** real RAM-index alive bitset with deleted early high scores and observed widening; growing Top1/Top10 thresholds and exact public TopDocs against exhaustive live scoring. Keep signed/exceptional canonical fallback and stale-tail cases. Frozen root exact gates and unchanged balanced controls remain mandatory for any eventual performance claim.

## Costs, holes, and stop boundaries

The clone avoids decoding and copies only fixed metadata state, but Arc increment/decrement is real work. `SkipReader::seek` loops through `advance`; each full-block advance updates remaining_docs, compressed byte and position offsets, parses the next skip entry, and advances the cloned OwnedBytes view (`skip.rs:392–445`). Cost is O(total visited metadata entries), plus the existing O(n) preparation and final prefix/partition pass. Repeated overlapping proposals can parse the same entries repeatedly. Dense and bitpacked metadata share this linear property. This is not the port's coarse impact-level range operation.

Native envelope construction can reconstruct a 256-entry inverse cache and allocate Arc storage once per eligible scorer; transformed pair evaluation still performs enclosed arithmetic. No new range result allocation, persistent state/cache, block budget, loaded-tail reduction, selector or hierarchical index is allowed in this first candidate. Its branch/clone/scan cost could exceed removed region setup. Bounds may remain global in many real workload policies/tails; no distribution was measured here. The old diagnostic's 89.05% optional-only caps motivates the experiment but is latency-unweighted and cannot quantify the range scan's net value.

This addendum leaves runtime holes intentionally open: actual scan-entry distribution, policy/fallback distribution, widened region count/width, resulting partition/candidate/probe growth, binary/frame change, correctness receipts, and balanced latency all require root's later authorized trial. If12 does not demonstrate that looser crossed globals are a meaningful cause of lost work, B may be unjustified. If12 is kept, B is a single distinct follow-up against that accepted parent; if12 is rejected, retain the same A endpoint/scratch/preparation shape in a new isolated accepted11 tree and compare only after full review. Do not claim an accepted12 parent before root accepts it.

## Source identity receipt

These SHA-256 values identify the immutable accepted11 Git objects; all except the concurrently root-grafted OR worktree file also matched the final working bytes:

| File | SHA-256 |
| --- | --- |
| `src/query/boolean_query/or_maxscore.rs` | `21153a76a7ae3091ac6400cc438f408b7ac367d9a534b1d290d30af5c3fb5901` |
| `src/query/term_query/term_scorer.rs` | `dd1e18b5b1d273cf2c09959ea207997a1e93f854b0fc459fb0a1e76129b87277` |
| `src/query/term_query/term_weight.rs` | `57510853a5a4c53633b5c053232d7061cdeb3df8822be57fd8f8889be5794f66` |
| `src/query/bm25.rs` | `64f8fc7922a31328c2410a4c7f8fea4ff599b5e31493829a0898a0ef660cd67c` |
| `src/postings/skip.rs` | `c0c6f8017aa2249fecc4076df372512fb9a9c09fa2292ad45cd5db8d34cf4eb9` |
| `src/postings/block_segment_postings.rs` | `55e3b9b64601cbaebdf97b1e704895740243359898f92bcd0893555b6af608ad` |
| `src/postings/segment_postings.rs` | `dc884fe8227d9ce0f79e5b8a4c7118b32654b43174078f6ac5e6872c981740a1` |
| `src/postings/postings.rs` | `65c925f08fc9dd886e6605bfb0e0c59542d91257b38000169b4195cc5159e788` |
| `src/positions/reader.rs` | `990b5ef4d9a0a9fe48071c99722bb4ed3938e383233c417853d15273bece51a6` |
| `src/postings/serializer.rs` | `ac96be1db50f7143829179d04e1cfcbf6f24eecaae43f3b6f6f45ec5e81b14b3` |
| `src/index/inverted_index_reader.rs` | `87ab13a39db0482eb55ea994714c5448fd6a1e28516b14ba19ca1e3322108b26` |
| `ownedbytes/src/lib.rs` | `e47dc3dc01310b7336680f2b02294c461d538809edf8f468e98353285ccd69c5` |
