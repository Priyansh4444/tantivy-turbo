# Grounding: region caps and metadata range bounds

A conservative range bound is possible without decoding or advancing the real
postings cursor: use a query-global maximum, or inspect compatible full-block
metadata through an independent cloned `SkipReader` and take the maximum of the
covered blocks. The latter is a **linear metadata scan**, not an existing
constant-time range-max operation. Missing metadata, tails, unsupported selection
provenance, and unsupported scoring profiles require conservative fallback.

This is grounding for a later independent hypothesis. It selects no architecture
and implements nothing. Accepted06 remains the source baseline; attempt09's heap
runtime outcome is still pending. Root must measure actual region statistics
before choosing a follow-up mechanism, and must not stack it on an unaccepted
attempt. The source alone does not establish a performance gain.

throughput checkpoint: n/a, read-only investigation

All Tantivy references below are under
`/home/pronsh/Coding/playground/search/tantivy-pr2937` at accepted06
`a674aaf2f6187f3010a135dc25fdacf9cab27e53`. Port references are under
`/home/pronsh/Coding/playground/search/lucene-rs-idf-fix-oct06`. Only this report
was written; no implementation, source edits, tests, builds, profiles,
compression, browser work, or child agents were performed.

## Why accepted06 obtains tight regions

`src/query/boolean_query/or_maxscore.rs:146–235` first derives a global optional
prefix, optionally narrows that certificate through its density policy, and then
visits every clause. A globally optional clause receives its global bound and
does not cap the region. Every other live clause shallow-selects at
`max(lo, clause.doc)`, supplies its selected block end, and caps `hi`. The local
prefix is assembled only afterward, and determines `first_essential`.

Consequently a clause that becomes **locally optional** can already have supplied
the tightest cap. Essential status is not known when the initial cap is selected.
The cap is needed because the selected block maximum certifies that block, not
all later blocks. Simply ignoring locally optional block ends after partition
would be unsound: their optional bounds would no longer cover the widened range.
Similarly, extending beyond an essential's selected block needs a replacement
bound for its later postings. The existing scalar/batched candidate source does
not remove that requirement.

Root supplied the all-103-OR accepted06 profile attribution: caller about 80%,
prefix-store `vmovsd` about 6.33% of caller, bound cast/compare about 13%, and
decode about 10%. This suggests measuring region preparation, but these shares
do not identify which caps dominate, the number of repeated regions, or whether
looser bounds would increase candidate/optional work enough to erase any saving.
I did not rerun or synthesize profile numbers.

## Actual cursor and metadata ownership

| Owner | Existing capability | Consequence for a range-bound investigation |
| --- | --- | --- |
| `SegmentPostings` | Owns `cur` and positions bookkeeping; doc comes from its block decoder at `cur`. `read_postings_batch` reconciles shallow selection and consumes one suffix. | Retained leaves can precede the real first unread doc. A metadata operation must not pretend a buffered head is the real doc or consume postings. |
| `BlockSegmentPostings` | Owns decoders, `block_loaded`, one block-score cache, postings bytes, and `SkipReader`. | Its actual shallow selection is forward movement with later decode reconciliation. Lookahead through this object can skip unread blocks even if `doc()` initially looks unchanged. |
| `SkipReader` | Cloneable metadata cursor with current `BlockInfo`, last/previous block ends, remaining docs, byte and position offsets, and a sliced `OwnedBytes`. | A clone can visit future metadata while leaving the actual skip cursor and all decoders untouched. |
| `OwnedBytes` | Clone copies the slice view and shares an `Arc` backing owner. | Cloning metadata does not copy the postings file or decoded blocks, though the shared reference count is real work. |
| `TermScorer` | Holds immutable query weight and a query-bound bound-policy enum. | Selection provenance and native enclosure belong here; the generic cursor's public weight-taking API cannot assume them. |

Evidence: `src/postings/segment_postings.rs:17–75,225–265`;
`src/postings/block_segment_postings.rs:31–40,359–373,415–420,499–530`;
`src/postings/skip.rs:127–196,392–445`; `ownedbytes/src/lib.rs:11–15,121–126`.

`BlockSegmentPostings::seek_block` changes the actual skip reader, invalidates
the current score cache, and marks the selected block unloaded. It leaves the
old decoder contents in place. `BlockSegmentPostings::seek` later loads that
selected block. Thus calling actual `seek_block(hi - 1)` merely to inspect later
maxima can leave a stale first-unread mirror while making earlier unread postings
unreachable on the next reconciliation. `advance_skip_only` has the same
ownership hazard; its existence does not make it a safe lookahead operation.

By contrast, `skip_reader()` returns `&SkipReader`, and `SkipReader: Clone`.
The clone owns independent offsets/state, so `seek`, `advance`, and metadata
parsing on it do not mutate the real selected block. Its traversal is forward
only. A cloned reader taken after actual metadata has advanced cannot recover
earlier blocks or certify retained candidates behind it. It is suitable for
range preparation at a drained boundary, or with an explicitly preserved
metadata starting point; it is not permission to repartition while buffers exist.

## What bounds the current format actually supports

The index format constant is 11 (`src/lib.rs:240`). The format version alone is
not a numeric certificate. Selection comparator provenance and its average come
from field metadata, decoded by `src/index/field_statistics.rs:65–138` and bound
to the real term scorer by `src/query/term_query/term_weight.rs:199–209` through
the reader accessors in `src/index/inverted_index_reader.rs:265–270`.

`BlockMaxSelection` has two meanings: `LegacyTfFactor` and
`NativeSaturationInput`. The native side entry writes the native tag and exact
field statistics (`src/postings/serializer.rs:105–130`). Historical/missing-tag
sources retain legacy selection and its selection average. The writer chooses
one `(norm, TF)` maximizer using the relevant DEFAULT input comparator
(`serializer.rs:534–565`); it does not write a query-independent block-wide
maximum frequency paired with a minimum norm.

`SkipReader::BlockInfo` stores that pair for full BitPacked and Dense blocks.
Basic/no-frequency metadata has zero/absent pair information. VInt tails have
only a posting count, no stored score pair. The TF byte saturates at 255 and
decodes that code as `u32::MAX` (`skip.rs:65–79,142–164,291–381`).

| TermScorer bound policy / domain | Current full-block behavior | Decode-free range consequence |
| --- | --- | --- |
| `StoredDefault`: matching scoring/selection profile, exact DEFAULT parameters, finite safe nonnegative weight, positive finite identical query/selection average | Uses the selected pair through `SkipReader::block_max_score`. Both legacy and native DEFAULT can qualify. | Evaluate each compatible full-block pair with the same immutable weight and max the results. Missing pairs need fallback. Saturated legacy TF cannot use the stored pair; native DEFAULT supports the frequency ceiling. |
| `PendingNative` / `NativeTransform`: native nondefault query, native DEFAULT selection provenance, k1 > 0, safe positive finite weight and positive finite selection/query averages | Lazily builds an immutable input envelope when a complete selected pair exists, then evaluates its conservative upper input and literal native score. | The same validated envelope can evaluate cloned-reader pairs. Envelope initialization is cold work; it is not a free per-range action. Saturated/inconsistent/unusable inputs retain global fallback. |
| `Global`: mismatched provenance, DEFAULT average mismatch, unsupported legacy custom parameters, native ineligible custom domains, or failed enclosure | Global bound on full blocks; loaded-tail policy can still compute an exact maximum. | Query-global maximum covers any range without metadata traversal. It can be much looser, and can change the optional/essential partition and candidate volume. |
| VInt tail, absent pair, Basic metadata, or a complete block with unusable pair | Existing implementation can reduce an already loaded block where its policy permits; otherwise returns global maximum. | A strictly metadata-only scan uses global maximum for the relevant unresolved part. An already loaded tail can be read without decoding, but literal rescoring/reduction is separate work and cannot bound future unloaded tails. Empty EOF tail is distinguishable from nonempty tail by its count. |

The rules are implemented by `src/query/term_query/term_scorer.rs:42–47,104–124,
160–210`, `src/query/bm25.rs:651–791`, and
`src/postings/block_segment_postings.rs:165–272`.

Native custom-parameter enclosure is more than substituting custom k1/b into
the stored pair. `NativeInputEnvelope` reconstructs the selection inverse cache
and upper-bounds the ratio between query and selection inverses across all norm
IDs. It encloses the rounded old input, ratio multiplication, and cast before
the monotone native subtraction (`bm25.rs:304–338,708–780`). The selected TF
ceiling covers only the selected posting. The DEFAULT input-selection maximum
and the ratio enclosure cover other postings. Treating the pair as raw extrema
would discard that proof.

Finite safe normalization alone does not imply transform eligibility. k1 = 0
is outside native envelope admission; b = 1 can produce infinite query inverses
and reject enclosure even when global score saturation is safe. Legacy custom
parameters have no corresponding enclosure here. DEFAULT with a changed average
also keeps its current global policy rather than activating the nondefault
transform. Signed/exceptional leaf domains keep the independent canonical route
before OR cursor mutation; range metadata must not create a new admission path.

The public `BlockSegmentPostings::block_max_score(fieldnorm_reader, weight)`
deliberately returns a global bound for complete blocks and does not use the
fixed TermScorer numeric cache. Arbitrary supplied weights/readers lack selection
provenance. Calling it repeatedly on a clone therefore does not obtain the
desired trusted metadata bounds. Reusing TermScorer's private policy is necessary.
The existing one-block cache is also not a range cache: a maximum over several
blocks must not be written under a single selected-block identity.

## What is cheap, and what is merely decode-free

`Scorer` only exposes `score`; `DocSet` exposes cursor operations. There is no
generic range-bound trait or multi-block maximum API in these interfaces.
The useful pieces are already private: immutable `Bm25Weight`, the trusted
`TermBlockBounds` enum, cloneable `SkipReader`, `selected_input_pair`, and the
native envelope.

Query-global fallback requires no metadata traversal and reuses a weight-derived
bound. Reading one selected pair and block end is metadata-only. A temporary
SkipReader clone copies fixed state and shares backing bytes. Its `seek` loops
through `advance`, and `advance` parses each full block entry and updates
compressed byte/position offsets. There is no multi-level summary or range-max
tree in Tantivy's current skip stream. A scan over k crossed blocks is O(k), even
if it avoids postings/frequency decompression. Re-cloning from the same point
for repeated overlapping probes repeats that parsing. A persistent lookahead or
cached summary could reduce repetition, but introduces another cursor/cache
lifecycle and is only a future mechanism possibility, not a selected design.

Do not clone the entire `TermScorer` or `BlockSegmentPostings` merely to get a
metadata lookahead: those objects include decoder state. The metadata clone is
the narrow ownership primitive. Native envelope construction additionally scans
the norm caches and allocates immutable cache storage; afterward evaluating a
pair still involves the existing enclosed arithmetic. Already-loaded tail
reduction scans scored docs, while a future tail with no pair needs global
fallback if decompression is prohibited.

## Conservative range proof and limits

A possible range-bound contract must specify whether it covers all original
term postings or only still-unread postings. At a drained query boundary only
the latter can still contribute; retired/skipped earlier postings need not be
recoverable through a forward metadata cursor. This distinction matters if the
real cursor is already beyond `lo`.

For a nonempty half-open range `[lo, hi)`, begin an independent metadata cursor
no later than the first block that can contain a relevant unread posting at or
after `lo`. Advance the clone to the block covering that floor. Include bounds
for that block and every subsequent block until the inclusive last doc reaches
`hi - 1`, or the remaining stream ends. Whole-block maxima cover supersets of
the clipped endpoints, so they remain conservative. For any unresolved nonempty
block/tail, use the query-global bound. The maximum of these conservative
per-block bounds covers every relevant literal leaf in the range. Empty streams
and ranges can return zero under the admitted nonnegative leaf domain.

The proof depends on complete metadata coverage, compatible selection, and
unchanged immutable query weight. Never omit the first partial block, ignore a
missing bound as zero, stop at a finite block end below `hi - 1`, or increment
the sentinel. Validate the exclusive range before subtracting one. Bounds can
over-cover consumed prefixes and docs outside the requested range; that hurts
pruning but not correctness.

The bound operation itself need not change real doc, decoded arrays, `cur`,
real skip/position offsets, or pending buffers. Native lazy envelope/cache work
may change query-owned bound policy, but does not authorize postings movement.
If requested during pending replay, cloning the **current** actual skip reader
can miss buffered leaves behind it. Cover those leaves explicitly or use a saved
start/global fallback, or restrict range preparation to the drained boundary.
Maintaining one fixed certificate until all retained candidates retire remains
the simplest existing contract. This investigation does not replace it.

## What the fixed port's range operation actually does

Port `src/search/bulk.rs:302–337` computes an outer cap from the current
essential preference suffix beginning at `first_window_lead`. With multiple
leaders, its observed candidates/outer-windows rule can enlarge a minimum width,
capped by `INNER_WINDOW_SIZE`; it then updates every term's range score using
`get_max_score(window_max - 1)`. Its outer preparation at `:191–205` partitions,
recomputes the cap, and tightens the proposed window when necessary.

`src/search/term_scorer.rs:98–122` first shallow-selects metadata, then
`get_max_score(up_to)` finds the first available impact level whose endpoint
covers `up_to`. It evaluates/caches that **level's** bound; if no level covers
the requested endpoint, it returns a global maximum. It does not walk all fine
blocks up to the requested endpoint.

`src/codec/postings_reader.rs:657–715` exposes up to two levels, with level-0
and level-1 endpoints and stored impact spans. `max_impact_x` scans compressed
Pareto impact pairs at the chosen level using the query's norm inverses. The
writer aggregates level-1 impacts across groups of blocks
(`src/codec/postings_writer.rs:1–4,41–75,302–342`). This is existing coarse-range
metadata that Tantivy's one-pair-per-full-block skip stream does not have.
The port's cache is keyed by the level endpoint, not by arbitrary `[lo, hi)`.

Port shallow selection also mutates its real metadata iterator and records
`needs_refilling` (`postings_reader.rs:454–468`), so its ownership is not an
immutable pure-range query. The port's scorer/candidate protocol owns that
reconciliation. Copying this call sequence into our real BlockSegmentPostings
would not establish that unread or pending docs stay reachable.

## Grounded possibilities, pending measurement and later design

These are alternatives to investigate, not a merged proposal:

* A chosen wider certificate with global maxima requires no new disk metadata
  or block lookahead, but can make many dense clauses essential and increase
  optional/candidate work. Accepted06 already deliberately balances global
  certificates against density; extending that idea indiscriminately can undo
  its useful tighter local bounds.
* Compatible full-block range maxima from a cloned SkipReader can keep tighter
  bounds without postings decode or actual-cursor movement. The price is
  per-block metadata traversal and unresolved-tail fallback. The operation must
  hide provenance and keep range cache identity separate from current-block
  caches.
* A hybrid can keep current-block bounds where they cover the range and use
  global fallback only for terms whose selected blocks do not cover it. This is
  conservative, but mixes bound quality across clauses and still needs a
  measured cap/partition policy; no choice is made here.
* Cached or precomputed multi-block maxima/coarse summaries could approach the
  port's coarse range support, but are absent from the current format/API. A
  query-local scan/cache would add setup and memory; persistent format metadata
  would change the frozen index/serialization experiment and is outside the
  next isolated runtime hypothesis.

Before implementation selection, root should measure actual regions per query,
physical widths, candidates and retained batch lengths per region, the term/rank
and eventual local role supplying `hi`, full/tail/missing-bound policy mix, and
how many cloned skip entries representative wider ranges would cross. Relate
these observations to sparse/disjoint versus dense overlap and to existing
global-certificate decisions. Record when a nonglobal locally optional term,
an essential term, or physical max_doc is the limiting cap. This is a request
for later measured statistics, not invented counts or production instrumentation
in this report.

Later verification must prove literal leaves stay below the range certificate,
strict callback traces match the independent union, actual cursor/decoder/skip/
position state survives metadata lookahead, and retained candidates are neither
lost nor reclassified. Include full/tail boundaries, missing and saturated
metadata, native DEFAULT and custom enclosure, average/profile mismatch,
legacy fallback, growing thresholds, and deletion behavior. Any runtime trial
still uses the unchanged frozen correctness and balanced performance ruler.
