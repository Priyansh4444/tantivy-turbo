"""Derived arithmetic group self-controls; never changes the frozen sampler."""
import argparse
import collections
import hashlib
import json
import math
import statistics
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--folder', required=True, type=Path)
args = parser.parse_args()
folder = args.folder.resolve(strict=True)
queries = {q['id']: q for q in map(json.loads, (Path(__file__).parent.parent / 'lucene-rs-balanced-oct06/queries.jsonl').read_text().splitlines())}
summary = json.loads((folder / 'port-summary.json').read_text())
assert summary['pass']
samplefile = folder / 'port-measure/samples.jsonl'
assert hashlib.sha256(samplefile.read_bytes()).hexdigest() == summary['samples_sha256']
cells = collections.defaultdict(list)
for line in samplefile.read_text().splitlines():
    row = json.loads(line)
    cells[row['pair'], row['order'], row['id'], row['side']].extend(row['ns'])
groups = {}
for group in summary['summaries']:
    if group['set'] != 'port-style':
        continue
    kind = group['kind']
    ids = [qid for qid, q in queries.items() if 'port-style' in q['tags'] and
           (kind == 'ALL' or q['kind'] == kind or q['kind'] == 'OR' and
            ((kind == 'OR_2' and len(q['terms']) == 2) or (kind == 'OR_3PLUS' and len(q['terms']) >= 3)))]
    def ratio(pair, order):
        return (sum(statistics.median(cells[pair, order, qid, 'A']) for qid in ids) /
                sum(statistics.median(cells[pair, order, qid, 'B']) for qid in ids))
    primary = {order: ratio(0, order) for order in ('AB', 'BA')}
    controls = {order: ratio(1, order) for order in ('AB', 'BA')}
    noise = max(abs(math.log(value)) for value in controls.values())
    for order in ('AB', 'BA'):
        assert abs(primary[order] - group[order]['ratio_of_mean_query_medians_candidate_over_port']) < 1e-12
    groups[kind] = {'queries': len(ids), 'candidate_over_port': primary,
                    'arithmetic_port_self_control': controls, 'control_log_spread': noise,
                    'candidate_win_both_beyond_port_self_controls': all(math.log(value) < -noise for value in primary.values())}
receipt = {'groups': groups, 'actual_port_target_met': all(groups[kind]['candidate_win_both_beyond_port_self_controls'] for kind in ('OR', 'OR_3PLUS')),
           'controls_scope': 'Contemporaneous fixed-port self comparisons only. Candidate self comparisons are retained in the separate parent-comparison lane.',
           'samples_sha256': summary['samples_sha256'], 'controller_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
destination = folder / 'port-group-audit.json'
assert not destination.exists()
destination.write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps(receipt, indent=2))
