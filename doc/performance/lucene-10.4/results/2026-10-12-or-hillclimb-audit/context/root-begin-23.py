import json,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
held=out.parents[1]/'tantivy-or-cutoff-all-bounds-oct12'
sys.path.insert(0,str(out))
from stage import sha
parent=json.loads((out/'parent-22.json').read_text())
static=json.loads((out/'attempt-23-static-receipt.json').read_text())
assert static['parent_commit']==parent['source_commit']
paths=['src/query/boolean_query/or_maxscore.rs','src/query/score_combiner.rs']
assert static['edited_tracked_files']==paths
for tree in (repo,held):
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=tree,text=True).strip()==parent['source_commit']
 assert not subprocess.check_output(['git','diff','--cached','--name-only'],cwd=tree)
assert not subprocess.check_output(['git','diff','--name-only'],cwd=repo)
assert subprocess.check_output(['git','diff','--name-only'],cwd=held,text=True).splitlines()==paths
for path in paths:assert sha(held/path)==static['source_sha256'][path]
patch=subprocess.check_output(['git','diff','--binary'],cwd=held)
assert patch==(out/'attempt-23-full.patch').read_bytes() and sha(out/'attempt-23-full.patch')==static['patch_sha256']
for name in ('attempt-23-integration.json','tests-targeted','tests-targeted-score','build-final','verify','pilot-final','measure','port-measure'):
 assert not (out/name).exists(),name
subprocess.run(['git','apply','-'],cwd=repo,input=patch,check=True)
for path in paths:assert sha(repo/path)==static['source_sha256'][path]
subprocess.run(['git','diff','--check'],cwd=repo,check=True)
(out/'attempt-23-integration.json').write_text(json.dumps({'parent':parent['source_commit'],'source_sha256':static['source_sha256'],'patch_sha256':static['patch_sha256'],'review':'Root read the complete authentic two-file diff. Explicit ordered greater handles NaN; short-circuited local at-most preserves array bounds. Same cutoff lifetime/refresh, original multiplication/sums, optional predicate, canonical callbacks/cursors/fallback. All13OR and four original numeric families remain byte-identical; strengthened tests compare both predicates with literal old successor/cast.'},indent=2)+'\n')
subprocess.run([sys.executable,'root-run-stage.py','tests-targeted'],cwd=out,check=True)
