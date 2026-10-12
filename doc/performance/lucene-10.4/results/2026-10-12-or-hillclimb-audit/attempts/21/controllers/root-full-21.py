import hashlib,json,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
expected={'src/query/boolean_query/or_maxscore.rs':'4bb058a4dd125ee819776b50f4506b01419bf42bba8cb2d6a0994a30cceae136'}
for path,digest in expected.items():assert hashlib.sha256((repo/path).read_bytes()).hexdigest()==digest
for gate in ('verify','configured-k0.9-b0.4','configured-k2-b1'):
    assert json.loads((out/gate/'receipt.json').read_text())['pass']
pilot=json.loads((out/'pilot-final/receipt.json').read_text())
group=next(r for r in pilot['summary'] if r['set']=='or3plus')
assert pilot['unchanged'] and all(group[o]['ratio_mean']<1 for o in ('AB','BA'))
for phase in ('tests','clippy','formatting'):
    subprocess.run([sys.executable,'root-run-stage.py',phase],cwd=out,check=True)
subprocess.run([sys.executable,'compare.py','measure','--parent','parent-11.json'],cwd=out,check=True)
subprocess.run([sys.executable,'analyze.py'],cwd=out,check=True)
subprocess.run([sys.executable,'measure_audit.py','--parent','parent-11.json','--attempt','21'],cwd=out,check=True)
