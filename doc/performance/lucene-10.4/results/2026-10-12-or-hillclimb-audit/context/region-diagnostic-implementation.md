# Region diagnostic implementation

Status: source is stable for root review. This is an intentionally unaccepted,
altered diagnostic binary; it is not a runtime optimization or latency evidence.
No compiler, tests, worker run, profile, timing, compression, browser work,
commit, or child agent was performed. Root alone owns those checks. Rustfmt
(`--edition 2021`) and `git diff --check` pass.

- Worktree: `/home/pronsh/Coding/playground/search/tantivy-or-region-diagnostic-oct07`
- Branch: `perf/or-region-diagnostic-oct07`
- Base: `a674aaf2f6187f3010a135dc25fdacf9cab27e53` (accepted06)
- Only changed tracked source: `src/query/boolean_query/or_maxscore.rs`
- Exact source SHA256: `cd2865e1364a66fadda587bc5a24eaaec6086b39fcfd48f1493fdeb8974b36bd`
- Diff: 195 insertions, 2 deletions. The deletions bind the existing scalar min
  expression to `scalar_doc` so observational reads can follow it; its expression
  and returned value are unchanged.
- Full actual final diff and actual resulting production section were read.

## Enable and lifecycle

Presence of `OR_REGION_DIAGNOSTIC` (any value, including empty) enables a
query-owned `Option<RegionDiagnostic>`. Its Drop emits exactly one stderr line
prefixed `OR_REGION_DIAGNOSTIC ` followed by explicit JSON fields. Fixed arrays
are formatted as numeric Debug arrays, which are valid JSON arrays. No dependency
is added. Drop covers normal return, fallback return, global-bound early finish,
and unwinding where applicable; panic-abort cannot run Drop.

The record covers one `or_maxscore` invocation, generally one eligible segment
search. It has no query ID: root must attribute it from the controlled worker
request/order. Count or another collector that does not enter `or_maxscore`
emits no record. A multi-segment query can emit several records. Enabled summaries
include `num_terms`, `max_doc`, and fallback status. Fallback reports zero OR
regions, since independent canonical union work is not instrumented.

## Inventory and interpretation

- `regions`: every prepared nonempty physical certificate `[lo, hi)`, including
  skipped regions, before essential cursor reconciliation.
- `physical_widths[33]`: bin 0 is zero width; bin i is
  `[2^(i-1), 2^i-1]`. Physical addresses include deleted docs and holes.
- `essential_counts[33]`: histogram of `num_terms-first_essential` after final
  local prefix partition. Bin 0 represents skipped regions.
- `cap_source_masks[8]`: per-region intrinsic endpoint owner categories,
  optional=1, essential=2, physical max_doc=4; combinations retain ties rather
  than assigning a winner. Optional/essential is the eventual local role.
  Only finite selected block endpoints are owners. A sentinel tail supplies no
  intrinsic finite end, and clipping an endpoint above max_doc does not fabricate
  a tie. A finite endpoint exactly at max_doc legitimately ties with bit4.
  Globally optional clauses never supply a selected cap. Within-category
  multiple-owner ties are aggregated into that category, not individually counted.
- `skipped_regions`: local prefix proves all terms optional.
- `globally_finished`: query exits when the global prefix proves all optional,
  before preparing another region; not included in regions.
- `selected_finite_blocks` / `selected_sentinel_tails`: selection observations
  immediately after existing `seek_block`, before tail reconciliation. These count
  visits, not distinct blocks. Sentinel includes empty EOF and nonempty tails.
- `batch_lengths[129]`: exact returned lengths from unchanged single-essential
  `fill_scored_batch` calls; bin0 includes the terminal empty fill.
- `scalar_candidates`: actual multi-essential candidates with doc below hi.
- `matching_essentials[33]`: number of essential clauses with the exact candidate
  doc, before any scoring or advance.
- `optional_bound_checks`: optional-loop bound checks, including the one that
  rejects a candidate.
- `optional_probes`: iterations that pass the optional bound check and inspect
  the optional clause; not necessarily a match or a seek.
- `optional_seeks`: existing optional `seek(doc)` calls only. Shallow region
  selections, tail reconciliation, and essential reconciliation are not counted.
- `scalar_boundaries`: drained multi-essential boundaries whose earliest doc is
  within the physical region; equal to scalar_candidates by construction.
- `unique_earliest`: those boundaries with exactly one earliest owner.
- `gap_ge_128` / `gap_ge_2048`: unique-earliest boundaries where
  `min(hi, second_doc)-first_doc` reaches each physical-address width. These are
  observational eligibility counts only, not retained batch lengths, amortized
  spans, or expected speedups. Consecutive candidate opportunities overlap.

## Semantic isolation and limits

All bounds, literal score expressions, incoming-order publication folds,
threshold callbacks, essential/optional partition decisions, actual cursor
operations and ordering, single-essential batch operations, and physical hi
calculation remain unchanged. Counters read copies or already coherent clause
records and never enter a semantic decision. Scalar boundary observations add
read-only scans only when enabled. Per-region cap scratch is fixed32 entries;
all histograms and counters are fixed-size arrays/u64 fields, with no per-region
allocation or per-candidate/per-region trace. Environment lookup and summary
formatting are diagnostic overhead; stderr output has no performance meaning.
Even disabled diagnostic source is an altered binary and cannot be used for
acceptance latency comparisons.

The existing last_doc API exposes finite endpoint versus sentinel selection.
TermScorer's stored-default/native-transform/global bound policy and pair validity
are private. This patch adds no API and cannot distinguish them. It does not
count metadata lookahead entries, loaded-tail rescore cost, frequency saturation,
or native enclosure creation. No runtime correctness or JSON parse success is
claimed until root builds and runs the diagnostic separately. Expected sanity
checks for root: sum(widths)=sum(essential_counts)=sum(cap_source_masks)=regions;
skipped_regions=essential_counts[0]; sum(matching_essentials)=scalar_candidates
=scalar_boundaries; gap_ge_2048<=gap_ge_128<=unique_earliest<=scalar_boundaries;
optional_seeks<=optional_probes<=optional_bound_checks.
