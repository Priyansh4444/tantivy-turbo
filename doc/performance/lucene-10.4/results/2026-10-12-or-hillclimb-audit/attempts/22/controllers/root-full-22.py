import hashlib,json,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
static=json.loads((out/'attempt-22-static-receipt.json').read_text())
for path,digest in static['source_sha256'].items():assert hashlib.sha256((repo/path).read_bytes()).hexdigest()==digest
for gate in ('verify','configured-k0.9-b0.4','configured-k2-b1'):
 assert json.loads((out/gate/'receipt.json').read_text())['pass']
pilot=json.loads((out/'pilot-final/receipt.json').read_text())
group=next(r for r in pilot['summary'] if r['set']=='or3plus')
assert pilot['unchanged'] and all(group[o]['ratio_mean']<1 for o in ('AB','BA'))
review=(out/'attempt-22-independent-review.md').read_text()
assert 'No concrete production correctness blocker' in review and all(digest in review for digest in static['source_sha256'].values())
for phase in ('tests','clippy','formatting'):
 subprocess.run([sys.executable,'root-run-stage.py',phase],cwd=out,check=True)
command=json.loads((out/'attempt-11/formatting/command.json').read_text())
command=[arg.replace('src/query/boolean_query/or_maxscore.rs','src/query/score_combiner.rs') for arg in command]
subprocess.run([sys.executable,'stage.py','formatting-score','120',*command],cwd=out,check=True)
subprocess.run([sys.executable,'compare.py','measure','--parent','parent-11.json'],cwd=out,check=True)
subprocess.run([sys.executable,'analyze.py'],cwd=out,check=True)
subprocess.run([sys.executable,'measure_audit.py','--parent','parent-11.json','--attempt','22'],cwd=out,check=True)
