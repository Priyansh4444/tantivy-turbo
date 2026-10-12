"""Accept only the fully verified bound specialization; retain immutable evidence."""
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

out = Path('/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07')
repo = out.parents[1] / 'tantivy-or-hillclimb-oct07'
prod = out.parents[1] / 'tantivy-pr2937'
sys.path.insert(0, str(out))
from stage import sha, sources

assert not (out / 'attempt-06').exists()
parent = json.loads((out / 'parent-03.json').read_text())
assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip() == parent['source_commit']
for name in ('build-final', 'tests', 'clippy', 'formatting', 'tests-targeted'):
    receipt = json.loads((out / name / 'receipt.json').read_text())
    assert receipt['returncode'] == 0 and receipt['source_unchanged'], name
for name in ('verify', 'configured-k0.9-b0.4', 'configured-k2-b1', 'measure', 'port-measure'):
    assert json.loads((out / name / 'receipt.json').read_text())['pass'], name
compiled = json.loads((out / 'build-final/after.json').read_text())
assert compiled == sources()
metrics = json.loads((out / 'attempt-06-metrics.json').read_text())
group = metrics['groups']['OR_3PLUS']
assert group['candidate_win_both_beyond_group_controls']
assert all(row['primary'] < 1 for row in group['rounds'])
assert all(not metrics['groups'][kind]['parent_win_both_beyond_group_controls'] for kind in ('TERM', 'AND', 'PHRASE'))
paths = ['src/query/score_combiner.rs', 'src/query/boolean_query/or_maxscore.rs']
assert set(subprocess.check_output(['git', 'diff', '--name-only'], cwd=repo, text=True).splitlines()) == set(paths)
assert sum(map(int, re.findall(r'test result: ok\. (\d+) passed', (out / 'tests/stdout.log').read_text()))) == 1440
review = (out / 'attempt-06-review.md').read_text()
assert 'no correctness or API blocker found' in review
assert all(sha(repo / path) in review for path in paths)
subprocess.run(['git', 'diff', '--check'], cwd=repo, check=True)
delta = metrics['efficiency']['candidate_file_bytes'] - metrics['efficiency']['baseline_file_bytes']
ratios = group['primary']
message = Path('/tmp/or-hill-06-commit.txt')
message.write_text(f'''perf(query): specialize admitted small score bounds

Copy the existing conservative factor once for admitted 3..32-term sums.
Use the exact positive finite double bit successor and unchanged float cast
to remove repeated classification branches from OR pruning bounds. Preserve
the generic helper for other domains and every published score expression.

Direct old/new raw-bit checks cover all admitted counts, signed zeros,
subnormal/normal/overflow cast boundaries and disjoint leaf sum orders.
All 1440 tests, three 1227-query exact profiles, clippy and formatting pass.

The complete frozen longer-OR group takes {(1-ratios['AB'])*100:.2f}% and
{(1-ratios['BA'])*100:.2f}% less time than {parent['source_commit'][:7]} in AB/BA,
beyond group self-controls. Binary file delta is {delta:+d} bytes, with no
new allocation or index-format change. Per-query losses remain in the full
untrimmed evidence; this partial win does not establish a lead over the port.
''')
subprocess.run(['git', 'add', *paths], cwd=repo, check=True)
subprocess.run(['git', 'commit', '--file', str(message)], cwd=repo, check=True)
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip()
assert compiled['tantivy-or-hillclimb-oct07']['files'] == sources()['tantivy-or-hillclimb-oct07']['files']
acceptance = {'verdict': 'kept_partial_win', 'commit': head,
    'candidate_binary_sha256': sha(out / 'target/release/tantivy_worker'),
    'correctness_tests': 1440, 'ignored_tests': 7, 'exact_profiles': 3,
    'queries_per_profile': 1227, 'longer_or_primary_ratios': ratios,
    'control_log_spread': group['control_log_spread'], 'target_met': False,
    'efficiency': metrics['efficiency'],
    'limitations': 'Per-query losses retained; no new allocation or index format; fixed-port gap reported separately.'}
(out / 'attempt-06-acceptance.json').write_text(json.dumps(acceptance, indent=2) + '\n')
summary = json.loads((out / 'summary.json').read_text())
group_summary = next(g for g in summary['summaries'] if g['set'] == 'port-style' and g['mode'] == 'top10' and g['kind'] == 'OR_3PLUS')
before = '/'.join(f"{group_summary[o]['mean_query_median_P_ns']/1000:.2f}" for o in ('AB', 'BA'))
after = '/'.join(f"{group_summary[o]['mean_query_median_T_ns']/1000:.2f}" for o in ('AB', 'BA'))
with (out / 'decision.tsv').open('a') as stream:
    stream.write(f'06\tremove redundant bound-domain checks after admission\texact specialized small nonnegative bound\t{before} us\t{after} us\t{(ratios["AB"]-1)*100:+.2f}%/{(ratios["BA"]-1)*100:+.2f}%\t1440 tests;3x1227 exact gates;full AB/BA controls\tkept\tno new allocation;{delta:+d} binary bytes;per-query losses retained;port target open\n')
folder = out / 'attempt-06'
folder.mkdir()
for name in ('build-final', 'tests-targeted', 'tests', 'clippy', 'formatting', 'verify', 'configured-k0.9-b0.4', 'configured-k2-b1', 'pilot-final', 'measure', 'port-measure'):
    shutil.move(out / name, folder / name)
for name in ('summary.json', 'per-query.csv', 'port-summary.json', 'port-per-query.csv', 'attempt-06-metrics.json', 'attempt-06-acceptance.json'):
    shutil.move(out / name, folder / name)
for name in ('attempt-06-implementation.md', 'attempt-06-review.md'):
    shutil.copy2(out / name, folder / name)
for name in ('tantivy_worker', 'configured_worker'):
    shutil.copy2(out / 'target/release' / name, folder / name)
controllers = folder / 'controllers'
controllers.mkdir()
for name in ('balanced.py', 'compare.py', 'stage.py', 'configured_gate.py', 'pilot.py', 'analyze.py', 'port_measure.py', 'analyze_port.py', 'measure_audit.py'):
    shutil.copy2(out / name, controllers / name)
shutil.copy2(Path(__file__), controllers / Path(__file__).name)
(folder / 'accepted.patch').write_bytes(subprocess.check_output(['git', 'show', '--format=', '--binary', head], cwd=repo))
descriptor = {'binary': str(folder / 'tantivy_worker'), 'binary_sha256': sha(folder / 'tantivy_worker'),
    'source_commit': head, 'compiled_sources': str(folder / 'build-final/after.json'),
    'compiled_sources_sha256': sha(folder / 'build-final/after.json'),
    'exact_gate': str(folder / 'verify/receipt.json'), 'exact_gate_sha256': sha(folder / 'verify/receipt.json')}
(out / 'parent-06.json').write_text(json.dumps(descriptor, indent=2) + '\n')
assert not subprocess.check_output(['git', 'status', '--porcelain'], cwd=prod)
subprocess.run(['git', 'merge', '--ff-only', head], cwd=prod, check=True)
subprocess.run(['git', 'push', 'fork', 'HEAD:main'], cwd=prod, check=True)
remote = subprocess.check_output(['git', 'ls-remote', 'fork', 'refs/heads/main'], cwd=prod, text=True).split()[0]
assert remote == head
print(json.dumps({'acceptance': acceptance, 'fork_main': remote}))
