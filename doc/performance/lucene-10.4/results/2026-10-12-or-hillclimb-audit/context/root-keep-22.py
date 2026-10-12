"""Keep only exact source bound to completed attempt-22 evidence."""
import json,re,shutil,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
prod=out.parents[1]/'tantivy-pr2937'
sys.path.insert(0,str(out))
from stage import sha,sources
read=lambda p:json.loads((out/p).read_text())
parent=read('parent-11.json')
folder=out/'attempt-22'
assert not folder.exists() and not (out/'parent-22.json').exists()
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()==parent['source_commit']
expected=read('attempt-22-static-receipt.json')['source_sha256']
assert subprocess.check_output(['git','diff','--name-only'],cwd=repo,text=True).splitlines()==sorted(expected)
assert not subprocess.check_output(['git','diff','--cached','--name-only'],cwd=repo)
for path,digest in expected.items():assert sha(repo/path)==digest
compiled=read('build-final/after.json');assert compiled==sources()
stages=('build-final','tests-targeted','tests-targeted-score','tests','clippy','formatting','formatting-score')
for name in stages:
 r=read(name+'/receipt.json');assert r['returncode']==0 and r['source_unchanged'],name
 assert read(name+'/before.json')==read(name+'/after.json')==compiled,name
for name in ('verify','configured-k0.9-b0.4','configured-k2-b1'):
 r=read(name+'/receipt.json');assert r['pass'] and r['queries']==1227 and r['failures']==0 and r['unchanged'],name
for name in ('measure','port-measure'):assert read(name+'/receipt.json')['pass'],name
assert read('measure/receipt.json')['source_index_binary_unchanged']
assert read('port-measure/receipt.json')['unchanged']
assert read('measure/receipt.json')['samples_sha256']==sha(out/'measure/samples.jsonl')
port=read('port-group-audit.json');assert port['samples_sha256']==sha(out/'port-measure/samples.jsonl')
counts=re.findall(r'test result: ok\. (\d+) passed; \d+ failed; (\d+) ignored',(out/'tests/stdout.log').read_text())
assert sum(int(a) for a,b in counts)==1446 and sum(int(b) for a,b in counts)==7
m=read('attempt-22-metrics.json');assert m['parent']==parent
for kind in ('OR','OR_3PLUS'):assert m['groups'][kind]['candidate_win_both_beyond_group_controls']
for kind in ('ALL','TERM','AND','PHRASE','OR_2'):assert not m['groups'][kind]['parent_win_both_beyond_group_controls']
assert len(m['regressions_beyond_per_query_controls'])==60
assert not any(r['kind']=='OR' and len(r['terms'].split())>=3 for r in m['regressions_beyond_per_query_controls'])
worker=out/'target/release/tantivy_worker'
assert sha(worker)==m['efficiency']['candidate_binary_sha256']=='005dd61a74e1885cb3e145987fc67399d89fe662bb2409aa00a9c1d174726b84'
review=(out/'attempt-22-root-decision.md').read_text()
assert all(d in review for d in expected.values()) and '60 full-suite' in review
assert not subprocess.check_output(['git','status','--porcelain'],cwd=prod)
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=prod,text=True).strip()==parent['source_commit']
remote=subprocess.check_output(['git','ls-remote','fork','refs/heads/main'],cwd=prod,text=True).split()[0]
assert remote==parent['source_commit'],remote
subprocess.run(['git','diff','--check'],cwd=repo,check=True)
message=out/'attempt-22-commit-message.txt'
message.write_text('''perf(query): cache exact optional score threshold cutoff

Move the f64 successor and f32 rounding comparison out of optional probes
into an exact threshold midpoint cutoff. Preserve the original rounded
factor multiplication, score order and callback/cursor behavior; refresh
the cutoff immediately after every callback threshold assignment.

Cover midpoint parity, signed zeros, exponent boundaries, finite overflow
and exceptional thresholds against the original bound. All 1446 tests,
three 1227-query exact profiles, Clippy and formatting pass.

Frozen longer OR arithmetic means take 2.77%/3.13% less time in AB/BA
against 43f0683, past 0.36% controls. All OR also clears controls. This is
a partial win: 60 full-suite per-query losses remain retained and the
actual lucene-rs longer OR gap is still 1.71-1.72x. Binary grows 448 bytes;
one f64 cache adds 8 logical bytes, no allocation or index format change.
''')
subprocess.run(['git','add',*sorted(expected)],cwd=repo,check=True)
subprocess.run(['git','commit','--file',str(message)],cwd=repo,check=True)
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()
assert compiled['tantivy-or-hillclimb-oct07']['files']==sources()['tantivy-or-hillclimb-oct07']['files']
g=m['groups']['OR_3PLUS'];ratios=g['primary']
a={'verdict':'kept_partial_win','commit':head,'parent':parent['source_commit'],
 'source_sha256':expected,'candidate_binary_sha256':sha(worker),'correctness_tests':1446,
 'ignored_tests':7,'exact_profiles':3,'queries_per_profile':1227,
 'longer_or_primary_ratios':ratios,'all_or_primary_ratios':m['groups']['OR']['primary'],
 'control_log_spread':g['control_log_spread'],'target_met':port['actual_port_target_met'],
 'efficiency':m['efficiency'],'persistent_logical_payload_bytes':8,
 'limitations':'60 full-suite controlled per-query losses retained, none in longer OR. Whole-suite and PHRASE mixed. No RSS advantage established. Existing optimized monotone-threshold domain. Configured own-oracle correctness only. Port tracks lower-bound hits.'}
(out/'attempt-22-acceptance.json').write_text(json.dumps(a,indent=2)+'\n')
s=next(x for x in read('summary.json')['summaries'] if x['set']=='port-style' and x['mode']=='top10' and x['kind']=='OR_3PLUS')
before='/'.join(f"{s[o]['mean_query_median_P_ns']/1000:.2f}" for o in ('AB','BA'))
after='/'.join(f"{s[o]['mean_query_median_T_ns']/1000:.2f}" for o in ('AB','BA'))
with (out/'decision.tsv').open('a') as f:
 f.write(f"22\tremove repeated optional successor/cast\texact cached post-multiply threshold cutoff\t{before} us full\t{after} us full\t{(ratios['AB']-1)*100:+.2f}%/{(ratios['BA']-1)*100:+.2f}%\t1446 tests;3x1227 exact gates;full AB/BA controls\tkept\t8B logical cache;binary+448B;60 fullsuite losses retained;actualport target open\n")
folder.mkdir()
for name in (*stages,'verify','configured-k0.9-b0.4','configured-k2-b1','pilot-final','measure','port-measure','summary.json','per-query.csv','port-summary.json','port-per-query.csv','port-group-audit.json','attempt-22-metrics.json','attempt-22-acceptance.json'):
 shutil.move(out/name,folder/name)
for p in sorted(out.glob('attempt-22-*')):
 if p.is_file():shutil.copy2(p,folder/p.name)
for name in ('tantivy_worker','configured_worker'):shutil.copy2(out/'target/release'/name,folder/name)
controllers=folder/'controllers';controllers.mkdir()
for name in ('balanced.py','compare.py','stage.py','configured_gate.py','pilot.py','analyze.py','port_measure.py','analyze_port.py','measure_audit.py','port_group_audit.py','root-run-stage.py','root-begin-22.py','root-primary-22.py','root-full-22.py','root-port-22.py','root-keep-22.py'):
 shutil.copy2(out/name,controllers/name)
(folder/'accepted.patch').write_bytes(subprocess.check_output(['git','show','--format=','--binary',head],cwd=repo))
d={'binary':str(folder/'tantivy_worker'),'binary_sha256':sha(folder/'tantivy_worker'),
 'source_commit':head,'compiled_sources':str(folder/'build-final/after.json'),
 'compiled_sources_sha256':sha(folder/'build-final/after.json'),
 'exact_gate':str(folder/'verify/receipt.json'),'exact_gate_sha256':sha(folder/'verify/receipt.json')}
(out/'parent-22.json').write_text(json.dumps(d,indent=2)+'\n')
subprocess.run(['git','merge','--ff-only',head],cwd=prod,check=True)
subprocess.run(['git','push','fork','HEAD:main'],cwd=prod,check=True)
remote=subprocess.check_output(['git','ls-remote','fork','refs/heads/main'],cwd=prod,text=True).split()[0]
assert remote==head
print(json.dumps({'acceptance':a,'fork_main':remote}),flush=True)
