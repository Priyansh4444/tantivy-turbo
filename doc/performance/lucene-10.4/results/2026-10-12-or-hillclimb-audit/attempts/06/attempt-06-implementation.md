# Attempt 06: admitted small nonnegative sum bound specialization

## Proof and API sketch before implementation

Caller is only the currently admitted 3..32 term `or_maxscore` numeric route. It
already rejects NaN thresholds, unsafe normalization/weight domains, and every
nonfinite/nonpositive global maximum BEFORE cursor mutation. Safe BM25 plus a
finite positive maximum certifies finite nonnegative matching f32 leaves. Local
bounds are sanitized to finite nonnegative f32 or replaced by finite global
maxima. No other caller gains this specialization.

Sketch:

```rust
pub(crate) struct SmallScoreSumUpperBound { factor: f64 }
impl SmallScoreSumUpperBound {
    pub(crate) fn new(num_terms: usize) -> Option<Self>;
    // Call only with a finite nonnegative sum of at most 32 f32 leaves/maxima.
    #[inline]
    pub(crate) fn score(&self, sum: f64) -> Score;
}
```

The constructor checks 3..=32 once and COPIES the factor returned by unchanged
`ScoreSumUpperBound::new(num_terms)`. Thus no new factor expression or different
rounding of factor construction is introduced. Factor is finite, greater than
one, and less than two throughout that range. The caller creates this checked
type only after its existing numeric admission; generic `ScoreSumUpperBound` and
its `score` remain unchanged for two-term/other/signed/exceptional callers.
The method documents and debug-checks its narrow numerical domain. All fields
remain private; there is no unsafe operation or public API change.

Each bound sum consists of disjoint known/remaining leaves or a prefix of original
clauses, never subtraction. Each input f32 is nonnegative finite. An n-leaf f64
sum remains finite and is <= n*f32::MAX <=32*f32::MAX. The endpoint n*f32::MAX is
exactly representable in f64 for integer n<=32 (at most 29 significand bits);
monotonic correctly rounded addition preserves that endpoint by induction.
A nonzero admitted sum is >= smallest positive f32, 2^-149, and hence a normal
positive f64. Multiplying by the unchanged factor remains positive normal finite
and <= 64*f32::MAX, far below f64 overflow. Nonzero multiplication cannot underflow.
The method can safely also tolerate smaller positive f64 inputs within the stated
finite magnitude envelope, which lets direct tests probe f32 cast-underflow ties.

For every positive finite f64 product in this envelope, IEEE next_up is EXACTLY
`f64::from_bits(product.to_bits() + 1)`. No sign/NaN/infinity/maximum checks are
needed after this boundary. u64 addition cannot overflow at these product bits.
The final f32 cast is the SAME cast as the generic implementation, so subnormal,
normal midpoint and f32 overflow rounding are unchanged; infinity is a valid
conservative bound result even when all f64 operations remain finite.

Both signed zeros can be handled without a hot zero branch while preserving the
RETURNED f32 bits: positive-zero product increments to the smallest positive f64,
which casts to positive f32 zero; negative-zero product increments its bit pattern
to the smallest negative f64, which casts to negative f32 zero. The generic helper
casts the input signed zero directly. The internal f64 intermediate differs only
in these zero cases; the public-to-caller f32 result is exactly identical. This
claim relies on returned bound bits, not identical intermediate doubles. Keep an
explicit raw-bit test for both zero signs. If source review considers this subtle
zero handling too obscure, prefer an explicit sum==0 branch in a separate later
trial rather than claiming it is the same measured binary.

Therefore release score can be:

```rust
let inflated = sum * self.factor;
f64::from_bits(inflated.to_bits() + 1) as Score
```

No factor==1 branch is needed because the constructor excludes 0..2. No generic
finite/sign branch is needed because only the admitted route constructs/uses this
type. Direct tests must reject unsupported constructor counts and compare OLD/NEW
returned bound raw bits over every admitted count, zeros, f32 subnormal/normal
boundaries, midpoint/overflow cast ties and deterministic actual f32-leaf sums.
NaN/signed inputs never enter the new method: preserve existing pre-mutation OR
fallback fixture and generic exceptional/signed tests. Published scores are still
original incoming-order f32 leaves folded in f64 then one f32 cast, independent of
bounds.

## Actual implementation

Worktree `/home/pronsh/Coding/playground/search/tantivy-or-bound-specialization-oct07`; branch `perf/or-bound-specialization-oct07`;
base `22f4a270926d513b2641cd44c5445008ecf3b276`. Proof/API above were written before these edits.

Only `score_combiner.rs` and `or_maxscore.rs` change. `SmallScoreSumUpperBound`
implements the checked constructor, copies the unchanged generic factor, and
uses branch-free product/bit-successor/f32 cast with domain debug assertions.
Only the already-numerically-admitted 3..32 `or_maxscore` constructs it; all three
existing global/prefix/optional-probe bound calls then dispatch through this type.
No score publication, region, threshold policy, ordering, heap, window or batch
logic changes. Generic ScoreSumUpperBound source, including new() and score(),
is BYTE-IDENTICAL to this accepted base. No production bound allocation is added;
the wrapper remains one double factor, the same data payload as its predecessor.
The checked constructor can panic through the caller's expect only for an invalid
private routing count; the real caller admits exactly 3..32 before this point.
Unsafe numeric scores return to the existing canonical fallback BEFORE construction.

The helper's debug-checked numerical envelope is slightly broader than real
nonzero f32 sums: any finite nonnegative f64 <=32*f32::MAX is safe for the bit
successor. For subnormal positive doubles multiplication cannot overflow; the
positive-bit successor still equals next_up and casts identically. Real admitted
nonzero sums are normal doubles, as proved above. This broader envelope lets
meaningful direct tests bracket the f32 zero-underflow tie without changing the
production domain. Signed zero handling deliberately compares RETURNED f32 bits.

### Direct tests added (root has not executed them yet)

`small_score_sum_bound_matches_generic_at_cast_boundaries` checks every count
3..32, exact copied factor bits, constructor rejection for 0/1/2/33/usize::MAX,
both signed zeros with literal output bits, n*f32::MAX sums, exact subnormal/normal
f32 anchors, midpoint product inputs and their neighbors, and the final finite-
to-infinity cast midpoint. Four-neighbor brackets assert literal lower/upper f32
result bits, so both sides of each cast boundary must actually be exercised.
Generic negative/NaN/infinite conservative outputs and its two-term factor-one
behavior are explicitly retained.

`small_score_sum_bound_matches_generic_for_disjoint_leaf_sums` compares old/new
returned raw bits for all counts, 256 deterministic corpora per count, every split
of actual finite nonnegative f32 leaves into forward prefixes plus reverse known
sums, and adversarial incoming orders of the existing midpoint leaves. This
mirrors inclusive optional-prefix grouping without copying the specialized bit
implementation into the oracle. Generic .score() is the unchanged direct oracle.

All existing ten `or_maxscore` independent exhaustive raw-bit callback fixtures
are untouched: full-block/tail boundaries, deferred stale optionals, global dense
widening, rising thresholds, comparable density, exhausted-high-cost remainder,
midpoint incoming orders, duplicates/boosts, deterministic mixed corpora, and
unsafe numeric pre-seek fallback. They remain essential integration references;
direct helper equality alone cannot prove leaf/cursor admission in the engine.
The accepted two-term six fixtures and its helper route are untouched.

### Checks actually run

- Read complete actual diff, caller admission and BM25 safe-domain source.
- Exact source comparison confirms generic factor/score implementation unchanged.
- Sole changed-path check confirms only the two expected source files.
- Rustfmt for both files completed successfully; git diff --check passed.

No compilation, tests, profiling, benchmarks, commits or child agents were run.
Root owns all runtime gates and separate measurements. There is no speed claim.

Source SHA256:

- `or_maxscore.rs`: `f53bbc7728d13f797c1d0975e6c63c7a29565cf1d167253b3141bbe29c94a108`
- `score_combiner.rs`: `9392f9d0c0f537252e27bdb68eccc2623d43607980e12d45e45458026aab6eb0`

Diff stat:

```
src/query/boolean_query/or_maxscore.rs |   7 +-
 src/query/score_combiner.rs            | 156 ++++++++++++++++++++++++++++++++-
 2 files changed, 160 insertions(+), 3 deletions(-)
```
