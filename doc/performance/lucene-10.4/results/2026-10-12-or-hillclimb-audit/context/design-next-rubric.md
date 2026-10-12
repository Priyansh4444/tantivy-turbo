# Next sparse essential merge: architect and arena rubric

This is design work only while attempt 08 receives its independent runtime
verdict. The frozen performance ruler, correctness gate and target remain
unchanged. No implementation may be stacked on an unaccepted experiment.

Grounded constraints: accepted 06 keeps exact f32 leaves, incoming-order f64
publication with one f32 cast, original-count conservative bounds, monotone
callback thresholds and valid indexed matching TF. Its multi-essential merge
still scans scalar cursors. Attempt 08 replaces that scan with complete gathers
into a 128-document exact-leaf bitmap window. The port also has a temporary
single-lead span when the second essential cursor is sufficiently far away.

Two structurally distinct packages compare drained temporary single-lead spans
and region-owned posting-bounded buffers with a small heap. Read both packages
in full before a fresh judge or root selects a base. Each criterion scores 1–5:

1. Exact semantic proof: incoming leaves, fixed preference bound sum, current
   threshold callback sequence, cursor/pending ownership and certificate expiry.
2. Interface depth and extension safety: complete ownership operations, minimal
   private surface, no pass-through phase API or invalid optional state bags.
3. Concrete mechanism: explain which repeated work disappears on actual sparse
   or disjoint posting streams; specify the workload that could lose.
4. Compactness and scope: bounded scratch, explicit allocation and byte costs,
   reuse existing buffers; one mechanism without unrelated bound/order changes.
5. Verification quality: discriminating independent raw callback oracles for
   ties, equality, growing thresholds, delayed deletions, tail/full blocks,
   exhausted real cursors with retained candidates and future role changes.

Architect phases: Ground complete; Sketch in progress; Agree is automatic;
Implement pending accepted-parent selection; Scrap if ownership needs repeated
workarounds. Arena phases: Frame complete; Fan out in progress; Cross-judge,
Pick, Graft and Verify pending. Runtime performance is not an architecture score.
