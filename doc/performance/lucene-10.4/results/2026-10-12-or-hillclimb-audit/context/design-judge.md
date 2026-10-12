# Fresh arena judgment

Read-only judgment. I read grounding.md, all three complete design packages,
and baseline-two-perf-report.txt. I also checked the current two-term entry and
the production 3+ admission, cursor ownership, and region builder. No source
changes, builds, tests, profiles, timings, or child tasks were performed.

## One first implementation

Implement **design-two A, fallback-only fixed-two local regions** first.
Replace the post-warm generic Block-WAND fallback, preserving the existing
128-candidate warm phase, global single-essential walk, and terminal global
intersection transfer. Add the required physical max_doc argument and safe
pre-movement numeric admission; these are correctness dependencies of the new
region kernel, not extra performance mechanisms. Do not add batches, local
sorting, threshold-driven repartition, or changes to the established global
single-essential branch.

The fresh five-query two-term profile puts 55.93% of self samples in generic
Block-WAND and 16.43% in posting seek, with no lost samples. This makes the
fallback the best observed target among the proposals. The fixed-two region
kernel can replace repeated pivot selection, bound alignment, and ordering with
one certificate and direct execution per physical region. It can also remove
single-term candidate visits when local bounds require the other stream.
Those are structural reductions, unlike the previous failed sorting-only trial.

This is a hypothesis, not a forecast of a 2x speedup. The WAND symbol contains
inlined scoring too, the profile aggregates five expensive cases rather than
all 198 two-term ORs, and short intersected block regions may retain most work.
The new kernel does not touch the 103 longer ORs and cannot by itself satisfy
the complete target. No mechanism here establishes victory before measurement.

## Rubric: 1 is weakest, 5 strongest

Scores assess each package's stated FIRST isolated attempt, not its larger
follow-on designs. The total is descriptive; causal evidence breaks the tie.

| First attempt | Exact bits / cursor-bound proof | Potential to halve relevant cost | Compactness | Ownership / API simplicity | Independent gates / controls | Total |
|---|---:|---:|---:|---:|---:|---:|
| Partition: pinned global prefix, local suffix permutation | 5 | 3 | 5 | 4 | 5 | 22 |
| Batch: exactly one essential, block-local scored batch | 4 | 3 | 4 | 3 | 5 | 19 |
| Two: fallback-only fixed-two local regions | 4 | 4 | 5 | 4 | 5 | 22 |

**Partition:** strong reuse of the incumbent numerical/cursor proof; 32 bounded
rank bytes and no new allocations. Pinning the globally certified prefix
preserves the historical density decision. Removing dense candidate work has
real upside, but no supplied trace shows local priority reversals dominate.
Sorting adds work to the already-hot region phase and adds rank indirection.
It is the best separate longer-OR trial, rather than a dependency of the first
two-term trial.

**Batch:** careful first-unread cursor ownership and unchanged BM25 expressions
are sound in intent, with good independent leaf and cursor oracles. The new
SegmentPostings protocol, scoring API, and buffered replay introduce a larger
four-module proof surface. Its roughly 2 KiB extra fixed scratch is reasonable
but exceeds both scalar alternatives. The first attempt only helps exactly-one
essential regions, whose dominance is not demonstrated, and leaves the measured
region-prefix overhead intact. The bitmap follow-on has greater scope but a
different scratch cost and must not inflate this first attempt's score.

**Two:** directly grounded in the newly measured fallback, two bounded stack
records, no additional allocation, and a small private call boundary. The
admission and original-order leaf arithmetic are explicit. Its proof score is
4 because this is a new region executor with promotion/intersection/tail modes,
not merely a reordering of an established exact executor. New raw-bit trace
coverage must establish those transitions; existing near-equality randomized
checks are insufficient.

## Scope and proof conditions before timing

1. Keep incoming scorer identities fixed. Publish the original two f32 leaves
   with one f64 sum and final f32 cast. Pruning uses forward conservative sums,
   never threshold subtraction. Numeric fallback must precede any warm or
   shallow movement. Leave the safe existing warm arithmetic unchanged unless
   exact equivalence is explicitly established; changing its kernel is not the
   speed experiment.
2. The selected block cap and maximum have the same lifetime: one half-open
   physical region. Save the selected cap before tail reconciliation. A shallow
   selected block does not legitimize stale decoded doc/frequency values.
   Movement owners update cached docs; debug assertions check coherence.
3. Local required-optional/intersection membership ends at hi. A later region
   must permit a one-term winner after a local intersection region. Reconcile
   newly essential stale cursors before scoring. No callback-driven
   repartitioning in the first attempt.
4. Preserve the existing warm/global intersection hand-offs. **Do not add a
   new intersection transfer from the region kernel in the first attempt.**
   After shallow region skips, an optional decoded cursor may remain below
   the next unprocessed floor; handing it to the existing intersection code
   can revisit processed docs or combine stale decoded state with a newer
   selected block. Retaining conservative local modes avoids that additional
   proof obligation. A later transfer must actually reconcile EVERY cursor to
   the next unprocessed floor, prove the hand-off state, and be terminal; do
   not call intersection and resume the region loop. Preserve ascending callback
   order, strict comparison, collector-owned deletion filtering, and physical
   max_doc even with trailing deleted IDs.
5. Use the independent buffered-union FULL callback trace with raw score bits
   for reversal, local-intersection exit, stale tail, exhaustion, warm boundary,
   rounded-threshold neighbors, reversed incoming order, and deletion cases.
   Prove the intended modes are reached using test-only observation if needed.
   Frozen default/configured full gates and timed checksums remain mandatory.

## Grafts and subsequent choices

Graft **verification ideas only** now: the partition package's explicit
mode-transition observer and old policy regression visibility, plus the batch
package's first-unread/cursor boundary checklist. Do not graft their production
permutation, batch API, arrays, or threshold promotion into the fixed-two diff.

Run the local-order partition as a separate longer-OR mechanism. Prioritize
single-essential batching only after an observed regime histogram or a residual
profile establishes enough driver arithmetic/cursor cost to matter. The cheap
fallback-only generic MaxScore-for-two experiment is a useful rejecting
comparison if the fixed-two trial loses; preserve its old outer warm/global
routes too. Do not confuse that reuse trial with directly rerouting every
two-term query, which changes multiple established decisions at once.

For acceptance, retain all queries and both timing orders, self-controls,
actual fixed-port comparison, named historical bad IDs, index/source/bin
identities, and measured scratch/binary changes. Reject incorrect bits and
material regressions; record and preserve a failed attempt before reverting.
An accepted partial gain advances the hillclimb but leaves the target pending.
