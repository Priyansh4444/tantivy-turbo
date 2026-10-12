"""Root-only bounded publication stages; outputs always outside the packet."""
import argparse,hashlib,json,subprocess,sys,time
from pathlib import Path
out=Path(__file__).resolve().parent
parser=argparse.ArgumentParser()
parser.add_argument('phase',choices=['assemble','audit','negative','clean'])
parser.add_argument('label')
parser.add_argument('packet',type=Path)
parser.add_argument('--revision')
args=parser.parse_args()
assert args.label and all(c.isalnum() or c=='-' for c in args.label)
packet=args.packet.resolve()
folder=out/(args.phase+'-'+args.label);folder.mkdir()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
if args.phase=='assemble':
 assert not packet.exists()
 command=[sys.executable,'-B',str(out/'portable-pack.py'),str(packet)]
 inputs=[out/'portable-pack.py',out/'portable-audit-draft/recompute.py',out/'portable-audit-draft/README.md',out/'decision.tsv',out/'results-current.md',out/'search-architecture-current.html']
elif args.phase=='audit':
 command=[sys.executable,'-B',str(packet/'recompute.py')]
 inputs=[packet/'packet-manifest.json',packet/'recompute.py']
elif args.phase=='negative':
 command=[sys.executable,'-B',str(out/'newportable-negative-probes.py'),str(packet)]
 inputs=[packet/'packet-manifest.json',packet/'recompute.py',out/'newportable-negative-probes.py']
else:
 assert args.revision
 repo=out.parents[1]/'tantivy-pr2937'
 command=[sys.executable,'-B',str(out/'verify-clean-publication.py'),str(repo),args.revision,str(packet.relative_to(repo))]
 inputs=[packet/'packet-manifest.json',out/'verify-clean-publication.py']
before={str(p):sha(p) for p in inputs}
(folder/'command.json').write_text(json.dumps(command,indent=2)+'\n')
(folder/'before.json').write_text(json.dumps(before,indent=2)+'\n')
start=time.monotonic()
with (folder/'stdout.json').open('w') as stdout,(folder/'stderr.log').open('w') as stderr:
 try:r=subprocess.run(command,cwd=out,stdout=stdout,stderr=stderr,timeout=1800);code=r.returncode
 except subprocess.TimeoutExpired:code=124
after={str(p):sha(p) for p in inputs}
(folder/'after.json').write_text(json.dumps(after,indent=2)+'\n')
receipt={'phase':args.phase,'returncode':code,'seconds':time.monotonic()-start,'inputs_unchanged':before==after,
 'stdout_sha256':sha(folder/'stdout.json'),'stderr_sha256':sha(folder/'stderr.log'),'command':command}
(folder/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt),flush=True)
assert before==after
if code:sys.exit(code)
result=json.loads((folder/'stdout.json').read_text())
if args.phase in ('audit','clean'):assert result['integrity_pass'] is True
if args.phase=='negative':assert result['negative_probes_pass'] is True
print(json.dumps({k:result[k] for k in ('files','stored_bytes','raw_bytes','manifest_sha256','integrity_pass','completed_attempts','accepted_attempts','rejected_attempts','latest_accepted_attempt','actual_port_target_met','negative_probes_pass','count','passed','commit','tracked_files','archive_sha256') if k in result}),flush=True)
