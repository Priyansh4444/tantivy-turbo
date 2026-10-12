# Required optional reach observer — source only

This is a throwaway diagnostic, not a production candidate or speed measurement.
No compilation, tests, query execution, profiling, compression, or benchmark was
run by its author. Root owns all runtime verification. The observer is based on
accepted parent 11, commit `43f06833a325d2a18c03bd7e4fa65f2f27944292`.

Workflow prototype steps:

1. Scope the decision the prototype exists to make: determine whether a lazy
   first-optional required-gap screen reaches substantial pending batch gaps.
2. Gather references when the design space is open: skip; the design was fixed
   by the full `ground-required-optionals.md` source investigation.
3. Build throwaway in an isolated scratch dir: isolated root-created tree
   `search/tantivy-or-required-diagnostic-oct11`, branch
   `perf/or-required-diagnostic-oct11`.
4. When comparing alternatives, build them behind one switcher: skip; this is
   one observer, with no alternative production algorithm.
5. Verify on the matching surface: pending root compilation and execution.
   Author verification is limited to full diff review, rustfmt, diff-check,
   source snapshots, and checksums.
6. Present alternatives, tradeoffs, and a recommendation: defer the decision
   until root retains the frozen 103-case diagnostic and verifies its scores.

## What changed

The sole edited tracked file is
`src/query/boolean_query/or_maxscore.rs`: 269 additions, zero deleted lines.
Every parent line, including admission, sorting, prefix construction, driver
batch fill, cursor operations, per-candidate clearing, optional probes,
incoming-order f64 score fold, callback, batch increment, and region exit is
retained. The original test module is unchanged. No feature, dependency,
public API, worker source, TermScorer, or postings owner changes were made.

The diagnostic is unconditionally enabled only in this isolated tree. It
allocates two 129-element histogram vectors and performs extra counter work;
its latency, allocation, binary size, and RSS cannot describe production.
Do not graft it into the accepted tree or use it for latency comparisons.

## Certificate and gap accounting

Only a single-essential region can build a diagnostic certificate. Its selected
optional rank is `first_essential - 1`, the first optional already probed by the
ordinary descending loop. The observer waits until that ordinary probe has
passed its existing competitiveness test and its ordinary forward seek reports
a miss. It then builds an excluded-clause sum from zero, in rank order, using
every immutable current `local_max` except that rank. Conversion uses the
existing `SmallScoreSumUpperBound` created with the original number of terms.
There is no subtraction and no shortened error count. This bound is computed
at most once per qualifying region and compared with the real current
nondecreasing callback threshold at later misses.

A passing certificate (`other_bound <= threshold`) suggests that driver docs
before `min(optional_real_doc, hi)` cannot compete. The observer finds those
entries in the already owned pending batch with `partition_point`, but never
changes the real batch index, any scorer/cursor, or a kernel decision. Every
baseline candidate continues to execute, including the existing next-lower
prefix test. Driver postings decoding and BM25 scoring were already paid when
the batch was filled.

`certificate_pass_misses` counts all actual passing misses, including repeated
observations inside one potential skip gap. It is **not** a count of unique
opportunities. `shadow_skip_until` is diagnostic bookkeeping that deduplicates
those gaps; it has no effect on the baseline loop. At a passing miss outside
an existing shadow gap, the histogram counts the pending entries strictly
before the target, including the current triggering miss.

The trigger has already paid contribution clearing, its essential leaf load,
and the first optional probe. Thus
`unique_additional_candidates_after_trigger`, not the trigger-inclusive total,
is the useful count of future candidate visits whose repeated work could be
avoided. No driver decoding or driver BM25 saving is available here.

`shadow_covered_candidate_visits` counts later baseline visits that lie in
deduplicated proposed gaps. `shadow_covered_callbacks` should remain zero;
these counters let root check the simulated skip bookkeeping against the
actual unchanged callback execution. They do not replace the independent
score oracle.

## Counter meanings

- Region denominators: `regions`, globally certified terminal, all-optional,
  single-essential, and multiple-essential counts. A global certificate return
  is counted separately because no local partition is then constructed.
- Existing global policy: single-essential regions with any globally optional
  prefix, and whether the selected first optional rank uses that prefix's
  global bound. This is policy provenance, not a new classification of
  TermScorer native/custom envelope internals.
- Existing probe reach: single-essential candidate visits, first-optional
  competitiveness rejections before probing, actual probes, matches, misses,
  certificate builds, passing misses, equality passes, regions with a pass,
  and regions whose first certificate failed but later passed after threshold
  growth.
- Existing rejection: actual next-lower prefix rejection immediately after a
  first-optional miss, with a separate certified subset. Lower optional probes
  after those misses are also retained. This measures what the baseline really
  does instead of assuming it would score the remaining optionals.
- Deduplicated gap: gap count, trigger-inclusive candidates, additional future
  candidates, an exact 129-bin pending-entry histogram (index is gap length),
  largest pending-entry gap, largest physical target-minus-trigger span,
  exhausted pending batches, and gaps reaching the exclusive region end.
- Optional endpoint: the next real optional doc exactly at `hi`, beyond `hi`,
  or TERMINATED. These classifications are disjoint; a TERMINATED doc is not
  additionally classified as equal to `hi` when `hi` itself is TERMINATED.
- Driver batches: exact 129-bin length histogram including empty fills, counts
  of full 128-entry/partial batches, finite/tail selected driver block, and
  first unread driver doc equal to `hi`, beyond it, or TERMINATED. The real
  first-unread mirror is read only after the unchanged owner fill operation.
- Ownership check: regions with a second nonempty driver fill are counted.
  Accepted-11 selected-block caps predict zero; do not infer repeated driver
  decoded blocks/refills or decoding savings from pending-entry gaps.
- `callbacks` counts actual baseline callbacks. `completed` is set only at
  ordinary exits, and `admitted` distinguishes canonical numeric fallback.

The tail indicator is read from the already shallow-selected driver before
its ordinary fill. The observer does not call block selection, bound creation,
score, advance, or seek. Its only scorer accessor is the immutable
`last_doc_in_block()` getter in that batch observation.

## Stderr emission and worker contract

`RequiredReach::Drop` writes exactly one JSON line to stderr for each invocation
of this kernel. Its marker is
`"schema":"tantivy-required-optional-reach-v1"`. No query IDs, query text,
global call sequence, or worker protocol fields are introduced into the engine.
It emits even for an admitted invocation with zero regions/opportunities and
for an ordinary numeric fallback. Unwind after observer creation emits
`completed:false`; a panic-abort or a failure before creation may emit nothing.
Root must reject failed processes and incomplete records, not label them as
zero opportunity. The existing initial debug assertion precedes creation.

The inspected frozen worker is
`OUT/harness/src/bin/tantivy_worker.rs`, byte-identical in structure to the
tracked 2026-10-07 harness. Its `Dump` evaluates:

1. `query.count(&searcher)` with scoring disabled; this does not enter this
   pruning kernel.
2. `hits(...)` through scored `TopDocs` search; this invokes the pruning path.
3. `exhaustive(...)` through `weight.scorer(...)` and sequential scalar
   iteration; this does not enter the pruning kernel.

In the worker's required one segment and no deletions, a resolved pure OR of
3..32 terms normally emits one record for a `dump`; there is no warmup or
implicit repeat in `dump`. `Run Top10` emits once per iteration; `Run Count`
does not. A term query, two-term OR, different scorer specialization, or a
query whose resolved term union has fewer than three scorers can emit none.
Root must check actual emission cardinality, rather than assign a missing
record a fabricated all-zero kernel result.

For the frozen 103 long OR cases, root should issue one sequential dump per
case using the complete original query file and retain the original case ID,
request/response, raw stderr, and exactly associated marker record. Associate
via order in a dedicated single-worker process with no concurrent requests.
Read stderr continuously or redirect it to a file: each record contains two
129-bin histograms and can exceed pipe buffering over 103 requests. A robust
simple association is one process per query (ready, one dump, close stdin,
wait, then parse stderr). Engine records remain ID independent either way.
No diagnostic `ns` value should be interpreted as a latency result.

Root should compare each dump's count, ordered hit documents and score bits,
and independent exhaustive oracle with the accepted-11 frozen results. Retain
all 103 cases, including zero certificates/gaps. The worker's top-10 exhaustive
comparison proves those query results; full callback trace equivalence remains
the responsibility of the unchanged independent kernel fixtures, not the
diagnostic counters.

## Suggested root counter validation

For each successful, complete admitted record:

- `regions` equals globally terminal + all-optional + single-essential +
  multiple-essential regions.
- First-optional probes equal matches + misses. Single-essential candidate
  visits equal probes + first-optional preprobe rejections.
- Both 129-bin histograms have integer nonnegative bins. Driver entries equal
  the weighted sum of the driver histogram and single-essential visits.
  Nonempty batches equal the sum of bins 1..128; empty fills equal bin 0.
- Deduplicated gaps equal the sum of gap bins, with bin zero equal to zero.
  Trigger-inclusive candidates equal their weighted sum. Additional candidates
  equal trigger-inclusive candidates minus deduplicated gaps, and equal
  `shadow_covered_candidate_visits` after the full unchanged traversal.
- `shadow_covered_callbacks` and lower optional probes after certified misses
  are zero. Passing misses equal the actual immediate next-lower prefix
  rejections after those certified misses.
- Certificate builds do not exceed single-essential regions; passing regions
  do not exceed builds; deduplicated gaps do not exceed passing misses.
- Gap/end classes are disjoint, and exact/beyond/terminated optional endpoint
  counts sum to gaps reaching `hi`. First-unread driver endpoint classes sum
  to nonempty driver batches under accepted-11 ownership.
- Regions with multiple nonempty driver batches are zero. Treat a violation
  as evidence requiring investigation, not as proof of an available speedup.

These identities are proposed runtime checks based on source reasoning. They
have not been executed. Even many long gaps would establish only reach and
candidate-work opportunity; isolated exact gates and frozen AB/BA latency
controls are still needed before any production performance claim.

## Static artifacts

- Parent snapshot `required-reach-parent-or_maxscore.rs`: SHA256
  `21153a76a7ae3091ac6400cc438f408b7ac367d9a534b1d290d30af5c3fb5901`.
- Diagnostic snapshot `required-reach-source-or_maxscore.rs`: SHA256
  `1991289b0e36bea08849549cc8ff6ab9e6b156063dc5d1ff0260be2482d9b6cf`.
- Authentic full diff `required-reach-source.patch`: SHA256
  `6b669b8be6f6a87d51b1df34b20632ea382aa09920c4442f83857400a2b881f8`.
- `required-reach-static-receipt.json` distinguishes completed source checks
  from unexecuted build/runtime work.

The isolated worktree and snapshot are held for root review. No commit exists
for this diagnostic and no production tree was edited.
