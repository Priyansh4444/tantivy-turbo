import hashlib
import json
import subprocess
from pathlib import Path

out = Path('/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07')
symbol = '<tantivy::query::boolean_query::boolean_weight::BooleanWeight<tantivy::query::score_combiner::SumCombiner> as tantivy::query::weight::Weight>::for_each_pruning'
for label in ('attempt-06-or3-clean-before-pivot', 'attempt-08-bitmap-or3-clean'):
    destination = out / (label + '-annotate.txt')
    assert not destination.exists()
    result = subprocess.run(['perf', 'annotate', '--stdio', '-i', str(out / (label + '-perf.data')), '--symbol', symbol], capture_output=True, text=True, check=True)
    destination.write_text(result.stdout)
    (out / (label + '-annotate.stderr')).write_text(result.stderr)
    hot = []
    for line in result.stdout.splitlines():
        parts = line.strip().split(':', 1)
        try:
            percent = float(parts[0])
        except ValueError:
            continue
        if percent >= 0.5:
            hot.append({'percent': percent, 'assembly': parts[1].strip()})
    hot.sort(key=lambda row: -row['percent'])
    (out / (label + '-hot-instructions.json')).write_text(json.dumps(hot, indent=2) + '\n')
    print(label, json.dumps(hot[:16]))
label = 'attempt-08-bitmap-or3-clean'
files = [out / (label + '-' + suffix) for suffix in ('perf.data', 'perf-report.txt', 'perf.stderr', 'profile-receipt.json', 'profile-replies.json', 'controller.py')]
digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
before = {str(path): digest(path) for path in files}
guard = subprocess.run(['python', 'profile-cases-safe.py', '--label', label, '--binary', 'target/release/tantivy_worker', '--cases', 'profile-or3-all.json'], cwd=out, capture_output=True, text=True)
after = {str(path): digest(path) for path in files}
assert guard.returncode != 0 and 'Profile label already exists' in guard.stderr and before == after
receipt = {'label': label, 'returncode': guard.returncode, 'all_six_files_unchanged': before == after, 'before': before, 'after': after}
(out / 'attempt-08-profile-overwrite-guard.json').write_text(json.dumps(receipt, indent=2) + '\n')
print('All six profile artifacts protected by duplicate-label guard.')
