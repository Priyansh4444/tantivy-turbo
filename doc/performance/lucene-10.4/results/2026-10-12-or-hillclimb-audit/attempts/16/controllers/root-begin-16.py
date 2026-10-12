"""Integrate exactly the held capped-expansion source, then run its owned fixtures."""
import hashlib,json,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
trial=out.parents[1]/'tantivy-or-capped-expansion-oct10'
parent=json.loads((out/'parent-11.json').read_text())
held=json.loads((out/'attempt-16-static-receipt.json').read_text())
paths=sorted(held['source_sha256'])
assert held['parent']==parent['source_commit']
for tree in (repo,trial):
    assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=tree,text=True).strip()==parent['source_commit']
assert not subprocess.check_output(['git','diff','--name-only'],cwd=repo)
assert sorted(subprocess.check_output(['git','diff','--name-only'],cwd=trial,text=True).splitlines())==paths
for path in paths:
    assert hashlib.sha256((trial/path).read_bytes()).hexdigest()==held['source_sha256'][path]
patch=subprocess.check_output(['git','diff','--binary'],cwd=trial)
assert patch==(out/'attempt-16-full.patch').read_bytes()
assert hashlib.sha256(patch).hexdigest()==held['patch_sha256']
integration=out/'attempt-16-integration.json'
assert not integration.exists()
for phase in ('tests-targeted','tests-targeted-term','tests-targeted-postings','build-final','verify','pilot-final'):
    assert not (out/phase).exists(),phase
subprocess.run(['git','apply','-'],cwd=repo,input=patch,check=True)
for path in paths:
    assert hashlib.sha256((repo/path).read_bytes()).hexdigest()==held['source_sha256'][path]
subprocess.run(['git','diff','--check'],cwd=repo,check=True)
integration.write_text(json.dumps({'parent':parent['source_commit'],'source_sha256':held['source_sha256'],'patch_sha256':held['patch_sha256'],'review':'Root read full13 inherited mechanism, final16-vs13 patch, implementation matrix, actual production cap and certificate preparation. Independent source review also requested; root owns runtime.'},indent=2)+'\n')
subprocess.run([sys.executable,'root-run-stage.py','tests-targeted'],cwd=out,check=True)
for phase in ('tests-targeted-term','tests-targeted-postings'):
    command=json.loads((out/'attempt-13'/phase/'command.json').read_text())
    subprocess.run([sys.executable,'stage.py',phase,'1200',*command],cwd=out,check=True)
