import hashlib,json,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
held=out.parents[1]/'tantivy-or-threshold-cutoff-oct11'
parent=json.loads((out/'parent-11.json').read_text())
static=json.loads((out/'attempt-22-static-receipt.json').read_text())
assert static['parent_commit']==parent['source_commit']
paths=static['edited_tracked_files']
assert paths==['src/query/boolean_query/or_maxscore.rs','src/query/score_combiner.rs']
for tree in (repo,held):
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=tree,text=True).strip()==parent['source_commit']
 assert not subprocess.check_output(['git','diff','--cached','--name-only'],cwd=tree)
assert not subprocess.check_output(['git','diff','--name-only'],cwd=repo)
assert subprocess.check_output(['git','diff','--name-only'],cwd=held,text=True).splitlines()==paths
for path in paths:assert hashlib.sha256((held/path).read_bytes()).hexdigest()==static['source_sha256'][path]
term='src/query/term_query/term_scorer.rs'
assert hashlib.sha256((held/term).read_bytes()).hexdigest()==static['term_source_unchanged_sha256']
patch=subprocess.check_output(['git','diff','--binary'],cwd=held)
assert patch==(out/'attempt-22-full.patch').read_bytes()
assert hashlib.sha256(patch).hexdigest()==static['patch_sha256']
for name in ('attempt-22-integration.json','tests-targeted','build-final','verify','pilot-final'):
 assert not (out/name).exists(),name
subprocess.run(['git','apply','-'],cwd=repo,input=patch,check=True)
for path in paths:assert hashlib.sha256((repo/path).read_bytes()).hexdigest()==static['source_sha256'][path]
subprocess.run(['git','diff','--check'],cwd=repo,check=True)
(out/'attempt-22-integration.json').write_text(json.dumps({'parent':parent['source_commit'],'source_sha256':static['source_sha256'],'patch_sha256':static['patch_sha256'],'review':'Root read full authentic 259-addition/3-deletion two-file diff, all four independent numeric tests, exact classified cutoff proof and static identities. Original13OR and4numeric tests/helpers unchanged. Only optional predicate representation and immediate callback cache refresh; original factor/sums/cursors/API unchanged. Runtime owned by root.'},indent=2)+'\n')
subprocess.run([sys.executable,'root-run-stage.py','tests-targeted'],cwd=out,check=True)
