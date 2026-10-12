import hashlib,json,re,shutil,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
trial=out.parents[1]/'tantivy-or-region-widen-oct07'
path='src/query/boolean_query/or_maxscore.rs'
sha=lambda b:hashlib.sha256(b).hexdigest()
before='6c71f32394bac8d1c5998c511585904e09ac68757063b2f2bc96afb375258d1d'
after='a53374a75b5dbedb3e8f148b94f8da92a6d4383142e69cc8b50a92330c2f96d1'
old=(repo/path).read_bytes();new=(trial/path).read_bytes()
assert sha(old)==before and sha(new)==after
assert old==(out/'attempt-12-pre-witness-fix/source.rs').read_bytes()
assert new==(out/'attempt-12-source-witness-fixed.rs').read_bytes()
assert subprocess.check_output(['git','diff','--name-only'],cwd=repo,text=True).splitlines()==[path]
receipt=json.loads((out/'attempt-12-witness-fix-receipt.json').read_text())
oldtext=old.decode();reconstructed=new.decode()
for name in receipt['only_changed_functions']:
 def span(text):
  start=text.index('    #[test]\n    fn '+name)
  finish=text.index('    #[test]',start+len('    #[test]'))
  return start,finish
 a,b=span(oldtext);c,d=span(reconstructed)
 reconstructed=reconstructed[:c]+oldtext[a:b]+reconstructed[d:]
assert reconstructed.encode()==old and sha(reconstructed.encode())==before
shutil.copy2(out/'attempt-12-integration.json',out/'attempt-12-pre-witness-fix/integration.json')
(repo/path).write_bytes(new)
assert sha((repo/path).read_bytes())==after
subprocess.run(['git','diff','--check'],cwd=repo,check=True)
(out/'attempt-12-integration.json').write_text(json.dumps({'parent':'43f06833a325d2a18c03bd7e4fa65f2f27944292','source_sha256':after,'adaptation':'Reviewed combined12 graft followed by ONLY two justified newtest witnessgeometry corrections. Independent rootreverse functionreplacement reconstructs entire failedsourceexactly; production/otherfixtures unchanged. Initial21pass2fail retained.'},indent=2)+'\n')
subprocess.run([sys.executable,'root-run-stage.py','tests-targeted'],cwd=out,check=True)
