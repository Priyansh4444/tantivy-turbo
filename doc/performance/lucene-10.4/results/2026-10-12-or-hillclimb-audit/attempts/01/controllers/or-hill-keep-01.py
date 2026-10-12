import json
import subprocess
import sys
from pathlib import Path

out = Path('/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07')
repo = out.parents[1] / 'tantivy-or-hillclimb-oct07'
sys.path.insert(0, str(out))
from stage import sources, sha

for name in ('build-final', 'tests', 'clippy', 'formatting', 'layout'):
    receipt = json.loads((out / name / 'receipt.json').read_text())
    assert receipt['returncode'] == 0 and receipt['source_unchanged'], name
for name in ('verify', 'configured-k0.9-b0.4', 'configured-k2-b1', 'measure', 'port-measure'):
    assert json.loads((out / name / 'receipt.json').read_text())['pass'], name
compiled = json.loads((out / 'build-final/after.json').read_text())
assert compiled == sources()
metrics = json.loads((out / 'attempt-01-metrics.json').read_text())
assert all(r['primary'] < min(r['TT'], r['PP']) for r in metrics['groups']['OR_2']['rounds'])
changed = subprocess.check_output(['git', 'diff', '--name-only'], cwd=repo, text=True).splitlines()
assert set(changed) == {'src/query/boolean_query/block_wand_union.rs', 'src/query/boolean_query/boolean_weight.rs'}
subprocess.run(['git', 'diff', '--check'], cwd=repo, check=True)
message = Path('/tmp/or-hill-01-commit.txt')
message.write_text('''perf(query): use fixed two-term local score regions

Replace the post-warm generic WAND fallback with two fixed cursors and
region-local score certificates. Keep the existing global paths, replay
incoming score leaves exactly, and fall back before unsafe cursor changes.

Six independent full callback fixtures cover roles, stale tails, thresholds,
warm boundaries, numeric fallback, and collector rejection. All 1434 tests
and 1227 exact queries in default and two configured profiles pass.

The frozen two-term workload takes 62.2-63.5% less time in both orders;
185/198 queries beat their observed controls. This reaches arithmetic
parity with the fixed port, not a reliable port lead. Longer OR remains
about 2.2-2.3x slower. Some unchanged AND cases regress; retain those
negative controls rather than claim a universal win.
''')
subprocess.run(['git', 'add', *changed], cwd=repo, check=True)
subprocess.run(['git', 'commit', '--file', str(message)], cwd=repo, check=True)
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip()
assert compiled['tantivy-or-hillclimb-oct07']['files'] == sources()['tantivy-or-hillclimb-oct07']['files']
receipt = {'verdict': 'kept_partial_win', 'commit': head,
           'candidate_binary_sha256': sha(out / 'target/release/tantivy_worker'),
           'correctness_tests': 1434, 'ignored_tests': 7,
           'exact_profiles': 3, 'queries_per_profile': 1227,
           'two_term_primary_ratios': metrics['groups']['OR_2']['primary'],
           'target_met': False,
           'tradeoffs': 'AND per-query regressions retained; +4872 binary file bytes, +4736 .text bytes; no index changes or per-region allocation.',
           'profile_correction': 'profile-evidence-correction.json; clean baseline/candidate reruns retained separately.'}
(out / 'attempt-01-acceptance.json').write_text(json.dumps(receipt, indent=2) + '\n')
with (out / 'decision.tsv').open('a') as stream:
    stream.write('01\tfixed two-term regions remove repeated WAND pivot/alignment work\tfallback-only fixed-two region kernel\t186.41/190.22 us\t70.46/69.51 us\t-62.20%/-63.46%\t1434 tests; 3x1227 exact gates; full AB/BA controls\tkept\tport arithmetic parity only; long OR still slower; AND regressions and binary growth retained\n')
print(json.dumps(receipt))
