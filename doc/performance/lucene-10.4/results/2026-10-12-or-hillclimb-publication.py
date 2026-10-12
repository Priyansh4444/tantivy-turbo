"""Audit committed packet bytes from a clean Git archive, independently of checkout.

Root supplies repository, committed revision and packet-relative directory.
Standalone recompute.py itself requires neither Git nor engine binaries.
"""
import argparse,hashlib,json,subprocess,tarfile,tempfile
from pathlib import Path,PurePosixPath

parser=argparse.ArgumentParser()
parser.add_argument('repository',type=Path)
parser.add_argument('revision')
parser.add_argument('packet')
args=parser.parse_args()
packet=PurePosixPath(args.packet)
assert not packet.is_absolute() and all(p not in ('','..','.') for p in args.packet.split('/'))
assert str(packet)==args.packet and '\\' not in args.packet
commit=subprocess.check_output(['git','rev-parse',args.revision+'^{commit}'],cwd=args.repository,text=True).strip()
assert not subprocess.check_output(['git','status','--porcelain','--',args.packet],cwd=args.repository)
tracked=subprocess.check_output(['git','ls-tree','-r','--name-only',commit,'--',args.packet],cwd=args.repository,text=True).splitlines()
assert tracked and all(p.startswith(args.packet+'/') for p in tracked)
# The repository ignores Cargo.lock even inside retained evidence. Bind the
# committed inventory before extraction; a clean status alone cannot prove it.
manifest=json.loads(subprocess.check_output(['git','show',commit+':'+args.packet+'/packet-manifest.json'],cwd=args.repository))
expected={args.packet+'/'+name for name in manifest['files']}|{args.packet+'/packet-manifest.json'}
assert set(tracked)==expected, json.dumps({'missing_committed_files':sorted(expected-set(tracked)),
                                         'extra_committed_files':sorted(set(tracked)-expected)})
with tempfile.TemporaryDirectory(prefix='tantivy-packet-audit-') as temporary:
 base=Path(temporary);archive=base/'packet.tar'
 with archive.open('wb') as stream:
  subprocess.run(['git','archive','--format=tar',commit,args.packet],cwd=args.repository,stdout=stream,check=True)
 archive_sha=hashlib.sha256(archive.read_bytes()).hexdigest()
 with tarfile.open(archive) as tar:
  assert all(m.isfile() or m.isdir() for m in tar.getmembers())
  tar.extractall(base/'extracted',filter='data')
 extracted=base/'extracted'/args.packet
 result_path=base/'audit-result.json'
 with result_path.open('w') as stream:
  subprocess.run(['python3','-B',str(extracted/'recompute.py')],stdout=stream,check=True)
 result=json.loads(result_path.read_text())
 assert result['integrity_pass'] is True
 print(json.dumps({'commit':commit,'archive_sha256':archive_sha,'tracked_files':len(tracked),
  'integrity_pass':True,'completed_attempts':result['completed_attempts'],
  'latest_accepted_attempt':result['latest_accepted_attempt'],
  'actual_port_target_met':result['actual_port_target_met']},indent=2))
