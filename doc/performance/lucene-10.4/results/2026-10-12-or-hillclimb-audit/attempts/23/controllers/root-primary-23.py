import json,re,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
static=json.loads((out/'attempt-23-static-receipt.json').read_text())
for path,prefix,field in (('src/query/boolean_query/or_maxscore.rs','query::boolean_query::or_maxscore::tests','test_names'),):
 source_names=re.findall(r'#\[test\]\s+fn\s+(\w+)',(repo/path).read_text())
 actual_names=re.findall(r'test '+re.escape(prefix)+r'::(\w+) \.\.\. ok',(out/'tests-targeted/stdout.log').read_text())
 assert sorted(source_names)==sorted(static[field])==sorted(actual_names)
 assert len(source_names)==len(set(source_names))==13
r=json.loads((out/'tests-targeted/receipt.json').read_text());assert r['returncode']==0 and r['source_unchanged']
command=json.loads((out/'attempt-22/tests-targeted-score/command.json').read_text())
subprocess.run([sys.executable,'stage.py','tests-targeted-score','1200',*command],cwd=out,check=True)
names=re.findall(r'#\[test\]\s+fn\s+(\w+)',(repo/'src/query/score_combiner.rs').read_text())
actual=re.findall(r'test query::score_combiner::tests::(\w+) \.\.\. ok',(out/'tests-targeted-score/stdout.log').read_text())
assert sorted(names)==sorted(static['numeric_test_names'])==sorted(actual) and len(names)==len(set(names))==8
r=json.loads((out/'tests-targeted-score/receipt.json').read_text());assert r['returncode']==0 and r['source_unchanged']
subprocess.run([sys.executable,'root-run-stage.py','build-final'],cwd=out,check=True)
assert (out/'build-final/compiler.txt').read_bytes()==(out/'attempt-22/build-final/compiler.txt').read_bytes()
for phase in ('verify','configured','pilot'):
 subprocess.run([sys.executable,'compare.py',phase,'--parent','parent-22.json'],cwd=out,check=True)
