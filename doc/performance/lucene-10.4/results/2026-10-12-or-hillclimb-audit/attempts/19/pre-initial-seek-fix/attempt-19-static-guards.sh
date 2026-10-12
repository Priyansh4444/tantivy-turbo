#!/usr/bin/env bash
set -euo pipefail
tree=/home/pronsh/Coding/playground/search/tantivy-or-capped-partition-oct11
out=/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07
cd "$tree"
test "$(git rev-parse HEAD)" = 43f06833a325d2a18c03bd7e4fa65f2f27944292
test "$(git status --short)" = $' M src/query/boolean_query/or_maxscore.rs\n M src/query/term_query/term_scorer.rs'
git diff --check
cmp src/query/term_query/term_scorer.rs "$out/attempt-17-source-term.rs"
for artifact in attempt-19-static-spans attempt-19-source-or.rs attempt-19-source-term.rs attempt-19-full.patch attempt-19-vs-17.patch attempt-19-static-receipt.json; do
  test ! -e "$out/$artifact"
done
mkdir "$out/attempt-19-static-spans"
spans=$out/attempt-19-static-spans
for lane in 17 19; do
  source=$out/attempt-17-source-or.rs
  if test "$lane" = 19; then source=src/query/boolean_query/or_maxscore.rs; fi
  awk '/^struct ClauseState \{/ {active=1} active && /^impl ClauseState/ {exit} active {print}' "$source" > "$spans/record-$lane.rs"
  awk '/^    \/\/ Global maxima and preference order are immutable/ {active=1} active && /^        let mut hi = max_doc;/ {exit} active {print}' "$source" > "$spans/global-density-$lane.rs"
  awk '/^        for clause in &mut clauses\[first_essential\.\.\] \{/ {active=1} active && /^#\[cfg\(test\)\]/ {exit} active {print}' "$source" > "$spans/replay-$lane.rs"
  awk '/^    fn trace\(/ {active=1} active && /^    type ScalarLeaf/ {exit} active {print}' "$source" > "$spans/canonical-$lane.rs"
  awk '/^    fn scalar_leaves\(/ {active=1} active && /^    fn finish_widening_evidence/ {exit} active {print}' "$source" > "$spans/scalar-$lane.rs"
  awk '/^        let at = \|ordinal/ {active=1} active && /^    fn check_widening/ {exit} active {print}' "$source" > "$spans/cursor-$lane.rs"
  awk '/^    fn indexed_widening_fixture\(/ {active=1} active && /^    #\[test\]/ {exit} active {print}' "$source" > "$spans/indexed-$lane.rs"
  awk '/^        let mut admitted = proposed_hi > tight_hi;/ {active=1} active && /^        if admitted \{/ {count++; if(count==2) exit} active {print}' "$source" > "$spans/admission-$lane.rs"
done
for fact in record global-density replay canonical scalar cursor indexed admission; do
  test -s "$spans/$fact-17.rs"
  cmp "$spans/$fact-17.rs" "$spans/$fact-19.rs"
done
cp src/query/boolean_query/or_maxscore.rs "$out/attempt-19-source-or.rs"
cp src/query/term_query/term_scorer.rs "$out/attempt-19-source-term.rs"
git diff --binary -- src/query/boolean_query/or_maxscore.rs src/query/term_query/term_scorer.rs > "$out/attempt-19-full.patch"
set +e
git diff --no-index "$out/attempt-17-source-or.rs" src/query/boolean_query/or_maxscore.rs > "$out/attempt-19-vs-17.patch"
diff_status=$?
set -e
test "$diff_status" = 1
or_sha=$(sha256sum "$out/attempt-19-source-or.rs" | cut -d' ' -f1)
term_sha=$(sha256sum "$out/attempt-19-source-term.rs" | cut -d' ' -f1)
full_sha=$(sha256sum "$out/attempt-19-full.patch" | cut -d' ' -f1)
delta_sha=$(sha256sum "$out/attempt-19-vs-17.patch" | cut -d' ' -f1)
replay_sha=$(sha256sum "$spans/replay-19.rs" | cut -d' ' -f1)
test_count=$(rg -c '^    #\[test\]' src/query/boolean_query/or_maxscore.rs)
cat > "$out/attempt-19-static-receipt.json" <<EOF
{
  "attempt": "19",
  "status": "source_only_unmeasured_unaccepted",
  "parent_commit": "43f06833a325d2a18c03bd7e4fa65f2f27944292",
  "worktree": "$tree",
  "branch": "perf/or-capped-partition-oct11",
  "source_sha256": {"src/query/boolean_query/or_maxscore.rs": "$or_sha", "src/query/term_query/term_scorer.rs": "$term_sha"},
  "full_patch_sha256": "$full_sha",
  "vs17_patch_sha256": "$delta_sha",
  "shared_replay_span_sha256": "$replay_sha",
  "static_guards": {
    "exact_two_file_unstaged_scope": true,
    "term_entire_source_same_as_17": true,
    "clause_record_same_as_17": true,
    "global_cache_live_density_span_same_as_17": true,
    "shared_replay_same_as_17": true,
    "canonical_check_scalar_and_cursor_oracles_same_as_17": true,
    "indexed_corpus_and_scorer_helpers_same_as_17": true,
    "optional_admission_loop_same_as_17": true,
    "rustfmt_completed": true,
    "git_diff_check_passed": true
  },
  "cold_scratch_logical_bytes": {"selected_ends": 128, "proposed_optional_bounds": 128, "total": 256},
  "new_or_fixture_families": 6,
  "or_test_count_source_only": $test_count,
  "runtime": {"build": false, "tests": false, "worker": false, "benchmark": false, "profile": false},
  "no_compiled_layout_memory_or_speed_claim": true,
  "no_commit_or_push": true
}
EOF
cat "$out/attempt-19-static-receipt.json"
git diff --numstat
