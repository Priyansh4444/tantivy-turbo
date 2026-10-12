import hashlib,json,subprocess
from pathlib import Path
out=Path(__file__).resolve().parent
folder=out/'accepted11-long-or-annotation';folder.mkdir()
parent=json.loads((out/'parent-11.json').read_text());binary=Path(parent['binary'])
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
profile=out/'accepted11-long-or-oct11-profile-receipt.json'
trace=out/'accepted11-long-or-oct11-perf.data'
r=json.loads(profile.read_text());assert r['binary_sha256']==sha(binary)==parent['binary_sha256']
assert r['returncode']==0 and r['binary_unchanged'] and r['inputs_unchanged'] and r['correctness_checksums_match'] and not r['latency_acceptance']
assert len(r['cases'])==103
before={'trace':sha(trace),'profile':sha(profile),'binary':sha(binary)}
symbol='<tantivy::query::boolean_query::boolean_weight::BooleanWeight<tantivy::query::score_combiner::SumCombiner> as tantivy::query::weight::Weight>::for_each_pruning'
command=['perf','annotate','--stdio','--no-source','--percent-type','global-period','--show-total-period','--symbol',symbol,'-i',str(trace)]
(folder/'command.json').write_text(json.dumps(command,indent=2)+'\n')
r=subprocess.run(command,capture_output=True,timeout=120)
(folder/'stdout.log').write_bytes(r.stdout);(folder/'stderr.log').write_bytes(r.stderr)
after={'trace':sha(trace),'profile':sha(profile),'binary':sha(binary)}
(folder/'receipt.json').write_text(json.dumps({'pass':r.returncode==0 and before==after,'returncode':r.returncode,'before':before,'after':after,'purpose':'Instruction sample attribution in existing accepted11 longOR profile; no comparative latency or causal speed claim'},indent=2)+'\n')
assert r.returncode==0 and before==after
print('accepted11 long OR instruction annotation retained')
