import json,re,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
source=out.parents[1]/'tantivy-or-hillclimb-oct07/src/query/boolean_query/or_maxscore.rs'
static=json.loads((out/'attempt-21-static-receipt.json').read_text())
source_names=re.findall(r'#\[test\]\s+fn\s+(\w+)',source.read_text())
actual_names=re.findall(r'test query::boolean_query::or_maxscore::tests::(\w+) \.\.\. ok',(out/'tests-targeted/stdout.log').read_text())
assert sorted(source_names)==sorted(static['test_names'])==sorted(actual_names)
assert len(source_names)==len(set(source_names))==static['test_count']==13
r=json.loads((out/'tests-targeted/receipt.json').read_text());assert r['returncode']==0 and r['source_unchanged']
assert (out/'tests-targeted/compiler.txt').read_bytes()==(out/'attempt-11/tests-targeted/compiler.txt').read_bytes()
subprocess.run([sys.executable,'root-run-stage.py','build-final'],cwd=out,check=True)
assert (out/'build-final/compiler.txt').read_bytes()==(out/'attempt-11/build-final/compiler.txt').read_bytes()
for phase in ('verify','configured','pilot'):
 subprocess.run([sys.executable,'compare.py',phase,'--parent','parent-11.json'],cwd=out,check=True)
