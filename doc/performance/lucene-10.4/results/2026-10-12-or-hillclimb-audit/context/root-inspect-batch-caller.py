import hashlib,json,subprocess
from pathlib import Path
out=Path(__file__).resolve().parent
folder=out/'batch-caller-inspection'
folder.mkdir()
parent=json.loads((out/'parent-11.json').read_text())
binary=Path(parent['binary'])
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(binary)==parent['binary_sha256']
for key in ('compiled_sources','exact_gate'):assert sha(Path(parent[key]))==parent[key+'_sha256']
commands=[]
def run(argv,name):
 commands.append({'argv':argv,'stdout':name+'.stdout','stderr':name+'.stderr'})
 r=subprocess.run(argv,capture_output=True,timeout=60)
 (folder/(name+'.stdout')).write_bytes(r.stdout)
 (folder/(name+'.stderr')).write_bytes(r.stderr)
 assert r.returncode==0
 return r.stdout.decode()
try:
 text=run(['nm','-S','--demangle',str(binary)],'symbols')
 symbols=[]
 for line in text.splitlines():
  parts=line.split(maxsplit=3)
  if len(parts)!=4:continue
  address,size,kind,name=parts
  if name == '<tantivy::query::boolean_query::boolean_weight::BooleanWeight<tantivy::query::score_combiner::SumCombiner> as tantivy::query::weight::Weight>::for_each_pruning' and kind.lower()=='t':
   symbols.append({'address':int(address,16),'bytes':int(size,16),'name':name})
 assert len(symbols)==1, symbols
 for i,symbol in enumerate(symbols):
  run(['objdump','-d','--demangle','--start-address='+hex(symbol['address']),'--stop-address='+hex(symbol['address']+symbol['bytes']),str(binary)],'route-'+str(i))
 assert sha(binary)==parent['binary_sha256']
 receipt={'pass':True,'purpose':'Inspect the enclosing sum-OR pruning caller in immutable accepted11; commands alone do not attribute instructions to its owned batch route or establish savings','parent':parent,'symbols':symbols,'commands':commands,'binary_unchanged':True,'source_to_binary_authenticity_claimed':False}
except BaseException as e:
 receipt={'pass':False,'error':{'type':type(e).__name__,'message':str(e)},'commands':commands}
(folder/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt),flush=True)
assert receipt['pass']
