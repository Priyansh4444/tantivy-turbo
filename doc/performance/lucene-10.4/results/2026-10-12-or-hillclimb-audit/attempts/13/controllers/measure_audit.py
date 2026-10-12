"""Derived arithmetic group controls; leaves the frozen sampler unchanged."""
import argparse
import collections
import csv
import hashlib
import json
import math
import statistics
import subprocess
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--parent', required=True, type=Path)
parser.add_argument('--attempt', required=True)
args = parser.parse_args()
out = Path(__file__).resolve().parent
parent = json.loads(args.parent.read_text())
queries = {q['id']: q for q in map(json.loads, (out.parent / 'lucene-rs-balanced-oct06/queries.jsonl').read_text().splitlines())}
cells = collections.defaultdict(list)
rounds = collections.defaultdict(list)
with (out / 'measure/samples.jsonl').open() as stream:
    for line in stream:
        row = json.loads(line)
        if row['mode'] == 'top10':
            key = (row['pair'], row['order'], row['id'], row['side'])
            cells[key].extend(row['ns'])
            rounds[(row['round'], *key)].extend(row['ns'])
groups = {}
for kind in ('ALL', 'OR', 'OR_2', 'OR_3PLUS', 'AND', 'TERM', 'PHRASE'):
    ids = [qid for qid, q in queries.items() if 'port-style' in q['tags'] and
           (kind == 'ALL' or q['kind'] == kind or q['kind'] == 'OR' and
            ((kind == 'OR_2' and len(q['terms']) == 2) or (kind == 'OR_3PLUS' and len(q['terms']) >= 3)))]
    def ratio(data, prefix, pair, order):
        return (sum(statistics.median(data[(*prefix, pair, order, qid, 'A')]) for qid in ids) /
                sum(statistics.median(data[(*prefix, pair, order, qid, 'B')]) for qid in ids))
    primary = {order: ratio(cells, (), 0, order) for order in ('AB', 'BA')}
    controls = {label: {order: ratio(cells, (), pair, order) for order in ('AB', 'BA')}
                for pair, label in ((1, 'TT'), (2, 'PP'))}
    noise = max(abs(math.log(value)) for orders in controls.values() for value in orders.values())
    groups[kind] = {
        'queries': len(ids), 'primary': primary, 'controls': controls,
        'control_log_spread': noise,
        'candidate_win_both_beyond_group_controls': all(math.log(value) < -noise for value in primary.values()),
        'parent_win_both_beyond_group_controls': all(math.log(value) > noise for value in primary.values()),
        'rounds': [{'round': rnd, 'order': order, 'primary': ratio(rounds, (rnd,), 0, order),
                    'TT': ratio(rounds, (rnd,), 1, order), 'PP': ratio(rounds, (rnd,), 2, order)}
                   for rnd in range(3) for order in ('AB', 'BA')],
    }
summary = json.loads((out / 'summary.json').read_text())
for group in summary['summaries']:
    if group['set'] == 'port-style' and group['mode'] == 'top10':
        for order in ('AB', 'BA'):
            assert abs(groups[group['kind']]['primary'][order] - group[order]['ratio_of_mean_query_medians']) < 1e-12
negative = []
for row in csv.DictReader((out / 'per-query.csv').open()):
    if row['mode'] == 'top10' and row['P_wins_beyond_controls'] == 'True':
        negative.append({key: row[key] for key in ('id', 'kind', 'terms', 'T_over_P_AB', 'T_over_P_BA', 'control_log_spread')})
candidate = out / 'target/release/tantivy_worker'
baseline = Path(parent['binary'])
digest = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
assert digest(baseline) == parent['binary_sha256']
def sections(path):
    rows = subprocess.check_output(['size', '-A', str(path)], text=True).splitlines()
    return {r.split()[0]: int(r.split()[1]) for r in rows if r.strip().startswith(('.text ', '.rodata '))}
result = {'parent': parent, 'groups': groups, 'regressions_beyond_per_query_controls': negative,
          'efficiency': {'candidate_file_bytes': candidate.stat().st_size, 'baseline_file_bytes': baseline.stat().st_size,
                         'candidate_sections': sections(candidate), 'baseline_sections': sections(baseline),
                         'candidate_binary_sha256': digest(candidate), 'baseline_binary_sha256': digest(baseline),
                         'index_change': False, 'per_region_allocation': False}}
(out / f'attempt-{args.attempt}-metrics.json').write_text(json.dumps(result, indent=2) + '\n')
for kind in ('OR_2', 'OR_3PLUS', 'AND', 'PHRASE'):
    print(kind, json.dumps(groups[kind]))
print('Regressions beyond per-query controls:', len(negative))
print('File byte delta:', candidate.stat().st_size - baseline.stat().st_size)
