import json,re,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
source=out.parents[1]/'tantivy-or-hillclimb-oct07/src/query/boolean_query/or_maxscore.rs'
expected_names=set(re.findall(r'#\[test\]\s+fn\s+(\w+)',source.read_text()))
actual_names=set(re.findall(r'test query::boolean_query::or_maxscore::tests::(\w+) \.\.\. ok', (out/'tests-targeted/stdout.log').read_text()))
assert expected_names==actual_names and len(expected_names)==32
for phase,passed in (('tests-targeted',len(expected_names)),('tests-targeted-term',21),('tests-targeted-postings',15)):
    r=json.loads((out/phase/'receipt.json').read_text())
    assert r['returncode']==0 and r['source_unchanged']
    assert (out/phase/'compiler.txt').read_bytes()==(out/'attempt-11/tests-targeted/compiler.txt').read_bytes()
    assert f'{passed} passed; 0 failed' in (out/phase/'stdout.log').read_text()
subprocess.run([sys.executable,'root-run-stage.py','build-final'],cwd=out,check=True)
assert (out/'build-final/compiler.txt').read_bytes()==(out/'attempt-11/build-final/compiler.txt').read_bytes()
for phase in ('verify','configured','pilot'):
    subprocess.run([sys.executable,'compare.py',phase,'--parent','parent-11.json'],cwd=out,check=True)
