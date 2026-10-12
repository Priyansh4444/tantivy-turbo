# Attempt 19 targeted failure analysis

The two reported panics are invalid fixture positioning before the optimization runs. They do not establish a production defect in capped partition expansion. The proposed patch repairs only test setup and leaves the release implementation and every semantic expectation unchanged. Corrected execution is still required and belongs to root.

Throughput checkpoint: n/a, source-only investigation.

Reviewed the complete actual parent-relative `attempt-19-full.patch` (2,421 lines), complete `attempt-19-vs-17.patch`, the two failing fixture bodies, construction and seek implementations, canonical callback and scalar cursor oracles, and the archived attempt 09 oracle correction. No tests, build, profiling, benchmark, Python, browser or compression were run. Neither root's worktree nor held attempt 19 was modified.

## Causal path

Root's original targeted execution recorded 30 passing and two failing tests. The original stdout/stderr and source are retained at `attempt-19/pre-initial-seek-fix`. Stderr identifies `term_scorer.rs:326`, `assertion failed: target >= self.doc()`. No backtrace was enabled, so there is no recorded dynamic stack beyond this panic site.

`TermScorer::create_for_test` serializes the supplied real doc/TF postings unchanged, opens `SegmentPostings`, and constructs a scorer already positioned at its first actual posting (`term_scorer.rs:131`, `segment_postings.rs:161`). The additive `capped_partition_admits_and_rejects_zero_restoration_n3_n32` starts its future list at doc 132, but the loop at original `or_maxscore.rs:1828` calls `seek(100)` on every scorer. Dense and driver heads start at 0 and legitimately advance to 100. The future scorer is already at 132: target 100 violates the forward-only contract and triggers the panic.

The additive `capped_partition_stops_before_the_later_optional_after_overflow` repeats the same unconditional setup at original `or_maxscore.rs:1892`. Its future and later-optional lists start at 132 and 133. The first future call `seek(100)` fails before reaching the later optional. Both setup loops precede `check_widening`, so neither failure enters `or_maxscore`, computes proposed bounds, invokes a callback, or runs the post-execution cursor oracle.

`DocSet::seek` explicitly requires target >= current doc (`docset.rs:44–59`), and `TermScorer::seek` asserts that contract before calling postings seek (`term_scorer.rs:325–327`). The patch must not relax this invariant. Attempt 09 had the same class of mistake in a candidate observer; its archived review records the correct conditional forward seek and retains its failed original run. The existing independent `range_encloses` helper also guards `scalar.seek(floor)` with `scalar.doc() < floor`.

## Proposed categorical fixture correction

`attempt-19-failure-analysis.patch` applies to the original held OR source SHA-256 `4c56911f9cabe4fd06f8d1fe2f999564267c8fa3196600f29fe1b42ba1e848f7`. It adds one helper inside `#[cfg(test)] mod tests`:

```rust
fn seek_fixture_floor(scorers: &mut [TermScorer], floor: DocId) {
    for scorer in scorers {
        if scorer.doc() < floor {
            scorer.seek(floor);
        }
    }
}
```

All three additive bulk positioning loops at floor 100 call it. Sharing the legal lower-bound positioning operation prevents repeating the same invalid assumption at another bulk call; the two local guards would fix the current failures but leave the third copy ready to regress when a fixture gains a later initial head. This test-only helper neither changes the forward-only scorer API nor hides invalid production seeks.

The two previously failing setups will start with real heads `[100,...,132,100]` and `[100,132,133,100]`; the minimum remains 100, and future heads remain exactly 132/133. The third, already passing exclusive-end setup has weak heads 0 and driver head 64, so every head still legitimately seeks to 100, identically to the original. No release code is changed.

Every posting doc, TF, boost, norm, threshold, incoming order, reverse-order case, limit 0/1/10, doc 200 TF1000 winner, proposal endpoint, admitted/rejected witness, raw score bits, callback trace, canonical BufferedUnionScorer reference, selected-cache assertion and scalar real-cursor oracle is preserved. The patch changes no assertions and removes no coverage.

## Required root verification

After archiving the failed original, apply and inspect the patch, verify the production prefix before `#[cfg(test)] mod tests` is byte-identical, and rerun the complete 32-test OR target in a fresh stage. All original checks remain live and may expose a later independent issue now that the setup can reach the mechanism. This source explanation does not claim the corrected run passes, nor establish correctness or performance acceptance.
