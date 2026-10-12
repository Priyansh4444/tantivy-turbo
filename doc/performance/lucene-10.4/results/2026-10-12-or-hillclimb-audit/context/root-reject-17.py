import hashlib,json,shutil,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
sys.path.insert(0,str(out))
from stage import sha,sources
parent=json.loads((out/'parent-11.json').read_text())
paths={'src/query/boolean_query/or_maxscore.rs':'25e73721d3ac6e4d830594ba399195d5a64082031726d0883fb67042c1e4acbe','src/query/term_query/term_scorer.rs':'21b05a2b6513223a81613ee0f40c4a31f6990a427bb0d4b60324154a55753cdb'}
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()==parent['source_commit']
assert subprocess.check_output(['git','diff','--name-only'],cwd=repo,text=True).splitlines()==list(paths)
for path,digest in paths.items():assert sha(repo/path)==digest
assert json.loads((out/'build-final/after.json').read_text())==sources()
for stage in ('tests-targeted','tests-targeted-term','tests-targeted-postings','build-final'):
 r=json.loads((out/stage/'receipt.json').read_text());assert r['returncode']==0 and r['source_unchanged']
assert '26 passed; 0 failed' in (out/'tests-targeted/stdout.log').read_text()
for gate in ('verify','configured-k0.9-b0.4','configured-k2-b1'):
 assert json.loads((out/gate/'receipt.json').read_text())['pass']
pilot=json.loads((out/'pilot-final/receipt.json').read_text());assert pilot['unchanged']
group=next(row for row in pilot['summary'] if row['set']=='or3plus')
ratios={o:group[o]['ratio_mean'] for o in ('AB','BA')}
assert all(ratios[o]<1 for o in ('AB','BA'))
severe={r['id']:r for r in pilot['worst_regressions'] if r['id'] in (817,909)}
assert len(severe)==2 and all(severe[i][o]>1.7 for i in severe for o in ('AB','BA'))
folder=out/'attempt-17';folder.mkdir()
for name in ('tests-targeted','tests-targeted-term','tests-targeted-postings','build-final','verify','configured-k0.9-b0.4','configured-k2-b1','pilot-final'):
 shutil.move(out/name,folder/name)
for f in sorted(out.glob('attempt-17*')):
 if f.is_file():shutil.copy2(f,folder/f.name)
for name in ('tantivy_worker','configured_worker'):shutil.copy2(out/'target/release'/name,folder/name)
controllers=folder/'controllers';controllers.mkdir()
for name in ('balanced.py','compare.py','stage.py','configured_gate.py','pilot.py','analyze.py','measure_audit.py','root-run-stage.py','root-begin-17.py','root-primary-17.py','root-reject-17.py'):
 shutil.copy2(out/name,controllers/name)
(folder/'rejected.patch').write_bytes(subprocess.check_output(['git','diff','--binary'],cwd=repo))
r={'verdict':'reverted_pilot_regression_risk','parent':parent['source_commit'],'source_sha256':paths,
 'reason':'Frozen pilot long-OR average improves in both orders, but817 and909 regress1.8–2.8x in both orders. Reject this expansion on the retained negative screen before spending a full sampler. Pilot has no self-controls and does not prove an intrinsic per-query loss. No selective repeat or workload change.',
 'or3_pilot_ratios':ratios,'pilot':pilot,'targeted_tests':62,'targeted_ignored':1,'worst_pilot_regressions':pilot['worst_regressions'],'exact_profiles':3,'queries_per_profile':1227,
 'candidate_binary_sha256':sha(folder/'tantivy_worker'),'candidate_file_delta_bytes':(folder/'tantivy_worker').stat().st_size-Path(parent['binary']).stat().st_size,
 'limits':'Pilot has no self controls; no intrinsic loss or full-suite speed claim. Broad tests, Clippy, full sampler and actual-port lane not run for this rejected candidate. 256 logical cold-scratch bytes are not a measured RSS result. Index and replay unchanged.'}
(folder/'rejection.json').write_text(json.dumps(r,indent=2)+'\n')
before='/'.join(f"{group[o]['baseline_mean_ns']/1000:.2f}" for o in ('AB','BA'))
after='/'.join(f"{group[o]['candidate_mean_ns']/1000:.2f}" for o in ('AB','BA'))
with (out/'decision.tsv').open('a') as f:
 f.write(f"17\tprevent optional promotion during expansion\tadmit wider metadata region only with exact original partition\t{before} us pilot\t{after} us pilot\t-9.61%/-9.51% raw aggregate; sparse risks\t62 targeted;3x1227 exact gates;full sampler not run\treverted\t8172.79/2.75x;9091.80/1.90x rawpilot;256B scratch;no selfcontrols/intrinsicloss/port claim\n")
subprocess.run(['git','restore','--',*paths],cwd=repo,check=True)
assert not subprocess.check_output(['git','diff','--name-only'],cwd=repo)
for path in paths:
 assert sha(repo/path)==hashlib.sha256(subprocess.check_output(['git','show',parent['source_commit']+':'+path],cwd=repo)).hexdigest()
print(json.dumps({'verdict':r['verdict'],'root_restored_to':parent['source_commit'],'or3_pilot':ratios,'binary_delta':r['candidate_file_delta_bytes']}),flush=True)
