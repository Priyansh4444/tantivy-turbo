"""Root-only diagnostic profile with explicit cases and unique output labels."""
import argparse
import json
import os
import re
import select
import subprocess
import time
from pathlib import Path
from stage import OUT, ROOT, sha

parser = argparse.ArgumentParser()
parser.add_argument('--label', required=True)
parser.add_argument('--binary', required=True, type=Path)
parser.add_argument('--cases', required=True, type=Path)
args = parser.parse_args()
if not re.fullmatch(r'[a-z0-9-]+', args.label):
    parser.error('label must contain only lowercase letters, digits, and hyphens')
paths = {suffix: OUT / f'{args.label}-{suffix}' for suffix in
         ('perf.data', 'perf-report.txt', 'perf.stderr', 'profile-receipt.json', 'profile-replies.json', 'controller.py')}
if any(path.exists() for path in paths.values()):
    raise FileExistsError('Profile label already exists; use a new label.')
paths['controller.py'].write_bytes(Path(__file__).read_bytes())
old = OUT.parent / 'lucene-rs-balanced-oct06'
casefile = args.cases.resolve(strict=True)
cases = json.loads(casefile.read_text())
proof = {r['case']['id']: r['tantivy']['hits'] for r in json.loads((old / 'verify/dumps.json').read_text())}
binary = args.binary.resolve(strict=True)
binary_before = sha(binary)
query_before = sha(old / 'queries.jsonl')
cases_before = sha(casefile)
command = ['perf', 'record', '-e', 'cycles:u', '-F', '499', '-o', str(paths['perf.data']), '--',
           str(binary), str(ROOT / 'bench/wiki-1m-native-aligned-v11-oct05.idx'), str(old / 'queries.jsonl')]
os.sched_setaffinity(0, {6})
start = time.monotonic()
replies = []
with paths['perf.stderr'].open('x') as stderr:
    worker = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                              text=True, bufsize=1, preexec_fn=lambda: os.sched_setaffinity(0, {4}))
    def read():
        if not select.select([worker.stdout], [], [], 60)[0]:
            raise TimeoutError('profile worker')
        line = worker.stdout.readline()
        if not line:
            raise RuntimeError('perf worker exited')
        value = json.loads(line)
        replies.append(value)
        return value
    def run(case, iterations):
        worker.stdin.write(json.dumps({'op': 'run', 'id': case['id'], 'mode': 'top10', 'iterations': iterations}) + '\n')
        worker.stdin.flush()
        value = read()
        expected = sum(doc ^ bits for doc, bits in proof[case['id']]) * iterations % 2**64
        if value['checksum'] != expected:
            raise RuntimeError('profile checksum')
    try:
        read()
        for case in cases:
            run(case, 10)
        for _ in range(3):
            for case in cases:
                run(case, 100)
        worker.stdin.close()
        returncode = worker.wait(timeout=10)
    finally:
        if worker.poll() is None:
            worker.terminate()
            worker.wait(timeout=10)
paths['profile-replies.json'].write_text(json.dumps(replies, indent=2) + '\n')
receipt = {'command': command, 'returncode': returncode, 'seconds': time.monotonic() - start,
           'binary_sha256': binary_before, 'binary_unchanged': binary_before == sha(binary),
           'script_sha256': sha(paths['controller.py']), 'cases_sha256': cases_before,
           'query_sha256': query_before, 'inputs_unchanged': query_before == sha(old / 'queries.jsonl') and cases_before == sha(casefile),
           'correctness_checksums_match': True, 'cases': cases,
           'first_preparation_and_warmup_included_in_trace': True,
           'latency_acceptance': False, 'purpose': 'diagnostic cycles only; sequential profiles under existing desktop load'}
paths['profile-receipt.json'].write_text(json.dumps(receipt, indent=2) + '\n')
if returncode or not receipt['binary_unchanged'] or not receipt['inputs_unchanged']:
    raise RuntimeError('profile failure or changed inputs')
report = subprocess.check_output(['perf', 'report', '--stdio', '-i', str(paths['perf.data']), '--no-children',
                                  '--sort', 'symbol', '--percent-limit', '0.5'], text=True, stderr=subprocess.PIPE)
paths['perf-report.txt'].write_text(report)
print(json.dumps({key: value for key, value in receipt.items() if key != 'cases'}))
print(report[:6000])
