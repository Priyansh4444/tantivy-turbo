import hashlib,json,re,shutil,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
parent=json.loads((out/'parent-11.json').read_text())
sys.path.insert(0,str(out))
from stage import sha,sources
path='src/query/boolean_query/or_maxscore.rs'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()==parent['source_commit']
assert subprocess.check_output(['git','diff','--name-only'],cwd=repo,text=True).splitlines()==[path]
assert sha(repo/path)=='6db826184ecfc6e745940b8049da6fd3755ad622085429ffca8fbb794bcb5010'
assert json.loads((out/'build-final/after.json').read_text())==sources()
for name in ('build-final','tests-targeted','formatting','tests','clippy'):
 r=json.loads((out/name/'receipt.json').read_text());assert r['returncode']==0 and r['source_unchanged']
for name in ('verify','configured-k0.9-b0.4','configured-k2-b1'):
 assert json.loads((out/name/'receipt.json').read_text())['pass']
assert json.loads((out/'measure/receipt.json').read_text())['pass']
metrics=json.loads((out/'attempt-14-metrics.json').read_text())
g=metrics['groups']['OR_3PLUS']
assert not g['candidate_win_both_beyond_group_controls']
summary=json.loads((out/'summary.json').read_text())
assert summary['complete_cells']==87912 and summary['raw_samples']==1406592
counts=[tuple(map(int,m)) for m in re.findall(r'test result: ok\. (\d+) passed; 0 failed; (\d+) ignored;', (out/'tests/stdout.log').read_text())]
assert sum(p for p,_ in counts)==1445 and sum(i for _,i in counts)==7
folder=out/'attempt-14';folder.mkdir()
for name in ('build-final','tests-targeted','formatting','tests','clippy','verify','configured-k0.9-b0.4','configured-k2-b1','pilot-final','measure'):
 shutil.move(out/name,folder/name)
for name in ('summary.json','per-query.csv'):
 shutil.move(out/name,folder/name)
for f in sorted(out.glob('attempt-14*')):
 if f.is_file():shutil.copy2(f,folder/f.name)
for name in ('tantivy_worker','configured_worker'):shutil.copy2(out/'target/release'/name,folder/name)
controllers=folder/'controllers';controllers.mkdir()
for name in ('balanced.py','compare.py','stage.py','configured_gate.py','pilot.py','analyze.py','measure_audit.py','root-run-stage.py','root-begin-14.py','root-primary-14.py','root-full-14.py','root-reject-14.py'):
 shutil.copy2(out/name,controllers/name)
(folder/'rejected.patch').write_bytes(subprocess.check_output(['git','diff','--binary'],cwd=repo))
r={'verdict':'reverted_no_reliable_win','parent':parent['source_commit'],'source_sha256':sha(repo/path),
 'reason':'Full frozen OR3+ aggregate does not clear matching arithmetic self controls in both orders. No relaxed target, exclusions or selective rerun.',
 'raw_or3_ratios':g['primary'],'control_log_spread':g['control_log_spread'],'all_groups':metrics['groups'],
 'targeted_tests':16,'tests_passed':1445,'tests_ignored':7,'exact_profiles':3,'queries_per_profile':1227,
 'candidate_binary_sha256':sha(folder/'tantivy_worker'),'candidate_file_delta_bytes':(folder/'tantivy_worker').stat().st_size-Path(parent['binary']).stat().st_size,
 'per_query_losses':metrics['regressions_beyond_per_query_controls'],
 'limits':'Both raw long-OR ratios improve but AB is inside controls under current background load. No intrinsic loss or port comparison claim. Actual-port lane not run for rejected candidate. Index format and contribution allocation unchanged; 4B logical mask, no measured RSS advantage.'}
(folder/'rejection.json').write_text(json.dumps(r,indent=2)+'\n')
group=next(row for row in summary['summaries'] if row['set']=='port-style' and row['mode']=='top10' and row['kind']=='OR_3PLUS')
before='/'.join(f"{group[o]['mean_query_median_P_ns']/1000:.2f}" for o in ('AB','BA'))
after='/'.join(f"{group[o]['mean_query_median_T_ns']/1000:.2f}" for o in ('AB','BA'))
with (out/'decision.tsv').open('a') as f:
 f.write(f"14\tavoid absent score slot clearing and zero additions\tcandidate-local matchedordinal mask and orderedpublication\t{before} us full\t{after} us full\t-0.79%/-4.11% raw; failscontrols\t1445 tests;3x1227 exact gates;full AB/BA controls\treverted\t2.45% groupcontrolspread;13 fullsuite losses retained;binary+160B;no actualport lane/no intrinsiclossclaim\n")
subprocess.run(['git','restore','--',path],cwd=repo,check=True)
assert not subprocess.check_output(['git','diff','--name-only'],cwd=repo)
assert sha(repo/path)==hashlib.sha256(subprocess.check_output(['git','show',parent['source_commit']+':'+path],cwd=repo)).hexdigest()
print(json.dumps({'verdict':r['verdict'],'root_restored_to':parent['source_commit'],'full_test_count':1445,'raw_or3':g['primary'],'controls':g['controls']}),flush=True)
