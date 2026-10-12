# Attempt 11 implementation — stable for root review

Worktree: /home/pronsh/Coding/playground/search/tantivy-or-global-prefix-oct07
Branch: perf/or-global-prefix-oct07
Base: a674aaf2f6187f3010a135dc25fdacf9cab27e53
Only changed file: src/query/boolean_query/or_maxscore.rs
Source SHA-256: 21153a76a7ae3091ac6400cc438f408b7ac367d9a534b1d290d30af5c3fb5901
Diff: src/query/boolean_query/or_maxscore.rs | 232 +++++++++++++++++++++++++++++++--
 1 file changed, 222 insertions(+), 10 deletions(-)

The modeled production change is implemented: certified global prefix count and
its exact sequential f64 sum persist for the query; extension tests the unchanged
SmallScoreSumUpperBound with the original term count and current threshold.
The effective optional count is copied afresh each region. The old live-remainder
cost filter/density policy is byte-unchanged except its source count; it may reset
only the effective count. Global all-certified exit remains before local/cursor
work. No scoring/cursor/local-prefix/callback/batch code is changed.

Proof relies on the existing monotone callback threshold contract, immutable
copied global maxima, and immutable preference order. A successful append is
never repeated: sequential sum bits match the original from-zero fold. Initial
negative/infinite/zero thresholds use the same comparison, and initial NaN still
falls back before this state exists. Invalid decreasing callback thresholds are
not a supported domain; no extra bound fallback or clamp was introduced.

Production persistent payload is usize + f64, 16 bytes on this 64-bit host. No
new allocation, per-region scratch, index change or public API. This is payload
accounting, not a measured stack/register or RSS claim. The next rejected maximum
still incurs one test each visited region; density and local prep remain.

Two new fixture families compare EVERY callback/doc/raw score bit against the
independent BufferedUnionScorer, with monotone deterministic schedules:
- n=4 and n=32, native and legacy weights, forward/reversed incoming order,
  -infinity/-1/-0/+0 starts, equal/neighbor global-prefix bounds across128-doc
  regions, larger finite jump and+infinity all-certified termination. Initial
  +infinity and NaN are also checked with the existing independent trace helper.
- A four-term corpus forces effective choice0->1->0: high-cost actual exhaustion
  first enables widening under an unchanged count1 certificate, then rising
  threshold admits a low-cost optional and density rejects widening at count3.

Test-only evidence independently recomputes the original from-zero prefix with
GENERIC ScoreSumUpperBound at every entered region (including all-certified
termination), asserting rank count and exact f64 sum bits. Successful append
count is observed and bounded by original n; release has no observer/counter.
Existing meaningful boundary/threshold/boost/exhaustion/fallback tests remain.

I read the full actual diff end-to-end and resulting source, including targeted
re-read of the new fixture span after a tool output abbreviated that span.
`rustfmt --edition2021` and `git diff --check` passed. No compiler, test, timing,
profiling, compression, browser job or child task was run. No speed claim.
Root must review, integrate byte-exact, run targeted tests/exact gates/pilot and
retain or reject using the frozen parent06 ruler. Source is stable; no further
edits will occur unless root requests a correction.
