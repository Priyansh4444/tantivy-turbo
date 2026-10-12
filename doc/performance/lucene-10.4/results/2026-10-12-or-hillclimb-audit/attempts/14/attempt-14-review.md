# Independent attempt 14 source review

No correctness blocker found in the stable candidate. Proceed to root-owned
targeted tests, exact gates and the frozen pilot. This is a static review; no
runtime success or speed advantage is established here.

Identity checked independently:

- Worktree `/home/pronsh/Coding/playground/search/tantivy-or-contribution-mask-oct08`.
- HEAD/base accepted11 `43f06833a325d2a18c03bd7e4fa65f2f27944292`.
- Sole modified tracked path `src/query/boolean_query/or_maxscore.rs`.
- Source SHA256 `6db826184ecfc6e745940b8049da6fd3755ad622085429ffca8fbb794bcb5010`.
- Full actual diff and resulting kernel/new fixtures read, together with the
  implementation report and actual SumCombiner/scalar/batch BM25 owner code.
- All 13 existing accepted11 fixture bodies, from the first cached-prefix
  fixture through EOF, are independently byte-identical to HEAD. Diff-check passes.

## Numerical identity

The unchanged admission requires safe normalization and a finite positive query
maximum. For valid positive indexed frequencies, both literal score expressions
produce finite nonnegative matching leaves, possibly +0 through underflow or
native subtraction cancellation. A sum of at most32 such f32 leaves cannot
overflow f64. The canonical SumCombiner starts +0, converts each leaf exactly to
f64, adds in incoming order and rounds once to f32.

The old contribution Vec publication adds absent +0 entries between those same
matching leaves. Adding +0 to any finite nonnegative partial sum preserves its
value and bits. Removing only those identity additions consequently preserves
each matching leaf's relative position and every material f64 rounding step.
The bit iteration visits original ordinals in increasing order; it introduces
no reassociation, preference-order fold, subtraction or f32 intermediate sum.
Marked matching zero leaves are still read and visited. Explicit +0 agrees with
canonical SumCombiner, and the admitted leaf signs prevent an all-negative-zero
iterator identity distinction. Exceptional/signed/unsafe score domains still
fall back before mask state exists. The existing monotone callback assumption
and valid positive-TF domain are not broadened by this patch.

## Candidate ownership and bit operations

The mask is initialized after `doc < hi` on every entry to the candidate loop.
All three contribution sites first overwrite the addressed leaf slot and then
mark its incoming ordinal: owned single-essential batch, each matching scalar
essential, and each matching optional probe. No publication can read a stale
slot unless this candidate has overwritten that same slot. Early optional
pruning suppresses publication; the next pending/scalar candidate initializes
a new mask despite retained Vec contents. Callback threshold changes and batch
index retirement remain unchanged.

Admitted ordinals are0..31. `1u32 << 31` is valid; no shift by32 occurs. The
publication loop calls trailing_zeros only on nonzero masks, reads index0..31,
then clears one bit with `matched - 1` under that same nonzero guard. This handles
u32::MAX and bit31 without a sentinel or overflow. The consumed mask is not used
by later cursor logic. Original bounds, known/pruning accumulation, prefix
partition, cursors, buffer heads and final strict comparison are unchanged.

## Independent fixture surface

The 32-term scalar fixture derives matching sets from its postings predicate and
expected publication from literal per-term leaves in original incoming order,
in addition to full BufferedUnionScorer callback/raw-bit equality. It requires
all-scalar competitive replay, every physical doc, full mask, bit31 alone and
multiple actual region floors. Forward/reverse midpoint sensitivity plus
predecessor/equality/successor thresholds retain numerical evidence rather than
checking only mask bookkeeping.

The optional/buffer fixture makes the weak clauses dense enough for the existing
global density certificate and the driver sparse enough for the single route.
It requires a128-row buffer, changed floors and all four driver/optional masks
under fixed thresholds. Its growing run requires actual early optional pruning
and a later stronger winner, checking that partially populated pruned masks
contain no bit outside the independent actual matching set. Complete callback
bits remain compared to the canonical union. The native arm tests the native
literal DEFAULT scoring expression; because create_for_test uses legacy
serialization, it is not a new native stored-pair provenance witness or policy
test. The implementation report does not claim otherwise.

The underflow fixture uses a positive minimum-subnormal boost and valid positive
TF with a large physical field norm. It explicitly requires positive finite
query maximum/safe admission but an actual +0 leaf. Mask101 atdoc0 proves that
matching zero is marked, then110/111 distinguish subsequent touched sets. This
does not admit zero weights or invalid frequencies. The actual underflow and
all route/geometry observations remain runtime assertions for root to execute.

## Scope and limits

No source change, build, test, worker, profiling, timing, browser, compression,
child, commit or push was performed by this reviewer. Only this report was
written outside the source repository. The mask is4 logical bytes while the
existing contribution allocation remains. Compiler placement, binary size and
RSS are unmeasured. Dense matches add bit writes/scans, so source simplicity
does not predict a latency win; frozen paired measurements and all per-query
regressions must decide acceptance. No full Lucene API parity claim follows.
