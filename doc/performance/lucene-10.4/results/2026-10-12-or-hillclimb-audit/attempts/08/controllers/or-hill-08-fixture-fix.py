import json
import shutil
import subprocess
import sys
from pathlib import Path

out = Path('/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07')
repo = out.parents[1] / 'tantivy-or-hillclimb-oct07'
trial = out.parents[1] / 'tantivy-or-multi-bitmap-oct07'
sys.path.insert(0, str(out))
from stage import sha

path = 'src/query/boolean_query/or_maxscore.rs'
assert sha(repo / path) == 'bf9d2a3c345e27a20bb3d974b5492f9b05ce780081f54bd811892e40ed774f5d'
assert sha(trial / path) == 'c4f7402f22938906d111472b690285074a01ea2e3a224f5a0102773f1855abf0'
assert json.loads((out / 'tests-targeted/receipt.json').read_text())['returncode'] == 101
folder = out / 'attempt-08-pre-fixture-fix'
assert folder.exists()
assert json.loads((folder / 'tests-targeted/receipt.json').read_text())['returncode'] == 101
shutil.move(out / 'tests-targeted', folder / 'duplicate-unfixed-tests')
before = (repo / path).read_text()
old = '        let run = |bitmap: bool| {\n            let mut accepted = Vec::new();\n            let mut current = 0.1;'
new = old.replace('let mut current = 0.1;', 'let mut current: Score = 0.1;')
assert before.count(old) == 1
after = before.replace(old, new)
assert after.replace(new, old) == before
prepared = (trial / path).read_text()
parent = json.loads((out / 'parent-06.json').read_text())
accepted = subprocess.check_output(['git', 'show', parent['source_commit'] + ':' + path], cwd=repo, text=True)
a = accepted.index('    // Numeric admission above')
b = accepted.index('    let mut clauses:', a)
assert after.replace('use crate::query::score_combiner::{SmallScoreSumUpperBound, SumCombiner};', 'use crate::query::score_combiner::{ScoreSumUpperBound, SumCombiner};').replace(accepted[a:b], '    let upper_bound = ScoreSumUpperBound::new(num_terms);\n') == prepared
(repo / path).write_text(after)
subprocess.run(['git', 'diff', '--check'], cwd=repo, check=True)
(out / 'attempt-08.patch').write_bytes(subprocess.check_output(['git', 'diff', '--binary'], cwd=repo))
(out / 'attempt-08-integration.json').write_text(json.dumps({'parent': parent['source_commit'], 'source_sha256': sha(repo / path), 'prepared_source_sha256': sha(trial / path), 'adaptation': 'Accepted06 import/constructor retained; isolated fixture E0689 corrected by explicit Score annotation. Reversal of06 substitutions reproduces reviewed prepared08 source byte-exact.', 'failed_gate_preserved': str(folder)}, indent=2) + '\n')
print('Grafted one fixture type annotation; complete failed receipts retained.')
