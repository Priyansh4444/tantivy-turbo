import hashlib,json,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
trial=out.parents[1]/'tantivy-or-density-certificate-oct08'
parent=json.loads((out/'parent-11.json').read_text())
path='src/query/boolean_query/or_maxscore.rs'
expected='602867a5085c86c6923e15234a6ce381519b2834af6c627ef910db393e6d3477'
for tree in (repo,trial):
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=tree,text=True).strip()==parent['source_commit']
assert not subprocess.check_output(['git','diff','--name-only'],cwd=repo)
assert subprocess.check_output(['git','diff','--name-only'],cwd=trial,text=True).splitlines()==[path]
assert hashlib.sha256((trial/path).read_bytes()).hexdigest()==expected
patch=subprocess.check_output(['git','diff','--binary'],cwd=trial)
assert patch==(out/'attempt-15-full.patch').read_bytes()
subprocess.run(['git','apply','-'],cwd=repo,input=patch,check=True)
assert hashlib.sha256((repo/path).read_bytes()).hexdigest()==expected
subprocess.run(['git','diff','--check'],cwd=repo,check=True)
(out/'attempt-15-integration.json').write_text(json.dumps({'parent':parent['source_commit'],'source_sha256':{path:expected},'review':'Root full actual diff; independent density15 source review. Attempt13 rejected and both files restored before this graft.'},indent=2)+'\n')
subprocess.run([sys.executable,'root-run-stage.py','tests-targeted'],cwd=out,check=True)
