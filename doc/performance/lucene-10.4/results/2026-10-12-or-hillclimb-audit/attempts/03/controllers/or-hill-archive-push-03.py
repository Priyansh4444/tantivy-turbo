import json,shutil,subprocess,sys
from pathlib import Path
out=Path('/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07');repo=out.parents[1]/'tantivy-or-hillclimb-oct07';prod=out.parents[1]/'tantivy-pr2937'
sys.path.insert(0,str(out));from stage import sha,sources
message=Path('/tmp/or-hill-03-commit.txt')
text=message.read_text().replace('than22f4a','than 22f4a').replace('controls.40','controls. Forty').replace('controls and2','controls and 2').replace('Adds2KiB fixed scratch and2584','Adds 2 KiB fixed scratch and 2,584')
message.write_text(text)
subprocess.run(['git','commit','--amend','--file',str(message)],cwd=repo,check=True)
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()
a=json.loads((out/'attempt-03-acceptance.json').read_text());a['commit']=head
(out/'attempt-03-acceptance.json').write_text(json.dumps(a,indent=2)+'\n')
compiled=json.loads((out/'build-final/after.json').read_text())
assert compiled['tantivy-or-hillclimb-oct07']['files']==sources()['tantivy-or-hillclimb-oct07']['files']
folder=out/'attempt-03';folder.mkdir()
for name in ('build-final','tests-targeted','tests','clippy','formatting','verify','configured-k0.9-b0.4','configured-k2-b1','pilot-final','measure','port-measure'):
 shutil.move(out/name,folder/name)
for name in ('summary.json','per-query.csv','port-summary.json','port-per-query.csv','attempt-03-metrics.json','attempt-03-acceptance.json','attempt-03-implementation.md','attempt-03-review.md'):
 shutil.move(out/name,folder/name)
for name in ('tantivy_worker','configured_worker'): shutil.copy2(out/'target/release'/name,folder/name)
c=folder/'controllers';c.mkdir()
for name in ('balanced.py','compare.py','stage.py','configured_gate.py','pilot.py','analyze.py','port_measure.py','analyze_port.py','measure_audit.py'): shutil.copy2(out/name,c/name)
for name in ('or-hill-keep-03.py','or-hill-archive-push-03.py'): shutil.copy2(Path('/tmp')/name,c/name)
(folder/'accepted.patch').write_bytes(subprocess.check_output(['git','show','--format=','--binary',head],cwd=repo))
p={'binary':str(folder/'tantivy_worker'),'binary_sha256':sha(folder/'tantivy_worker'),'source_commit':head,'compiled_sources':str(folder/'build-final/after.json'),'compiled_sources_sha256':sha(folder/'build-final/after.json'),'exact_gate':str(folder/'verify/receipt.json'),'exact_gate_sha256':sha(folder/'verify/receipt.json')}
(out/'parent-03.json').write_text(json.dumps(p,indent=2)+'\n')
assert not subprocess.check_output(['git','status','--porcelain'],cwd=prod)
subprocess.run(['git','merge','--ff-only',head],cwd=prod,check=True)
subprocess.run(['git','push','fork','HEAD:main'],cwd=prod,check=True)
remote=subprocess.check_output(['git','ls-remote','fork','refs/heads/main'],cwd=prod,text=True).split()[0]
assert remote==head
print(json.dumps({'parent':p,'fork_main':remote,'production_clean':not subprocess.check_output(['git','status','--porcelain'],cwd=prod)}))
