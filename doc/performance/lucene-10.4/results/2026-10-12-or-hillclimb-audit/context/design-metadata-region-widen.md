# Candidate B: widen to old essential ends using trusted metadata ranges

Design package only. Parent is accepted06 `a674aaf2f6187f3010a135dc25fdacf9cab27e53`; no production implementation or speed claim. This is the metadata-range alternative to candidate A's crossed-block global fallback, with the same caller endpoint rule so the comparison isolates bound quality and metadata traversal.

Architect phases: Ground and Sketch complete for this candidate; Agree, Implement and conditional Scrap belong to root's later synthesis. Arena Frame/Fan-out are root-owned; this package is one independent whole shape for fresh judging, not a selection decision. Throughput checkpoint: n/a, read-only design. I read `ground-region-caps.md` and the complete `region-diagnostic/analysis.json` end to end, plus the actual accepted scorer, batch owner, skip traversal, block-bound policy, native envelope and scalar arithmetic sources. Only this design file was written for the design task; no build, tests, timing, profile, compression, browser, source implementation, children or commits.

## Caller first

```text
at a drained boundary, keep accepted06 admission/global-density policy
prepare the ordinary selected blocks and original tight_hi
retain each selected exclusive end and its PRE-CLEANUP local bound
build tight prefix: use zero only when actual doc >= tight_hi
derive old_first_essential with the unchanged original-num_terms helper

if old_first_essential == n:
    keep the original tight-region skip; no widening proposal
else:
    proposed_hi = min(selected_end of clauses[old_first_essential..], max_doc)
    if proposed_hi > tight_hi:
        for each clause in fixed preference order:
            if authoritative doc >= proposed_hi: expanded bound = 0
            else if globally optional: expanded bound = query global max
            else if selected_end >= proposed_hi: expanded bound = retained local max
            else:
                expanded bound = max(retained local max,
                    incoming_scorer.range_max_score(lo, proposed_hi))
        build the complete expanded prefix, then choose first_essential ONCE
        hi = proposed_hi
    else:
        keep tight prefix/partition and hi = tight_hi

reconcile essential actual cursors at lo through the existing owner
run accepted06 single scored-batch or scalar multi candidate protocol
run the one unchanged current-threshold optional/publication body
drain the entire fixed certificate, then discard it and set lo = hi
```

The old partition proposes an endpoint; it is not reused as the expanded partition. Crossed old optionals can become essential only during preparation, before pending batches exist. Their expanded bounds cover every crossed block, so their original selected end need not cap the new certificate. No fixed-point loop, second endpoint proposal, callback repartition, wider posting buffer, optional batching, heap or new density rule is included. An expanded all-optional partition may safely skip the expanded range; its bounds, not the old role names, justify that skip.

The endpoint is naturally capped by all old essentials' selected blocks. This keeps the independent A/B comparison fair and excludes another window-width policy. It can still span many dense optional blocks when the essential postings are sparse. That is the central risk, not an assumption of cheap metadata scans.

## Private signatures and module shape

```rust
// or_maxscore.rs: existing metadata plus one physical certificate endpoint.
struct ClauseState {
    ordinal: usize,
    global_max: Score,
    cost: u32,
    doc: DocId,             // authoritative first unread, never a range cursor
    local_max: Score,       // preserve the bound from preparation; do not zero it
    selected_end: DocId,    // exclusive, clipped to max_doc; max_doc if no cap
}

// term_scorer.rs: private query-bound capability, not a generic Scorer trait.
impl TermScorer {
    /// Covers still-unread matching postings in [floor, up_to).
    /// Called at drained boundaries; may resolve existing native bound policy.
    /// Never moves actual postings, real skip/position state, or decoders.
    pub(crate) fn range_max_score(&mut self, floor: DocId, up_to: DocId) -> Score {
        /* not implemented */
    }

    // Private policy evaluation over a borrowed, independent metadata reader.
    // No caller-supplied weight/fieldnorms or block-cache write.
    fn trusted_metadata_max(&mut self, reader: &SkipReader) -> Option<Score> {
        /* not implemented */
    }
}
```

The helper is a numerical policy chunk inside the same deep owner, not a pass-through chain. `None` means that this nonempty block cannot be certified from trusted metadata; the range owner substitutes global. No public range trait, decoder clone, loaded-tail reduction API, weight-taking public cursor call, new numeric cache or separately exposed lookahead protocol is needed. `TermScorer` already has direct private access to `postings.block_cursor.skip_reader()` and its immutable similarity weight; generic `BlockSegmentPostings` need not learn selection provenance.

| Module | Responsibility/change |
| --- | --- |
| `boolean_query/or_maxscore.rs` | Retain selected ends/pre-cleanup maxima; one endpoint proposal; complete bound replacement and prefix/partition rebuild before unchanged replay. |
| `term_query/term_scorer.rs` | Trusted, clone-only range operation; reuse existing `TermBlockBounds` validation/native enclosure; hide all policy and cache identity. |
| `postings/skip.rs` | Existing clone, seek, advance, block info, pair, endpoints and offsets only. No format or parsing change. |
| `postings/block_segment_postings.rs` | Existing borrowed reader accessor; real cursor and one-block score cache untouched by range lookahead. |
| `query/bm25.rs` | Existing scalar/default pair and native-envelope formulas unchanged. No cache arithmetic, fieldnorm gather or selection statistic changes. |
| serializer/reader/public APIs | No change; fixed v11 index used unchanged. |

The mutable `local_max` continues to mean the prepared certificate bound, not an effective zero from one range endpoint. Tight prefix construction uses a local `effective = if clause.doc >= tight_hi { 0 } else { clause.local_max }`. During widening replace local_max with the expanded effective bound only after the initial partition was obtained. No second bound array or per-region allocation is required. The selected_end field is physical coverage, not a sentinel numeric score or buffer head.

## Range operation pseudocode

```text
validate nonempty half-open range before subtracting one
if floor >= up_to, or authoritative doc >= up_to, or doc == TERMINATED:
    return 0 in the admitted nonnegative domain
global = immutable weight.max_score()
if weight/policy cannot supply trusted metadata:
    return global without cloning/scanning

start = max(floor, authoritative doc)
reader = clone ONLY actual SkipReader
if metadata starting point could already have skipped relevant docs >= start:
    return global                         // never attempt backwards recovery
reader.seek(start)                        // independent forward cursor
maximum = 0
loop:
    if reader.remaining_docs == 0: return maximum
    if nonempty VInt tail: return global  // no pair; never decode it
    block_max = trusted_metadata_max(reader), or return global
    if nonfinite/negative result: return global
    maximum = max(maximum, block_max)
    if maximum >= global: return global
    if reader.last_doc_in_block >= up_to - 1: return maximum
    reader.advance()                     // clone offsets/bytes only
```

First partial and final partial full blocks are deliberately included whole. `seek(start)` may skip irrelevant metadata blocks strictly before start; it never skips a block that could contain a relevant posting. A partial VInt tail has no finite endpoint or pair; its count distinguishes nonempty unknown coverage from empty EOF. A nonempty unresolved tail makes the whole range bound global, so nothing after a sentinel is traversed and no sentinel arithmetic occurs.

The starting-point guard hides the cursor limitation inside TermScorer. For a noninitial selected block, `last_doc_in_previous_block >= start` means the current clone may start after relevant postings, so return global rather than claiming coverage. Initial block identity is `reader.remaining_docs() == self.postings.doc_freq()`, which includes legal doc 0 even though its previous-last field is initialized to 0. Do not use byte_offset 0 as initial identity: compressed bytes are not a semantic block ordinal. The existing outer preparation normally selects a block at `max(lo, actual doc)`, so its starting point covers the required start. The guard makes an accidentally later actual shallow selection conservative rather than recovering skipped postings. It does not authorize calling range preparation while retained leaf buffers exist: such leaves can precede both the authoritative doc and current real metadata, and the caller's drained-boundary contract excludes them.

No range result is cached in the existing one-block numeric cache. The clone may visit arbitrary future endpoints, while that cache is tied to actual selected-block movement. Keeping range maxima call-local categorically prevents identity collisions and avoids a persistent lookahead/cache lifecycle in this first trial. Existing `PendingNative` can resolve to `NativeTransform` or `Global` through the existing validated constructor upon a usable full pair; subsequent pairs reuse that immutable envelope. This query-owned policy mutation moves no actual cursor and changes no scoring weight. Cold envelope allocation remains actual work, not a free scan.

## Trusted numeric policy

`TermBlockBounds` is the certificate: no caller-provided weight or format-version guess replaces it.

* `StoredDefault` means matching native/legacy selection comparator, exact DEFAULT parameters, safe nonnegative weight, and positive finite identical query/selection average. Evaluate a compatible unsaturated full pair using the existing `SkipReader.block_max_score(weight)` expression. Missing/zero pair returns global.
* `PendingNative`/`NativeTransform` mean the existing admitted nondefault native domain and native DEFAULT-input selection provenance. Reuse `NativeInputEnvelope.input_upper(norm, selected_tf_ceiling)` and `score_from_native_input_upper`, including all existing validity checks. Do not rescore the stored pair under arbitrary custom parameters as if it were raw TF/norm extrema.
* `Global` includes unsupported legacy custom queries, selection/profile mismatch, DEFAULT shifted-average mismatch, invalid or failed enclosure and ineligible native parameters. Return global without traversal.
* All saturated selected TF ceilings (`u32::MAX`), missing metadata, unknown nonempty tails and unusable pair/enclosure results return global for this first range mechanism. Native DEFAULT's existing current-block ceiling can be safe; this design deliberately chooses a conservative range fallback rather than adding saturation handling to the new path. Empty EOF contributes zero.

For each compatible full block, the validated stored maximizer or existing enclosed native input bounds every literal matching f32 leaf in that block. Whole-block maximum over a clipped interval is conservative. For unresolved coverage, the immutable global maximum bounds every admitted valid positive-TF leaf. Taking the maximum of all covered block bounds therefore bounds the requested unread range. `max(retained_initial_bound, range_bound)` on a crossed clause preserves the old prepared certificate as well, avoiding an incidental tighter-bound experiment.

All score sums retain original incoming leaves, preference order, original n inflation, inclusive prefix, strict threshold comparisons and one final f32 publication cast. Widening changes only certificate coverage/quality and preparation frequency. Earlier clauses with actual doc >= old tight_hi may contribute before proposed_hi: their preserved nonzero initial bound or complete range certificate is used. Reusing their old cleanup zero would be unsound.

## State invariants and costs

1. Every local_max used by prefix construction covers the complete chosen [lo,hi) for every still-relevant posting; zero requires actual first unread >=hi or proved EOF.
2. Every real clause/scorer doc mirrors authoritative first unread. Metadata range work alters only an independent cloned reader and, optionally, existing immutable bound policy initialization.
3. One partition and one certificate remain fixed throughout accepted06 pending replay. No range lookup sees undrained retained leaves.
4. Range results never enter a single-block cache. Immutable weight/provenance remain within the scorer that validated them.
5. Exclusive endpoints are clipped to physical max_doc, including deletes; `up_to-1` is used only for a validated nonempty range.

Source-grounded cost is O(n) initial preparation plus O(sum crossed metadata entries) for lookahead, plus O(n) expanded prefix/partition. SkipReader.advance parses entries and updates compressed-byte and position offsets; it is not constant-time/coarse range metadata like the port's level1 impacts. Repeated overlapping proposals may reparse entries. No persistent summaries are added to hide that cost. Early global fallback or reaching global can end a scan, but that is a conservative algebraic outcome, not an unmeasured block-budget selector.

The existing heap attempt is not imported. ScoredTermBatch stays 2048B; accepted scalar multi replay remains. One added DocId field may grow ClauseState 24→32B on this 64-bit host, about256B across n32 (actual ABI requires root measurement). Temporary SkipReader clone shares OwnedBytes backing, performs Arc reference-count work and carries fixed state; it does not allocate a postings copy. Existing native envelope may allocate once per eligible query. There is no per-region Vec/cache allocation. No RSS or total compactness claim follows from this accounting.

## Grounding and limits

The supplied diagnostic records 103 kernel invocations and 95,315 regions: 84,881 (89.053%) capped solely by eventual locally optional clauses, 50.258% physical width<128, and 53.002% of nonempty single batches length <= 8. It also records 23,387 skipped regions, 907,568 finite block selections and 6,343 selected sentinel tails. These motivate trying a wider certificate, but are counter-instrumented observations unweighted by latency. They establish neither a causal speedup nor range-scan cost or bound-policy distribution.

The largest example, query 920 (21 terms), supplies 30,242 regions and 631,838 finite selections, with 29,629 locally-optional-only caps. Its tightest optional block may be cheap to advance, while a wider certificate could scan many dense entries and promote more essential terms. Query 919 has 8,690 single-essential regions; query 917 has many multi-essential regions. Both patterns matter because a smaller setup count can still lose to larger candidate/probe volume. The design cannot claim that a range will retain the old partition or that metadata traversal is cheaper than repeated certified setup.

## Minimal meaningful runtime fixtures for root

1. **Literal range enclosure:** actual serialized dense/bitpacked lists spanning full blocks and nonempty tails; lower/upper endpoints at 0,127,128,129,255,256,257, inside a block, exactly exclusive block end, sparse million-address gaps and physical max_doc. Compare every still-unread scalar leaf in the range to the returned bound. Include an actual cursor already inside/beyond the range and empty EOF; do not infer emptiness from a sentinel alone.
2. **Real-state preservation:** clone scorer at a loaded nonzero posting position with frequencies/positions; capture actual doc, real skip previous/current ends, remaining docs, offsets, block-loaded status, decoder arrays and current position state. Call several overlapping/wider range requests, then compare all state and the remaining `(doc,TF,positions,scorebits)` trace to an untouched clone. Include stale shallow selection at lo, first partial selected block and the conservative later-start fallback. Bound policy initialization may change; actual posting/metadata state may not.
3. **Policy provenance:** trusted legacy/default and native/default selection; shifted query average, opposite scoring-selection tags, absent/Basic pair, saturated TF, native custom profiles (.9/.4 and2/1), failed enclosure/k1=0/b1 norm0, and legacy custom. Assert which paths fall back and bound every literal valid positive-TF leaf. Do not synthesize impossible zero-TF witnesses or silently rebuild normalization arithmetic.
4. **Caller coverage correction:** a dense eventual optional supplies old tight_hi; sparse old essentials supply proposed_hi. Put a strong doc of the crossed term after old tight_hi but before proposed_hi, including a term whose actual doc was already>=old tight_hi. Assert complete current-threshold callbacks/raw bits equal independent union; observe promoted essential role only before replay and genuinely wider certificates. This catches cleanup-zero reuse and old-partition reuse.
5. **Replay boundary/deletion integration:** new essential scalar traversal across full blocks; single accepted batching across 128 refill boundaries inside one expanded certificate; doc exactly proposed_hi appears only next region; tail/missing/saturation fallback cannot omit candidates; growing Top1/Top10 thresholds and real indexed deleted early winners preserve complete raw callback traces and public TopDocs. Keep accepted signed/exceptional fallback and stale-tail fixtures.

Root's existing frozen three 1227 exact gates, full targeted/broad checks as appropriate and unchanged balanced AB/BA pilot/full ruler remain required. Counters for proposals, successful widening, metadata entries and partition/candidate/probe growth may diagnose a trial separately, never serve as latency acceptance evidence.

## Alternatives considered and rejected

Candidate A's query-global crossed bound is a viable simpler whole shape: zero metadata traversal and fewer provenance obligations, but potentially looser expanded prefixes. This candidate chooses existing trusted pairs/enclosures for crossed terms to preserve useful optionality where possible; that is its one distinctive mechanism.

Rejected cloning TermScorer/BlockSegmentPostings because decoder state and unrelated scoring ownership are unnecessary. Rejected real seek_block lookahead because it can hide unread postings behind stale docs. Rejected new fieldnorm caches, direct custom rescoring of stored pairs, current-block cache reuse for a multi-block result, persistent range summaries/coarse index metadata, loaded-tail rescoring, adaptive scan budgets/windows, repeated fixed-point endpoint changes and pending repartition because each adds another mechanism or weakens the certificate invariant.

Synthesis input: choose B only if the deeper TermScorer boundary and possible tighter bounds justify the linear scan, extra preparation and verification scope. Choose A if the simpler global certificate is a better cheap first ruler probe. No synthesis decision or performance prediction is made here.
