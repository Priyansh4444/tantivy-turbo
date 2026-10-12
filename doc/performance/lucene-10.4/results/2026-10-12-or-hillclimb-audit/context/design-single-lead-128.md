# Attempt 10: drain a certified single-lead span, capped cutoff 128

This is a revised design before implementation. Parent is accepted06
`a674aaf2f6187f3010a135dc25fdacf9cab27e53`; isolated worktree is
`search/tantivy-or-single-lead-oct07`, branch `perf/or-single-lead-oct07`.
The worktree was absent and the branch did not exist before creation. Only this
design has been written; production sources remain unchanged. Root agreement
is required by the delegated brief before implementation, then root owns every
runtime job and the measured verdict.

Architect phases: Ground complete; Sketch complete; Agree pending; Implement
pending; Scrap conditional on repeated shape friction. The prior arena produced
distinct heap and single-lead shapes; this revision keeps the smaller rejected
shape as a separate experiment, not an addition to attempt09. Workflow hillclimb
steps1–4 are inherited from root's frozen run; step5 is pending this isolated
unit; steps6–8 remain root-owned. One mechanism, one measurement, keep or revert.

throughput checkpoint: n/a, source/design only

## Caller usage first

Keep the established region, prefix and essential suffix fixed. At a drained
candidate boundary, choose either a lead span or one scalar minimum using one
two-minimum pass over the essential suffix. A tie is counted twice. A suffix with
one clause keeps the accepted single-essential route without any cutoff.

```text
reconcile essentials at lo, exactly as accepted06
replay = Drained
loop:
    Drained => dispatch one scalar doc, a fixed lead span, or Done
    SingleLead => refill only its fixed rank below its fixed cap when empty
                  if a nonempty batch exists, expose its next retained row
    ScalarMulti => expose the minimum carried by the same two-minimum pass
    all candidates => shared current-threshold optional probes and incoming sum
    SingleLead => retire row; redispatch only after all rows drain AND cap reached
    ScalarMulti => advance every matching essential, then become Drained
Done is reachable only from Drained; pending rows always outlive real cursors
```

The ONLY cutoff for a multi-essential suffix is
`min(hi, second_doc) - first_doc >= 128`, evaluated after proving
`first_doc < min(hi, second_doc)`. `128` is fixed before this trial: it matches our
existing decoded posting batch capacity and full-block region granularity,
instead of borrowing half the port's4096-address bitmap window. Physical address
width is not posting count; a one-hit128-wide span may lose. No measured optimum
or speed claim is implied. The cutoff may not be tuned mid-trial.

## Grounding and fixed constraints

I read `design-single-lead.md`, the fresh sparse-merge judge, synthesis, and
`ground-region-caps.md` end to end, then accepted06's whole OR merge and fixtures.
`or_maxscore.rs:236–295` batches only a truly single essential; otherwise its
minimum, matching-leaf and matching-advance scans revisit the full suffix for
each candidate. A certified suffix can temporarily have only one populated
driver below the second first-unread doc. Sorted postings certify absence of
all other essential leaves below that exclusive boundary.

`or_maxscore.rs:178–235` caps the region using every nonglobal live term's
selected block before local partitioning. Locally optional terms can therefore
provide the tightest cap. This experiment retains those bounds and caps exactly;
it does not borrow the port's coarse range metadata, extend a block certificate,
use global maxima to widen regions, or mutate metadata to create an opportunity.

The existing `ClauseState::fill_scored_batch` owns the authoritative mirror
update. `SegmentPostings::read_postings_batch` consumes one decoded suffix below
the exclusive cap and leaves first unread. Literal f32 scores in
`ScoredTermBatch` remain valid after cursor advancement. No retained row may be
scored from the advanced cursor. The same scratch remains exclusive to the
current span until every row retires.

## Private shape and signatures

Everything is private in `or_maxscore.rs`; scorer/reader/public APIs are unchanged.

```rust
struct SingleLeadSpan {
    rank: usize,       // fixed preference rank, never incoming ordinal
    up_to: DocId,      // exclusive fixed cap <= current hi
    len: usize,        // accepted batch rows currently retained, <=128
    next: usize,       // next unretired row, <=len
}

enum ReplaySource {
    Drained,
    SingleLead(SingleLeadSpan),
    ScalarMulti { doc: DocId },
}

enum EssentialDispatch {
    Done,
    SingleLead { rank: usize, up_to: DocId },
    ScalarMulti { doc: DocId },
}

fn choose_essential_dispatch(
    clauses: &[ClauseState], first_essential: usize, hi: DocId,
) -> EssentialDispatch { /* not implemented */ }
```

Dispatch returns true-single batching immediately. Otherwise one pass records
first doc, its rank, and second doc; `doc < first` shifts old first into second,
else `doc < second` records second (including equality with first). If first is
outside hi, Done. A unique lead with capped width>=128 becomes a span; all other
cases carry exactly that pass's first doc in ScalarMulti. The fallback must not
repeat a minimum scan. No release route observer or density selector.

Rank/cap/len/next form one exhaustive owner instead of parallel booleans. The
owner stays SingleLead after a nonempty fill even if actual cursor is beyond hi
or terminated. Once next==len, refill the same rank/cap while its actual doc is
below the cap; otherwise transition Drained. A zero fill reconciles beyond the
cap and transitions Drained without emitting, so it cannot loop on stale state.
Callback and retirement happen before any new dispatch or refill. ScalarMulti
owns one doc until every matching essential advances after callback.

No new allocation or per-clause field, heap, matrix, cursor clone, repartition,
buffer copy, format/index/public API change. The existing contribution row and
2048B scored batch are reused. O(1) owner metadata grows the stack frame; the
root-owned fixture reports actual host sizes, not an RSS benefit.

## Exactness and progress

For selected unique lead d1, all other essential cursors are at least d2. Thus
below `min(hi,d2)` only the lead can match. Store its retained literal f32 leaf at
its original incoming ordinal and set known to that leaf converted to f64. This
equals accepted scalar accumulation because all other essential leaves are zero.
In scalar fallback, preserve existing preference-order known accumulation and
all matching leaves. Then use ONE unchanged optional/current-threshold/incoming
publication body for either route: reverse optional probes, original-count
SmallScoreSumUpperBound, incoming-order f64 fold, one f32 cast, strict `>`.

The partition and cap stay fixed while replay is pending, even after threshold
growth. The monotone callback threshold makes the old certificate conservative;
deleted physical docs remain candidates and collector rejection may retain the
threshold. Essential cursors cannot move through optional probes. The unchanged
unsafe-domain admission falls back before any seek.

Every scalar candidate advances matching cursors. Every successful fill consumes
postings and creates finite retained rows; every replay retires one. A zero fill
or cursor at cap ends the span. No addition/subtraction of TERMINATED occurs:
first>=hi is handled before capped subtraction. Exact first-doc ties select scalar,
and the second-doc boundary is excluded from all span fills, then coalesced with
all matching essential leaves at the next dispatch or numerical region.

## Meaningful distinguishing fixtures

Compare every callback doc/raw score bit to independent BufferedUnionScorer /
SumCombiner, with cfg(test)-only dispatch/fill/retirement evidence. Include:

1. Classifier cutoff127/128/129, exact first-doc ties, a large uncapped gap with
   capped127 span, single essential with width1 (no cutoff), exhausted runner-up,
   all exhausted, and near-sentinel arithmetic without giant norm storage.
2. Real scorer lead127/128/129 postings ending at shared runner doc, ties among
   at least two other essentials, exclusive cap both at runner and at hi, and
   retained future heads. Direct explicit-region fixture distinguishes boundaries
   that natural block caps might otherwise hide.
3. Pending real lead exhaustion/hi crossing before the first row is replayed,
   later winner inside retained rows, current Top1 threshold growth, no duplicate
   or lost candidate, and no extra per-row advance of the batched lead.
4. Lead span -> scalar tie -> different lead span, dirty contribution row reuse,
   optional absent/present leaves, incoming order different from preference order,
   suffix-role changes across unchanged region setup.
5. Positive indexed TF legacy/native literal midpoint leaves and adjacent/equal
   strict thresholds, proving the temporary multi-essential lead branch actually
   handles the tested midpoint; no TF0 construction.
6. Callback-modeled deleted early winners AND real RAM index alive-bitset/TopDocs
   integration, with deleted physical docs in a selected multi-essential span.

Retain accepted existing fixtures. Root runs targeted tests, exact default plus
two configured1227-case gates, release build and pilot against parent06; broad
checks and balanced whole suite only if promising. The frozen acceptance ruler
and all regressions remain unchanged. Losses include second-min overhead in dense
interleaving and short capped regions, sparse one-hit spans that fail to amortize
fill, region setup/optional scoring untouched, and code-size effects on non-OR.

## Agreement request

Root: review this whole shape and agree before implementation. If ownership needs
repeated escape hatches or duplicated completion, stop and re-ground rather than
quietly reshaping it. Keep attempt09 and10 source/runtime verdicts independent.
