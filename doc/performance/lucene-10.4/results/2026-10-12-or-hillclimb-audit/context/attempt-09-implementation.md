# Attempt 09 implementation receipt — isolated posting heap

Status: source implementation finished and stable for root's end-to-end diff review and runtime gates. No speed or acceptance claim. Root alone owns all compiler, test, profiling and benchmark jobs.

* Worktree: `/home/pronsh/Coding/playground/search/tantivy-or-posting-heap-oct07`
* Branch: `perf/or-posting-heap-oct07`
* Exact implementation parent: `a674aaf2f6187f3010a135dc25fdacf9cab27e53` (accepted06).
* Only modified source: `src/query/boolean_query/or_maxscore.rs`.
* Current source SHA-256: `4e97064413426debda620071e274215f64e1e4bddfdde5624e5a3dd86d56cad5`.
* Selection contract: full `design-posting-heap-v2.md` plus `design-sparse-merge-synthesis.md`, root/fresh judge selected heap 22/25 versus SingleLead 21/25. 08 bitmap was rejected and is not imported.

## Implemented owner and caller

One query-lazy `Vec<EssentialLeafBuffer>` holds n docs/leaves arrays and usize next/len counters, with a fixed 32-Head array. `EssentialRegion` borrows scratch and the fixed ClauseState essential suffix, owns heap length/floor/hi, and exposes one `next_candidate` operation. It fills perowner posting suffixes only below current hi, gathers every matching owner at one ascending doc, writes exact f32 leaves into the existing incoming row, accumulates pruning-known in fixed suffix rank order, and repairs/refills the drained owner before returning. Heap comparison is `(doc, slot)`.

Scorer/shared-batch borrows are call-local. The optional metadata slice is disjoint and the sole original optional/publication completion block follows either replay branch. Every authoritative ClauseState doc still mirrors scorer first unread, even while retained heads are earlier. The heap, not minimum authoritative doc, decides pending completion. Explicit replay drop ends the region metadata/scratch borrow before coherence assertions and the next certificate preparation. Debug Drop rejects an undrained region but does not mutate/drain cursors; it preserves an original invariant panic instead of causing a second panic during unwinding.

Deleted obsolete scalar ClauseState.advance plus multi minimum/match/advance scans. No physical matrix/bitmap/window selector, gap dispatch, optional batching, cursor exposure, new numerical bound, format change, public interface change or mutable ordering. The original SmallScoreSumUpperBound factor uses original num_terms. Exact publication remains original ordinal f64 fold followed by one f32 cast; callback threshold is current and strict `>` unchanged. Single batch cursor mutation remains owned by the accepted fill_scored_batch path; its retained leaf is copied into the row before private index retirement.

## One agreed sketch refinement

Before implementation I surfaced the Rust metadata-borrow conflict: Option<EssentialRegion> cannot coexist with an independently borrowed single suffix even when a bool selects mutually exclusive paths. Root agreed to private exhaustive `EssentialReplay::Single(&mut ClauseState)` versus `EssentialReplay::Heap(EssentialRegion)`. The caller matches it for `(doc, known)`, then runs one common completion body. Existing single batch len/index stay local and update after completion. This encodes mutual exclusivity; there are no alias casts, scorer references retained across completion, duplicate completion functions or caller finish protocol. No further ownership escape hatch was needed.

The direct boundary fixture uses an explicitly valid global-max certificate for [0,length) then [length,512), so it can exercise guarded refill across a 128-posting decoder edge even when today's production selected-block caps usually require only one owner fill. It changes no production region rule.

## Semantic verification written, not executed

Nine new posting_heap fixtures plus the accepted fallback fixture's zero-heap-event assertion:

1. Pending retained leaves after all authoritative cursors reach TERMINATED; fixed preference order differs from incoming ordinal order. Host layout prints buffer/head/scratch/region, Vec len/capacity, foundation and total array/record payload while this oracle actually exercises three owners.
2. Whole raw callback oracle for n3/n32, overlap/disjoint lists and 127/128/129/255/256/257 boundaries; fixed 0 and rising Top1/Top10 thresholds.
3. Dirty contribution row clearing across essential and optional nonmatches.
4. Query buffer reuse when fixed suffix grows then shrinks and a clause changes roles only between drained regions.
5. Million-address sparse gaps use three owner fills for three tail suffixes, with no physical-window traversal.
6. Grafted SingleLead runner-up case: 127/128/129 lead postings meet two other owners below hi and exactly at the next region floor; future 300 heads remain retained. Exactly one callback per tie and full incoming row/known order are proved. Direct owner replay compares complete raw traces to BufferedUnionScorer.
7. Legacy/native valid positive-TF midpoint and disparate-magnitude incoming permutations; threshold next_down/equal/next_up, route proof with two substantial essential leaves, high-TF 127/255/256 witnesses. Uses unchanged native DEFAULT parameters and accepted fixture construction; no zero TF.
8. Fresh threshold and simulated deleted-hit callback rejection; explicit Score type annotation retained for current.max(score).
9. Real indexed deletes with NoMergePolicy, physical max_doc 160/live 158, real alive bitset callback trace, actual BooleanQuery TopDocs Top1 raw score and physical doc 127.

cfg(test)-only evidence records perfill docs/leaves/rank/unread and percandidate ordinal suffix, popped slots, exact incoming row, pruning-known, authoritative docs and retained heads. Independent scalar clones verify leaf bits and known f64 bits, expected ascending rank ties, strict in-certificate docs, batch len<=128, increasing docs, unique heap slots/heap order, and existence of exactly one next head for every remaining essential posting below hi. Debug owner assertions enforce refill only when previous next==len and actual progress/zero fill contract. No observer or counter remains in release.

Existing accepted tests remain, including single batching, stale full/tail floors, global optional density policy, duplicates, changing thresholds, strict midpoint ties and unsafe fallback. Generic/specialized bound helper source and tests are untouched and remain required in root's runtime gate.

## Lightweight checks completed

* rustfmt on the sole source (edition2021), successful.
* git diff --check, successful.
* Source-only exact text comparison against accepted06: full eligibility, original preference ordering, all region/density/prefix/floor preparation unchanged; common optional/publication block unchanged except ClauseState lookup through the disjoint optionals slice.
* Read back the actual resulting owner, caller, semantic fixtures and fallback extension. Source hash above is for final formatted bytes.

No cargo/build/rustc job, runtime test, profile, benchmark, pilot, browser, compression job, child agent, commit, root-trial edit or canonical-source mutation was run. Runtime correctness and host size numbers remain unverified until root executes them. Planned n32 array/record payload remains about 37,000B; Vec headers, references/padding, call frames, allocator overhead, scorer storage and RSS are excluded. Dense overlap, e2 repairs and 1–2-hit certified regions may lose; hi is never widened to finish a buffer and no batch is assumed large.

Root next: read the whole actual diff, graft stable bytes with overwrite/source hash evidence, run targeted semantic/untouched helper gates and exact frozen correctness, then the prescribed pilot before broad acceptance work. Reject or redesign if runtime evidence refutes the owner shape or performance hypothesis.
