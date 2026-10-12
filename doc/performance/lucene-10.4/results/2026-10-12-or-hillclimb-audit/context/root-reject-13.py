import hashlib,json,re,shutil,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
parent=json.loads((out/'parent-11.json').read_text())
sys.path.insert(0,str(out))
from stage import sha,sources
expected=json.loads((out/'attempt-13-integration.json').read_text())['source_sha256']
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()==parent['source_commit']
assert subprocess.check_output(['git','diff','--name-only'],cwd=repo,text=True).splitlines()==list(expected)
for path,digest in expected.items():assert sha(repo/path)==digest
assert json.loads((out/'build-final/after.json').read_text())==sources()
stages=('build-final','tests-targeted','tests-targeted-term','tests-targeted-postings','formatting','formatting-term','tests','clippy')
for name in stages:
 r=json.loads((out/name/'receipt.json').read_text());assert r['returncode']==0 and r['source_unchanged']
for name in ('verify','configured-k0.9-b0.4','configured-k2-b1','measure'):
 assert json.loads((out/name/'receipt.json').read_text())['pass']
metrics=json.loads((out/'attempt-13-metrics.json').read_text());g=metrics['groups']['OR_3PLUS']
assert g['candidate_win_both_beyond_group_controls']
losses=metrics['regressions_beyond_per_query_controls']
severe={row['id']:row for row in losses if row['id'] in ('817','909')}
assert len(severe)==2 and all(float(row['T_over_P_'+order])>9 for row in severe.values() for order in ('AB','BA'))
summary=json.loads((out/'summary.json').read_text())
assert summary['complete_cells']==87912 and summary['raw_samples']==1406592
counts=[tuple(map(int,m)) for m in re.findall(r'test result: ok\. (\d+) passed; 0 failed; (\d+) ignored;', (out/'tests/stdout.log').read_text())]
assert sum(p for p,_ in counts)==1457 and sum(i for _,i in counts)==7
folder=out/'attempt-13';folder.mkdir()
for name in (*stages,'verify','configured-k0.9-b0.4','configured-k2-b1','pilot-final','measure','summary.json','per-query.csv'):
 shutil.move(out/name,folder/name)
for f in sorted(out.glob('attempt-13*')):
 if f.is_file():shutil.copy2(f,folder/f.name)
for name in ('tantivy_worker','configured_worker'):shutil.copy2(out/'target/release'/name,folder/name)
controllers=folder/'controllers';controllers.mkdir()
for name in ('balanced.py','compare.py','stage.py','configured_gate.py','pilot.py','analyze.py','measure_audit.py','root-run-stage.py','root-begin-13-parent11.py','root-primary-13.py','root-full-13.py','root-reject-13.py'):
 shutil.copy2(out/name,controllers/name)
(folder/'rejected.patch').write_bytes(subprocess.check_output(['git','diff','--binary'],cwd=repo))
r={'verdict':'reverted_material_per_query_regressions','parent':parent['source_commit'],'source_sha256':expected,
 'reason':'OR aggregates clear controls, but retained sparse OR cases regress 9–15x in both orders. Reject on the whole-suite negative/regression gate; no query exclusions, metric change or selective rerun.',
 'raw_or3_ratios':g['primary'],'control_log_spread':g['control_log_spread'],'all_groups':metrics['groups'],
 'targeted_tests':59,'targeted_ignored':1,'tests_passed':1457,'tests_ignored':7,'exact_profiles':3,'queries_per_profile':1227,
 'candidate_binary_sha256':sha(folder/'tantivy_worker'),'efficiency':metrics['efficiency'],'per_query_losses':losses,
 'limits':'No intrinsic cycle attribution. Actual-port lane not run for rejected candidate. Index bytes unchanged; no measured RSS advantage. Aggregate improvements do not establish universal speed or API parity.'}
(folder/'rejection.json').write_text(json.dumps(r,indent=2)+'\n')
group=next(row for row in summary['summaries'] if row['set']=='port-style' and row['mode']=='top10' and row['kind']=='OR_3PLUS')
before='/'.join(f"{group[o]['mean_query_median_P_ns']/1000:.2f}" for o in ('AB','BA'))
after='/'.join(f"{group[o]['mean_query_median_T_ns']/1000:.2f}" for o in ('AB','BA'))
with (out/'decision.tsv').open('a') as f:
 f.write(f"13\ttighter metadata certificates for wider regions\tindependent SkipReader range max with conservative provenance\t{before} us full\t{after} us full\t-8.05%/-8.94% aggregate; severe losses\t1457 tests;3x1227 exact gates;full AB/BA controls\treverted\t60 fullsuite controlledlosses;817 14.82/14.51x;909 9.24/9.14x;binary+4544B;no actualport lane\n")
subprocess.run(['git','restore','--',*expected],cwd=repo,check=True)
assert not subprocess.check_output(['git','diff','--name-only'],cwd=repo)
for path in expected:
 assert sha(repo/path)==hashlib.sha256(subprocess.check_output(['git','show',parent['source_commit']+':'+path],cwd=repo)).hexdigest()
print(json.dumps({'verdict':r['verdict'],'root_restored_to':parent['source_commit'],'raw_or3':g['primary'],'retained_losses':len(losses)}),flush=True)
