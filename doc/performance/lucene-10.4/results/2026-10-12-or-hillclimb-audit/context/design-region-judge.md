# Fresh wider-region architecture judgment

Choose **candidate A, the global replacement certificate, as the cheap first
isolated trial**. Candidate B has a credible deeper numerical range capability,
but it buys potentially tighter bounds with linear metadata traversal and a much
larger verification surface. A can settle whether removing optional block caps
helps enough to justify another range owner, without changing scorer APIs or hot
clause layout. This is an experiment ordering decision, **not a speed prediction**.

No semantic blocker was identified in either complete package as written. Both
remain unimplemented designs requiring root review and runtime proof. A can lose
badly when global replacement promotes dense clauses to essential status. If it
does, a cheap loss is informative; it does not prove that B will win.

I read the rubric, both whole-shape designs, grounding report and full diagnostic
analysis end to end. The analysis was decoded and printed in compact separate
pieces covering all top-level keys and all 10 largest-region query records, with
no truncation. I inspected accepted 06's scorer policy, current-block bounds,
batch reconciliation, SkipReader fields/traversal and current OR preparation.
Only this judge report was written. No source/design edits elsewhere, build,
test, worker, profile, latency, compression, browser, children or commit job ran.

Parent is accepted 06 `a674aaf2f6187f3010a135dc25fdacf9cab27e53` with the frozen
v11 index. Root reports attempt 10's 17 targeted fixtures and three exact gates
green; its pilot outcome is pending. Attempt 11 is separate and unmeasured.
Neither is a dependency of this wider-region judgment.

throughput checkpoint: n/a, read-only architecture judgment

## Equal-weight rubric, 1–5 per criterion

| Criterion | A | B | Whole-shape assessment |
| --- | ---: | ---: | --- |
| Exact semantic proof | 5 | 5 | Both preserve pre-cleanup selected bounds, recompute the complete widened partition, keep original-count conservative sums and literal incoming publication, and freeze certificates through pending replay. A uses existing global validity; B specifies trusted policy, complete metadata coverage and global fallback. |
| Ownership and interface | 5 | 5 | A keeps the full region lifecycle and cold endpoint metadata in the existing OR owner. B correctly hides clone traversal, starting-point validity, provenance and range-cache identity in TermScorer rather than exposing caller choreography. Neither changes real cursors to inspect future bounds. |
| Concrete performance mechanism | 3 | 3 | Both target observed optional-owned caps with one endpoint proposal. A adds endpoint/min/final-prefix work and may multiply candidates; B adds linear pair parsing/enclosure work and can still fall back globally. Counters do not reveal net saved latency, candidate growth, or hypothetical metadata scan cost. |
| Compactness and scope | 5 | 3 | A changes one preparation owner with fixed 128 B scratch and unchanged hot ClauseState. B adds an internal scorer API/policy operation and a DocId field that may grow hot records 24→32 B; clone/Arc work and existing cold native-envelope allocation are additional costs. Both keep disk format and posting scratch unchanged. |
| Meaningful verification | 5 | 5 | Both explicitly require actual widening, later-block and stale-zero winners, exact callback traces, profile/configuration fallback, pending refill and deletion behavior. B additionally makes real metadata/decoder/position preservation explicit; A can borrow that evidence without borrowing its range arithmetic. |
| **Total** | **23** | **21** | Select A for lower isolated cost, retaining B as a structurally distinct later trial. |

These scores do not quantify expected throughput or probability of winning.
Giving B 5 for proof/interface means its proposed contract is coherent, not that
an implementation of its numeric policy or start guard has already passed tests.

## The diagnostic supports a question, not an answer

The observed 103 long-OR invocations prepared 95,315 regions and visited 907,568
finite selected blocks. Optional-only caps account for 84,881 regions, 89.05%;
50.26% of regions are narrower than 128 physical addresses. Those facts establish
that eventual optional clauses often constrain accepted 06's preparation cadence.
They do not establish that their caps can be removed cheaply.

First, 23,387 regions are complete skips; both proposals deliberately retain the
old skip endpoint, so their preparation count is not removable by this shape.
Second, the counts are strongly skewed: query 920's 21 terms contribute 30,242
regions and 631,838 finite selections, with 9,893 skips. That query has only 18
single-essential regions, so the familiar batching argument cannot explain its
dominant preparation volume. Its scalar path has 47,771 unique earliest candidates
but only 7 capped gaps>=128. Query 919 instead has 8,690 single-essential regions;
query 793 has 5,480, mostly very short nonempty batches; query 917 is predominantly
multi-essential. The benefit and cost can differ across those regimes.

The aggregates do not cross-tabulate optional cap ownership with nonempty old
suffixes and hypothetical essential-only endpoints. They do not measure which
profiles supply usable pairs, how many skip entries a proposed range would cross,
or which clauses would become essential under A/B. The counter binary alters
runtime, and counts are unweighted by query latency. Neither the 89.05% cap share
nor the short-batch distribution predicts a win. Keep per-query regressions and
the frozen balanced ruler intact.

## Semantic obligations shared by both proposals

The old local preparation must retain each selected bound and its exclusive
coverage end before cleanup. Tentative prefix construction can use a virtual zero
when the authoritative first unread doc is at/after old_hi. It cannot overwrite
the only saved bound with that zero before final_hi is known. A clause starting
between old_hi and final_hi can match in the expanded region; reusing its old
cleanup zero would skip a later winner. Both packages eliminate that error in
the state shape rather than asking reviewers to remember to restore lost data.

The old suffix proposes final_hi only once. Rebuild every final effective bound,
prefix and essential partition before any candidate replay. A formerly optional
term can become essential. A newly essential term whose selected end is below
final_hi is safe only because its replacement bound certifies the wider range;
it must not reuse the short local certificate merely because its role changed.
No fixed-point width loop or mid-callback repartition belongs in the first trial.

For either proposal, zero requires actual first unread>=final_hi or genuine
exhaustion. A stale decoded doc below lo is not absence. Existing preparation
selects metadata at max(lo,doc); existing actual seek(lo) must reconcile any newly
essential stale cursor before replay. Actual `ClauseState.doc` remains the real
first unread, never a buffered replay head or hypothetical metadata endpoint.

The numerical certificate stays fixed until all batch rows drain. Callback
threshold growth changes every subsequent optional probe and strict publication
immediately; it does not authorize discarding retained leaves, changing roles,
or expiring the region early. Physical deletes remain inside the endpoint range
and collector rejection can leave the threshold unchanged. Original pruning
preference, incoming f32 leaves/f64 fold/f32 publication and original num_terms
inflation remain intact. Admission/unsafe-domain canonical fallback must remain
before metadata mutation.

## Why A is the simpler first owner

A uses already admitted query-global maxima whenever a selected end cannot cover
final_hi. There is no raw-pair reinterpretation or new provenance admission.
Covered clauses retain their trusted pre-cleanup local bound; uncovered clauses
receive a conservative global bound; only first-unread>=final_hi becomes zero.
This proves all final leaves enclosed without a second numerical policy path.

Its query-local 32-entry endpoint scratch is appropriate because endpoint values
are consumed during preparation, while clauses remain hot during scoring/probes.
A selected_end field in each clause would keep cold data in every hot record.
The plain scratch also avoids methods that scatter preparation, width choice and
restoration into a temporal caller protocol. The existing OR owner is already the
right depth for those operations. No additional range helper or cache lifecycle
is justified in A.

This minimality does not make its runtime cost negligible. Every region writes
n endpoints; every nonempty old suffix scans its ends; widening adds a full final
bound/prefix pass and another partition. Short no-widen regions still pay some
overhead. Global bounds can promote very dense old optionals into essentials,
**explode scalar candidate volume**, increase matching scans and optional work,
or erase the old single-essential route. A sparse tail can choose max_doc and
make that effect extreme. Query 920 is an explicit adverse regime to retain.
One source mechanism may lose across the entire workload even with fewer regions.

## Why B remains credible, and why it comes later

B's strongest architecture choice is placing trusted range evaluation inside
TermScorer, where immutable query weight, selection provenance and validated
TermBlockBounds already live. Its clone is only SkipReader metadata; calling real
seek_block at a future endpoint would risk skipping unread postings while decoded
doc looks stale. The proposed internal start guard returns global when the clone
could begin after relevant unread postings. This is a conservative boundary,
not a backward-recovery scheme. Initial identity must use remaining_docs/doc_freq,
including legal doc 0, rather than interpreting byte offsets as block numbers.

The range must include the first partial full block and every covered full block
through up_to-1. Nonempty VInt tail, absent/saturated pair, incompatible/default
average mismatch, legacy custom profile, or unusable native enclosure returns
global; empty EOF is separate. Native custom bounds reuse the validated input
envelope, not an arbitrary custom rescore of one stored norm/TF pair. Its result
must never populate the real selected-block numeric cache. These choices make
the proposed API deeper and prevent metadata/provenance information leaking
into the OR caller.

However, this is **linear metadata and pair-policy work**, not the port's coarse
level 1 impact lookup. Every cloned-reader advance parses entries and updates
offsets; repeated ranges can repeat parsing. OwnedBytes sharing still incurs Arc
work; native envelope initialization has its existing cold allocation and cache
scan. Missing/tail/global domains can spend some traversal and then produce A's
loose bound anyway. Even a tighter maximum need not preserve old optionality.
The added ClauseState endpoint may expand 24 B records to 32 B, affecting all hot
candidate/probe loops. Host layout and total costs require root measurement.

B would be a sensible later independent trial if A's losing evidence shows
large candidate/essential promotion while genuinely removing region setup, or
if deeper measurements show ranges are short and trusted pairs dominate. A loss
alone does not prove either premise. Do not secretly add B's scan as a rescue
inside A, add a scan budget, widen repeatedly, or change format/cache scope.

## Graft exactly two verification ideas from B into A

1. **Prove every actual widened certificate covers literal unread leaves.**
   In test-only region evidence record old/final endpoints, authoritative docs,
   retained selected ends, final per-clause bounds and final roles. From independent
   untouched scalar scorers, enumerate relevant positive-TF leaves in each observed
   widened half-open range and compare them to those final bounds. Include the
   first-unread-between-old/new witness, later high-TF/low-norm block, configured
   native queries and profile mismatch fallback. This exercises A's existing
   local/global policy; it adds no metadata-range numeric operation.
2. **Prove real state and replay progress across expanded intervals.**
   B's untouched-clone remaining `(doc,TF,positions,scorebits)` trace is stronger
   than checking only a doc mirror. Adapt it to A's widened caller: verify new
   essentials reconcile stale selected blocks at lo, a globally bounded former
   optional crosses multiple loaded blocks, a sole essential refills batches
   without overwriting pending rows, and exact-final_hi docs wait for the next
   region. Include real indexed deletion rejection/current Top1 and independent
   full callback traces. A introduces no lookahead, so its ordinary cursor moves
   are expected; the proof is that they follow canonical remaining postings and
   preserve position/frequency alignment, not that the replay cursor stays fixed.

Keep A's cold 128 B scratch, virtual cleanup and one existing-owner preparation
block. Do not graft B's selected_end hot field, range scorer API, cloned-reader
scan, pair helper or native policy operation. Their stronger bound is a different
runtime mechanism and belongs to a later isolated unit.

## Root agreement and runtime boundary

Implement A only after root reads both packages and this judgment and agrees the
whole shape. Review actual source, especially cleanup ordering and final partition
construction, before running it. Both designs' proposed assertions and fixtures
are plans, not runtime evidence. Root owns targeted/broad checks, 3×1227 exact
gates, release build, frozen pilot and balanced AB/BA acceptance. Keep actual-port
comparison, full 1201 regressions, configured-oracle limitation, collector-work
limitation and binary/scratch accounting. Diagnostic route changes can explain
the outcome separately; they earn no latency credit. Reject a measured loss in
full and retain B as an unimplemented alternative rather than declaring victory.
