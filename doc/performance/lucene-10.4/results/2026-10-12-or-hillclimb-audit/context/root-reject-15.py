import hashlib,json,shutil,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
sys.path.insert(0,str(out))
from stage import sha,sources
parent=json.loads((out/'parent-11.json').read_text())
path='src/query/boolean_query/or_maxscore.rs'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()==parent['source_commit']
assert subprocess.check_output(['git','diff','--name-only'],cwd=repo,text=True).splitlines()==[path]
assert sha(repo/path)=='602867a5085c86c6923e15234a6ce381519b2834af6c627ef910db393e6d3477'
assert json.loads((out/'build-final/after.json').read_text())==sources()
for stage in ('tests-targeted','build-final'):
 r=json.loads((out/stage/'receipt.json').read_text());assert r['returncode']==0 and r['source_unchanged']
assert '17 passed; 0 failed' in (out/'tests-targeted/stdout.log').read_text()
for gate in ('verify','configured-k0.9-b0.4','configured-k2-b1'):
 assert json.loads((out/gate/'receipt.json').read_text())['pass']
pilot=json.loads((out/'pilot-final/receipt.json').read_text());assert pilot['unchanged']
group=next(row for row in pilot['summary'] if row['set']=='or3plus')
ratios={o:group[o]['ratio_mean'] for o in ('AB','BA')}
assert ratios['AB']>1 and ratios['BA']<1
folder=out/'attempt-15';folder.mkdir()
for name in ('tests-targeted','build-final','verify','configured-k0.9-b0.4','configured-k2-b1','pilot-final'):
 shutil.move(out/name,folder/name)
for f in sorted(out.glob('attempt-15*')):
 if f.is_file():shutil.copy2(f,folder/f.name)
for name in ('tantivy_worker','configured_worker'):shutil.copy2(out/'target/release'/name,folder/name)
controllers=folder/'controllers';controllers.mkdir()
for name in ('balanced.py','compare.py','stage.py','configured_gate.py','pilot.py','analyze.py','measure_audit.py','root-run-stage.py','root-begin-15.py','root-primary-15.py','root-reject-15.py'):
 shutil.copy2(out/name,controllers/name)
(folder/'rejected.patch').write_bytes(subprocess.check_output(['git','diff','--binary'],cwd=repo))
r={'verdict':'reverted_no_reliable_win','parent':parent['source_commit'],'source_sha256':sha(repo/path),
 'reason':'Frozen unchanged pilot has mixed OR3+ order deltas, with median per-query ratios above one in both orders. Added state has no demonstrated benefit. No selective repeat or workload change.',
 'or3_pilot_ratios':ratios,'pilot':pilot,'targeted_tests':17,'exact_profiles':3,'queries_per_profile':1227,
 'candidate_binary_sha256':sha(folder/'tantivy_worker'),'candidate_file_delta_bytes':(folder/'tantivy_worker').stat().st_size-Path(parent['binary']).stat().st_size,
 'limits':'Pilot has no self controls; no intrinsic loss or full-suite speed claim. Broad tests, Clippy, full sampler and actual-port lane not run for this rejected candidate. 13 logical bytes are not a measured RSS result. Index and replay unchanged.'}
(folder/'rejection.json').write_text(json.dumps(r,indent=2)+'\n')
before='/'.join(f"{group[o]['baseline_mean_ns']/1000:.2f}" for o in ('AB','BA'))
after='/'.join(f"{group[o]['candidate_mean_ns']/1000:.2f}" for o in ('AB','BA'))
with (out/'decision.tsv').open('a') as f:
 f.write(f"15\tavoid repeated immutable density scans\texact prefix minimum and live suffix winner cache\t{before} us pilot\t{after} us pilot\t+0.48%/-0.79% mixed\t17 targeted;3x1227 exact gates;full sampler not run\treverted\tno demonstrated gain;median per-query slower bothorders;13B logical state;no intrinsicloss/port claim\n")
subprocess.run(['git','restore','--',path],cwd=repo,check=True)
assert not subprocess.check_output(['git','diff','--name-only'],cwd=repo)
assert sha(repo/path)==hashlib.sha256(subprocess.check_output(['git','show',parent['source_commit']+':'+path],cwd=repo)).hexdigest()
print(json.dumps({'verdict':r['verdict'],'root_restored_to':parent['source_commit'],'or3_pilot':ratios,'binary_delta':r['candidate_file_delta_bytes']}),flush=True)
