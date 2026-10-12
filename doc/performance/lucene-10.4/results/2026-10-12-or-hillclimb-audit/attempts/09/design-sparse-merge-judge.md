# Fresh sparse-merge design judgment

Recommend **posting-bounded heap v2 as the base**, narrowly, against the packages
as written. SingleLead is the smaller experiment and remains a credible
alternative, but its capped 2048-address selector is poorly matched to the
accepted source's region boundaries. Heap v2 has the stronger complete owner
interface and removes scalar essential scans on interleaved as well as separated
posting streams. This recommendation is an architecture judgment, **not a
prediction that the heap is faster**.

Both complete stable packages and `design-next-rubric.md` were read end to end.
The rubric path in the task's compressed text resolved to that existing file.
I inspected the relevant accepted production source and existing oracle, without
editing production code or running tests, builds, profiles, browser work,
compression, or child agents. The only created artifact is this report.

throughput checkpoint: n/a, read-only investigation

## Scores: each criterion is 1–5

| Rubric criterion | SingleLead | Heap v2 | Reason |
| --- | ---: | ---: | --- |
| Exact semantic proof | 5 | 5 | Both retain literal f32 leaves, incoming-order f64 publication, conservative original-count bounds, current monotone callback thresholds, fixed certificates, and pending candidates independent of actual cursors. |
| Interface depth and extension safety | 4 | 5 | SingleLead's private enum is small and sensible, but the hot loop must coordinate dispatch, refill, candidate completion, and retirement. Heap v2's scoped owner makes head repair, equal-doc coalescing, and drained-owner refill one operation, while leaving optional probes and callbacks outside. |
| Concrete mechanism and losing workload | 3 | 4 | SingleLead removes three suffix scans only inside selected isolated spans; 2048 can suppress activation under actual region caps. Heap replaces the scans for every multi-essential stream, including interleaving, but adds copies and O(log e) repair per matching posting. Neither addresses repeated short region setup. |
| Compactness and scope | 5 | 3 | SingleLead reuses the accepted batch and contribution row with O(1) metadata and no additional allocation. Heap adds a lazy n-buffer allocation and approximately 37,000B total array/record payload at n32, versus accepted06's approximately 3,208B foundation, excluding headers and frames. Both preserve bounds, preference order, and public APIs. |
| Verification quality | 4 | 5 | Both require independent full callback traces and branch evidence. Heap additionally spells out valid indexed positive TF in both scoring profiles, multiple populated owners, rank-tie known order, actual deletion integration, suffix-role reuse, mirror/head assertions, and measured host layout. SingleLead's boundary tests are good but its deletion coverage is described primarily through callback modeling. |
| **Total** | **21** | **22** | The margin is small; runtime can reject either design. |

## Grounding that determines the judgment

The inspected accepted tree is
`/home/pronsh/Coding/playground/search/tantivy-pr2937`, HEAD
`a674aaf2f6187f3010a135dc25fdacf9cab27e53`. The next implementation must start
there after root removes rejected08. Its passing 19 fixtures and three 1227-case
gates do not overcome the root-reported 12.79%/13.55% pilot regression. The
accepted06/actual-port long-OR ratio of approximately 1.74–1.81 remains open.
These measurements are supplied status evidence; this judge reran none.

* `src/query/boolean_query/or_maxscore.rs:9–51` owns the authoritative cursor
  mirror through `ClauseState` mutations. `:119–136` fixes pruning preference
  order separately from incoming scorer/leaf order. Neither candidate needs a
  reordered scorer vector or per-clause replay doc.
* `or_maxscore.rs:178–235` shallow-selects every nonglobal live term, caps `hi`
  by selected block ends, clones local maxima, clears bounds for real positions
  beyond `hi`, and establishes the essential suffix. A locally optional
  nonglobal term can still supply the earliest cap. This constraint survives
  either merge replacement.
* `or_maxscore.rs:236–295` reconciles essentials below `lo`, batches only the
  one-essential case, and otherwise scans for minimum and matching leaves.
  `:325–331` is the third matching-cursor advance scan. `:297–323` is the exact
  optional/probe/publication body that must remain shared.
* `src/postings/segment_postings.rs:25–75` consumes one decoded block suffix
  below an exclusive cap. It reconciles unloaded shallow-selected blocks,
  copies real posting prefixes, and leaves the first unread doc. Zero fill
  follows reconciliation to a doc at/above the cap or termination. A successful
  fill may already advance beyond the certified region while retained leaves
  remain.
* `src/query/term_query/term_scorer.rs:13–33,51–69` supplies the existing 128-posting
  scored scratch; `src/query/bm25.rs:568–614` confirms literal scalar-expression
  leaf arithmetic for both legacy and native profiles. Batch leaves stay valid
  after the scorer advances, but shared scratch must be copied or exclusively
  retained before its next fill.
* `search/lucene-rs-idf-fix-oct06/src/search/bulk.rs:53,223–299` has a 4096-address
  window, a two-head 2048 separation test, and a competing bitmap route.
  This explains its cutoff. It does not make 2048 an amortization threshold for
  our block-bounded caller. Its score accumulation is also not our publication
  arithmetic.
* Accepted `or_maxscore.rs:388–404` already compares every callback doc and score
  bits against independent `BufferedUnionScorer`/`SumCombiner`. Its midpoint
  fixture at `:637–665` uses positive indexed TF; retain that valid construction
  and prove the new route actually handles the tested candidates.

## Semantic and interface audit

There is no identified semantic blocker in either design as specified. For
SingleLead, the two-minimum scan must count ties as two occupied positions.
`min(hi, second_doc)` excludes the tie boundary, so no unrecorded essential leaf
can appear in a selected span. The shared completion body consumes the retained
leaf at its original ordinal; scalar fallback consumes the minimum already
computed by dispatch. Pending rows must be retired before redispatch or region
completion. Storing rank, cap, length, and next position in one exhaustive replay
variant is materially safer than adding flags to the existing boolean/index
state.

For heap v2, `(doc, slot)` ordering is essential, not cosmetic: repeated root
consumption of the same document visits matching leaves in fixed ascending
preference order, preserving pruning-known accumulation. An owner's replacement
head is strictly later than the consumed document, so root-down repair is enough
and cannot hide another equal-doc owner. All matching leaves enter the incoming
row before `next_candidate` returns. Its immediate refill may advance actual
cursors before the callback; that is safe because the literal leaf was copied
to the row and the certificate is fixed. Importing SingleLead's later retirement
timing mechanically would add caller protocol without improving this proof.

The region borrow prevents metadata repartition during replay. It does not by
itself prevent arbitrary mutations through the transient full scorer slice;
the caller must continue to use it solely for the unchanged optional suffix
between merge calls. That is a real internal contract, not proof that Rust's
borrow checker alone certifies the entire query. Keep the mirror assertions at
actual mutations, and do not store replay heads in `ClauseState.doc`.

Neither an exhausted actual cursor nor `doc >= hi` proves replay exhaustion.
SingleLead needs drained replay state; heap needs an empty head heap with every
owner empty and no unread essential below `hi`. Callback threshold growth changes
the next candidate's optional/publish decision immediately, but does not expire
the fixed numerical certificate or reclassify pending leaves. Deleted physical
docs remain in both mechanisms and collector rejection may leave the threshold
unchanged.

## Base, grafts, and rejections

Use heap v2's single scoped region owner, retained posting buffers, and fixed
head ordering as the base. Keep the existing one-essential batch route. Do not
combine heap, bitmap, scalar multi, and gap-dispatch mechanisms in one trial.

Two useful SingleLead inputs to carry across:

1. Preserve its explicit **one common completion body** for both accepted
   single-essential batching and the new multi merge. Heap v2 already states
   this intent; treat it as a synthesis constraint that must survive code fill-in.
   Avoid a completion helper that merely takes the entire query state or two
   duplicated optional/publication blocks.
2. Adapt its **exact runner-up transition fixture** to heap ownership: a unique
   owner emits 127/128/129 postings, its consumed suffix meets a document shared
   with two other owners, and another owner remains pending. Test both the
   shared doc below `hi` and the doc exactly equal to `hi`. Verify one exact
   callback, preference-order known accumulation, and the incoming leaf fold.
   This concentrates refill, tie coalescing, and region boundary obligations
   that separate generic overlap and block-length cases can miss.

Reject SingleLead's 2048 selector in this heap trial: it adds a second multi
protocol and neither reduces retained-buffer payload nor fixes short regions.
Reject changing region caps, threshold-driven repartition, optional batching,
packing fields, or adapting the port's score sums. Those change additional
mechanisms and weaken attribution. Rejected08's semantic pending-cursor cases
remain valuable fixtures; its matrix is not a foundation.

If root instead prefers the lower-scratch SingleLead experiment, the strongest
revision is to choose a fixed initial capped span such as **128**, recorded
before implementation, and replace its classifier cases with 127/128/129 plus
the uncapped-large/capped-small case. The source-grounded reason is the existing
128-posting replay unit and full-block region caps, rather than half a competing
4096-address bitmap. Physical addresses still do not count postings; 128 can
select a one-hit span and is neither a measured optimum nor a speed claim. This
would be a revised candidate, not the 2048 package scored above. Do not lower it
mid-trial or widen certificates to manufacture activation.

## What still requires runtime proof

Heap v2 pays approximately 8 copied bytes per posting before later buffer reads,
one extra lazy allocation, and root repair for each matching owner. Dense overlap
can turn O(e) scalar matching work into O(e log e) heap work. For e2, two scalar
heads can beat heap machinery; clipped 1–2-hit regions can fail to amortize batch
setup and copying. Its n32 payload is also larger than rejected08's approximately
20,632B matrix foundation. These costs are reasons to reject it if measured,
not an architecture proof that it loses. Likewise, eliminating scan instructions
does not prove closing the current port gap.

Root's existing independent callbacks, valid-TF/strict-tie cases, pending-real-
cursor cases, dynamic thresholds and actual deletion integration must pass with
test-only route evidence. Preserve shallow full/tail reconciliation and dirty
region role changes. Measure actual host scratch and buffer lengths in semantic
fixtures rather than inferring them from proposed structs. Then use the frozen
default and two configured exact gates, mixed pilot, and balanced AB/BA whole-
query ruler with existing negative controls. No query exclusions and no release
observer. A pilot regression ends this mechanism; another exact pass alone does
not establish a win.

The two designers produced distinct complete artifacts with no dropout. The
source audit verifies their ownership claims at design level. Implementation
correctness, scratch layout, activation/amortization, and speed remain unverified
until root performs the authorized implementation and runtime acceptance work.
