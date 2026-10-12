"""Select an immutable parent artifact; reuse the frozen sampler and statistics."""
import argparse
import json
import os
import runpy
from pathlib import Path
import balanced
from stage import OUT, sha

parser = argparse.ArgumentParser()
parser.add_argument('operation', choices=('verify', 'configured', 'pilot', 'measure'))
parser.add_argument('--parent', required=True, type=Path)
args = parser.parse_args()
descriptor_path = args.parent.resolve(strict=True)
descriptor = json.loads(descriptor_path.read_text())
binary = Path(descriptor['binary']).resolve(strict=True)
if sha(binary) != descriptor['binary_sha256']:
    raise RuntimeError('parent binary differs from accepted descriptor')
for field in ('compiled_sources', 'exact_gate'):
    if sha(Path(descriptor[field])) != descriptor[field + '_sha256']:
        raise RuntimeError(f'parent proof changed: {field}')
if not json.loads(Path(descriptor['exact_gate']).read_text())['pass']:
    raise RuntimeError('parent must have passed its exact gate')
balanced.BINS['port'] = binary
original_identity = balanced.identity
def identity():
    return {**original_identity(),
            'comparison_controller_sha256': sha(Path(__file__)),
            'selected_parent': {'descriptor': str(descriptor_path),
                                'descriptor_sha256': sha(descriptor_path),
                                'source_commit': descriptor['source_commit'],
                                'compiled_sources_sha256': sha(Path(descriptor['compiled_sources'])),
                                'exact_gate_sha256': sha(Path(descriptor['exact_gate']))}}
balanced.identity = identity
os.sched_setaffinity(0, {6})
if args.operation in ('verify', 'measure'):
    folder = OUT / args.operation
    folder.mkdir()
    try:
        getattr(balanced, args.operation)(folder)
    except BaseException as error:
        (folder / 'error.json').write_text(json.dumps({'type': type(error).__name__, 'message': str(error)}, indent=2) + '\n')
        raise
else:
    runpy.run_path(str(OUT / ('configured_gate.py' if args.operation == 'configured' else 'pilot.py')), run_name='__main__')
