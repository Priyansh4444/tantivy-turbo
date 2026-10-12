import hashlib,json,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
held=out.parents[1]/'tantivy-or-prefix-register-oct11'
parent=json.loads((out/'parent-11.json').read_text())
static=json.loads((out/'attempt-21-static-receipt.json').read_text())
assert static['parent_commit']==parent['source_commit']
paths=static['edited_tracked_files']
assert paths==['src/query/boolean_query/or_maxscore.rs']
for tree in (repo,held):
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=tree,text=True).strip()==parent['source_commit']
 assert not subprocess.check_output(['git','diff','--cached','--name-only'],cwd=tree)
assert not subprocess.check_output(['git','diff','--name-only'],cwd=repo)
assert subprocess.check_output(['git','diff','--name-only'],cwd=held,text=True).splitlines()==paths
for path in paths:assert hashlib.sha256((held/path).read_bytes()).hexdigest()==static['source_sha256'][path]
term='src/query/term_query/term_scorer.rs'
assert hashlib.sha256((held/term).read_bytes()).hexdigest()==static['term_source_unchanged_sha256']
patch=subprocess.check_output(['git','diff','--binary'],cwd=held)
assert patch==(out/'attempt-21-full.patch').read_bytes()
assert hashlib.sha256(patch).hexdigest()==static['patch_sha256']
for name in ('attempt-21-integration.json','tests-targeted','build-final','verify','pilot-final'):
 assert not (out/name).exists(),name
subprocess.run(['git','apply','-'],cwd=repo,input=patch,check=True)
for path in paths:assert hashlib.sha256((repo/path).read_bytes()).hexdigest()==static['source_sha256'][path]
subprocess.run(['git','diff','--check'],cwd=repo,check=True)
(out/'attempt-21-integration.json').write_text(json.dumps({'parent':parent['source_commit'],'source_sha256':static['source_sha256'],'patch_sha256':static['patch_sha256'],'review':'Root read full authentic three-statement diff, exact reset/sequential-add proof and static identities. All parent13test bodies and helpers unchanged; no scoring/cursor/API change. Runtime owned by root.'},indent=2)+'\n')
subprocess.run([sys.executable,'root-run-stage.py','tests-targeted'],cwd=out,check=True)
