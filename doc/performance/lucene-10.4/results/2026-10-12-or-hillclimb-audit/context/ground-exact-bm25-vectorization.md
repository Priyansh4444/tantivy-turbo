# Exact BM25 batch vectorization — source-only category grounding

Status: recommendation from source inspection only. No engine implementation,
build, compiler/assembly inspection, profiling, query execution, Python,
browser, benchmark, or child task was performed. The required-reach observer
and controller were not edited. Accepted parent 11 is commit
`43f06833a325d2a18c03bd7e4fa65f2f27944292`.

throughput checkpoint: n/a, read-only investigation

Investigation steps: source/configuration/path and accepted-03 rationale
traced; read-only checkpoint preserved; recommendation and tradeoffs recorded;
plain-writing review applied. Runtime and generated-code facts remain pending.

## Recommendation

Do not immediately write a SIMD scoring kernel. Accepted 11 already has the
main architecture needed for automatic vectorization: gather cached norms
first, then run an independent contiguous arithmetic loop with its scoring
policy selected once. There is no per-leaf BM25 function call or policy
dispatch left to remove from this batch route. Source cannot tell us whether
the frozen compiler actually emitted SIMD, or whether its division throughput
is the remaining cost.

The inexpensive prerequisite is for root, after latency jobs finish, to inspect
the **existing immutable accepted-11 binary's actual batch arithmetic route**.
If its native/legacy loop already evaluates several leaves with packed add,
multiply, divide, and subtract instructions, a hand-written equivalent loop
does not introduce a new mechanism. Merely finding a vector divide elsewhere
in the binary is insufficient: it must be attributed to this route. No such
inspection was done here.

If source-level specialization is still worth an isolated trial, the smaller
first candidate is a private **FieldNormReader batch gather** that matches its
data/constant representation once per batch. It can retain every scalar norm
lookup and the existing arithmetic loops, adds no scratch or dispatch state,
and makes reader ownership explicit. The compiler may already hoist that
representation check, so even this has no source-established speed benefit.
An explicit exact SIMD arithmetic route is a conditional later candidate only
if generated code and batch-length reach show something concrete to fix.

| Choice | Possible new mechanism | Cost and uncertainty |
|---|---|---|
| Keep accepted-11 batch and inspect its existing artifact | Determine whether arithmetic SIMD/gather hoisting is already present | No source change; root still must attribute instructions to this route |
| Private reader-owned norm gather | Match data/constant representation once per batch | Small private change, same scratch and arithmetic; compiler may already hoist it |
| Explicit exact SIMD arithmetic | Packed independent leaf evaluation when compiler missed it | New unsafe/dispatch/tail/conversion proof; cannot remove the irregular norm gather or promise a win |

## Actual accepted path and prior work

`TermScorer::fill_scored_batch` (`term_scorer.rs:51`) first calls the postings
owner's `read_postings_batch`. That owner copies at most one decoded block
suffix, with actual TF values or scalar-equivalent ones when frequencies are
not read, and advances its real cursor to the first unread posting. The scorer
then calls `Bm25Weight::score_batch` once with the emitted `docs`, `freqs`,
existing cached-norm scratch, and leaf-score output slices.

`ScoredTermBatch` already owns four fixed 128-entry arrays: docs, TFs, cached
normalization components, and literal leaf scores (2 KiB total). A vectorized
arithmetic loop need not add a fifth array or change posting ownership.

`Bm25Weight::score_batch` (`bm25.rs:575`) has two phases:

1. For each actual doc, gather exactly
   `self.cache[fieldnorms.fieldnorm_id(doc) as usize]` into the existing norm
   scratch. The cache contains 256 f32 components behind an immutable Arc.
2. Match `self.scoring` once, then evaluate a slice loop using only contiguous
   TFs, gathered norms, and score outputs plus the invariant weight. The native
   and legacy loops have no cross-leaf sum or loop-carried floating dependency.

`FieldNormReader::fieldnorm_id` (`reader.rs:134`) is inline and matches an
immutable private enum: `FromData(OwnedBytes)` reads one indexed byte;
`Const` returns its stored byte. This is an enum match and indexed lookup,
not a trait-object call. The source currently expresses the match inside the
gather loop. It is unknown whether LLVM unswitches/hoists it in the frozen
optimized artifact.

Accepted attempt 03 introduced this exact two-pass batch route. Its retained
source report explicitly listed vectorization as unresolved; it did not
establish that emitted machine code remained scalar. The decision trail records
that mechanism as accepted after full gates and controlled timing. An explicit
SIMD proposal must therefore improve this existing route, not claim the
already completed per-leaf-call removal as a new optimization.

The scalar `score` and `score_with_frequency` methods are inline as well; this
does not prove all scalar callers inline or vectorize. Only the owned-batch
route is in this proposed scope. Multi-essential scalar replay, optional
probes, phrase fractional frequency, two-term OR, and bound calculation remain
unchanged.

## Exact lane arithmetic contract

All operations below are independent f32 operations, with the same literal
cached component and existing weight. `RN32` denotes the scalar operation's
f32 rounding; it does not authorize choosing a new floating environment.

For **both** policies, first compute:

```
F = u32 TF converted exactly as the existing Rust `TF as f32`
N = the already gathered literal f32 cache entry
W = the existing literal f32 weight
```

For **legacy/custom classic** scoring:

```
D = RN32(F + N)
Q = RN32(F / D)          // actual division
S = RN32(W * Q)
```

For **native Lucene** scoring:

```
P = RN32(F * N)
D = RN32(1.0f32 + P)
Q = RN32(W / D)          // actual division
S = RN32(W - Q)
```

There is no horizontal operation between leaves. Store each resulting S at
the same batch index, including zero/subnormal outputs and signed-zero bits.
The existing OR replay still promotes original f32 leaves to f64, folds them
in incoming ordinal order, and casts once to f32. Neither a SIMD leaf routine
nor a norm gather should alter that separate publication contract.

Do not substitute reciprocal estimates/Newton iterations, fused multiply-add,
`W * (1 - 1 / D)`, an algebraic classic/native conversion, clamp, saturation,
f64 intermediates, or approximate TF conversion. Those change either an
operation's rounding or cancellation and require a different score contract.
The frozen `target-cpu=native` flag offers host instructions; it is not a
request to change arithmetic into fast math or add FMA expressions.

### Full unsigned TF range is a real constraint

The current loop casts **u32**, including values above `i32::MAX`. Existing
raw-bit tests include `16_777_217` and `u32::MAX`; the native-bound tests also
explicitly observe `u32::MAX as f32 == 4294967296.0`.

A direct signed packed conversion of the same bits is wrong for those TFs.
The locally inspected stdarch `_mm256_cvtepi32_ps` explicitly casts
`a.as_i32x8()`. Do not silently require small TFs because the measured corpus
usually has them. A cheap first explicit-vector prototype could assemble
lanes from the original scalar `freq as f32` conversions and vectorize only
the subsequent arithmetic; it would preserve the unsigned conversion without
introducing a new integer-conversion proof or another full scratch array.
Its insertion/conversion cost could erase any gain.

A future packed unsigned conversion would need its own all-u32 correctness
argument and adversarial bit tests. Neither the existing integer filter's
unsigned-to-signed comparison mapping nor a guessed high-bit adjustment is a
valid substitute for that conversion proof.

## What SIMD could and could not remove

The arithmetic phase can, in principle, evaluate independent leaves in packed
lanes with real vector division. This may reduce instruction/loop overhead or
improve arithmetic throughput **if the existing compiler has not already done
it**. The source gives no speed percentage, cycle attribution, or proof that
division is the bottleneck.

This arithmetic-only route does not remove the irregular doc-to-byte norm
lookup or the byte-to-256-entry cache lookup. Gathering f32 cache entries with
AVX2 still requires first obtaining valid norm IDs. Gathering 32-bit words from
the byte norm file to emulate byte reads can cross the file boundary or read
unrelated bytes, especially at the last doc; such a route needs a new safety
proof and is unnecessary for a small candidate. Keep the existing safe gather
or improve its representation dispatch separately.

Precomputing a per-doc f32 norm array for each query weight would add startup,
allocation and memory proportional to the segment, replacing the current
256-entry cache and small fixed scratch. That is a different architecture and
compactness tradeoff, not a small exact SIMD implementation.

An unsafe unchecked norm access would also require a corruption/lifetime/range
contract stronger than the present safe indexing. It should not be bundled
with arithmetic vectorization merely to remove bounds checks.

## Smaller owner-level gather candidate

A private method conceptually shaped as

```
FieldNormReader::gather_cache(docs, cache: &[f32; 256], out: &mut [f32])
```

would own its private reader representation, validate equal slice lengths, and
match it once:

- `FromData`: retain safe `data.as_slice()[doc as usize]`, then the same cache
  lookup, for every supplied doc in order.
- `Const`: load `cache[fieldnorm_id as usize]` once and fill the output with
  those exact bits. Do not add an out-of-range panic contract that differs from
  today's constant `fieldnorm_id`, which simply returns the constant byte.

`Bm25Weight::score_batch` would call that method and keep both arithmetic loops
unchanged. This is private code at the fieldnorm owner, not a new public
scoring API, per-doc cache, SIMD framework, or index format. It categorically
removes the source-level per-doc reader match, but cannot establish additional
machine-code savings without comparing actual generated code/runtime.

One mechanism per attempt: do not combine this gather change with explicit
SIMD, new TF conversion, or region/partition changes. Against accepted 11,
scores and cursor ownership can remain independent of this optimization.

## Dispatch and architecture constraints for a later SIMD attempt

The repository's existing `bitpacker/src/filter_vec` is the applicable idiom:
architecture-specific private modules, a scalar fallback, runtime feature
detection before unsafe target-feature calls, cached selection, and scalar
tails. Its x86-64 route selects AVX2; AArch64 uses NEON (and non-Apple SVE when
available). Its instructions operate on integer filtering/compaction. They
are not a ready-made floating BM25 kernel or a public reusable selector.

For the smallest potential x86-only exact arithmetic experiment, there are
two defensible deployment choices, to be picked before implementation:

1. Compile-time `cfg(target_arch = "x86_64", target_feature = "avx")` selection
   for an AVX arithmetic helper, with the unchanged scalar loop everywhere
   else. This can exercise the same native-host build as the ruler and adds no
   runtime feature cache or per-weight pointer. A generic build stays scalar
   even on an AVX-capable machine; that is safe, with limited acceleration.
2. A private runtime AVX check before an `#[target_feature(enable = "avx")]`
   helper, once per batch rather than per leaf, with scalar fallback. If a
   later packed integer conversion uses AVX2 operations, require AVX2 instead.
   Do not call an AVX2 helper merely because AVX was detected. Cached feature
   selection can follow the existing filtering idiom, but that selection cost
   and any extra weight state must be measured rather than assumed free.

Use unaligned full-width loads/stores only for complete in-bounds chunks of
the actual emitted batch length. A 128-entry backing array does not authorize
scoring padding, reading unavailable slice elements, or modifying outputs past
the emitted length. Evaluate remaining entries with the literal scalar route.
Never change the first-unread driver cursor while processing a vector chunk.

AArch64 can initially remain on the unchanged scalar/auto-vectorized code.
That is an architecture-safe fallback without claiming a NEON speedup. An
explicit NEON route would require its own exact TF conversion, real division,
separate multiply/add/subtract sequence and host raw-bit validation. SVE is
unnecessary for this bounded 128-entry experiment. Other architectures retain
the scalar route; do not force an x86 instruction set into a shared build.

The local older stdarch source shows AVX packed `mul/add/div/sub` and AArch64
NEON division implemented as the corresponding SIMD operations, including real
division, and exposes their target-feature requirements. This establishes
the existence of an idiom, not frozen-1.99 code generation or cross-host bit
parity. Rust 1.99 source was not installed at the checked source path; no
toolchain was installed or compiler invoked here.

## Numeric domains and needed verification

Native DEFAULT, native configured profiles, and legacy/custom classic use
different already-built cache contents; the SIMD loop must consume those
literal contents. It must not regenerate norm caches, substitute native input
envelopes, or conflate persisted block bounds with scoring components.

`has_safe_score_bounds` certifies positive matching TFs and finite weight, not
that every cache component is finite normal. Safe configured domains include
zero/subnormal native inverses and certain k1 zero cases with infinite inverse
normalization. The admitted OR batch can encounter exact zero leaves and
overflowed intermediate denominators. Do not silently reject those valid
cases, change denormal handling, or alter the floating control state. Public
invalid/custom exceptional or signed inputs that do not enter this admitted
route must keep the scalar/canonical fallback behavior.

The existing raw-bit batch test covers all 256 norm IDs, three profiles,
native/legacy policies, boosts 0/1/3.25/-2, and several TF magnitudes including
the full unsigned maximum. A later explicit SIMD attempt needs independent
scalar-bit comparisons for varied mixed lanes and lengths 0/1/3/4/7/8/15/16/
127/128, slice offsets/unaligned starts, unsigned-conversion boundaries and
rounding neighbors, zero/subnormal leaves/components, overflow and valid
zero-k1 profiles. Preserve existing independent OR callback traces and batch
cursor/position/tail fixtures. Keep actual native/legacy/custom-safe dispatch
witnesses rather than assuming every passing score test took the new path.

Cross-architecture claims require execution on those architectures. The frozen
three exact profiles and all-query AB/BA parent controls remain mandatory
before accepting a local performance result. No reciprocal, FMA, new score
contract, per-query special casing, or changed index belongs in that trial.

## Measurable hypothesis and remaining-cost decision

The pending required-reach diagnostic's driver batch-length histogram and
already-scored entry counts can show arithmetic **reach**, without proving
cost. For vector width w, the histogram gives
`sum(floor(length / w) * w * batch_count[length])` entries that fit complete
chunks, and the remainder count shows tail work. Count every query, including
ones with no batch reach. This does not identify cycles spent in norm gather,
division, replay, or optional probes.

An isolated owner-gather trial hypothesizes that moving the immutable reader
match outside the irregular lookup loop reduces redundant branch/reader work.
An isolated explicit SIMD trial hypothesizes that the actual existing code is
scalar or uses a less effective width, and that exact packed arithmetic wins
after dispatch and full-u32 conversion costs. Both hypotheses can be false;
neither follows from Rust being faster than Java.

If existing code already vectorizes and gather dispatch is already hoisted,
stop this category at source/code-generation grounding. Replacing it with
more unsafe architecture-specific code would have low expected value. If the
diagnostic shows mostly tiny/clipped batches, that also raises the remaining
cost of a SIMD-only trial. If root finds a concrete missed transformation and
sufficient batch reach, the existing bounded scratch plus unchanged score
sequence offers a relatively small, independently testable candidate.

## Source identity and references

Inspected tree: `/home/pronsh/Coding/playground/search/tantivy-pr2937`, HEAD
`43f06833a325d2a18c03bd7e4fa65f2f27944292`.

- `src/query/bm25.rs`: SHA256
  `64f8fc7922a31328c2410a4c7f8fea4ff599b5e31493829a0898a0ef660cd67c`;
  cache construction 230..285, batch/scalar literal loops 567..615,
  safety admission 652..670, raw-bit batch fixtures 843..897.
- `src/query/term_query/term_scorer.rs`: SHA256
  `dd1e18b5b1d273cf2c09959ea207997a1e93f854b0fc459fb0a1e76129b87277`;
  owned scratch/fill 13..66.
- `src/fieldnorm/reader.rs`: SHA256
  `e22d1a8a521e57043dd069f7ddb4b1e9ddefe94fdb28bdcd6bbef6da1bb74c3f`;
  private representation 56..76, reader operations 96..147.
- `src/postings/segment_postings.rs:26..68`: owner of batch emission,
  exact frequencies and real cursor advance.
- `bitpacker/src/filter_vec/mod.rs`: SHA256
  `f5ab3a17187f7408331ceca54c8eaf485d79bca44b0f0c53f1fcf0f2deaef318`;
  architecture selection 1..132.
- `bitpacker/src/filter_vec/avx2.rs`: SHA256
  `3235edd234c4c2ba6bd3b2898cd5f3dd7ae169c90216bf48311b7c7f8d82eb8c`;
  guarded helper/tail 1..100.
- `bitpacker/src/filter_vec/neon.rs`: SHA256
  `84204668972b817ee15dd6314023d3de4c3a4ee31d1c4f2318ecaa71b25d8966`;
  target feature, helper, scalar remainder 1..98.
- `OUT/attempt-03/attempt-03-implementation.md`: retained prior source report,
  two-pass layout and originally unresolved vectorization question.
- `OUT/attempt-11/build-final/{command.json,receipt.json,compiler.txt}`:
  actual accepted compiler/profile/flags receipts; no new compiler invocation.
- Local 1.98.1 stdarch source under
  `/home/pronsh/.rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/stdarch/crates/core_arch/src/`:
  `x86/avx.rs` packed ops 35..52,268..285,334..362, signed conversion 918..932;
  `aarch64/neon/generated.rs` actual vector division 7905..7922.

No measured SIMD use, missing SIMD, remaining-cycle attribution, speedup,
compactness advantage, or cross-architecture correctness is claimed.
