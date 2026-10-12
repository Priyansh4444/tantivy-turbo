import hashlib,json,shutil,subprocess,sys
from pathlib import Path
out=Path('/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07')
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
trial=out.parents[1]/'tantivy-or-posting-heap-oct07'
path='src/query/boolean_query/or_maxscore.rs'
sha=lambda data:hashlib.sha256(data).hexdigest()
old=(repo/path).read_bytes();new=(trial/path).read_bytes()
assert sha(old)=='4e97064413426debda620071e274215f64e1e4bddfdde5624e5a3dd86d56cad5'
assert sha(new)=='41065df1d96bb7c585e0593c68b0df09ee717b97366efc253bd036f062725f12'
before=b'                if scorer.seek(candidate.doc) == candidate.doc {'
after=b'                if scorer.doc() < candidate.doc {\n                    scorer.seek(candidate.doc);\n                }\n                if scorer.doc() == candidate.doc {'
assert old.count(before)==1 and old.replace(before,after)==new
assert old.split(b'#[cfg(test)]\nmod tests')[0]==new.split(b'#[cfg(test)]\nmod tests')[0]
failed=out/'attempt-09-pre-oracle-fix';failed.mkdir()
shutil.move(out/'tests-targeted',failed/'tests-targeted')
(failed/'prepared-source.rs').write_bytes(old)
(repo/path).write_bytes(new)
subprocess.run(['git','diff','--check'],cwd=repo,check=True)
(out/'attempt-09.patch').write_bytes(subprocess.check_output(['git','diff','--binary'],cwd=repo))
(out/'attempt-09-integration.json').write_text(json.dumps({'parent':'a674aaf2f6187f3010a135dc25fdacf9cab27e53','source_sha256':sha(new),'adaptation':'None relative to corrected isolated source. Observer seeks forward only; production byte-identical to pre-fix version.'},indent=2)+'\n')
command=['cargo','test','--locked','--manifest-path','../../../tantivy-or-hillclimb-oct07/Cargo.toml','--target-dir',str(out/'test-target'),'--lib','query::boolean_query::or_maxscore::tests','--','--test-threads=2','--nocapture']
subprocess.run([sys.executable,'stage.py','tests-targeted','1200',*command],cwd=out,check=True)
subprocess.run([sys.executable,'/tmp/or-hill-run.py','build'],check=True)
subprocess.run([sys.executable,'/tmp/or-hill-gates.py','parent-06.json'],check=True)
subprocess.run([sys.executable,'compare.py','pilot','--parent','parent-06.json'],cwd=out,check=True)
