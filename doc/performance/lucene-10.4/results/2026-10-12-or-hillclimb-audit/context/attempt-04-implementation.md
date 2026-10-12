# Attempt 04: leave the two-term warm phase when one term is globally optional

Isolated worktree `search/tantivy-or-warm-handoff-oct07`, branch
`perf/or-warm-handoff-oct07`, accepted parent
`22f4a270926d513b2641cd44c5445008ecf3b276`. Worktree was already prepared and
verified clean before this edit. This is an unaccepted trial, not a measured win.

Only `src/query/boolean_query/block_wand_union.rs` changed. The release behavior
change is one OR comparison and break AFTER the existing BOTH-global maximum
intersection check. Each matching scorer has already advanced through the
just-processed warm candidate, so the existing single-essential branch receives
actual cursors at the next unprocessed state. A singleton term whose global
maximum equals threshold cannot produce a strict `score > threshold` winner.
No shallow movement occurs before this handoff; it introduces no stale selected
block/cursor protocol. There is no pre-first-doc handoff.

The existing both-global intersection branch retains precedence and its body
is unchanged. Warm constant128, first candidate scoring, threshold/callback
math, safe numeric admission, globally single-essential branch, local-region
kernel, incoming scorer order, index format and public APIs are unchanged.
No new bound, batching or cursor API is added.

One bounded new test reuses the existing independent full raw-bit callback
oracle. An opt-in cfg(test) observer records only processed warm count, next
physical doc, and exit kind. The observer, counter and recording calls are
absent in release builds; no production allocation or runtime counter is added.
Both incoming orders cover:

* Fixed threshold at/above the weak term's exact cached global maximum: single-
  essential handoff after1 candidate with next actual doc1, later winner250.
* One representable float below that certificate: no early handoff, full128
  warm candidates and unchanged local fallback at physical doc128.
* Top-10 callback threshold growth: threshold first certifies the weak term
  after10 candidates, with next doc10 and the later stronger hit retained.
* Both terms certified optional after the first strong candidate: existing
  terminal global intersection exit takes precedence after1 candidate.
* Fixed low threshold that certifies neither term: all300 callbacks exactly
  match canonical union and warm phase still consumes128 before local fallback.

Actual observed outputs and speed are not claimed until root runs the fixture,
full default/configured gates, frozen paired comparisons and controls against
the accepted parent. No build, test, benchmark, profile, commit, child task or
formatting command was run by this implementation agent. Root owns all heavy
commands and formatting. Source edits are stable and ready for review.
