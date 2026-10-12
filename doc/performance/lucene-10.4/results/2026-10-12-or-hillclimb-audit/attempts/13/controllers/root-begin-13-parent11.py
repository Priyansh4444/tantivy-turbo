import hashlib,json,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
trial=out.parents[1]/'tantivy-or-metadata-range-oct08'
parent=json.loads((out/'parent-11.json').read_text())
expected={
 'src/query/boolean_query/or_maxscore.rs':'1a1ce59b330b4a207d353385bf90cac96333cd4c6da49ed52a7c40cd5bb30804',
 'src/query/term_query/term_scorer.rs':'21b05a2b6513223a81613ee0f40c4a31f6990a427bb0d4b60324154a55753cdb'}
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
for r in (repo,trial):
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=r,text=True).strip()==parent['source_commit']
assert not subprocess.check_output(['git','diff','--name-only'],cwd=repo)
assert subprocess.check_output(['git','diff','--name-only'],cwd=trial,text=True).splitlines()==list(expected)
for path,digest in expected.items():assert sha(trial/path)==digest
patch=subprocess.check_output(['git','diff','--binary'],cwd=trial)
assert patch==(out/'attempt-13-full.patch').read_bytes()
subprocess.run(['git','apply','-'],cwd=repo,input=patch,check=True)
for path,digest in expected.items():assert sha(repo/path)==digest
subprocess.run(['git','diff','--check'],cwd=repo,check=True)
(out/'attempt-13-integration.json').write_text(json.dumps({'parent':parent['source_commit'],'source_sha256':expected,'review':'Root full actual diff and independent13 review; rejected12 geometry plus one metadata-range mechanism. No candidate14 imported.'},indent=2)+'\n')
subprocess.run([sys.executable,'root-run-stage.py','tests-targeted'],cwd=out,check=True)
command=json.loads((out/'attempt-11/tests-targeted/command.json').read_text())
for phase,selector in [('tests-targeted-term','query::term_query::term_scorer::tests'),('tests-targeted-postings','postings::segment_postings::tests')]:
 next_command=[selector if arg=='query::boolean_query::or_maxscore::tests' else arg for arg in command]
 subprocess.run([sys.executable,'stage.py',phase,'1200',*next_command],cwd=out,check=True)
