# Attempt 06 independent source review

Verdict: no correctness or API blocker found for the current admitted, valid-index
3..=32-term OR route. Acceptance as a performance win remains the root's frozen
harness decision; this review makes no speed claim.

Reviewed worktree: `/home/pronsh/Coding/playground/search/tantivy-or-hillclimb-oct07`.
HEAD/base: `05835b971452bd5bda75037cec2925b7ba6785bf`. I read the entire actual
diff against this base, the new helper/tests, all production bound calls, the
production router, and BM25 scalar/batch/cache admission source. Only the two
expected source files are modified (160 insertions, 3 deletions). The status also
contains an untracked `tests/compat_tests_data/index_v11/` fixture directory;
it is not a third modified source file and should not be swept into this attempt.

This is the delegated read-only review step of the workflow hillclimb. The
boundary-discipline principle shaped the audit: verify the caller's admitted
domain rather than reintroducing hot classification in the specialized method.
The prove-it-works principle shaped the evidence boundary: I inspected actual
source and performed a byte comparison; runtime and performance claims are
separately owned by root. I ran no build, test, profiler, compression job,
browser, or benchmark, made no source edits, and spawned no agents.

## Source identity

SHA256 of files actually inspected:

| Source | SHA256 |
| --- | --- |
| `src/query/score_combiner.rs` | `9392f9d0c0f537252e27bdb68eccc2623d43607980e12d45e45458026aab6eb0` |
| `src/query/boolean_query/or_maxscore.rs` | `5dd474782bd1ed076fee86d62b67fe231512dbcba168da73b3baec7781f5ebc9` |
| `src/query/bm25.rs` | `64f8fc7922a31328c2410a4c7f8fea4ff599b5e31493829a0898a0ef660cd67c` |
| `src/query/bm25_parameters.rs` | `0ec7f79e6a9a46a9669d0ca66363a78ab1a1d5b1cca82660bf9f2f574ca3ef8c` |
| `src/query/boolean_query/boolean_weight.rs` | `d31ec3d108611cc6b0efd37ae172fffc27a3af71070d44fbff616650b5f9a279` |
| `src/query/term_query/term_scorer.rs` | `dd1e18b5b1d273cf2c09959ea207997a1e93f854b0fc459fb0a1e76129b87277` |
| `src/postings/segment_postings.rs` | `dc884fe8227d9ce0f79e5b8a4c7118b32654b43174078f6ac5e6872c981740a1` |
| `src/postings/recorder.rs` | `ec00c72628fc3d50cf4e1c5851814af4c1bfba1bf45ba05b4b192e5e8883b2bf` |
| `src/postings/postings_writer.rs` | `10c7e2daa6917ae2e497e0c0b2a5501e155bc6e8422e06615709268b504a2c3e` |

Line references below refer to these contents. The implementation note records
an earlier implementation worktree/base and an earlier `or_maxscore.rs` hash
(`f53bbc...`); those are provenance, not the identity of the root candidate
reviewed here. Its `score_combiner.rs` hash does match this candidate.

## Domain and arithmetic proof

1. `boolean_weight.rs:690-703` routes exactly 3..=32 terms to this OR function,
   and only for a combiner advertising summation with f64 accumulation. The
   specialized constructor (`score_combiner.rs:134-140`) rejects all other
   counts and copies the factor from the existing helper. An actual byte
   comparison confirms the entire generic `ScoreSumUpperBound` struct/impl
   (`score_combiner.rs:87-125`) is unchanged from the stated root base.

2. `or_maxscore.rs:75-88` admits a non-NaN starting threshold and weights with
   safe normalization and finite positive global maxima before any shallow seek
   or ordinary seek. `bm25.rs:652-669` makes this maximum the finite positive
   weight. Crucially, this establishes finite nonnegative *matching term*
   leaves only together with the valid indexed-posting invariant TF > 0. It
   is not a certification of arbitrary calls to the public BM25 scoring method
   with literal zero frequency. The recorder's contract and positive-TF assert
   are at `recorder.rs:65-70,186-194`; occurrence insertion records a position
   even when creating a new recorder (`postings_writer.rs:230-247`). Missing
   stored frequencies decode as one in both scalar and batch paths
   (`segment_postings.rs:51-60,488-506`). Malformed/synthetic zero-TF postings
   fall outside this proof and are not newly checked by this optimization.

3. The actual normalization classification is at `bm25.rs:260-285`, and the
   literal scalar and batch score expressions agree at `bm25.rs:589-611`.
   For classic scoring, finite positive frequency divided by frequency plus
   nonnegative norm is finite in [0,1], including an infinite norm yielding
   zero. Multiplication by the admitted finite positive weight therefore stays
   finite and nonnegative. For native scoring, a nonnegative inverse gives a
   denominator at least one, possibly +infinity, so the quotient and subtraction
   remain finite in [0,weight]. Valid native k1 = -0 is an explicitly admitted
   exception: its -infinity inverse times positive TF is -infinity; division
   returns -0 and the subtraction returns the finite positive weight. Native
   b=1 with a zero norm similarly admits a +infinity inverse and returns weight
   for positive TF. Thus a blanket “all admitted inverses are finite and
   nonnegative” proof would be false, but the required finite-leaf proof holds.
   Zero TF multiplied by either infinite inverse could instead produce NaN;
   that is why the structural matching-frequency assumption matters.

4. Global sums (`or_maxscore.rs:130-138`) contain a prefix of finite positive
   global maxima. Local bounds are explicitly sanitized to finite nonnegative
   f32 values or replaced with the admitted finite global maximum
   (`or_maxscore.rs:175-220`). The optional candidate bound
   (`or_maxscore.rs:280-311`) adds known actual leaves for essential/already
   probed clauses to `prefix[rank+1]` for the disjoint not-yet-probed prefix.
   There is no subtraction and no duplicated original term. Consequently every
   bound uses at most the constructor's original term count, never merely a
   dynamically reduced count.

5. Each input is a finite nonnegative f32, so these addition trees stay in
   [0,n*f32::MAX] for n<=32. The n*f32::MAX endpoint is exactly representable
   in f64 (at most 29 significant bits); monotonic correctly rounded addition
   preserves the per-subtree endpoint. A nonzero real sum is at least 2^-149,
   hence normal in f64. For every admitted count the copied factor is finite,
   greater than one, and less than two. The product is therefore positive
   finite and below 64*f32::MAX, far from f64 overflow. Its positive IEEE bit
   successor (`score_combiner.rs:148-152`) is precisely `next_up`, without
   integer overflow. The final f32 cast is the same as the generic helper,
   including f32 subnormal rounding and conservative +infinity after f32
   overflow. The documented broader envelope of positive subnormal f64 inputs
   is also safe: multiplication by a factor greater than one cannot round a
   positive value to zero, and the product remains finite.

6. Signed zero is an intentional returned-value equivalence, not equality of
   the intermediate f64 operation. Generic returns the input zero directly.
   The specialized product preserves the zero sign, and bit increment yields
   +least-subnormal or -least-subnormal f64; its f32 cast gives the corresponding
   signed zero. The returned raw bits are identical. The new comment states
   this distinction correctly (`score_combiner.rs:149-151`).

## API blast radius and verification limits

The helper is crate-private, its factor is private, and source search finds one
production constructor/caller. The three bound applications share it; actual
score publication still replays original incoming-order f32 leaves in f64 and
casts once (`or_maxscore.rs:314-323`). No cursor, region, threshold, ordering,
batch, allocation, or published-score logic changes. Two-term OR, unrestricted
Block-WAND, intersections, and signed/exceptional generic bound users retain
the unchanged generic helper. The new `expect` can panic for an unsupported
internal count, but the sole production router exhaustively limits it to
3..=32; this does not create a reachable public API panic on the audited route.

The type checks constructor count, not the numeric sum or number of represented
leaves in release. Its debug assertions are diagnostics, not admission. A future
caller must preserve both finite/nonnegative magnitude and the original-count
allowance. An unsafe future reuse could break pruning even if a numeric sum
passed the magnitude check. This is a documented internal precondition, not a
new public validity guarantee.

The two new direct fixtures (`score_combiner.rs:207-321`) use the unchanged
generic helper as the raw-bit oracle for every admitted count. They include
unsupported counts, factor bits, both zero signs, cast midpoint neighbors with
literal assertions on both sides, f32 overflow, deterministic finite f32-leaf
sums and disjoint grouping, plus adversarial incoming orders. They are finite
sample coverage backed by the arithmetic proof, not exhaustive coverage of all
f64 values. Existing independent OR callback comparisons use exhaustive
`BufferedUnionScorer`/`SumCombiner` results (`or_maxscore.rs:356-403`) and are
unchanged, including unsafe-domain fallback (`or_maxscore.rs:724-750`). Those
OR fixtures use classic weights; native/custom-parameter coverage also depends
on the root-owned exact gates and existing BM25 tests, not these direct helper
fixtures alone.

Root reports a full run with 1,440 tests passed and 7 ignored, default and two
configuration exact gates with 1,227 comparisons passed, and formatting/clippy
passing for the candidate. I did not rerun or independently inspect those
runtime logs. Full timing was still running when this review was requested.
Only root's completed frozen-harness measurements can establish the performance
verdict. The implementation note's wording about “signed inputs” should be read
as negative nonzero inputs: signed zero deliberately remains supported. Its
numeric-admission claims also require the valid matching-TF domain described
above; they must not be presented as covering arbitrary BM25 zero-frequency
calls or corrupted postings.
