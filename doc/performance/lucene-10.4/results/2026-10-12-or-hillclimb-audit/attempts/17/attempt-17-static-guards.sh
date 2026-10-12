#!/usr/bin/env bash
set -euo pipefail
out=/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07
tree=/home/pronsh/Coding/playground/search/tantivy-or-preserve-partition-oct10
held13=/home/pronsh/Coding/playground/search/tantivy-or-metadata-range-oct08
or_rel=src/query/boolean_query/or_maxscore.rs
term_rel=src/query/term_query/term_scorer.rs
expected_parent=43f06833a325d2a18c03bd7e4fa65f2f27944292
test "$(git -C "$tree" rev-parse HEAD)" = "$expected_parent"
test "$(git -C "$tree" branch --show-current)" = perf/or-preserve-partition-oct10
test -z "$(git -C "$tree" diff --cached --name-only)"
test "$(git -C "$tree" diff --name-only)" = "$(printf '%s\n%s' "$or_rel" "$term_rel")"
git -C "$tree" diff --check
cmp "$tree/$term_rel" "$held13/$term_rel"
test "$(sha256sum "$tree/$term_rel" | cut -d ' ' -f 1)" = 21b05a2b6513223a81613ee0f40c4a31f6990a427bb0d4b60324154a55753cdb
scratch=$(mktemp -d /tmp/attempt17-static.XXXXXX)
trap 'rm -rf "$scratch"' EXIT
git -C "$tree" show "$expected_parent:$or_rel" > "$scratch/parent-or.rs"
extract_spans() {
    local source=$1
    local prefix=$2
    awk '/^struct ClauseState \{/ { on=1 } on { print } on && /^\}/ { exit }' "$source" > "$scratch/$prefix-record"
    awk '/    \/\/ Global maxima and preference order are immutable/ { on=1 } on && /        let mut hi = max_doc;/ { exit } on { print }' "$source" > "$scratch/$prefix-global"
    awk '/        for clause in &mut clauses\[first_essential\.\.\] \{/ { on=1 } on && /^#\[cfg\(test\)\]/ { exit } on { print }' "$source" > "$scratch/$prefix-replay"
    awk '/    fn check\(/ { on=1 } on && /    type ScalarLeaf =/ { exit } on { print }' "$source" > "$scratch/$prefix-check"
    awk '/    fn scalar_leaves\(/ { on=1 } on && /    fn begin_widening_evidence/ { exit } on { print }' "$source" > "$scratch/$prefix-scalar"
    awk '/        let at = \|ordinal: usize, floor: DocId\|/ { on=1 } on && /    fn check_widening/ { exit } on { print }' "$source" > "$scratch/$prefix-cursor-oracle"
    awk '/    fn indexed_widening_fixture\(/ { on=1 } on && /    fn wider_native_indexed_default/ { exit } on { print }' "$source" > "$scratch/$prefix-indexed-corpus"
    awk '/    fn cached_global_prefix_preserves_equality_neighbors_and_large_threshold_jumps/ { on=1 } on { print }' "$source" > "$scratch/$prefix-inherited-suffix"
}
extract_spans "$scratch/parent-or.rs" parent
extract_spans "$held13/$or_rel" held13
extract_spans "$tree/$or_rel" candidate
for name in record global; do
    test -s "$scratch/candidate-$name"
    cmp "$scratch/parent-$name" "$scratch/candidate-$name"
done
for name in replay check scalar cursor-oracle indexed-corpus inherited-suffix; do
    test -s "$scratch/candidate-$name"
    cmp "$scratch/held13-$name" "$scratch/candidate-$name"
done
test "$(rg -c '^    #\[test\]' "$tree/$or_rel")" = 26
if rg -q 'min_certified_cost|density_certificate_ready|COMPRESSION_BLOCK_SIZE|saturating_add' "$tree/$or_rel"; then
    printf '%s\n' 'Unexpected independent cap/density mechanism' >&2
    exit 1
fi
for file in attempt-17-full.patch attempt-17-vs-13.patch attempt-17-source-or.rs attempt-17-source-term.rs attempt-17-static-receipt.json; do
    test ! -e "$out/$file"
done
git -C "$tree" diff -- "$or_rel" "$term_rel" > "$out/attempt-17-full.patch"
diff_status=0
git diff --no-index "$held13/$or_rel" "$tree/$or_rel" > "$out/attempt-17-vs-13.patch" || diff_status=$?
test "$diff_status" = 1
cp "$tree/$or_rel" "$out/attempt-17-source-or.rs"
cp "$tree/$term_rel" "$out/attempt-17-source-term.rs"
or_sha=$(sha256sum "$tree/$or_rel" | cut -d ' ' -f 1)
term_sha=$(sha256sum "$tree/$term_rel" | cut -d ' ' -f 1)
patch_sha=$(sha256sum "$out/attempt-17-full.patch" | cut -d ' ' -f 1)
delta_sha=$(sha256sum "$out/attempt-17-vs-13.patch" | cut -d ' ' -f 1)
replay_sha=$(sha256sum "$scratch/candidate-replay" | cut -d ' ' -f 1)
cat > "$out/attempt-17-static-receipt.json" <<EOF
{
  "attempt": "17",
  "status": "source_only_unmeasured_unaccepted",
  "parent_commit": "$expected_parent",
  "worktree": "$tree",
  "branch": "perf/or-preserve-partition-oct10",
  "source_sha256": {"$or_rel": "$or_sha", "$term_rel": "$term_sha"},
  "full_patch_sha256": "$patch_sha",
  "vs13_patch_sha256": "$delta_sha",
  "shared_replay_span_sha256": "$replay_sha",
  "static_guards": {
    "exact_two_file_unstaged_scope": true,
    "term_entire_source_same_as_13": true,
    "clause_record_same_as_accepted11": true,
    "global_cache_live_density_span_same_as_accepted11": true,
    "shared_replay_same_as_13": true,
    "canonical_check_scalar_and_cursor_oracles_same_as_13": true,
    "indexed_corpus_and_scorer_helpers_same_as_13": true,
    "inherited_fixture_suffix_same_as_13": true,
    "cap16_and_density15_absent": true,
    "rustfmt_completed": true,
    "git_diff_check_passed": true
  },
  "cold_scratch_logical_bytes": {"selected_ends_from13": 128, "proposed_optional_bounds_added": 128, "total": 256},
  "new_or_fixture_families": 3,
  "or_test_count_source_only": 26,
  "runtime": {"build": false, "tests": false, "worker": false, "benchmark": false, "profile": false},
  "no_compiled_layout_memory_or_speed_claim": true,
  "no_commit_or_push": true
}
EOF
cat "$out/attempt-17-static-receipt.json"
