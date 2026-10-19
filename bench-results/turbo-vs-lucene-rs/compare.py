"""tantivy-turbo (v8) vs lucene-rs (Rust port of Lucene 10) on the same 1M Wikipedia docs and 1,201+20 queries.
Both engines return the exact same top-10 IDs/scores and counts (verified against frozen dumps on every timed call).
Interleaved AB/BA rounds per query, TOP_10 and COUNT, plus RSS."""
import json, os, random, statistics, sys, time, math
H = '/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07/attempt-34'
sys.path.insert(0, H)
import balanced
from balanced import Worker, CASES, native_run
from pathlib import Path
OUT = Path(__file__).resolve().parent
balanced.BINS = {'tantivy': Path(H) / 'attempt-34-v8-tantivy_worker',
                 'port': Path(H).parent.parent / 'lucene-rs-balanced-oct06/target/release/port_worker'}
balanced.INDICES = {'tantivy': balanced.TINDEX, 'port': balanced.PINDEX}
os.sched_setaffinity(0, {6})
proof = {r['case']['id']: r for r in json.loads((Path(H).parent.parent / 'lucene-rs-balanced-oct06/verify/dumps.json').read_text())}
work = [q for q in CASES if 'correctness-only' not in q['tags']]
folder = OUT / 'run'; folder.mkdir(exist_ok=True)
workers = {}
for e in ('tantivy', 'port'):
    w = Worker(e, folder, e, deadline=time.time() + 7200); w.expected = {i: r[e] for i, r in proof.items()}; workers[e] = w
raw = []
try:
    for q in work:
        for w in workers.values():
            native_run(w, q, 'top10', 4); native_run(w, q, 'count', 2)
    for order in ('AB', 'BA'):
        cases = work.copy(); random.Random(20261018).shuffle(cases)
        for q in cases:
            for mode in ('top10', 'count'):
                for side in order:
                    e = 'tantivy' if side == 'A' else 'port'
                    raw.append({'order': order, 'engine': e, 'mode': mode, 'id': q['id'], 'ns': native_run(workers[e], q, mode, 12)['ns']})
    rss = {e: w.rss() for e, w in workers.items()}
finally:
    for w in workers.values(): w.close()
(folder / 'samples.json').write_text(json.dumps(raw))
med = {(r['order'], r['engine'], r['mode'], r['id']): statistics.median(r['ns']) for r in raw}
qs = {q['id']: q for q in work}
def group(q):
    if 'wiki20' in q['tags'] and 'port-style' not in q['tags']: return 'wiki20'
    if q['kind'] == 'OR': return 'OR3+' if len(q['terms']) >= 3 else 'OR2'
    return q['kind']
summary = {}
per_query = []
for mode in ('top10', 'count'):
    for g in ('ALL', 'TERM', 'AND', 'OR2', 'OR3+', 'PHRASE', 'wiki20'):
        ids = [i for i in qs if (g == 'ALL' and 'port-style' in qs[i]['tags']) or group(qs[i]) == g]
        res = {}
        for o in ('AB', 'BA'):
            t = sum(med[o, 'tantivy', mode, i] for i in ids); p = sum(med[o, 'port', mode, i] for i in ids)
            gm = math.exp(statistics.mean(math.log(med[o, 'tantivy', mode, i] / med[o, 'port', mode, i]) for i in ids))
            wins = sum(med[o, 'tantivy', mode, i] < med[o, 'port', mode, i] for i in ids)
            res[o] = {'turbo_mean_us': t / len(ids) / 1e3, 'port_mean_us': p / len(ids) / 1e3, 'sum_ratio': t / p, 'geomean_ratio': gm, 'turbo_faster': wins}
        summary[f'{mode}/{g}'] = {'queries': len(ids), **res}
for i in qs:
    for mode in ('top10', 'count'):
        per_query.append({'id': i, 'group': group(qs[i]), 'terms': ' '.join(qs[i]['terms']), 'mode': mode,
                          **{f'{e}_{o}_ns': med[o, e, mode, i] for e in ('tantivy', 'port') for o in ('AB', 'BA')}})
summary['rss'] = rss
(OUT / 'summary.json').write_text(json.dumps(summary, indent=1))
(OUT / 'per-query.json').write_text(json.dumps(per_query))
print(json.dumps(summary, indent=1))
