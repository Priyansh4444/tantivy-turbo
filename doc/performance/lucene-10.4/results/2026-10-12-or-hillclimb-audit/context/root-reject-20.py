import hashlib,json,shutil,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
sys.path.insert(0,str(out))
from stage import sha,sources
parent=json.loads((out/'parent-11.json').read_text())
paths={'src/query/boolean_query/or_maxscore.rs':'8cecd8b7f48b70018c18bbced1800e7f057a3f99d6f576107afc2b156f32e8c3'}
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()==parent['source_commit']
assert subprocess.check_output(['git','diff','--name-only'],cwd=repo,text=True).splitlines()==list(paths)
for path,digest in paths.items():assert sha(repo/path)==digest
assert json.loads((out/'build-final/after.json').read_text())==sources()
for stage in ('tests-targeted','build-final'):
 r=json.loads((out/stage/'receipt.json').read_text());assert r['returncode']==0 and r['source_unchanged']
assert '18 passed; 0 failed' in (out/'tests-targeted/stdout.log').read_text()
for gate in ('verify','configured-k0.9-b0.4','configured-k2-b1'):
 assert json.loads((out/gate/'receipt.json').read_text())['pass']
pilot=json.loads((out/'pilot-final/receipt.json').read_text());assert pilot['unchanged']
group=next(row for row in pilot['summary'] if row['set']=='or3plus')
ratios={o:group[o]['ratio_mean'] for o in ('AB','BA')}
assert all(ratios[o]>=1 for o in ('AB','BA'))
folder=out/'attempt-20';folder.mkdir()
for name in ('tests-targeted','build-final','verify','configured-k0.9-b0.4','configured-k2-b1','pilot-final'):
 shutil.move(out/name,folder/name)
for f in sorted(out.glob('attempt-20*')):
 if f.is_file():shutil.copy2(f,folder/f.name)
for name in ('tantivy_worker','configured_worker'):shutil.copy2(out/'target/release'/name,folder/name)
controllers=folder/'controllers';controllers.mkdir()
for name in ('balanced.py','compare.py','stage.py','configured_gate.py','pilot.py','analyze.py','measure_audit.py','root-run-stage.py','root-begin-20.py','root-primary-20.py','root-reject-20.py'):
 shutil.copy2(out/name,controllers/name)
(folder/'rejected.patch').write_bytes(subprocess.check_output(['git','diff','--binary'],cwd=repo))
r={'verdict':'reverted_no_reliable_win','parent':parent['source_commit'],'source_sha256':paths,
 'reason':'Frozen pilot long-OR arithmetic mean of query medians is slower in both orders. Reject at unchanged untrimmed pilot screen; no selective repeat or workload change. Pilot has no self controls and does not prove an intrinsic loss.',
 'or3_pilot_ratios':ratios,'pilot':pilot,'targeted_tests':18,'targeted_ignored':0,'worst_pilot_regressions':pilot['worst_regressions'],'exact_profiles':3,'queries_per_profile':1227,
 'candidate_binary_sha256':sha(folder/'tantivy_worker'),'candidate_file_delta_bytes':(folder/'tantivy_worker').stat().st_size-Path(parent['binary']).stat().st_size,
 'limits':'Pilot has no self controls; no intrinsic loss or full-suite speed claim. Broad tests, Clippy, full sampler and actual-port lane not run for this rejected candidate. One Option<f32> logical region cache is not a measured RSS result. No release allocation/index format change. Elevated external CPU load was observed during pilot; no intrinsic slowdown attribution.'}
(folder/'rejection.json').write_text(json.dumps(r,indent=2)+'\n')
before='/'.join(f"{group[o]['baseline_mean_ns']/1000:.2f}" for o in ('AB','BA'))
after='/'.join(f"{group[o]['candidate_mean_ns']/1000:.2f}" for o in ('AB','BA'))
with (out/'decision.tsv').open('a') as f:
 f.write(f"20\tskip certified first-optional misses inside owned batches\tregion-local original-n excluded bound and structural future-entry guard\t{before} us pilot\t{after} us pilot\t+4.82%/+5.34% raw; pilot screen rejected\t18 targeted;3x1227 exact gates;full sampler not run\treverted\tOption<f32> region cache;external CPU load rose during pilot;no selfcontrols/intrinsicloss/port claim\n")
subprocess.run(['git','restore','--',*paths],cwd=repo,check=True)
assert not subprocess.check_output(['git','diff','--name-only'],cwd=repo)
for path in paths:
 assert sha(repo/path)==hashlib.sha256(subprocess.check_output(['git','show',parent['source_commit']+':'+path],cwd=repo)).hexdigest()
print(json.dumps({'verdict':r['verdict'],'root_restored_to':parent['source_commit'],'or3_pilot':ratios,'binary_delta':r['candidate_file_delta_bytes']}),flush=True)
