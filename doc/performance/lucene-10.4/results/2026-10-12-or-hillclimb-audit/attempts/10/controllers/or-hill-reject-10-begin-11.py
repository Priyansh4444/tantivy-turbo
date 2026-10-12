import json,shutil,subprocess,sys
from pathlib import Path
out=Path('/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07')
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
trial=out.parents[1]/'tantivy-or-global-prefix-oct07'
sys.path.insert(0,str(out))
from stage import sha,sources
parent=json.loads((out/'parent-06.json').read_text())
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()==parent['source_commit']
assert json.loads((out/'build-final/after.json').read_text())==sources()
for name in ('build-final','tests-targeted'):
    receipt=json.loads((out/name/'receipt.json').read_text());assert receipt['returncode']==0 and receipt['source_unchanged']
for name in ('verify','configured-k0.9-b0.4','configured-k2-b1'):
    assert json.loads((out/name/'receipt.json').read_text())['pass']
pilots=[json.loads((out/name/'receipt.json').read_text()) for name in ('attempt-10-pilot-first-load-heavy','pilot-final')]
for pilot in pilots:
    assert pilot['unchanged']
    row=next(row for row in pilot['summary'] if row['set']=='or3plus')
    assert row['AB']['ratio_mean']>1 and row['BA']['ratio_mean']<1
path='src/query/boolean_query/or_maxscore.rs'
assert subprocess.check_output(['git','diff','--name-only'],cwd=repo,text=True).splitlines()==[path]
folder=out/'attempt-10';folder.mkdir()
for name in ('build-final','tests-targeted','verify','configured-k0.9-b0.4','configured-k2-b1','pilot-final'):
    shutil.move(out/name,folder/name)
shutil.move(out/'attempt-10-pre-fixture-fix',folder/'pre-fixture-fix')
shutil.move(out/'attempt-10-pilot-first-load-heavy',folder/'pilot-first-load-heavy')
for name in ('tantivy_worker','configured_worker'):
    shutil.copy2(out/'target/release'/name,folder/name)
for name in ('attempt-10-implementation.md','attempt-10-review.md','attempt-10-integration.json','attempt-10-load-processes.json','design-single-lead-128.md'):
    shutil.copy2(out/name,folder/name)
controllers=folder/'controllers';controllers.mkdir()
for name in ('balanced.py','compare.py','stage.py','configured_gate.py','pilot.py'):
    shutil.copy2(out/name,controllers/name)
for name in ('or-hill-begin-10.py','or-hill-10-fixture-fix.py','or-hill-reject-10-begin-11.py'):
    shutil.copy2(Path('/tmp')/name,controllers/name)
(folder/'rejected.patch').write_bytes(subprocess.check_output(['git','diff','--binary'],cwd=repo))
rejection={'verdict':'reverted','parent':parent['source_commit'],'reason':'No reliable win in two unchanged pilots: longer OR AB/BA initially +11.21%/-2.15%; repeat +7.75%/-8.51%. Substantial background compiler load retained; no intrinsic loss claim.',
    'pilots':[p['summary'] for p in pilots],'targeted_tests':17,'exact_profiles':3,'queries_per_profile':1227,
    'candidate_binary_sha256':sha(folder/'tantivy_worker'),
    'candidate_file_delta_bytes':(folder/'tantivy_worker').stat().st_size-Path(parent['binary']).stat().st_size,
    'source_sha256':sha(repo/path),'memory':'No extra allocation; reported span/replay sizes in targeted stderr, not RSS.',
    'fixture_correction':'Explicit .order_by_score() on new TopDocs fixture. Initial E0277 retained; no production change.',
    'limits':'Full sampler/broad tests/actualport not run; no causal performance claim under load.'}
(folder/'rejection.json').write_text(json.dumps(rejection,indent=2)+'\n')
with (out/'decision.tsv').open('a') as stream:
    stream.write('10\tuniquelead span bypasses repeated multiessential scans\tfixed128 cappedgap lead replay\t728.96/836.64 us repeat pilot\t785.41/765.43 us repeat pilot\t+7.75%/-8.51%\t17 targeted;3x1227 exact gates;2 unchanged pilots\treverted\tbothpilotsmixed underbackgroundcompilerload;no reliablewin/no intrinsiclossclaim;failed collector fixture compile retained\n')
subprocess.run(['git','restore','--',path],cwd=repo,check=True)
assert not subprocess.check_output(['git','diff','--name-only'],cwd=repo)
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=trial,text=True).strip()==parent['source_commit']
assert subprocess.check_output(['git','diff','--name-only'],cwd=trial,text=True).splitlines()==[path]
expected='21153a76a7ae3091ac6400cc438f408b7ac367d9a534b1d290d30af5c3fb5901'
assert sha(trial/path)==expected
patch=subprocess.check_output(['git','diff','--binary','--',path],cwd=trial)
(out/'attempt-11-original.patch').write_bytes(patch)
subprocess.run(['git','apply','-'],cwd=repo,input=patch,check=True)
assert sha(repo/path)==expected
subprocess.run(['git','diff','--check'],cwd=repo,check=True)
(out/'attempt-11.patch').write_bytes(subprocess.check_output(['git','diff','--binary'],cwd=repo))
(out/'attempt-11-integration.json').write_text(json.dumps({'parent':parent['source_commit'],'source_sha256':expected,'adaptation':'None; integrated source byte-identical to isolated reviewed source.'},indent=2)+'\n')
print(json.dumps(rejection),flush=True)
command=['cargo','test','--locked','--manifest-path','../../../tantivy-or-hillclimb-oct07/Cargo.toml','--target-dir',str(out/'test-target'),'--lib','query::boolean_query::or_maxscore::tests','--','--test-threads=2','--nocapture']
subprocess.run([sys.executable,'stage.py','tests-targeted','1200',*command],cwd=out,check=True)
subprocess.run([sys.executable,'/tmp/or-hill-run.py','build'],check=True)
subprocess.run([sys.executable,'/tmp/or-hill-gates.py','parent-06.json'],check=True)
subprocess.run([sys.executable,'compare.py','pilot','--parent','parent-06.json'],cwd=out,check=True)
