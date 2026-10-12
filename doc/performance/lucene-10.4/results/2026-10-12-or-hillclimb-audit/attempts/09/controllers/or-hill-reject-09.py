import json,shutil,subprocess,sys
from pathlib import Path
out=Path('/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07')
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
sys.path.insert(0,str(out))
from stage import sha,sources
parent=json.loads((out/'parent-06.json').read_text())
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()==parent['source_commit']
assert json.loads((out/'build-final/after.json').read_text())==sources()
for name in ('build-final','tests-targeted'):
    receipt=json.loads((out/name/'receipt.json').read_text());assert receipt['returncode']==0 and receipt['source_unchanged']
for name in ('verify','configured-k0.9-b0.4','configured-k2-b1'):
    assert json.loads((out/name/'receipt.json').read_text())['pass']
pilot=json.loads((out/'pilot-final/receipt.json').read_text());assert pilot['unchanged']
row=next(row for row in pilot['summary'] if row['set']=='or3plus')
assert all(row[order]['ratio_mean']>1.04 for order in ('AB','BA'))
path='src/query/boolean_query/or_maxscore.rs'
assert subprocess.check_output(['git','diff','--name-only'],cwd=repo,text=True).splitlines()==[path]
folder=out/'attempt-09';folder.mkdir()
for name in ('build-final','tests-targeted','verify','configured-k0.9-b0.4','configured-k2-b1','pilot-final'):
    shutil.move(out/name,folder/name)
shutil.move(out/'attempt-09-pre-oracle-fix',folder/'pre-oracle-fix')
for name in ('tantivy_worker','configured_worker'):
    shutil.copy2(out/'target/release'/name,folder/name)
for name in ('attempt-09-implementation.md','attempt-09-review.md','attempt-09-integration.json','design-posting-heap-v2.md','design-single-lead.md','design-sparse-merge-synthesis.md','design-sparse-merge-judge.md','design-next-rubric.md'):
    shutil.copy2(out/name,folder/name)
controllers=folder/'controllers';controllers.mkdir()
for name in ('balanced.py','compare.py','stage.py','configured_gate.py','pilot.py'):
    shutil.copy2(out/name,controllers/name)
for name in ('or-hill-begin-09.py','or-hill-09-oracle-fix.py','or-hill-reject-09.py'):
    shutil.copy2(Path('/tmp')/name,controllers/name)
(folder/'rejected.patch').write_bytes(subprocess.check_output(['git','diff','--binary'],cwd=repo))
rejection={'verdict':'reverted','parent':parent['source_commit'],'reason':'Longer OR pilot 4.32%/8.27% slower in AB/BA. No full sampler/broad tests run.',
    'pilot':pilot['summary'],'targeted_tests':20,'exact_profiles':3,'queries_per_profile':1227,
    'candidate_binary_sha256':sha(folder/'tantivy_worker'),
    'candidate_file_delta_bytes':(folder/'tantivy_worker').stat().st_size-Path(parent['binary']).stat().st_size,
    'source_sha256':sha(repo/path),'memory':'Measured array/record payload 37000B at n32 vs foundation3208B; full RSS not measured.',
    'oracle_correction':'Guard observer seek when original term starts after candidate; initial17pass3fail retained; no production change.'}
(folder/'rejection.json').write_text(json.dumps(rejection,indent=2)+'\n')
with (out/'decision.tsv').open('a') as stream:
    stream.write('09\tposting-bounded merge replaces repeated essential scalar scans\tperessential exact leaf batches and32headheap\t274.35/278.60 us pilot\t286.20/301.65 us pilot\t+4.32%/+8.27%\t20 targeted;3x1227 exact gates;full sampler not run\treverted\t37000B n32 array payload;forward-only oracle corrected;failed run retained;no broad tests or port claim\n')
subprocess.run(['git','restore','--',path],cwd=repo,check=True)
assert not subprocess.check_output(['git','diff','--name-only'],cwd=repo)
print(json.dumps(rejection))
