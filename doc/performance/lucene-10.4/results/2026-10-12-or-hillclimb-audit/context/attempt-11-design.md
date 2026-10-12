# Attempt 11: retain the certified global prefix

Base: a674aaf2f6187f3010a135dc25fdacf9cab27e53. One mechanism only.

Query-owned state is a `usize certified_global` and `f64 certified_global_sum`:
16 bytes on this host, no allocation. The count identifies the immutable pruning
preference prefix; the sum is precisely its sequential f64 fold from +0.0. At
region entry, append the next immutable global maximum only when the unchanged
SmallScoreSumUpperBound(original term count) is <= the current threshold.
Append and count update happen together. A rejected next clause stays unconsumed.
All-certified termination remains before density policy or shallow cursor work.

Callback thresholds are monotonically nondecreasing, as already asserted by the
existing kernel. A previously certified prefix remains certified; restarting at
its exact sum performs the identical floating operations as recomputing from
zero. Finite positive admitted maxima and immutable order are the prerequisites.
Initial -infinity, negative thresholds, both signed zeros, and +infinity retain
the original comparisons. NaN initial threshold still falls back unchanged.
No proof is offered for a callback that violates the existing monotone contract.

`globally_optional` is a fresh region-local effective count copied from the
certified count. Existing live-remainder costs and density policy recompute each
region and may reset only this effective count to zero. Actual cursor exhaustion
can change density selection without invalidating or shrinking the certificate.
Local prefix construction, local partition, buffered/scalar candidate paths,
optional probes, original-order score fold and publication are unchanged.

Test-only evidence independently recomputes the old from-zero rank prefix using
the generic pre-specialization bound helper at every visited region, verifies
count and f64 sum bits, and records current threshold/count/sum/append count.
Fixtures compare full callback traces with independent BufferedUnionScorer;
they cover threshold equality and neighbors, jumps across regions, density
selection changes after actual exhaustion, negative/zero/infinite starts and
both native and legacy valid positive boost domains. No runtime jobs are run by
the implementation agent; root owns build, exact gates and frozen measurements.

Likely marginal result: the first rejected global maximum is still retested at
each region, and density/local preparation costs remain. This attempt does not
claim that global-prefix recomputation dominates, nor predict a speedup.
