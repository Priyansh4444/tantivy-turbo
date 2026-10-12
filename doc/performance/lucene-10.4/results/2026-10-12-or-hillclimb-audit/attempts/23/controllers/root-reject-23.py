import json,re,shutil,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
sys.path.insert(0,str(out))
from stage import sha,sources
parent=json.loads((out/'parent-22.json').read_text())
static=json.loads((out/'attempt-23-static-receipt.json').read_text())
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()==parent['source_commit']
assert subprocess.check_output(['git','diff','--name-only'],cwd=repo,text=True).splitlines()==static['edited_tracked_files']
for path,digest in static['source_sha256'].items():assert sha(repo/path)==digest
assert json.loads((out/'build-final/after.json').read_text())==sources()
for name in ('tests-targeted','tests-targeted-score','build-final'):
 r=json.loads((out/name/'receipt.json').read_text());assert r['returncode']==0 and r['source_unchanged']
for folder,count in (('tests-targeted',13),('tests-targeted-score',8)):
 assert f'{count} passed; 0 failed; 0 ignored;' in (out/folder/'stdout.log').read_text()
for name in ('verify','configured-k0.9-b0.4','configured-k2-b1'):
 r=json.loads((out/name/'receipt.json').read_text());assert r['pass'] and r['queries']==1227 and r['failures']==0 and r['unchanged']
pilot=json.loads((out/'pilot-final/receipt.json').read_text());assert pilot['unchanged']
g=next(row for row in pilot['summary'] if row['set']=='or3plus')
assert not all(g[o]['ratio_mean']<1 for o in ('AB','BA'))
for name in ('tests','clippy','formatting','measure','port-measure','attempt-23'):
 assert not (out/name).exists(),name
folder=out/'attempt-23';folder.mkdir()
for name in ('tests-targeted','tests-targeted-score','build-final','verify','configured-k0.9-b0.4','configured-k2-b1','pilot-final'):
 shutil.move(out/name,folder/name)
for p in sorted(out.glob('attempt-23-*')):
 if p.is_file():shutil.copy2(p,folder/p.name)
for name in ('tantivy_worker','configured_worker'):shutil.copy2(out/'target/release'/name,folder/name)
controllers=folder/'controllers';controllers.mkdir()
for name in ('balanced.py','compare.py','stage.py','configured_gate.py','pilot.py','analyze.py','measure_audit.py','root-run-stage.py','root-begin-23.py','root-primary-23.py','root-full-23.py','root-reject-23.py'):
 shutil.copy2(out/name,controllers/name)
patch=subprocess.check_output(['git','diff','--binary'],cwd=repo)
assert patch==(out/'attempt-23-full.patch').read_bytes()
(folder/'rejected.patch').write_bytes(patch)
r={'verdict':'reverted_no_reliable_win','parent':parent['source_commit'],'source_sha256':static['source_sha256'],
 'reason':'Frozen pilot longer OR is mixed across orders and median query ratios exceed one in both. Conservative screen rejection; no full self-control or actual-port lane ran, so no intrinsic slowdown or controlled performance claim.',
 'pilot_or3_ratios':{o:g[o]['ratio_mean'] for o in ('AB','BA')},'targeted_tests':21,
 'exact_profiles':3,'queries_per_profile':1227,'candidate_binary_sha256':sha(folder/'tantivy_worker'),
 'candidate_file_delta_bytes':(folder/'tantivy_worker').stat().st_size-Path(parent['binary']).stat().st_size,
 'limits':'No added state/allocation/index change. User background load retained. No broad test, full timing, actual-port, RSS, compiled register/frame or intrinsic-loss claim.'}
(folder/'rejection.json').write_text(json.dumps(r,indent=2)+'\n')
before='/'.join(f"{g[o]['baseline_mean_ns']/1000:.2f}" for o in ('AB','BA'))
after='/'.join(f"{g[o]['candidate_mean_ns']/1000:.2f}" for o in ('AB','BA'))
delta='/'.join(f"{(g[o]['ratio_mean']-1)*100:+.2f}%" for o in ('AB','BA'))
with (out/'decision.tsv').open('a') as f:
 f.write(f"23\treuse exact cutoff for remaining bound comparisons\tordered greater global and at-most local\t{before} us pilot\t{after} us pilot\t{delta} mixed; pilot screen rejected\t21 targeted;3x1227 exact gates;full sampler not run\treverted\tno added state;binary{r['candidate_file_delta_bytes']:+d}B;no selfcontrols/intrinsicloss/actualport claim\n")
subprocess.run(['git','restore','--',*static['edited_tracked_files']],cwd=repo,check=True)
assert not subprocess.check_output(['git','diff','--name-only'],cwd=repo)
for path in static['edited_tracked_files']:
 assert (repo/path).read_bytes()==subprocess.check_output(['git','show',parent['source_commit']+':'+path],cwd=repo)
print(json.dumps({'verdict':r['verdict'],'root_restored_to':parent['source_commit'],'pilot_ratios':r['pilot_or3_ratios'],'file_delta_bytes':r['candidate_file_delta_bytes']}),flush=True)
