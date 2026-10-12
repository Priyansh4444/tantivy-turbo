import json,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
sys.path.insert(0,str(out))
from stage import sha
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
static=json.loads((out/'attempt-23-static-receipt.json').read_text())
for path,digest in static['source_sha256'].items():assert sha(repo/path)==digest
for gate in ('verify','configured-k0.9-b0.4','configured-k2-b1'):
 assert json.loads((out/gate/'receipt.json').read_text())['pass']
pilot=json.loads((out/'pilot-final/receipt.json').read_text())
group=next(r for r in pilot['summary'] if r['set']=='or3plus')
assert pilot['unchanged'] and all(group[o]['ratio_mean']<1 for o in ('AB','BA'))
review=(out/'attempt-23-independent-review.md').read_text()
assert 'No concrete correctness blocker found' in review and all(d in review for d in static['source_sha256'].values())
for phase in ('tests','clippy','formatting'):
 subprocess.run([sys.executable,'root-run-stage.py',phase],cwd=out,check=True)
command=json.loads((out/'attempt-22/formatting-score/command.json').read_text())
subprocess.run([sys.executable,'stage.py','formatting-score','120',*command],cwd=out,check=True)
subprocess.run([sys.executable,'compare.py','measure','--parent','parent-22.json'],cwd=out,check=True)
subprocess.run([sys.executable,'analyze.py'],cwd=out,check=True)
subprocess.run([sys.executable,'measure_audit.py','--parent','parent-22.json','--attempt','23'],cwd=out,check=True)
