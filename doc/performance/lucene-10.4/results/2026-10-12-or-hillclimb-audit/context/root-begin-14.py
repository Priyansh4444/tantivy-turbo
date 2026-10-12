import hashlib,json,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
trial=out.parents[1]/'tantivy-or-contribution-mask-oct08'
parent=json.loads((out/'parent-11.json').read_text())
path='src/query/boolean_query/or_maxscore.rs'
expected='6db826184ecfc6e745940b8049da6fd3755ad622085429ffca8fbb794bcb5010'
sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest()
for r in (repo,trial):
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=r,text=True).strip()==parent['source_commit']
assert not subprocess.check_output(['git','diff','--name-only'],cwd=repo)
assert subprocess.check_output(['git','diff','--name-only'],cwd=trial,text=True).splitlines()==[path]
assert sha(trial/path)==expected
patch=subprocess.check_output(['git','diff','--binary','--',path],cwd=trial)
(out/'attempt-14-full.patch').write_bytes(patch)
(out/'attempt-14-source.rs').write_bytes((trial/path).read_bytes())
subprocess.run(['git','apply','-'],cwd=repo,input=patch,check=True)
assert sha(repo/path)==expected
subprocess.run(['git','diff','--check'],cwd=repo,check=True)
(out/'attempt-14-integration.json').write_text(json.dumps({'parent':parent['source_commit'],'source_sha256':expected,'review':'Root read full actual diff and numerical proof; isolated candidate, no attempt12/13 geometry.'},indent=2)+'\n')
subprocess.run([sys.executable,'root-run-stage.py','tests-targeted'],cwd=out,check=True)
