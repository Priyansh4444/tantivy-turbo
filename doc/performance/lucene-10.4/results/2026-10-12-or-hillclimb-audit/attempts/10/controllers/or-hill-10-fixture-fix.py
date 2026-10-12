import hashlib,json,shutil,subprocess,sys
from pathlib import Path
out=Path('/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07')
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
trial=out.parents[1]/'tantivy-or-single-lead-oct07'
path='src/query/boolean_query/or_maxscore.rs'
sha=lambda data:hashlib.sha256(data).hexdigest()
old=(repo/path).read_bytes();new=(trial/path).read_bytes()
assert sha(old)=='1e57a16764161506a8dc1e62a909e13f5b21604c161b6992eea8d53ebc4bc7dd'
assert sha(new)=='81cb888e53e467a03515f0224aeefbe3bd3b05a25eaf6cd294491eea578a9729'
before=b'let top = searcher.search(&query, &TopDocs::with_limit(1))?;'
after=b'let top = searcher.search(&query, &TopDocs::with_limit(1).order_by_score())?;'
assert old.count(before)==1 and old.replace(before,after)==new
assert old.split(b'#[cfg(test)]\nmod tests')[0]==new.split(b'#[cfg(test)]\nmod tests')[0]
failed=out/'attempt-10-pre-fixture-fix';failed.mkdir()
shutil.move(out/'tests-targeted',failed/'tests-targeted')
(failed/'prepared-source.rs').write_bytes(old)
(repo/path).write_bytes(new)
subprocess.run(['git','diff','--check'],cwd=repo,check=True)
(out/'attempt-10.patch').write_bytes(subprocess.check_output(['git','diff','--binary'],cwd=repo))
(out/'attempt-10-integration.json').write_text(json.dumps({'parent':'a674aaf2f6187f3010a135dc25fdacf9cab27e53','source_sha256':sha(new),'adaptation':'None relative to corrected isolated source. Explicit score-order collector in new test; production unchanged.'},indent=2)+'\n')
command=['cargo','test','--locked','--manifest-path','../../../tantivy-or-hillclimb-oct07/Cargo.toml','--target-dir',str(out/'test-target'),'--lib','query::boolean_query::or_maxscore::tests','--','--test-threads=2','--nocapture']
subprocess.run([sys.executable,'stage.py','tests-targeted','1200',*command],cwd=out,check=True)
subprocess.run([sys.executable,'/tmp/or-hill-run.py','build'],check=True)
subprocess.run([sys.executable,'/tmp/or-hill-gates.py','parent-06.json'],check=True)
subprocess.run([sys.executable,'compare.py','pilot','--parent','parent-06.json'],cwd=out,check=True)
