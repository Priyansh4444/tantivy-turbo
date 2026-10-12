"""Root-only unaccepted instrumentation; never a latency acceptance ruler."""
import hashlib,json,os,shutil,subprocess,sys,time
from pathlib import Path
out=Path('/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07')
repo=out.parents[1]/'tantivy-or-region-diagnostic-oct07'
sys.path.insert(0,str(out))
from stage import sha
import balanced
op=sys.argv[1]
folder=out/'region-diagnostic'
def snapshot():
    files=subprocess.check_output(['git','ls-files','-z'],cwd=repo).split(b'\0')
    return {'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),
        'status':subprocess.check_output(['git','status','--porcelain'],cwd=repo,text=True),
        'files':{os.fsdecode(p):sha(repo/os.fsdecode(p)) for p in files if p}}
if op=='build':
    assert sha(repo/'src/query/boolean_query/or_maxscore.rs')=='cd2865e1364a66fadda587bc5a24eaaec6086b39fcfd48f1493fdeb8974b36bd'
    folder.mkdir()
    harness=out/'region-diagnostic-harness'
    shutil.copytree(out/'harness',harness)
    text=(harness/'Cargo.toml').read_text()
    original='tantivy = { path = "../../../tantivy-or-hillclimb-oct07" }'
    replacement='tantivy = { path = "../../../tantivy-or-region-diagnostic-oct07" }'
    assert text.count(original)==1
    (harness/'Cargo.toml').write_text(text.replace(original,replacement))
    before=snapshot();assert before['head']=='a674aaf2f6187f3010a135dc25fdacf9cab27e53'
    (folder/'source-before.json').write_text(json.dumps(before,indent=2)+'\n')
    subprocess.run([sys.executable,'stage.py','region-diagnostic-build','1200','cargo','build','--locked','--release','--manifest-path',str(harness/'Cargo.toml'),'--bin','tantivy_worker'],cwd=out,check=True)
    after=snapshot();(folder/'source-after.json').write_text(json.dumps(after,indent=2)+'\n');assert before==after
    shutil.copy2(out/'target/release/tantivy_worker',folder/'tantivy_worker')
    shutil.copy2(out/'region-diagnostic-implementation.md',folder/'implementation.md')
    (folder/'unaccepted.patch').write_bytes(subprocess.check_output(['git','diff','--binary'],cwd=repo))
    shutil.copy2(__file__,folder/'controller.py')
    (folder/'build-identity.json').write_text(json.dumps({'binary_sha256':sha(folder/'tantivy_worker'),'controller_sha256':sha(Path(__file__)),'source_sha256':sha(repo/'src/query/boolean_query/or_maxscore.rs'),'purpose':'Aggregate instrumentation only. Excluded from latency acceptance.'},indent=2)+'\n')
elif op=='run':
    proof=json.loads((folder/'build-identity.json').read_text())
    assert proof['binary_sha256']==sha(folder/'tantivy_worker')
    assert proof['controller_sha256']==sha(Path(__file__))
    assert json.loads((folder/'source-after.json').read_text())==snapshot()
    run=folder/'run';run.mkdir()
    before={'source':snapshot(),'binary':sha(folder/'tantivy_worker'),'queries':sha(balanced.QUERIES),'index':balanced.index_files(balanced.TINDEX)}
    (run/'before.json').write_text(json.dumps(before,indent=2)+'\n')
    cases=[q for q in balanced.CASES if q['kind']=='OR' and len(q['terms'])>=3 and 'port-style' in q['tags'] and 'correctness-only' not in q['tags']]
    assert len(cases)==103
    refs={r['case']['id']:r['tantivy'] for r in json.loads((out.parent/'lucene-rs-balanced-oct06/verify/dumps.json').read_text())}
    os.environ['OR_REGION_DIAGNOSTIC']='1';os.sched_setaffinity(0,{6})
    balanced.BINS['tantivy']=folder/'tantivy_worker'
    worker=balanced.Worker('tantivy',run,'diagnostic',deadline=time.monotonic()+1200)
    records=[];line_offset=0
    try:
        for case in cases:
            dump=worker.request({'op':'dump','id':case['id']},120)
            balanced.validate_dump(case,dump)
            assert dump['hits']==dump['oracle']['hits']==refs[case['id']]['hits']
            assert dump['count']==dump['oracle']['count']==refs[case['id']]['count']
            lines=(run/'diagnostic.stderr').read_text().splitlines()
            additions=lines[line_offset:];line_offset=len(lines)
            records.append({'case':case,'dump':dump,'diagnostic_lines':additions})
    finally:worker.close()
    (run/'records.json').write_text(json.dumps(records,indent=2)+'\n')
    after={'source':snapshot(),'binary':sha(folder/'tantivy_worker'),'queries':sha(balanced.QUERIES),'index':balanced.index_files(balanced.TINDEX)}
    (run/'after.json').write_text(json.dumps(after,indent=2)+'\n')
    assert before==after
    (run/'receipt.json').write_text(json.dumps({'pass':True,'queries':len(records),'artifacts_unchanged':before==after,'diagnostic_only':True,'record_sha256':sha(run/'records.json')},indent=2)+'\n')
    print('Completed 103 exact diagnostic dumps; no latency claim.')
else:raise ValueError(op)
