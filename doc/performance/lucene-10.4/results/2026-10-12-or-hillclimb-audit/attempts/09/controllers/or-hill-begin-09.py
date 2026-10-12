import hashlib,json,subprocess,sys
from pathlib import Path
out=Path('/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07')
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
trial=out.parents[1]/'tantivy-or-posting-heap-oct07'
path='src/query/boolean_query/or_maxscore.rs'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
expected='4e97064413426debda620071e274215f64e1e4bddfdde5624e5a3dd86d56cad5'
parent=json.loads((out/'parent-06.json').read_text())
for tree in (repo,trial):
    assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=tree,text=True).strip()==parent['source_commit']
assert not subprocess.check_output(['git','diff','--name-only'],cwd=repo)
assert subprocess.check_output(['git','diff','--name-only'],cwd=trial,text=True).splitlines()==[path]
assert sha(trial/path)==expected
patch=subprocess.check_output(['git','diff','--binary','--',path],cwd=trial)
(out/'attempt-09-original.patch').write_bytes(patch)
subprocess.run(['git','apply','-'],cwd=repo,input=patch,check=True)
assert sha(repo/path)==expected
subprocess.run(['git','diff','--check'],cwd=repo,check=True)
(out/'attempt-09.patch').write_bytes(subprocess.check_output(['git','diff','--binary'],cwd=repo))
(out/'attempt-09-integration.json').write_text(json.dumps({'parent':parent['source_commit'],'source_sha256':expected,'adaptation':'None; integrated source byte-identical to isolated reviewed source.'},indent=2)+'\n')
command=['cargo','test','--locked','--manifest-path','../../../tantivy-or-hillclimb-oct07/Cargo.toml','--target-dir',str(out/'test-target'),'--lib','query::boolean_query::or_maxscore::tests','--','--test-threads=2','--nocapture']
subprocess.run([sys.executable,'stage.py','tests-targeted','1200',*command],cwd=out,check=True)
