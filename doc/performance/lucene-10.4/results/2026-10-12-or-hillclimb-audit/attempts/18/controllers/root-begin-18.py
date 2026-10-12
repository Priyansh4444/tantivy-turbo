import hashlib,json,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
trial=out.parents[1]/'tantivy-or-two-essential-min-oct11'
parent=json.loads((out/'parent-11.json').read_text())
path='src/query/boolean_query/or_maxscore.rs'
digest='252c08f4e6401bd5b1c497fefe6e3885c2a10bf233ecd87085ddcca7ef514819'
patch=(out/'attempt-18-source.patch').read_bytes()
assert hashlib.sha256(patch).hexdigest()=='fbb5f88a6b6fea9c321e9efa9e3518db7269ffa7047d3af1b96cbafb106704e4'
for tree in (repo,trial):
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=tree,text=True).strip()==parent['source_commit']
assert not subprocess.check_output(['git','diff','--name-only'],cwd=repo)
assert subprocess.check_output(['git','diff','--name-only'],cwd=trial,text=True).splitlines()==[path]
assert hashlib.sha256((trial/path).read_bytes()).hexdigest()==digest
assert subprocess.check_output(['git','diff','--binary'],cwd=trial)==patch
assert not (out/'attempt-18-integration.json').exists()
for phase in ('tests-targeted','build-final','verify','pilot-final'):assert not (out/phase).exists()
subprocess.run(['git','apply','-'],cwd=repo,input=patch,check=True)
assert hashlib.sha256((repo/path).read_bytes()).hexdigest()==digest
subprocess.run(['git','diff','--check'],cwd=repo,check=True)
(out/'attempt-18-integration.json').write_text(json.dumps({'parent':parent['source_commit'],'source_sha256':{path:digest},'patch_sha256':hashlib.sha256(patch).hexdigest(),'review':'Root read complete source patch and report. Exactly-two fixed-partition direct cursor minimum; all old replay and fixtures unchanged; independent canonical trace and actual route fixture.'},indent=2)+'\n')
subprocess.run([sys.executable,'root-run-stage.py','tests-targeted'],cwd=out,check=True)
