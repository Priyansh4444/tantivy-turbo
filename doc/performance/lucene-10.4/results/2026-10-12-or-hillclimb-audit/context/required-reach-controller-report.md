# Reach controller — authored, not executed

`root-run-required-reach.py` is held for root's full review. Its SHA256 is
`f2f3339e1c5fb177c99e59173c38bbbeb446f491643f4a23f0ee6bc912c71983`
(505 lines). The author read back the complete script in sequential chunks.
No Python execution, AST/bytecode compilation, build, query execution, test,
profiling, or timing was performed.

Root invocation after all own latency jobs finish:

```
python3 -B /home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07/root-run-required-reach.py
```

The script accepts no arguments and creates a new `OUT/required-reach-run`.
An existing directory causes immediate failure; a failed run is retained and
cannot be rerun selectively or overwritten by this controller.

The staged Cargo project is directly in that run directory, at the same
depth as the frozen `OUT/harness`. This preserves the unchanged relative
`lucene-rs` dependency. All five frozen harness files are copied both into the
staged project and a pristine `original-harness/`. Only the staged Tantivy
dependency path changes to the isolated diagnostic worktree. Original/staged
hashes and its single-line dependency diff are retained. The frozen harness,
frozen controllers, and accepted production tree are never written.

Build settings match the frozen release profile: Rust 1.99.0 exact commit and
LLVM text, native CPU, two Cargo jobs, opt-level 3, fat LTO, codegen-units 1,
panic abort, no overflow checks, no debug info, and locked Cargo dependencies.
Build cache and worker live only in `required-reach-run/target`. Compiler
stdout/stderr, commands, source snapshots, status, logs, hashes, and failure
receipts are retained. Only the newly created Cargo process group is terminated
on a build timeout; no user or unrelated process is changed.

Read-only before/after identities cover every tracked/nonignored source file
in the diagnostic and fixed port trees (with the existing generated index
exception), their HEAD/status, original/pristine/staged harness files, every
non-lock index file, complete frozen query manifest, original parent-11
descriptor and its immutable binary/compiled-source/gate artifacts, frozen
accepted-11 dumps, diagnostic source snapshot/patch/static receipt, and the
controller itself. Initial guards enforce parent 11, the known observer SHA,
its sole unstaged file scope, the fixed port commit, frozen harness hashes,
query/dump hashes, and full index hashes from the frozen ruler. Failed after
guard readbacks are retained explicitly instead of preventing stage receipts.

One dedicated worker is pinned to CPU 4. Its ready object must match the
frozen workload exactly. The complete original query file is passed unchanged;
the selected dump schedule comprises all 103 port-style OR cases with 3..32
terms, including cases yielding no successful certificate or gap. Each case is
requested once, in original case order. There are no warmups, run/timing
requests, query-specific selectors, or engine query IDs.

Worker stderr goes straight to a raw file. After each complete stdout response,
the controller reads only the newly appended stderr byte range and requires
exactly one completed, admitted observer record. It records those byte offsets,
the original request/response/case, and all records before validating. Missing
or multiple records fail; neither is replaced with a fabricated zero. Extra
startup/exit records, unexpected stderr, or extra stdout fail and remain in
the raw files. Failed association/validation records remain in `counters.jsonl`;
only successfully verified rows enter the summarized dump collection.

Each dump must retain accepted-11 count, ordered raw hit docs/score bits, and
the independent exhaustive oracle exactly. The proposed counter identities
in `required-reach-report.md` are checked, including histogram sums,
deduplicated additional visits, zero callbacks/probes that a certified gap
would suppress, actual immediate next-prefix rejection, and parent one-block
ownership. Global histograms and counters aggregate all successfully verified
cases; a valid final receipt requires all 103. Zero-opportunity case IDs remain
in the summary and full case schedule.

The summary explicitly makes no latency, driver-decoding, or driver-BM25 saving
claim. Build/stage wall times are command execution metadata only. Future
candidate-work reach is a screening input for root's judgment, never a speedup
result. Artifact hashes cover retained sources, commands, compiler, receipts,
protocol, counters, and raw output. Build cache/ELFs are excluded from that
inventory; the compiled worker's SHA is recorded and checked independently.
