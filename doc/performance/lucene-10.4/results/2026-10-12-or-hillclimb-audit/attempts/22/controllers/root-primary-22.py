import json,re,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
source=out.parents[1]/'tantivy-or-hillclimb-oct07/src/query/boolean_query/or_maxscore.rs'
static=json.loads((out/'attempt-22-static-receipt.json').read_text())
source_names=re.findall(r'#\[test\]\s+fn\s+(\w+)',source.read_text())
actual_names=re.findall(r'test query::boolean_query::or_maxscore::tests::(\w+) \.\.\. ok',(out/'tests-targeted/stdout.log').read_text())
assert sorted(source_names)==sorted(static['test_names'])==sorted(actual_names)
assert len(source_names)==len(set(source_names))==static['test_count']==13
r=json.loads((out/'tests-targeted/receipt.json').read_text());assert r['returncode']==0 and r['source_unchanged']
assert (out/'tests-targeted/compiler.txt').read_bytes()==(out/'attempt-11/tests-targeted/compiler.txt').read_bytes()
score_source=out.parents[1]/'tantivy-or-hillclimb-oct07/src/query/score_combiner.rs'
command=json.loads((out/'attempt-11/tests-targeted/command.json').read_text())
command=[arg.replace('query::boolean_query::or_maxscore::tests','query::score_combiner::tests') for arg in command]
subprocess.run([sys.executable,'stage.py','tests-targeted-score','1200',*command],cwd=out,check=True)
score_names=re.findall(r'#\[test\]\s+fn\s+(\w+)',score_source.read_text())
actual_score=re.findall(r'test query::score_combiner::tests::(\w+) \.\.\. ok',(out/'tests-targeted-score/stdout.log').read_text())
assert sorted(score_names)==sorted(static['numeric_test_names'])==sorted(actual_score)
assert len(score_names)==len(set(score_names))==static['numeric_test_count']==8
r=json.loads((out/'tests-targeted-score/receipt.json').read_text());assert r['returncode']==0 and r['source_unchanged']
subprocess.run([sys.executable,'root-run-stage.py','build-final'],cwd=out,check=True)
assert (out/'build-final/compiler.txt').read_bytes()==(out/'attempt-11/build-final/compiler.txt').read_bytes()
for phase in ('verify','configured','pilot'):
 subprocess.run([sys.executable,'compare.py',phase,'--parent','parent-11.json'],cwd=out,check=True)
