# Attempt20: skip proven first-optional gaps in owned driver batches

Source-only ready, uncompiled and untimed. The worktree is
`search/tantivy-or-required-gap-oct11`, branch `perf/or-required-gap-oct11`, at
accepted11 `43f06833a325d2a18c03bd7e4fa65f2f27944292`. No production/main/root worktree was edited.
Root owns all builds, tests, benchmark, profile, disassembly, compression, and
browser work. Author performed source reads, isolated source edits, rustfmt,
Git metadata/diff-check, source preservation, and SHA256 checks only.

## Actual release change

A local `Option<Score>` caches one immutable excluded-clause bound per region.
The existing candidate loop has a label. In the existing first optional's
actual miss arm, restricted to one essential, compute its target from the
ordinary post-seek real cursor capped at exclusive hi. First require another
pending entry strictly before that target. A trigger-only gap cannot save any
later candidate visit and bypasses both bound construction and comparison.

Only after that structural guard, independently sum every current local maximum
except this rank from f64 zero in fixed rank order. Use the existing bound helper
with the original number of leaves. Cache the resulting f32 bound, not its
Boolean decision; compare against the live monotone threshold. A certificate
advances only `batch_index` over the owned sorted slice's docs below target;
the labelled continue bypasses ordinary final +1. At least two entries must be
skipped. The first surviving entry retains its original aligned literal score.
The ordinary next iteration owns refill/empty exit. No real driver cursor,
scorer, score array, prefix, partition, callback, or metadata is changed.

No helper, new API, allocator, vector, score matrix, or release observation was
introduced. The logical added cache is Option<f32>; binary/frame/RSS require
root measurements. Decode and driver BM25 are already paid. The intended
saving is later contribution clearing, exact leaf load, and repeated cheap
miss/prefix processing, never decoding or driver scoring.

The independent original-n from-zero arithmetic proof and cursor proof are in
`required-gap-production-design.md`. The added guard narrows reach without
altering that proof: there is no action when no future pending doc can skip.
Original literal incoming-order f64 publication and final f32 conversion remain
byte-identical, as do strict callback, deletion-controlled thresholds, general
multi-essential path and eligibility fallback.

## Additive tests and independent expectations

All 13 original OR tests and all pre-existing fixture/trace/check/
scheduled helpers are byte-identical. The static check compared the entire old
test-module tail and entire pre-existing helper prefix, rather than matching
names alone. There are exactly 18 test names: five additive families.
All new expected full callback doc/raw-score-bit traces come from exhaustive
BufferedUnionScorer with canonical SumCombiner. Pruning certificates generate
path evidence only. A separate generic original-n helper checks a new independent
from-zero excluded fold when a cache is built.

1. Full128 owned driver batch, n3/n32, native/custom bounds, reversed incoming
   rank order, preserved match64 while the real first-unread cursor is ahead,
   optional next doc exactlyhi/beyondhi/TERMINATED; fixed and Top1/10 canonical
   traces plus equality/next-down/next-up thresholds.
2. Partial65 owned tail with match32 retained while real driver is TERMINATED;
   later gap exhausts pending entries at exclusive region end.
3. Cached bound initially fails by one representable threshold step; actual
   callbacks0/32 model deleted winners withholding a threshold raise; real
   callback64 raises to exact bound and later65 skip must reuse the old cache.
4. Alternating optional matches make every miss trigger-only: observer requires
   no bound construction/check, no skip, exact trace. A separate gapped corpus
   with fixed below-bound threshold witnesses cache construction and no skip.
5. An early certified gap retains match64. After the old driver is exhausted,
   the previous optional's later TF1000 block supplies all callbacks256..511;
   independent canonical trace catches stale decoder or retained-region-bound
   errors. Original exhaustive unsafe-domain test remains unchanged, covering
   fallback before any scorer seek.

Real serialized cursor geometry is asserted, not backseeked: initial block
selection is performed only on clones at floor0; no fixture seeks a term behind
its real doc. The new observer verifies strict progress, exclusive endpoint,
first-unread driver at/beyond hi, batch index limits, >=2 skipped entries and
bound<=live threshold. It is entirely cfg(test), with no release cost.
All assertions are proposed runtime obligations, not reported test results;
root must retain any failed original corpus/assertion before a correction.

## Preserved artifacts

- Full authentic Git diff: `attempt-20-full.patch`, SHA256
  `87692553b1dc640c814de4de6938b8dec3acccbfac1870a4bc2bc5c44f36b0e7`.
- Final OR snapshot: `attempt-20-source-or.rs`, SHA256
  `20b487c5f16cee50973632930a4828805f3f3cd258009b8b6911da23f783e174`.
- Parent OR snapshot: `attempt-20-parent-or.rs`.
- Untouched TermScorer SHA256: `dd1e18b5b1d273cf2c09959ea207997a1e93f854b0fc459fb0a1e76129b87277`.
- `attempt-20-static-receipt.json`: sole tracked source path, parent/branch,
  exact full test-name set, byte-identical old fixtures and helpers, static-only
  outcomes and explicit unrun runtime checks.

No speed, efficiency advantage, correctness runtime result, actual-port win,
index/API parity, or compiled source derivation is claimed. This is a cheap
isolated experiment justified by root's full103-query exact reach diagnostic,
whose limited reach does not itself predict a measurable win. Frozen ruler and
negative gates remain unchanged.
