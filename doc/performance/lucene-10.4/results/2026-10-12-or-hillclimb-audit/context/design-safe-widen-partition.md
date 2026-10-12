# Design A: publish a wider certificate only with the tight partition

## Caller usage first

`or_maxscore` continues to own preparation and replay in one function. At a
drained boundary it constructs the accepted11 tight certificate and its exact
rank prefix, then considers source13's wider endpoint. It computes proposed
bounds for old optional ranks into cold scratch and checks their prefix without
changing `hi`, `first_essential`, `ClauseState.local_max`, or the live prefix.
It publishes the proposal only if its final first essential rank equals the
tight first essential rank. Otherwise it materializes the tight virtual zeros
and enters the existing replay with the original endpoint, bounds and partition.

The caller does not obtain a proposal object or call a new pass-through module.
The only scorer capability remains source13's existing crate-private
`range_max_score(lo, proposed_hi)`. Its numeric provenance and owner contract
remain inside `TermScorer`; OR does not interpret stored metadata pairs.

This is one replacement admission policy for source13's one-shot proposal. It
does not include physical-width capping, a density cache, an entry budget,
preference sorting, a new score fold, or the rejected contribution mask.

## Grounding and phase

Design-only package: ground and sketch complete; root owns synthesis/selection,
implementation and any scrap decision. No runtime or source edit was performed.
Read the full design brief, implementation report, review and static receipt;
read the complete actual 1,699-line source13 diff and actual scorer helper; trace
accepted11's actual Git production OR body through retirement. The diff and
source checksums were independently read back with a read-only Python guard.

Accepted parent: `43f06833a325d2a18c03bd7e4fa65f2f27944292`.
Held source13 OR: `1a1ce59b330b4a207d353385bf90cac96333cd4c6da49ed52a7c40cd5bb30804`.
Held scorer: `21b05a2b6513223a81613ee0f40c4a31f6990a427bb0d4b60324154a55753cdb`.
Actual diff: `589305ea0face0e33545393ef474edf177c0af9803b8a9526113f37c6f6bfb4a`.

Accepted11 mutates real shallow selection during tight preparation, reconciles
stale selected tails when necessary, obtains the current-block bound and then
zeros a clause only if its authoritative real first unread doc is beyond `hi`.
Source13 preserves the raw selected bounds while forming those zeros virtually.
Its proposal can restore an old zero or replace a crossed old optional bound
with `max(raw, range)`. It currently publishes every proposed bound and rebuilds
the partition before replay, allowing a crossed optional to become a driver.

Source13's published shared replay is independently byte-identical to accepted11
(4,233-byte span SHA `63086168e3a7fbc768ca8d6529189aacc2d736d1c414c9d00b5796741462093e`).
Its cached global certificate/density span and hot ClauseState are also unchanged.
These are the spans to preserve when implementing this design.

Root's frozen pilot shows about 8.5% less aggregate long-OR latency with source13
and severe retained sparse-query regressions. This establishes a reason to try a
different admission policy, not causal evidence that promotion accounts for the
observed time. The gate structurally prevents that promotion while retaining
some wider regions. It provides no per-query timing floor.

## State and signatures

Keep all existing live state, the existing `[DocId; 32]` selected-end scratch,
the original `Vec<f64>` prefix, the original contribution vector and batch.
Add exactly one cold proposed certificate array:

```rust
let mut proposed_bounds: [Score; 32] = [0.0; 32];
// Scalar preparation temporaries, not persistent cursor state:
let tight_hi: DocId;
let tight_first_essential: usize;
let proposed_hi: DocId;
let mut proposed_sum: f64;
let mut proposal_fits: bool;
```

The array is 128 logical bytes with the actual `Score = f32`, so total extra
cold arrays relative to accepted11 are 256 logical bytes. Relative to source13
this is 128 bytes. Do not expand ClauseState or store bounds in a new hot record.
The source arithmetic does not establish a compiled frame or RSS difference.

Keep this signature and implementation byte-for-byte:

```rust
impl TermScorer {
    pub(crate) fn range_max_score(&mut self, floor: DocId, up_to: DocId) -> Score;
}
```

No new production type/signature/module is needed. Tests may use a private
`ProposalEvidence` value with attempted endpoint, tight/proposed first rank,
chosen endpoint, raw/proposed/chosen bounds, real doc mirrors and chosen prefix
bits. This observer exists only under `cfg(test)` and is not a release policy.

## Preparation pseudocode

The body below replaces only source13's proposal-publication block. The global
certificate/density decision and tight selection remain source13's existing
accepted11 behavior; its initial tight prefix still uses virtual cleanup.

```text
prepare selected_ends and raw ClauseState.local_max as source13 does
build tight prefix in the existing prefix vector, rank order, using
    effective = 0 if real doc >= tight_hi else raw local_max
compute tight first_essential with existing original-n SmallScoreSumUpperBound
if tight first_essential == num_terms:
    skip ONLY tight_hi, as accepted11 already does
    continue

tight_hi = hi
tight_first_essential = first_essential
proposed_hi = min(selected_ends[tight_first_essential .. num_terms])

admit = false
if proposed_hi > tight_hi:
    proposed_sum = +0.0 f64
    proposal_fits = true
    for rank in 0 .. tight_first_essential:
        raw = clauses[rank].local_max             // never overwritten here
        proposed =
            +0.0 if real doc >= proposed_hi
            max(raw, range_max_score(lo, proposed_hi))
                if selected_ends[rank] < proposed_hi
            raw otherwise
        proposed_bounds[rank] = proposed
        proposed_sum += f64(proposed)             // same rank sequence
        if upper_bound.score(proposed_sum) > threshold:
            proposal_fits = false
            break                                // no later clone scans needed
    admit = proposal_fits

if admit:
    hi = proposed_hi
    // first_essential is unchanged: the proof certifies exact equality.
    for rank in 0 .. num_terms:
        chosen =
            proposed_bounds[rank] if rank < tight_first_essential
            +0.0 if real doc >= hi
            clauses[rank].local_max otherwise
        clauses[rank].local_max = chosen
        prefix[rank + 1] = prefix[rank] + f64(chosen)
else:
    // hi, first_essential and prefix still are the tight originals
    for clause in clauses:
        if real clause.doc >= tight_hi:
            clause.local_max = +0.0
    // Other raw bounds are already the exact accepted11 tight bounds.

existing essential reconciliation/replay/optional completion/publication/
callback/retirement span, unchanged
```

The scalar optional-prefix sum avoids another 33-entry f64 prefix array and a
complete speculative prefix/partition pass. On an accepted proposal, the commit
pass reconstructs all required replay prefix entries in the
same rank order with identical conversion/addition operations, starting from the
unchanged prefix[0] = +0.0. On rejection, the original tight prefix is untouched.
Do not overwrite the live prefix with a speculative sum then reconstruct it on
rejection. Do not copy an entire TermScorer to recover a live cursor.

Only old optional ranks need proposed scratch; old-essential selected ends all
cover proposed_hi by construction, so those ranks never require range lookahead.
Their commit bounds are raw selected bounds, except real doc >= proposed_hi still
proves zero. Scratch beyond the optional prefix is never read, and an early
rejection never reads a partially computed prefix on the next region. With k=0
an expansion cannot occur: the old-essential minimum equals the original tight
minimum. There is no added numerical threshold or tuned early-exit budget.

## Proof obligations and ownership

**Chosen certificate coverage.** Tight fallback is accepted11's literal bound
and cleanup rule. For an admitted proposal, doc >= proposed_hi proves no unread
leaf in the interval; an uncrossed selected end covers the full proposal; a
crossed clause receives max(raw, source13 trusted range certificate). Global
optionals have selected end max_doc and cannot cross. Source13's helper either
certifies each included complete block through the existing policy or falls
back to immutable global for unresolved nonempty coverage. Every admitted
certificate therefore covers every still-relevant leaf.

**Partition equality.** Let k be the tight first essential rank, so its tight
prefix[k] fits the existing bound test and prefix[k+1] exceeds it. Every proposed
bound is at least its tight effective bound: old tight zeros remain zero or
become positive; nonzero raw bounds are retained or increased by max. In
particular rank k has selected end >= proposed_hi; its raw certificate remains
unchanged and its effective bound cannot decrease. Sequential nonnegative f64
addition and the existing upper-bound transform are monotone.

If every proposed optional prefix through k fits, all ranks before k remain
optional. Adding proposed rank k gives a sum >= the original tight prefix[k+1],
so that bound still exceeds the threshold and rank k remains essential. The final
first essential is therefore EXACTLY k, including strict/equality cases. If any
proposed optional prefix exceeds, the proposed first essential is earlier than k
and the proposal must be rejected; no later nonnegative leaf can make that prefix
fit. The early gate is exactly the equality policy, not approximate admission.
No subtraction or changed original term-count factor exists.

The existing SmallScoreSumUpperBound transform is monotone over this admitted
nonnegative finite-leaf sum domain. Threshold +infinity is handled by the existing
global certification early return; -infinity cannot admit optional ranks.
Preserve initial NaN/unsafe-domain fallback before preparation. Do not use a
reordered or subtracted optional sum to decide equality.

**Rollback boundary.** During proposal, clause raw bounds, live prefix, hi, rank
partition and real docs are untouched. The helper only moves a SkipReader clone,
never real skip selection/decoder/postings/positions/current-block score cache.
Consequently rejection requires only materializing the preexisting tight zeros;
there is no rollback seek and no speculative leaf/score cache.

The helper may initialize source13's existing PendingNative policy into its
validated NativeTransform or Global variant. That is an allowed deterministic
query-owned policy transition, not a cursor transition. Entire TermScorer object
bytes need not match an execution that never attempted lookahead. Do not claim
byte-identical scorer policy state after rejection. Tight certificate/prefix/
partition/cursor/replay must match; scalar arithmetic is unchanged and future
block queries reuse the same deterministic provenance policy. Retain source13's
interleaved range/current-block cache and complete remaining positions/raw-score
fixtures to establish this boundary.

**Single batch and scalar ownership.** Equality preserves the exact essential
suffix membership and hence preserves whether this boundary starts single-driver
batch replay or multi-driver scalar replay. No retained batch exists while the
proposal is evaluated. The last rank is old essential, and proposed_hi is capped
by every old essential selected end. An admitted sole driver cannot cross its
own selected full-block end inside a certificate. Actual full/partial suffixes,
exclusive first-unread heads and later-region refills remain the realizable
batch witnesses. A rejected promoted clause can become essential in a later
tight region; the design does not permanently suppress it.

**Publication and strict score.** Leave essential leaf extraction, optional
completion, contribution zeroing, original incoming-order f64 publication, final
single f32 cast, `score > threshold`, collector callback and nondecreasing
threshold assertion byte-identical. Widening uses the boundary threshold only;
live callback thresholds still apply to every replay candidate. Deleted winners
remain collector-owned and do not raise the threshold if rejected.

**Tails, EOF and address arithmetic.** Preserve accepted11/source13 selected
end calculation `last_doc.min(max_doc - 1) + 1` inside lo < max_doc. Empty max_doc
never enters it; TERMINATED/full/tail endpoints cannot overflow. The proposed
endpoint is the same minimum of old-essential ends, hence tight_hi <= proposed_hi
<= max_doc and lo < tight_hi. No new arithmetic or sentinel increment appears.
Keep helper's floor/up_to validation before up_to - 1, initial doc0 exception,
too-late selected metadata guard before EOF inference, and missing/Basic/tail/
saturated TF/global-policy fallback. No tail decoding is introduced.

## Realizable independent fixtures

1. **Reject actual promotion, then find the stronger later leaf.** Reuse the
   real source13 native indexed alpha/beta/gamma corpus (beta TF20 at doc150;
   gamma sparse addresses and strong selected full-block leaf). Retain the
   threshold midpoint derived from selected alpha/beta bounds and independently
   literal beta@150, not range helper output. Source13 currently requires old
   optional beta promotion and strict sub-global enclosure. Under A, record that
   this attempted proposal crosses the old beta end and would promote beta, but
   chosen hi/first/bounds/prefix bits equal the original tight values. Require
   the later stronger beta leaf and full strict callback trace to match independent
   BufferedUnionScorer for fixed/Top1/Top10 observers. The old assertion demanding
   published first-region promotion must change because that is precisely the
   rejected policy. Its numeric enclosure and later winning leaf stay required.
   This is a geometric policy adaptation, not removal of a score oracle.

2. **Reject restoration of a virtual zero.** Use the existing actual legacy
   dense0..1023 weak term, future300..899 with TF1000@428, sparse driver0,4,...516
   fixture. Its first tight region zeros the future doc300; source13's proposal
   restores it and promotes it under threshold .015. A must keep the future
   bound zero at the chosen tight endpoint, retain the original selected raw bound
   in observation, reject the attempted wider partition, and later emit the
   competitive TF1000 leaf@428 with canonical bits. Verify every cursor event
   against independent untouched scalar posting lists; no proposal event may
   advance/seek/fill the real cursor. This guards the exact rollback ownership.

3. **Admit real expansion and retain both batch sizes.** Reuse source13's weak
   dense lists with boosts .001/.002 and spaced strong driver i*3 (including
   head382 and competitive partial final TF30) at lengths127/128/129/255/256/257/
   768. Here the weak proposal stays below .01 and the strong driver keeps the
   same first essential. Require actual proposed_hi > tight_hi AND equality,
   choose proposed_hi, actual128-row batch ending exclusively at head382 for
   applicable lengths, later doc382 winner, and competitive partial batches.
   Enumerate literal leaves independently against every CHOSEN certificate;
   check real first unread cursor/TF/raw bits against untouched scalar lists.
   Length128/256/768 need not invent a partial-tail witness.

4. **Admit while restoring an old zero that stays optional.** Use norms[3000]=1,
   dense0..1023 boosted .01, future300..899 with TF1000@428 boosted .001, and a
   600-posting driver at4*i boosted .1, fixed threshold .025. The longer driver
   prevents the unchanged global density policy from admitting both weak global
   certificates: dense cost1024 < 2*driver cost600 (future cost600 likewise).
   Ground the actual admitted global sums and priority order independently first;
   under the existing fixture weight the weak global sum is about .0242 and the
   driver maximum about .22. Tight finite dense end128 lies before future doc300;
   the driver's first full-block end509 lies beyond it. Require old virtual zero,
   positive chosen future bound, equality and actual widening. Assert actual rank
   order and density rejection rather than assuming boosts alone choose rank.
   Independent bound/leaf enumeration prevents dropping the restored term.
   This demonstrates that the admission rule preserves useful zero restoration;
   it does not categorically reject every restored zero.

5. **n32 and strict numeric boundaries, mixed decisions.** Extend actual overlap/
   disjoint weak-term fixtures to32 with a sufficiently small independently
   verified weak sum, then include the promotion corpus and its reverse incoming
   order. Require at least one accepted and one rejected attempt overall. Keep
   existing original-n generic-bound observer, midpoint incoming leaves
   1,2^-24,2^-53,2^-53, strict next_down/equal/next_up thresholds, callback threshold
   jumps, stale shallow full selection, exhaustion, safe custom/native fallback,
   simulated deletions and real1536/1534 alive-bitset/public collector cases.
   Widening occurrence is asserted only for geometry whose literal certificate
   relations permit it. Retain canonical score/cursor oracle for every case.

Test observer logic should describe attempted and chosen intervals separately.
An early rejected attempt can record its first overflowing optional rank rather
than invent a complete uncomputed proposed count. For accepted attempts, test-only
independent recomputation of the final full prefix must confirm exact first-rank
equality. For rejected attempts, assert the observed overflowing prefix implies
promotion and the actual chosen partition remains tight.
For rejected proposals assert tight chosen certificates and prefix raw bits;
for admitted proposals assert every literal unread leaf is enclosed. A blanket
assertion that every attempted proposal is used would repeat the current policy
assumption. A stub that always rejects must fail fixture3/4; a stub that always
admits must fail fixture1/2. Existing scorer range fixtures remain byte-identical.

## Costs, limits, alternatives and synthesis choice

This policy directly prevents changing optional suffix membership inside the
widened preparation and preserves batch/scalar mode. It preserves plausible
region reduction where the optional sum still fits. It does not bound span,
metadata entries parsed, number of optional probes or wall time. A rejected
proposal still pays clone/Arc/drop/linear skip parsing and proposal fold work;
repeated tight regions can repeat rejected lookahead scans. An admitted proposal
can use looser optional bounds and perform more optional work despite unchanged
driver membership. Restored existing essential leaves can also enlarge work.
The original full sampler, severe query losses and self-controls must decide.

Additional scratch is one Score[32], no per-region allocation. Proposals pay up
to O(k) optional bound/fold work, stopping at the first overflowing prefix;
admitted proposals also pay one O(n) commit/prefix pass, rejected proposals only
the O(n) tight cleanup. Old-essential metadata is never scanned by the proposal.
Do not call that a speed gain
without measurement or hide the extra accepted-pass work. Existing once-per-query
native envelope allocation can remain a cost even after a rejected attempt.

Rejected whole-shape alternatives here: mutate live bounds/prefix then undo
(requires fragile snapshots); clone entire scorers (copies real decoding owners
and is unnecessary); cache per-leaf scores to undo proposal (wrong owner and
adds state); update the partition during retained replay (violates fixed batch
ownership); gate on corpus/query IDs or sampled latency (invalid generic policy);
adaptively tune widths or scan budgets (multiple mechanisms and new thresholds).
The separately assigned physical128-document cap is a legitimate alternative
whole policy, not part of A. This package leaves its comparison to root synthesis.

Selection rationale: A has an exact structural answer to optional promotion,
with a small private scratch array and no new API/format/provenance branch. Its
main weakness is speculative scan cost on rejected proposals; it should lose
selection if that cost outweighs the reduced candidate work under the frozen
ruler. No universal Rust/Lucene speed win or complete public API parity follows.

Root alone must implement, independently review, run targeted ownership/policy
fixtures, rebuild, execute all three1227 exact gates, frozen pilot and conditional
full controls. If geometry assertions need revision, preserve failed source/logs
and independent raw callback/leaf/cursor observers; do not relax acceptance.

## Smaller-shape synthesis within A

Root questioned whether full proposed first-rank computation was necessary. The
source-grounded equality proof above permits a smaller shape: fold only the old
optional prefix, stop on its first overflow, then perform one complete prefix
rebuild only after admission. This is equivalent to final first-rank equality
under the existing numeric domain and endpoint rule. It removes speculative
essential writes/folds and can avoid later optional metadata scans after a known
rejection. It keeps the same one Score[32] scratch and exact rollback boundary.
The full-proposal-count shape remains a clear correct fallback if implementation
review finds the monotonic predicate proof no longer holds after a source change;
do not silently broaden the numeric domain or change endpoint selection.
