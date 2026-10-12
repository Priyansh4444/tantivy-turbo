# Attempt23 independent source review

No concrete correctness blocker found in the reviewed two-file diff. The global and local substitutions preserve the old ordered predicates over the admitted sum domain, reuse the existing live cutoff, and retain the independent certificate and callback-trace oracles. This source review supports proceeding to root-owned runtime gates; it is not a runtime result or acceptance verdict.

Throughput checkpoint: n/a, read-only investigation.

## Artifact binding and scope

Reviewed held worktree `/home/pronsh/Coding/playground/search/tantivy-or-cutoff-all-bounds-oct12`, HEAD `3dff75c5cfddee6dab50c144839b4cec645d9dca` (selected accepted22 parent). Read the full actual two-file Git diff, the complete current OR source and its thirteen fixture families, the complete current score combiner and all eight numeric families, the parent production path and numeric construction, `ground-cutoff-all-bounds.md`, and the source author's static receipt. Source/Git metadata independently verifies the following hashes; the actual Git diff is byte-identical to the retained full patch.

| Artifact | SHA256 |
|---|---|
| Current `src/query/boolean_query/or_maxscore.rs` | `e8936f114df5bb4c09b27960afc126832f07fab73863b061ab9e3d55a6be23bb` |
| Current `src/query/score_combiner.rs` | `67a8070a9abad0d107eaebfc8ef731793b2d0f016e5216afc98d1b6ff3ac93b7` |
| Authentic `attempt-23-full.patch` | `7f673aea60d7ce7248b06042b098e3c096a175d1d8f3b97ad066f8a4595f4ad2` |
| Unchanged complete OR test-module tail | `2ba2bfa05759a9e09bd387a412da9ab8f8738b0400f234a1ec73ebdabc96779c` |
| Unchanged four original numeric families and helper tail | `a6ad2cd5c7851284af43ab0c936402f015bae523e62285c9fb64d29b5a170551` |
| Unchanged independent generic global certificate block | `8eaffdf70b5dff1e12dd82f282bc7b1862f6d2e0919027580ba2c1c66cced160` |

The tail/block comparisons were performed directly against `git show HEAD:<path>` with a metadata-only Perl script, independently of the author's asserted booleans. No engine files, root checkout, or production checkout were modified by this review. No build, test, Python, benchmark, disassembly, profile, browser, or compression work ran. Workflow investigation guided the source tracing and evidence binding; runtime proof remains root's responsibility.

## Ordered comparison and exceptional thresholds

At `score_combiner.rs:196`, the added greater method computes literal `(sum * self.factor) > cutoff.0`, with the same admitted-domain assertion as the unchanged at-most helper. It does not negate at-most, divide, change the original-n factor, or reassociate the multiplication. The constructor and old `score` implementation remain unchanged.

For finite positive post-multiply product p, the old operation increments its f64 bits then narrows to f32. The unchanged cutoff L is the last representable p whose old narrowed bound is at most T. Even T significands admit the upward midpoint tie after the successor step, yielding midpoint's first predecessor; odd T significands exclude the tie, yielding its second predecessor. The midpoint is exactly representable in f64. Thus the ordered greater complement follows for non-NaN values by total numeric order, including equality at L. MAX uses the finite hypothetical endpoint 2^128 and odd-tie exclusion, preserving the overflow-to-infinity boundary.

The exceptional cases also preserve both directions: negative T maps to -infinity and every admitted old bound exceeds it; +infinity maps to +infinity and neither finite p nor an old narrowed infinity exceeds it; either signed/payload NaN maps to NaN and both ordered predicates remain false. Both signed zeros normalize to the same positive finite cutoff. A signed-zero product's old bit-successor cast preserves zero's sign, and ordered comparisons treat either signed zero equally. The global greater method would be incorrect if implemented by negating at-most, but it is not.

These conclusions rely on the pre-existing admitted finite numerically nonnegative domain at most `32*f32::MAX`; the change does not make the helper safe for arbitrary negative, exceptional, or overflowing f64 sums.

## Live integration and preservation

The global test at `or_maxscore.rs:146` consumes precisely the unchanged `certified_global_sum + global_max` and now actively compares its result with the untouched old small helper under cfg(test), including optimized test builds. The independent from-zero generic certificate at line163 still checks count, exact sum bits, and total appends. It neither consumes the new cutoff nor reuses the cached count or sum for its expected value.

The local while at line274 retains `first_essential < num_terms` as the short-circuit guard before reading `prefix[first_essential+1]`. It cannot read prefix[num_terms+1]. Its prefix construction, zeroed out-of-region bounds, sequential sum order, partition count and original allowance are unchanged. Its active cfg(test) assertion at line278 checks the old small-helper predicate on the current actual prefix value and threshold.

The cache initializes after admission at line100. The sole live threshold assignment at line381 is immediately followed by unconditional reconstruction at line382. This includes callbacks returning the same threshold; subsequent regions therefore use current state for both changed decisions. The optional predicate and its pre-existing debug-only old-helper check at line354 are mechanically renamed only. Initial NaN and unsafe score domains still fall back before cursor movement. A callback NaN violates the pre-existing debug monotonicity contract; direct ordered comparisons nevertheless retain its old release predicate behavior. This change does not expand the supported callback contract to decreasing thresholds.

The actual diff changes no cursor operation, shallow selection, batch replay, contribution ownership, canonical incoming-order f64 fold/final f32 cast, strict publication comparison, density policy, or fallback path. It adds no new cache state or refresh site.

## Test quality and limits

The shared numeric check now compares greater against literal `small.score(sum) > threshold`, retaining the independent old successor/narrowing operation. Its original four cutoff families cover all n3..32, exceptional threshold classes, signed zeros, neighboring exponent/cast products, and deterministic valid raw-bit sums. The direct-product family additionally uses the literal old successor-and-cast independently of the new greater method; explicit even/odd witnesses and L/next-up checks anchor boundary direction. Explicit NaN assertions require both ordered predicates false, and the overflowing old bound is required to equal +infinity and remain not greater than a +infinity threshold. These checks would detect a negated-at-most implementation; they are not expectations derived solely by calling the new helper.

The thirteen unchanged OR families retain complete callback doc/raw-score-bit comparison against buffered union, rather than only final Top-K. Scheduled native/legacy n4/n32 fixtures exercise threshold equality neighbors, large changes through +infinity, and exact certificate counts; the batch fixture requires an observed full128 batch and unchanged deletion thresholds followed by raises. Density transitions, stale optional promotion, tails/exhaustion, strict ties, duplicates/max-n and pre-movement unsafe fallback remain covered.

The new live global/local assertions and unchanged generic certificate supply independent checks at actual execution inputs if root runs the tests. Source inspection alone does not establish that all desired branches execute in those runs. Numeric diversity is substantial rather than exhaustive over all f64 sums/thresholds. cfg(test) instrumentation is absent from the ordinary release worker; the optional check remains debug-only as in parent22. Root still needs targeted test receipts, three exact indexed profiles, frozen pilot/full controls and negative gates. There is no speed, binary-size, frame/register-placement, memory, index-format or actual-port win claim in this report.
