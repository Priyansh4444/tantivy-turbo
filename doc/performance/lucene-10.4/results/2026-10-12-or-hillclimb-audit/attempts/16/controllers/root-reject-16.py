import hashlib,json,re,shutil,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
parent=json.loads((out/'parent-11.json').read_text())
sys.path.insert(0,str(out))
from stage import sha,sources
expected={'src/query/boolean_query/or_maxscore.rs':'4ee0b044610d00a8f6fc508997bba250e23473d08b5f2c5c453372d6d9ff63e8','src/query/term_query/term_scorer.rs':'21b05a2b6513223a81613ee0f40c4a31f6990a427bb0d4b60324154a55753cdb'}
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()==parent['source_commit']
assert subprocess.check_output(['git','diff','--name-only'],cwd=repo,text=True).splitlines()==list(expected)
for path,digest in expected.items():assert sha(repo/path)==digest
assert json.loads((out/'build-final/after.json').read_text())==sources()
stages=('build-final','tests-targeted','tests-targeted-term','tests-targeted-postings','formatting','formatting-term','tests','clippy')
for name in stages:
 r=json.loads((out/name/'receipt.json').read_text());assert r['returncode']==0 and r['source_unchanged']
for name in ('verify','configured-k0.9-b0.4','configured-k2-b1','measure'):
 assert json.loads((out/name/'receipt.json').read_text())['pass']
metrics=json.loads((out/'attempt-16-metrics.json').read_text());g=metrics['groups']['OR_3PLUS']
assert g['candidate_win_both_beyond_group_controls']
losses=metrics['regressions_beyond_per_query_controls']
severe={row['id']:row for row in losses if row['id'] in ('917','919')}
assert len(severe)==2 and all(float(row['T_over_P_'+order])>1.09 for row in severe.values() for order in ('AB','BA'))
summary=json.loads((out/'summary.json').read_text())
assert summary['complete_cells']==87912 and summary['raw_samples']==1406592
counts=[tuple(map(int,m)) for m in re.findall(r'test result: ok\. (\d+) passed; 0 failed; (\d+) ignored;', (out/'tests/stdout.log').read_text())]
assert sum(p for p,_ in counts)==1463 and sum(i for _,i in counts)==7
folder=out/'attempt-16';assert folder.is_dir() and sorted(x.name for x in folder.iterdir())==['pre-capfixture-fix']
for name in (*stages,'verify','configured-k0.9-b0.4','configured-k2-b1','pilot-final','measure','summary.json','per-query.csv'):
 shutil.move(out/name,folder/name)
for f in sorted(out.glob('attempt-16*')):
 if f.is_file():shutil.copy2(f,folder/f.name)
for name in ('tantivy_worker','configured_worker'):shutil.copy2(out/'target/release'/name,folder/name)
controllers=folder/'controllers';controllers.mkdir()
for name in ('balanced.py','compare.py','stage.py','configured_gate.py','pilot.py','analyze.py','measure_audit.py','root-run-stage.py','root-fix-capfixture-16.py','root-begin-16.py','root-primary-16.py','root-full-16.py','root-reject-16.py'):
 shutil.copy2(out/name,controllers/name)
(folder/'rejected.patch').write_bytes(subprocess.check_output(['git','diff','--binary'],cwd=repo))
r={'verdict':'reverted_material_per_query_regressions','parent':parent['source_commit'],'source_sha256':expected,
 'reason':'Long-OR aggregate narrowly clears controls, but only one long query has a controlled win and twelve have controlled losses. Two expensive dense queries regress 9–11% in both orders. Their losses are material relative to the 2.3–3.0% aggregate saving, which is concentrated in query920. Revert on retained whole-suite negative/regression evidence; the descriptive concentration analysis does not change the frozen target or exclude any query.',
 'raw_or3_ratios':g['primary'],'control_log_spread':g['control_log_spread'],'all_groups':metrics['groups'],
 'targeted_tests':65,'targeted_ignored':1,'tests_passed':1463,'tests_ignored':7,'exact_profiles':3,'queries_per_profile':1227,
 'candidate_binary_sha256':sha(folder/'tantivy_worker'),'efficiency':metrics['efficiency'],'per_query_losses':losses,
 'limits':'No intrinsic cycle attribution. Actual-port lane not run for rejected candidate. Index bytes unchanged; no measured RSS advantage. Aggregate improvements do not establish universal speed or API parity.'}
(folder/'rejection.json').write_text(json.dumps(r,indent=2)+'\n')
group=next(row for row in summary['summaries'] if row['set']=='port-style' and row['mode']=='top10' and row['kind']=='OR_3PLUS')
before='/'.join(f"{group[o]['mean_query_median_P_ns']/1000:.2f}" for o in ('AB','BA'))
after='/'.join(f"{group[o]['mean_query_median_T_ns']/1000:.2f}" for o in ('AB','BA'))
with (out/'decision.tsv').open('a') as f:
 f.write(f"16\tbound new physical region expansion\tmetadata widening capped to128 physical docs\t{before} us full\t{after} us full\t-2.26%/-2.99% aggregate; material dense losses\t1463 tests;3x1227 exact gates;full AB/BA controls\treverted\t38 fullsuite controlledlosses;12 longlosses vs1win;917/919 9–11% slower;binary+4480B;no actualport lane\n")
subprocess.run(['git','restore','--',*expected],cwd=repo,check=True)
assert not subprocess.check_output(['git','diff','--name-only'],cwd=repo)
for path in expected:
 assert sha(repo/path)==hashlib.sha256(subprocess.check_output(['git','show',parent['source_commit']+':'+path],cwd=repo)).hexdigest()
print(json.dumps({'verdict':r['verdict'],'root_restored_to':parent['source_commit'],'raw_or3':g['primary'],'retained_losses':len(losses)}),flush=True)
