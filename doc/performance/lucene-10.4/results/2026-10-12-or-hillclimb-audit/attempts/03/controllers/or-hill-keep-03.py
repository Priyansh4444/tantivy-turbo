import json, re, subprocess, sys
from pathlib import Path
out=Path('/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07')
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
sys.path.insert(0,str(out)); from stage import sources,sha
for name in ('build-final','tests','clippy','formatting','tests-targeted'):
 r=json.loads((out/name/'receipt.json').read_text()); assert r['returncode']==0 and r['source_unchanged'],name
for name in ('verify','configured-k0.9-b0.4','configured-k2-b1','measure','port-measure'):
 assert json.loads((out/name/'receipt.json').read_text())['pass'],name
compiled=json.loads((out/'build-final/after.json').read_text()); assert compiled==sources()
m=json.loads((out/'attempt-03-metrics.json').read_text())
assert m['groups']['OR_3PLUS']['candidate_win_both_beyond_group_controls']
assert all(r['primary']<1 for r in m['groups']['OR_3PLUS']['rounds'])
assert all(not m['groups'][kind]['parent_win_both_beyond_group_controls'] for kind in ('TERM','AND','PHRASE'))
paths=['src/postings/segment_postings.rs','src/query/bm25.rs','src/query/boolean_query/or_maxscore.rs','src/query/term_query/mod.rs','src/query/term_query/term_scorer.rs']
assert set(subprocess.check_output(['git','diff','--name-only'],cwd=repo,text=True).splitlines())==set(paths)
assert sum(int(x) for x in re.findall(r'test result: ok\. (\d+) passed',(out/'tests/stdout.log').read_text()))==1438
subprocess.run(['git','diff','--check'],cwd=repo,check=True)
message=Path('/tmp/or-hill-03-commit.txt');message.write_text('''perf(query): batch exact single-essential OR leaves

Own block suffix consumption in SegmentPostings, gather actual cached norms,
and evaluate the literal native/legacy BM25 expressions in a separate loop.
Replay retained driver leaves with fresh callback thresholds and unchanged
incoming-order Boolean score publication; leave multi-essential routing intact.

Four independent fixture families cover first-unread cursors, positions and
SkipFreq, all norm IDs and frequency conversion edges, and full callback traces.
All 1438 tests, default/two configured 1227-query exact gates, and clippy pass.

The complete frozen 103 longer-OR group takes 3.5-7.0% less time than22f4a
in both orders beyond observed group controls.40 queries improve beyond their
own controls and2 regress. Adds2KiB fixed scratch and2584 binary file bytes;
no index-format change or per-region allocation. This is a partial improvement,
not a reliable speed lead over the fixed port or a universal per-query win.
''')
subprocess.run(['git','add',*paths],cwd=repo,check=True)
subprocess.run(['git','commit','--file',str(message)],cwd=repo,check=True)
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()
assert compiled['tantivy-or-hillclimb-oct07']['files']==sources()['tantivy-or-hillclimb-oct07']['files']
receipt={'verdict':'kept_partial_win','commit':head,'candidate_binary_sha256':sha(out/'target/release/tantivy_worker'),'correctness_tests':1438,'ignored_tests':7,'exact_profiles':3,'queries_per_profile':1227,'longer_or_primary_ratios':m['groups']['OR_3PLUS']['primary'],'control_log_spread':m['groups']['OR_3PLUS']['control_log_spread'],'target_met':False,'tradeoffs':'2KiB fixed scratch; +2584 binary file bytes;2 longer-OR regressions beyond own controls; no format/index/perregion allocation changes.'}
(out/'attempt-03-acceptance.json').write_text(json.dumps(receipt,indent=2)+'\n')
with (out/'decision.tsv').open('a') as f:f.write('03\tbulk exact leaves reduce cursor and scoring overhead\tsingle-essential block suffix and literal BM25 batch\t609.55/616.18 us\t588.33/572.94 us\t-3.48%/-7.02%\t1438 tests;3x1227 exact gates;full AB/BA controls\tkept\t40/103 beyond perquerycontrols;2 losses;2KiB fixed scratch;+2584 binarybytes;portlead stillpending\n')
print(json.dumps(receipt))
