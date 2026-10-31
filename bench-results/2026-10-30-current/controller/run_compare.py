import hashlib, json, math, os, platform, random, statistics, subprocess, sys, time
from pathlib import Path
import balanced
from balanced import Worker, CASES, native_run, validate_dump
workdir=Path('/path/to/benchmark-work')
base=Path('/path/to/retained-search/bench')
out=workdir/'rust/results'; out.mkdir(exist_ok=True)
worker=workdir/'rust/harness/target/release/tantivy_worker'
tindex=base/'wiki-1m-native-aligned-l1.idx'
portroot=base/'lucene-rs-balanced-oct06'
proof={r['case']['id']:r for r in json.loads((portroot/'verify/dumps.json').read_text())}
work=[q for q in CASES if 'correctness-only' not in q['tags']]
balanced.BINS={'tantivy':worker,'port':portroot/'target/release/port_worker','upstream':base/'turbo-vs-upstream/upstream-harness/target/release/tantivy_worker'}
balanced.INDICES={'tantivy':tindex,'port':portroot/'port.idx','upstream':base/'turbo-vs-upstream/upstream-027.idx'}
allowed=os.sched_getaffinity(0)
if not {4,6}.issubset(allowed): raise RuntimeError(f'expected CPUs 4/6 unavailable: {allowed}')
os.sched_setaffinity(0,{6})
class UpstreamWorker(Worker):
 def __init__(self,engine,folder,label,cpu=4,deadline=None):
  self.engine=engine; self.expected=None; self.deadline=deadline
  self.stderr=(folder/f'{label}.stderr').open('w'); self.log=(folder/f'{label}.protocol.jsonl').open('w')
  self.p=subprocess.Popen([str(balanced.BINS[engine]),str(balanced.INDICES[engine]),str(balanced.QUERIES)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=self.stderr,text=True,bufsize=1,preexec_fn=lambda:os.sched_setaffinity(0,{cpu}))
  self.ready=self.read(120)
  if self.ready['queries']!=len(CASES) or self.ready['num_docs']!=1000000: raise RuntimeError(f'upstream ready mismatch: {self.ready}')
def digest(path):
 h=hashlib.sha256()
 with path.open('rb') as f:
  for block in iter(lambda:f.read(1024*1024),b''): h.update(block)
 return h.hexdigest()
def identity(engine):
 return {'commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=workdir/'repo',text=True).strip(),
  'binary_hashes':{n:digest(balanced.BINS[n]) for n in ('tantivy',engine)},
  'queries_sha256':digest(balanced.QUERIES),'proof_sha256':digest(portroot/'verify/dumps.json'),
  'indices':{n:{str(p.relative_to(balanced.INDICES[n])):digest(p) for p in sorted(balanced.INDICES[n].rglob('*')) if p.is_file() and not p.name.endswith('.lock')} for n in ('tantivy',engine)},
  'controller_sha256':digest(Path(__file__))}
def launch(engine,folder):
 a=Worker('tantivy',folder,'candidate',deadline=time.monotonic()+7200)
 b=Worker('port',folder,'port',deadline=time.monotonic()+7200) if engine=='port' else UpstreamWorker('upstream',folder,'upstream',deadline=time.monotonic()+7200)
 return {'tantivy':a,engine:b}
def run(engine):
 folder=out/engine; folder.mkdir(exist_ok=False)
 before=identity(engine); workers=launch(engine,folder); dumps={}; raw=[]; oracle_differences=[]
 try:
  for i,q in enumerate(CASES):
   a=workers['tantivy'].request({'op':'dump','id':q['id']},timeout=120); validate_dump(q,a)
   b=workers[engine].request({'op':'dump','id':q['id']},timeout=120)
   if a['count']!=a['oracle']['count'] or a['hits']!=a['oracle']['hits']: raise RuntimeError(f"candidate differs from exhaustive at {q['id']}")
   f=proof[q['id']]
   if a['count']!=f['tantivy']['count'] or a['hits']!=f['tantivy']['hits']: raise RuntimeError(f"candidate differs from frozen at {q['id']}")
   if engine=='port' and (b['count']!=f['port']['count'] or b['hits']!=f['port']['hits']): raise RuntimeError(f"port differs from frozen at {q['id']}")
   if engine=='upstream':
    if a['count']!=b['count']: raise RuntimeError(f"upstream count differs at {q['id']}")
    if b['hits']!=b['oracle']['hits'] or b['count']!=b['oracle']['count']: oracle_differences.append(q['id'])
   dumps[q['id']]={'tantivy':a,engine:b}
   if i%100==0: print(engine,'verified',i,flush=True)
  (folder/'dumps.json').write_text(json.dumps(dumps))
  (folder/'verification.json').write_text(json.dumps({'queries':len(dumps),'candidate_frozen_and_oracle_failures':0,'cross_engine_count_failures':0,'upstream_oracle_score_differences':oracle_differences,'identity_before':before},indent=2))
  print(engine,'correctness verified; WAITING FOR TIMING PERMISSION',flush=True)
 finally:
  for w in workers.values(): w.close()
 # Parent creates explicit permission file only after compilation and competing work stops.
 if not (workdir/'rust/timing-allowed').exists(): print('timing not yet authorized; rerun with measure',flush=True); return
 measure(engine, before)
def measure(engine,before=None):
 folder=out/engine; folder.mkdir(exist_ok=True)
 verified=json.loads((folder/'verification.json').read_text())
 before=before or verified['identity_before']
 if before!=identity(engine):raise RuntimeError('benchmark inputs changed after verification')
 dumps={int(k):v for k,v in json.loads((folder/'dumps.json').read_text()).items()}
 workers=launch(engine,folder); raw=[]
 try:
  for name,w in workers.items():w.expected={qid:d[name] for qid,d in dumps.items()}
  for q in work:
   for w in workers.values(): native_run(w,q,'top10',4); native_run(w,q,'count',2)
  metadata={'start_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),'loadavg_before':os.getloadavg(),'uname':platform.uname()._asdict(),'allowed_cpu':4,'controller_cpu':6,'samples_per_order':12,'orders':['AB','BA']}
  for order in ('AB','BA'):
   cases=work.copy(); random.Random(20261018).shuffle(cases)
   for i,q in enumerate(cases):
    for mode in ('top10','count'):
     for side in order:
      name='tantivy' if side=='A' else engine
      raw.append({'order':order,'engine':name,'mode':mode,'id':q['id'],'ns':native_run(workers[name],q,mode,12)['ns']})
    if i%100==0:print(engine,order,i,flush=True)
  rss={name:w.rss() for name,w in workers.items()}; metadata['loadavg_after']=os.getloadavg(); metadata['end_utc']=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime())
 finally:
  for w in workers.values():w.close()
 (folder/'samples.json').write_text(json.dumps(raw)); (folder/'metadata.json').write_text(json.dumps(metadata,indent=2))
 med={(x['order'],x['engine'],x['mode'],x['id']):statistics.median(x['ns']) for x in raw}
 def group(q):
  if 'wiki20' in q['tags'] and 'port-style' not in q['tags']:return 'wiki20'
  if q['kind']=='OR':return 'OR3+' if len(q['terms'])>=3 else 'OR2'
  return q['kind']
 summary={}
 for mode in ('top10','count'):
  for g in ('ALL','TERM','AND','OR2','OR3+','PHRASE','wiki20'):
   ids=[q['id'] for q in work if (g=='ALL' and 'port-style' in q['tags']) or group(q)==g]
   if not ids:continue
   r={}
   for order in ('AB','BA'):
    t=sum(med[order,'tantivy',mode,i] for i in ids);b=sum(med[order,engine,mode,i] for i in ids)
    gm=math.exp(statistics.mean(math.log(med[order,'tantivy',mode,i]/med[order,engine,mode,i]) for i in ids))
    r[order]={'turbo_mean_us':t/len(ids)/1000,f'{engine}_mean_us':b/len(ids)/1000,'sum_ratio':t/b,'geomean_ratio':gm,'turbo_faster':sum(med[order,'tantivy',mode,i]<med[order,engine,mode,i] for i in ids)}
   summary[f'{mode}/{g}']={'queries':len(ids),**r}
 summary['rss']=rss
 (folder/'summary.json').write_text(json.dumps(summary,indent=2)); after=identity(engine)
 if before!=after:raise RuntimeError('benchmark inputs changed during run')
 (folder/'identity-after.json').write_text(json.dumps(after,indent=2));print(json.dumps(summary,indent=2),flush=True)
if __name__=='__main__':
 engine=sys.argv[2]; run(engine) if sys.argv[1]=='verify' else measure(engine)
