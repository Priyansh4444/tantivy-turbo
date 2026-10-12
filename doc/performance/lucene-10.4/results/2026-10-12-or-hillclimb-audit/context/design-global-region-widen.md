# Direction A: one wider certificate with global replacement bounds

Design only. No source was edited and no build, tests, worker run, profile,
timing, compression, browser work, child agent, or commit was performed. This
package is one whole-shape candidate for root's architect/arena comparison;
selection and implementation are pending. Base is accepted06
`a674aaf2f6187f3010a135dc25fdacf9cab27e53`, not attempts09/10/11.

## Caller usage first

`or_maxscore` retains ownership of clauses, scorers, prefix, contributions, batch,
and the whole region lifecycle. At each drained region boundary it first prepares
the existing tight certificate. A nonempty tentative essential suffix can then
choose a wider endpoint once. Final bounds certify that endpoint, and the existing
single-essential/scalar replay consumes the entire fixed certificate unchanged.

```text
query-owned selected_end: [DocId; 32]    // fixed128-byte scratch

while lo < max_doc:
    compute globally_optional with unchanged global-prefix/density policy
    if globally_optional == n: return   // unchanged early finish

    old_hi = max_doc
    selected_end[..n].fill(max_doc)
    for rank in original preference order:
        prepare EXACT existing local bound/metadata/tail reconciliation
        retain uncleaned local_max in ClauseState
        selected_end[rank] = selected block's half-open coverage end
        old_hi = min(old_hi, selected_end[rank])

    build tentative prefix using zero when actual doc >= old_hi
        BUT DO NOT overwrite ClauseState.local_max
    tentative_first = existing prefix/threshold partition
    if tentative_first == n:
        lo = old_hi
        continue                       // never widen an empty certificate

    final_hi = min(selected_end[tentative_first..n])
    assert lo < old_hi <= final_hi <= max_doc
    if final_hi > old_hi:
        for rank in original preference order:
            if actual doc >= final_hi: final bound = zero
            else if selected_end[rank] < final_hi: final bound = global_max
            else: final bound = preserved pre-cleanup local_max
        overwrite local_max with final bound; rebuild prefix
        final_first = existing prefix/threshold partition
    else:
        apply original doc >= old_hi cleanup to local_max
        final_first = tentative_first  // reuse existing tentative prefix

    if final_first == n:
        lo = final_hi; continue         // final certificate proves safe skip
    reconcile final essentials at lo using existing cursor protocol
    freeze final_hi/final_first/prefix until all candidate replay drains
    unchanged single-essential batches or unchanged scalar multi-essential loop
    unchanged optional probing/current threshold/original leaf fold/callback
    lo = final_hi
```

No feedback loop follows `final_first`. A newly essential term with an endpoint
below final_hi receives a global certificate. Its newly essential role does not
require shrinking final_hi back to its fine-block end. Recomputing width after
that partition would turn the proposal into another design and can oscillate
between locally tight and globally loose certificates.

## Types, signatures, and module map

One file changes: `src/query/boolean_query/or_maxscore.rs`. No public trait/API,
postings reader, BM25 implementation, serializer, or format changes.

```rust
// Design sketch; not implemented.
// Preference-rank addressed, reset each region. Each entry certifies the
// corresponding preserved local_max through an exclusive endpoint.
type SelectedBlockEnds = [DocId; 32];

// Existing ClauseState shape remains unchanged:
// ordinal, global_max, cost, doc, local_max.
// local_max holds the uncleaned selected bound until final_hi is decided.

pub(crate) fn or_maxscore(
    scorers: Vec<TermScorer>,
    max_doc: DocId,
    threshold: Score,
    callback: &mut dyn FnMut(DocId, Score) -> Score,
); // same signature, implementation not included in this design
```

Recommend a plain function-local `[DocId;32]`, allocated once before the region
loop, rather than a type alias/wrapper in implementation unless naming materially
helps readability. The sketch names its semantic role. Adding a method that only
returns a suffix minimum would be a shallow pass-through. Extracting separate
`prepare_tentative`, `choose_width`, and `restore_bounds` methods would scatter
ownership across temporal helper APIs; keep this short preparation block in the
existing owner. The complete candidate replay and common completion stay untouched.

`selected_end[rank]` means:

- Globally optional: max_doc, because its pre-cleanup bound is global and already
  certifies the whole physical range. It remains exempt from shallow selection.
- Exhausted at entry or after existing tail reconciliation: max_doc; its bound is
  zero and contributes no endpoint restriction.
- Live finite selection: `min(last_doc, max_doc-1)+1`, using exactly the existing
  safe clipping arithmetic. Preserve the endpoint observed before any existing
  tail reconciliation, as the current code does.
- Live sentinel tail: max_doc. The existing trusted tail/global policy covers
  the remaining tail; sentinel arithmetic is never incremented.

The entry is reset to max_doc before early continues. Preference order is frozen,
so it needs no ordinal remapping. `local_max` remains rank-addressed in the same
ClauseState, not in scorer order.

### Smaller alternatives considered

Appending a `selected_end: DocId` to every ClauseState can grow its layout from
24 to32 bytes on a64-bit host (one usize plus four existing32-bit fields, then
one more32-bit field). That would add24 bytes at3 terms and256 at32; an exact host
size_of receipt remains root's job. It permanently expands records touched by
every hot candidate/optional loop, even though the information is used only
during region preparation. No assumed performance benefit is claimed.

Retaking `last_doc_in_block()` from every tentative essential removes scratch,
but it is another scorer/skip-reader indirection. Retaking every pre-cleanup
`block_max_score()` after widening additionally touches bound caches and native
lazy policy again, and makes the proof depend on retaking identical selected
provenance. A 32-bit bitmap cannot recover the distinct essential endpoints and
cannot tell which bounds cover an arbitrary expanded endpoint. A second32-score
array adds128 bytes unnecessarily: virtual tentative cleanup preserves existing
`local_max` without losing the true selected bound. The proposed fixed128-byte
scratch keeps cold endpoint metadata out of hot ClauseState records and preserves
all needed information directly.

## Grounding from actual observations

The full `region-diagnostic/analysis.json` was read (compact JSON to avoid output
truncation), as was `ground-region-caps.md`. These are diagnostic counts from103
exact long OR queries, not a latency acceptance measurement:

- 95,315 prepared regions,907,568 finite selection visits.
- 84,881 regions (89.05%) capped only by eventually locally optional terms;
  10,216 essential-only,115 optional/essential ties,103 physical-cap-only.
- 50.26% of regions narrower than128 physical addresses;93.45% narrower than2048.
- 23,387 regions skipped entirely (24.54%). Those remain unchanged skips.
- Nonempty single-essential batches average21.68 postings,53.00% have at most8.

The mechanism hypothesis is specific: remove the short endpoint supplied by an
optional clause **only after replacing its short certificate with a global one**,
so fewer repeated region preparations may amortize over the unchanged essential
stream. It does not try to copy the port's coarse-range impact metadata, which our
frozen format lacks. Counts do not establish how many optional endpoints are
strictly shorter than the essential-only minimum in non-skipped regions, how
loose replacements will be, or whether candidate/probe growth will cost more than
the preparation saved. They cannot support a speed claim.

The aggregate is highly skewed. Query920 (21 terms) contributes30,242 regions and
631,838 finite selections;29,629 caps optional-only,9,893 regions skipped. Query919
has10,215 regions, mostly8,690 single-essential certificates. Query493 skips3,933
of5,551 regions. Those distribution differences require per-query regressions,
not extrapolation from the89.05% aggregate.

## Conservativeness proof and the zero-cleanup hazard

Numeric admission and canonical exceptional-domain fallback execute unchanged
before any metadata cursor mutation. Original global maxima remain valid for the
admitted nonnegative literal leaf domain, including safe custom native queries.
Selection provenance and current-block conservative arithmetic remain delegated
to existing `TermScorer::block_max_score`; this design never interprets raw pairs.

The first metadata pass is unchanged. For every clause it retains a conservative
pre-cleanup bound covering unread postings at/after lo through its selected end,
or a global bound through max_doc. A stale decoded doc below lo is not proof of
absence and never becomes zero merely for being stale.

Tentative prefix uses the original effective bound: zero if actual doc is at or
beyond old_hi, otherwise the selected bound. This computes exactly the existing
tentative partition. Crucially, it **does not store those zeros into local_max**.
A term can have its first unread doc beyond old_hi but inside final_hi. Reusing
an old cleanup zero for that future contribution would falsely certify absence,
lose a winner, and potentially skip the entire expanded region. The final pass
starts from preserved pre-cleanup bounds, so such a term gets its trusted local
bound if coverage suffices, otherwise global_max; only doc>=final_hi becomes zero.

Because selected ends all exceed lo, their minimum over the nonempty tentative
suffix is at least old_hi. Each old essential still has local coverage through
final_hi. For every other clause, selected_end>=final_hi allows its preserved
conservative local bound; otherwise global_max covers its future blocks. Thus
all final bounds cover every relevant literal leaf in `[lo,final_hi)`, regardless
of eventual role. The existing bound-sum inflation covers final prefixes and
optional probe sums with the original num_terms factor. Final partition may
change; conservativeness does not require it to retain tentative_first.

Global replacements can increase bounds, though numeric conservative estimates
can overlap and no strict numeric monotonicity between every stored/global bound
is needed. Do not assume final_first can move in only one direction as a semantic
invariant. An empty final suffix can safely skip only because **final** bounds
certify the wider range. The initial empty tentative suffix always keeps the old
skip range and is never expanded.

## Real cursor and replay ownership

No metadata lookahead occurs. `seek_block(lo.max(clause.doc))` and existing tail
reconciliation remain the only preparation mutations. No actual cursor is moved
to final_hi, and no SkipReader clone is created. Final essentials with a stale
actual doc below lo retain the original `seek(lo)` reconciliation before replay;
newly essential clauses follow that same protocol. Do not invent a mirror update
from shallow metadata or replace an actual doc with a retained batch doc.

Within a wider certificate, scalar `advance()` can cross decoded block boundaries:
`SegmentPostings::advance` delegates to `BlockSegmentPostings::advance`, which
advances skip metadata and loads the next block. A term upgraded to a global bound
may therefore become a multi-essential driver across several blocks safely.
The bound stored in ClauseState is a fixed region certificate, not a mutable
per-block cache.

Single-essential `fill_scored_batch` still emits at most one currently decoded
suffix per call. `read_postings_batch` reconciles an unloaded block, clips docs by
up_to, advances a consumed full block, and updates the authoritative first unread
cursor. After the pending batch drains, the existing outer batch loop calls it
again with the same lo/final_hi and can consume another block. No change is needed
to the literal batch scoring or2048-byte scratch. Pending leaves keep their own
docs/scores; no repartition occurs while they remain. A final sole essential may
be a globally bounded former optional and cross several blocks; that remains safe.

Optional terms can also cross blocks through existing actual `seek(doc)`, with
the fixed final bound covering every probe. Known leaves stay in preference
order for pruning; published leaves retain original incoming-order f64 fold and
f32 cast. Current threshold is read for every probe and publication; callback
changes never alter the region partition mid-replay. Deleted physical addresses
remain in the certified range and collector-owned rejection leaves threshold
handling unchanged.

## Cost, risks, and falsification

Fixed additional payload is128 bytes/query, no per-region allocation, no format
change. Each region writes n endpoints. Every nonempty tentative suffix adds one
suffix-min pass. Only actual widening adds one n-term final-bound/prefix pass and
partition scan. The old virtual cleanup/prefix pass already replaces the old
mutating cleanup/prefix pass. No block bound is retaken and no extra metadata
entry is parsed by the width choice.

The proposal deliberately has no density/width heuristic beyond the unchanged
global optional certificate and one old-essential-end choice. Adding a selector
before establishing this unit's outcome would introduce a second hypothesis.
When fine optional blocks are the sole cap, widening can promote very dense terms
to essential status under global maxima. Candidate volume and essential scans
can explode; more optional probes may erase saved metadata/prefix work. Query920
is an especially direct risk. A tail essential can choose final_hi=max_doc and
make the certificate very loose. Even a no-widen region pays endpoint/min overhead.
No speed or memory/RSS advantage is promised. Reject on frozen pilot/full ruler
if the mechanism loses; do not retain speculative broadening for later layering.

## Meaningful verification plan for root

Test-only observations must prove actual widening and final bounds/roles, rather
than merely comparing an unchanged route. Core fixture families:

1. A dense weak optional term owns repeated128-posting block caps while a sparse
   essential's selected endpoint is much later. Observe final_hi>old_hi and a
   final one-essential route consuming multiple batches/blocks. Exact callback
   trace/raw scores versus independent union, fixed and growing thresholds.
2. A term starts at a doc between old_hi and final_hi. It is tentatively cleanup
   zero, then contributes or becomes essential and wins later. Observe restored
   final bound, candidate doc, and strict threshold neighbors to catch stale-zero
   reuse. Include multiple such terms and reversed incoming order.
3. A weak term's current-block bound is tiny but its next block has high TF/low
   norm. Widening must upgrade it to global_max, promote it when required, and
   preserve winners across127/128/129 and255/256/257 posting boundaries. Include
   original essential endpoints tied with optional endpoints (no widening).
4. Initial first_essential==n skips exactly old_hi, even if hypothetical
   essential-only cap is undefined. Observe unchanged skipped-region endpoints.
5. Sparse high-doc terms, loaded finite blocks, stale decoded docs below lo,
   shallow-selected complete blocks, nonempty VInt tails, EOF reconciliation,
   max_doc clipping, million-address holes. Preserve mirrors/first unread after
   each actual cursor mutation; optional terms may become essential next region.
6. Original global-density exemption retained: globally optional terms have
   max_doc coverage without new shallow selection. A failed density test still
   performs the same existing local preparation. No new global admission route.
7. Literal native DEFAULT, legacy DEFAULT, native custom enclosure/global fallback,
   average/profile mismatch, missing/saturated pair metadata, k1=0 and b=1 safe
   global cases. Assert leaves under chosen certificate where observable and
   exact full traces; signed/negative/exceptional domains use canonical fallback
   before cursor mutation and do not enter widening.
8. Dirty contribution rows across changed regions/suffix roles; n3/n32; overlap
   and disjoint posting lists; optional equality pruning; callback deletion
   simulation plus real indexed deletes/TopDocs. Initialthreshold ties and f32
   rounding-neighbor cases require exact incoming-order raw bits.

Existing runtime admission/selection proofs are reused, not rewritten as trivial
mirror tests. Root's unchanged3×1227 exact gates and full suite remain mandatory
for an accepted candidate. Native pilot/full paired AB/BA timing compares a clean
candidate against immutable accepted06. Actualport comparison, full1201 and
per-query regressions, collector-work limitation, and binary/array compactness
remain part of the frozen acceptance packet. Diagnostic widened-region/bound-role
counts can explain routes separately; their altered binary has no latency credit.

## Synthesis decision

Pending root's independent alternative and fresh judge. This package proposes the
plain128-byte endpoint scratch and virtual tentative cleanup because they hide
cold metadata from hot records and eliminate the stale-zero bug categorically.
No source is ready for graft; this is a design only.


## Design-only correction from implementation grounding

After root agreed A, source grounding corrected the proposed single-driver
verification claim. Every nonempty old suffix contains the final preference rank;
a final sole suffix also contains exactly that rank. Therefore the old-essential
minimum endpoint cannot exceed the sole driver's own selected block end. The
unchanged reader can refill across blocks in general, but this particular endpoint
rule normally prevents multiple full driver blocks inside one widened certificate.
No production shape changes follow from this correction.

The attempt12 fixtures instead require full/partial retained batches, current
threshold replay, authoritative heads exactly at the exclusive endpoint, and
refills across successive regions. A test-only observed invariant checks that
every actual widened sole suffix remains within its last-rank selected endpoint.
Promoted crossed old optionals, which receive global certificates, can cross full
blocks on the multi-essential scalar route inside the wider certificate; that is
the actual within-region cross-block witness. This update is a design correction,
not a compiled test result or performance claim. B's runtime alternative is unchanged.
