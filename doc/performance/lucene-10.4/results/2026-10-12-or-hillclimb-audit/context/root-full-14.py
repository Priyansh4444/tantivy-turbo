import hashlib,json,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
assert hashlib.sha256((repo/'src/query/boolean_query/or_maxscore.rs').read_bytes()).hexdigest()=='6db826184ecfc6e745940b8049da6fd3755ad622085429ffca8fbb794bcb5010'
for name in ('verify','configured-k0.9-b0.4','configured-k2-b1'):
 assert json.loads((out/name/'receipt.json').read_text())['pass']
assert json.loads((out/'pilot-final/receipt.json').read_text())['unchanged']
for phase in ('formatting','tests','clippy'):
 subprocess.run([sys.executable,'root-run-stage.py',phase],cwd=out,check=True)
subprocess.run([sys.executable,'compare.py','measure','--parent','parent-11.json'],cwd=out,check=True)
subprocess.run([sys.executable,'analyze.py'],cwd=out,check=True)
subprocess.run([sys.executable,'measure_audit.py','--parent','parent-11.json','--attempt','14'],cwd=out,check=True)
