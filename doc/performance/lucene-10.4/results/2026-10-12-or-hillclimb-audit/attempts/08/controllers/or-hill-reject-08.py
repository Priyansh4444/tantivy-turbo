import json
import shutil
import subprocess
import sys
from pathlib import Path

out = Path('/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07')
repo = out.parents[1] / 'tantivy-or-hillclimb-oct07'
sys.path.insert(0, str(out))
from stage import sha, sources

parent = json.loads((out / 'parent-06.json').read_text())
assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip() == parent['source_commit']
assert json.loads((out / 'build-final/after.json').read_text()) == sources()
for name in ('build-final', 'tests-targeted', 'formatting'):
    receipt = json.loads((out / name / 'receipt.json').read_text())
    assert receipt['returncode'] == 0 and receipt['source_unchanged']
for name in ('verify', 'configured-k0.9-b0.4', 'configured-k2-b1'):
    assert json.loads((out / name / 'receipt.json').read_text())['pass']
pilot = json.loads((out / 'pilot-final/receipt.json').read_text())
assert pilot['unchanged']
longer = next(row for row in pilot['summary'] if row['set'] == 'or3plus')
assert all(longer[order]['ratio_mean'] > 1.1 for order in ('AB', 'BA'))
path = 'src/query/boolean_query/or_maxscore.rs'
assert set(subprocess.check_output(['git', 'diff', '--name-only'], cwd=repo, text=True).splitlines()) == {path}
folder = out / 'attempt-08'
folder.mkdir()
for name in ('build-final', 'tests-targeted', 'formatting', 'verify', 'configured-k0.9-b0.4', 'configured-k2-b1', 'pilot-final'):
    shutil.move(out / name, folder / name)
shutil.move(out / 'attempt-08-pre-fixture-fix', folder / 'pre-fixture-fix')
for name in ('tantivy_worker', 'configured_worker'):
    shutil.copy2(out / 'target/release' / name, folder / name)
for name in ('attempt-08-implementation.md', 'attempt-08-review.md', 'attempt-08-integration.json', 'attempt-08-profile-overwrite-guard.json', 'profile-or3-all.json'):
    shutil.copy2(out / name, folder / name)
diagnostics = folder / 'diagnostics'
diagnostics.mkdir()
for pattern in ('attempt-08-bitmap-or3-clean-*', 'attempt-06-or3-clean-before-pivot-*'):
    for source in sorted(out.glob(pattern)):
        shutil.move(source, diagnostics / source.name)
controllers = folder / 'controllers'
controllers.mkdir()
for name in ('balanced.py', 'compare.py', 'stage.py', 'configured_gate.py', 'pilot.py', 'profile-cases-safe.py'):
    shutil.copy2(out / name, controllers / name)
for name in ('or-hill-reject-08.py', 'or-hill-profile-08-audit.py', 'or-hill-08-fixture-fix.py'):
    shutil.copy2(Path('/tmp') / name, controllers / name)
(folder / 'rejected.patch').write_bytes(subprocess.check_output(['git', 'diff', '--binary'], cwd=repo))
rejection = {'verdict': 'reverted', 'parent': parent['source_commit'],
    'reason': 'Complete longer-OR pilot is 12.79%/13.55% slower in AB/BA, with sparse query losses around 2x. Full controlled sampler and broad tests not run.',
    'pilot': pilot['summary'], 'targeted_tests': 19, 'exact_profiles': 3, 'queries_per_profile': 1227,
    'candidate_binary_sha256': sha(folder / 'tantivy_worker'),
    'candidate_file_delta_bytes': (folder / 'tantivy_worker').stat().st_size - Path(parent['binary']).stat().st_size,
    'diagnostics': 'All 103 longer ORs, sequential diagnostic-only profiles, zero lost samples, exact checksums. Query loop/batch boundary dominate; no cache-miss causal proof.',
    'fixture_correction': 'Explicit Score annotation after E0689; first failed compilation and duplicate unfixed rerun retained. Production mechanism unchanged.'}
(folder / 'rejection.json').write_text(json.dumps(rejection, indent=2) + '\n')
with (out / 'decision.tsv').open('a') as stream:
    stream.write('08\tbulk multiple essential leaves avoid repeated scalar scans\t128-address exact-leaf bitmap gather/replay\t275.79/283.75 us pilot\t311.08/322.20 us pilot\t+12.79%/+13.55%\t19 targeted;3x1227 exact gates;full sampler not run\treverted\tsparse queries ~2x losses;lazy128xn score matrix;failed fixture build retained;diagnostic profiles do not establish latency\n')
subprocess.run(['git', 'restore', '--', path], cwd=repo, check=True)
assert not subprocess.check_output(['git', 'diff', '--name-only'], cwd=repo)
print(json.dumps(rejection))
