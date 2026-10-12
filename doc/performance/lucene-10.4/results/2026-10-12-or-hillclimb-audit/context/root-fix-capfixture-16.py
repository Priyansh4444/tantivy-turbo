"""Preserve failed geometry evidence, then require actual owned winning rows."""
import hashlib,json,shutil,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
path='src/query/boolean_query/or_maxscore.rs'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(repo/path)=='bfa821570827f2ca95590ee08ead7bbc2b854067f9a32db215a702081c6733d0'
receipt=json.loads((out/'tests-targeted/receipt.json').read_text())
assert receipt['returncode']==101 and receipt['source_unchanged']
assert '28 passed; 1 failed' in (out/'tests-targeted/stdout.log').read_text()
failed=out/'attempt-16/pre-capfixture-fix'
failed.mkdir(parents=True,exist_ok=False)
shutil.move(out/'tests-targeted',failed/'tests-targeted')
for f in sorted(out.glob('attempt-16-*')):
    if f.is_file():shutil.copy2(f,failed/f.name)
old='''CursorProgress::Batch { docs, doc: 382, up_to: 382, .. } if !docs.is_empty()
                    )),
                    "exact exclusive end must remain first unread: length={length}"'''
new='''CursorProgress::Batch { docs, doc, .. } if docs.contains(&382) && *doc > 382
                    )),
                    "winning row382 must be batch-owned with a later authoritative unread head: length={length}; {:?}",
                    evidence.progress'''
source=(repo/path).read_text()
assert source.count(old)==1
(repo/path).write_text(source.replace(old,new))
subprocess.run(['rustfmt','--edition','2021',str(repo/path)],check=True)
subprocess.run(['git','diff','--check'],cwd=repo,check=True)
patch=subprocess.check_output(['git','diff','--binary'],cwd=repo)
(out/'attempt-16-full.patch').write_bytes(patch)
(out/'attempt-16-source-or.rs').write_bytes((repo/path).read_bytes())
(out/'attempt-16-capfixture-fix.json').write_text(json.dumps({'failed_revision':'pre-capfixture-fix','failed_targeted':{'passed':28,'failed':1},'old_source_sha256':'bfa821570827f2ca95590ee08ead7bbc2b854067f9a32db215a702081c6733d0','final_source_sha256':sha(repo/path),'patch_sha256':hashlib.sha256(patch).hexdigest(),'production_changed':False,'reason':'Old stride3 corpus retains same TF30/TF20 and all literal callback/cursor oracles. Weak clauses become conservative tails after128, so capped geometry does not promise an exclusive382 interval. Require actual owned winner382 with real head strictly after it; the additive92-row fixture independently requires exclusive192 to remain unread and next refill owns192.'},indent=2)+'\n')
subprocess.run([sys.executable,'root-run-stage.py','tests-targeted'],cwd=out,check=True)
for phase in ('tests-targeted-term','tests-targeted-postings'):
    command=json.loads((out/'attempt-13'/phase/'command.json').read_text())
    subprocess.run([sys.executable,'stage.py',phase,'1200',*command],cwd=out,check=True)
