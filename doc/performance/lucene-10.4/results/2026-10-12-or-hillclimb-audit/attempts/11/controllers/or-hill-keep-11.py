"""Publish only the source matched to completed frozen acceptance evidence."""
import json,re,shutil,subprocess,sys
from pathlib import Path
out=Path('/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07')
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
prod=out.parents[1]/'tantivy-pr2937'
sys.path.insert(0,str(out))
from stage import sha,sources
parent=json.loads((out/'parent-06.json').read_text())
assert not (out/'attempt-11').exists()
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()==parent['source_commit']
for name in ('build-final','tests-targeted','tests','clippy','formatting'):
 r=json.loads((out/name/'receipt.json').read_text());assert r['returncode']==0 and r['source_unchanged'],name
for name in ('verify','configured-k0.9-b0.4','configured-k2-b1','measure','port-measure'):
 assert json.loads((out/name/'receipt.json').read_text())['pass'],name
compiled=json.loads((out/'build-final/after.json').read_text());assert compiled==sources()
m=json.loads((out/'attempt-11-metrics.json').read_text())
for kind in ('OR','OR_3PLUS'):assert m['groups'][kind]['candidate_win_both_beyond_group_controls']
for kind in ('TERM','AND','PHRASE','OR_2'):assert not m['groups'][kind]['parent_win_both_beyond_group_controls']
assert sum(map(int,re.findall(r'test result: ok\. (\d+) passed',(out/'tests/stdout.log').read_text())))==1442
path='src/query/boolean_query/or_maxscore.rs'
expected='21153a76a7ae3091ac6400cc438f408b7ac367d9a534b1d290d30af5c3fb5901'
assert sha(repo/path)==expected
assert subprocess.check_output(['git','diff','--name-only'],cwd=repo,text=True).splitlines()==[path]
review=(out/'attempt-11-review.md').read_text();assert expected in review and 'no concrete correctness blocker' in review
subprocess.run(['git','diff','--check'],cwd=repo,check=True)
g=m['groups']['OR_3PLUS'];ratios=g['primary']
port=json.loads((out/'port-group-audit.json').read_text())
message=Path('/tmp/or-hill-11-commit.txt')
message.write_text(f"""perf(query): retain certified global OR prefixes

Cache the accepted global prefix count and exact sequential double sum
across drained regions. Extend with the original term-count bound and
current monotone threshold; copy the effective count afresh so unchanged
live-cost density decisions can still reject or restore optional terms.
Keep local bounds, candidate replay, literal scoring and callbacks unchanged.

Independent old-helper checks compare prefix count and raw sum bits each
region, including density transitions and all-certified termination.
All 1442 tests, three 1227-query exact profiles, Clippy and formatting pass.
Frozen longer-OR arithmetic means take {(1-ratios['AB'])*100:.2f}% and
{(1-ratios['BA'])*100:.2f}% less time than {parent['source_commit'][:7]} in AB/BA,
beyond group self-controls. All OR also clears controls; full-suite and
per-query losses remain retained. One round is mixed. Binary size and index
format are unchanged; logical persistent state adds 16 bytes, no allocation.
The actual-port OR target remains open. This uses the existing optimized
kernel's monotone-threshold domain; it adds no universal callback guarantee.
""")
subprocess.run(['git','add',path],cwd=repo,check=True)
subprocess.run(['git','commit','--file',str(message)],cwd=repo,check=True)
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()
assert compiled['tantivy-or-hillclimb-oct07']['files']==sources()['tantivy-or-hillclimb-oct07']['files']
a={'verdict':'kept_partial_win','commit':head,'parent':parent['source_commit'],
 'source_sha256':expected,'candidate_binary_sha256':sha(out/'target/release/tantivy_worker'),
 'correctness_tests':1442,'ignored_tests':7,'exact_profiles':3,'queries_per_profile':1227,
 'longer_or_primary_ratios':ratios,'all_or_primary_ratios':m['groups']['OR']['primary'],
 'control_log_spread':g['control_log_spread'],'target_met':port['actual_port_target_met'],
 'efficiency':m['efficiency'],'persistent_logical_payload_bytes':16,
 'limitations':'One primary longer-OR round mixed; 19 full-suite per-query losses retained, none in longer-OR. No measured RSS advantage. Port collector also tracks lower-bound hits. Configured profiles are own-oracle correctness only; existing optimized monotone-threshold domain.'}
(out/'attempt-11-acceptance.json').write_text(json.dumps(a,indent=2)+'\n')
summary=json.loads((out/'summary.json').read_text())
gs=next(x for x in summary['summaries'] if x['set']=='port-style' and x['mode']=='top10' and x['kind']=='OR_3PLUS')
before='/'.join(f"{gs[o]['mean_query_median_P_ns']/1000:.2f}" for o in ('AB','BA'))
after='/'.join(f"{gs[o]['mean_query_median_T_ns']/1000:.2f}" for o in ('AB','BA'))
with (out/'decision.tsv').open('a') as f:
 f.write(f"11\tavoid recomputing immutable certified global prefix\texact cached count and sequential f64 sum\t{before} us\t{after} us\t{(ratios['AB']-1)*100:+.2f}%/{(ratios['BA']-1)*100:+.2f}%\t1442 tests;3x1227 exact gates;full AB/BA controls\tkept\t16B logical persistent state;binary unchanged;one mixed round;19 full-suite losses retained;port target open\n")
folder=out/'attempt-11';folder.mkdir()
for name in ('build-final','tests-targeted','tests','clippy','formatting','verify','configured-k0.9-b0.4','configured-k2-b1','pilot-final','measure','port-measure','summary.json','per-query.csv','port-summary.json','port-per-query.csv','port-group-audit.json','attempt-11-metrics.json','attempt-11-acceptance.json'):
 shutil.move(out/name,folder/name)
for name in ('attempt-11-design.md','attempt-11-implementation.md','attempt-11-review.md','attempt-11-integration.json','attempt-11-original.patch'):
 shutil.copy2(out/name,folder/name)
for name in ('tantivy_worker','configured_worker'):shutil.copy2(out/'target/release'/name,folder/name)
controllers=folder/'controllers';controllers.mkdir()
for name in ('balanced.py','compare.py','stage.py','configured_gate.py','pilot.py','analyze.py','port_measure.py','analyze_port.py','measure_audit.py','port_group_audit.py'):
 shutil.copy2(out/name,controllers/name)
for name in ('or-hill-full-11.py','or-hill-port-11.py','or-hill-reject-10-begin-11.py','or-hill-keep-11.py'):
 shutil.copy2(Path('/tmp')/name,controllers/name)
(folder/'accepted.patch').write_bytes(subprocess.check_output(['git','show','--format=','--binary',head],cwd=repo))
d={'binary':str(folder/'tantivy_worker'),'binary_sha256':sha(folder/'tantivy_worker'),
 'source_commit':head,'compiled_sources':str(folder/'build-final/after.json'),
 'compiled_sources_sha256':sha(folder/'build-final/after.json'),
 'exact_gate':str(folder/'verify/receipt.json'),'exact_gate_sha256':sha(folder/'verify/receipt.json')}
assert not (out/'parent-11.json').exists()
(out/'parent-11.json').write_text(json.dumps(d,indent=2)+'\n')
assert not subprocess.check_output(['git','status','--porcelain'],cwd=prod)
subprocess.run(['git','merge','--ff-only',head],cwd=prod,check=True)
subprocess.run(['git','push','fork','HEAD:main'],cwd=prod,check=True)
remote=subprocess.check_output(['git','ls-remote','fork','refs/heads/main'],cwd=prod,text=True).split()[0]
assert remote==head
print(json.dumps({'acceptance':a,'fork_main':remote}),flush=True)
