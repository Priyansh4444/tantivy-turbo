import hashlib,json,shutil,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
parent=json.loads((out/'parent-11.json').read_text())
sys.path.insert(0,str(out))
from stage import sha,sources
path='src/query/boolean_query/or_maxscore.rs'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()==parent['source_commit']
assert sha(repo/path)=='a53374a75b5dbedb3e8f148b94f8da92a6d4383142e69cc8b50a92330c2f96d1'
assert subprocess.check_output(['git','diff','--name-only'],cwd=repo,text=True).splitlines()==[path]
assert json.loads((out/'build-final/after.json').read_text())==sources()
for name in ('build-final','tests-targeted'):
 r=json.loads((out/name/'receipt.json').read_text());assert r['returncode']==0 and r['source_unchanged']
for name in ('verify','configured-k0.9-b0.4','configured-k2-b1'):
 r=json.loads((out/name/'receipt.json').read_text());assert r['pass']
pilot=json.loads((out/'pilot-final/receipt.json').read_text());assert pilot['unchanged']
g=next(row for row in pilot['summary'] if row['set']=='or3plus')
assert all(g[o]['ratio_mean']>1 for o in ('AB','BA'))
folder=out/'attempt-12';folder.mkdir()
for name in ('build-final','tests-targeted','verify','configured-k0.9-b0.4','configured-k2-b1','pilot-final'):
 shutil.move(out/name,folder/name)
shutil.move(out/'attempt-12-pre-witness-fix',folder/'pre-witness-fix')
for f in sorted(out.glob('attempt-12*')):
 if f.is_file():shutil.copy2(f,folder/f.name)
for name in ('tantivy_worker','configured_worker'):shutil.copy2(out/'target/release'/name,folder/name)
controllers=folder/'controllers';controllers.mkdir()
for name in ('balanced.py','compare.py','stage.py','configured_gate.py','pilot.py','root-run-stage.py','root-begin-12.py','root-witness-fix-12.py','root-primary-12.py','root-reject-12.py'):
 shutil.copy2(out/name,controllers/name)
(folder/'rejected.patch').write_bytes(subprocess.check_output(['git','diff','--binary'],cwd=repo))
r={'verdict':'reverted','parent':parent['source_commit'],'source_sha256':sha(repo/path),
 'reason':'Unchanged frozen longer-OR pilot loses in bothorders: +11.29%/+2.50%; medianperqueryratios +12.08%/+10.52%; severe sparse regressions retained, no queryexclusions.',
 'pilot':pilot,'targeted_tests':23,'exact_profiles':3,'queries_per_profile':1227,
 'candidate_binary_sha256':sha(folder/'tantivy_worker'),
 'candidate_file_delta_bytes':(folder/'tantivy_worker').stat().st_size-Path(parent['binary']).stat().st_size,
 'scratch':'128B extra fixed endpoint array; actual ClauseState24B; no new per-regionallocation/indexformat. No measuredRSSadvantage.',
 'fixture_corrections':'Original06→accepted11 adaptation preserves cache/replay and density states; retained21pass2fail run correctedONLY2newtest corpuspath assumptions, final23pass.',
 'limits':'Full sampler/broadtests/clippy/actualport not run for loser. External user/backgroundload retained. No causalcycleattribution or universal APIparity claim.'}
(folder/'rejection.json').write_text(json.dumps(r,indent=2)+'\n')
before='/'.join(f"{g[o]['baseline_mean_ns']/1000:.2f}" for o in ('AB','BA'))
after='/'.join(f"{g[o]['candidate_mean_ns']/1000:.2f}" for o in ('AB','BA'))
with (out/'decision.tsv').open('a') as f:
 f.write(f"12\twider local certificates reduce optional-end setup\tone-shot oldessential endpoint plus crossedglobal bounds\t{before} us pilot\t{after} us pilot\t+11.29%/+2.50%\t23 targeted;3x1227 exact gates;full sampler not run\treverted\t128B fixedcold ends; sparse losses up to14.68x; failed21pass2fail witnessrunretained; no fulltests or portclaim\n")
subprocess.run(['git','restore','--',path],cwd=repo,check=True)
assert not subprocess.check_output(['git','diff','--name-only'],cwd=repo)
assert sha(repo/path)==hashlib.sha256(subprocess.check_output(['git','show',parent['source_commit']+':'+path],cwd=repo)).hexdigest()
print(json.dumps({'verdict':r['verdict'],'delta_bytes':r['candidate_file_delta_bytes'],'root_restored_to':parent['source_commit']}),flush=True)
trial=out.parents[1]/'tantivy-or-metadata-range-oct08'
assert not trial.exists()
subprocess.run(['git','worktree','add','-b','perf/or-metadata-range-oct08',str(trial),parent['source_commit']],cwd=repo,check=True)
(trial/path).write_bytes((folder/'attempt-12-source-witness-fixed.rs').read_bytes())
assert sha(trial/path)==r['source_sha256']
print('Prepared isolated13 with reviewed12 geometry; unaccepted source, accepted11 base.',flush=True)
