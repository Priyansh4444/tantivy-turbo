# Attempt 01: fallback-only fixed-two local regions

Edits ready for root formatting, actual diff review and all runtime gates.
No build, test, benchmark, profile, commit, or child task run by implementation
agent. Existing five-query two-term baseline motivated this exact fallback.

Changed production files:

* `src/query/boolean_query/block_wand_union.rs`: physical max_doc argument;
  pre-mutation canonical fallback for unsafe/signed/exceptional domains;
  final post-warm generic block_wand call replaced by private fixed-two region
  kernel; stack metadata and cursor-owned mirror updates.
* `src/query/boolean_query/boolean_weight.rs`: pass reader.max_doc() at the
  actual two-term caller. The old module test helper also receives physical
  max_doc from its fixture, rather than reconstructing it from postings.

Existing warm-128, globally single-essential, and existing warm/global
intersection bodies are unchanged. There is NO intersection handoff from the
new region kernel. Rising thresholds retain a conservative local mode until
the physical half-open end. A local required-membership role expires there.

Two stack records preserve fixed incoming scorer identities and cache decoded
doc, immutable maximum and cost. Only actual seek/advance methods update each
mirror. Debug assertions surround shallow selection, bounds, leaf scores and
movement. The originally selected finite cap is retained through tail-floor
reconciliation. An optional global certificate can omit its boundary only when
exactly one term is globally optional, preserving the established single-
essential density policy after threshold rises inside the new kernel.

Local modes: direct union when each local maximum can win alone; one essential
when only one can; locally required optional membership driven by cheaper
postings when neither can win alone but their conservative sum can. Whole
regions whose sum cannot exceed threshold are skipped without actual decoding.
Exact scores replay incoming f32 leaves as f64 sum and one f32 cast. Bounds are
forward nonnegative sums; no subtraction, approximation or mutable reordering.

Six bounded new tests compare every callback's doc and raw score bits against
independent BufferedUnionScorer+SumCombiner traversal. Direct region calls
guarantee kernel coverage; entry calls exercise preserved warm/global behavior.
Fixtures cover essential identity reversal, local required-mode expiry before
a later one-term winner, full blocks/staggered tails/exhaustion, rounded midpoint
neighbors and strict ties, warm winners at127/128/129 with global handoff,
signed/exceptional fallback, both incoming orders, and collector rejection with
trailing physical IDs. Two opt-in thread-local mode observers prove reversal
and required-mode expiry; all observer code is cfg(test) and absent in release.
The collector-rejection fixture emits actual host TwoTermState size for root's
scratch receipt. Existing randomized near-equality tests remain intact and
their helper now receives physical max_doc; frozen raw-bit gates add coverage.

No new public API, index bits, per-region allocation, batch API, local sort,
callback-driven repartition or changes to longer-OR engine were introduced.
The reduced first attempt deliberately leaves terminal intersection transfer
out, superseding the larger shape's suggested handoff due fresh judge's
cursor-staleness objection. Admission and max_doc are correctness dependencies
of the region kernel, as approved by the fresh judge.

Root should reject the attempt on any raw-bit/cursor failure or paired material
regression, retain its source/binary/receipts, and only then move to another
mechanism. No performance gain is claimed by this source-only implementation.
