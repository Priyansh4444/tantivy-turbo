"""tantivy-turbo vs upstream tantivy 0.27 on the same 1M Wikipedia docs and 1,201 queries.
Interleaved AB/BA rounds per query (as in the frozen pilot), TOP_10 and COUNT, plus RSS and overlap."""
import json, os, random, statistics, sys, time, math
H = '/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07/attempt-34'
sys.path.insert(0, H)
import balanced
from balanced import Worker, CASES
from pathlib import Path
OUT = Path(__file__).resolve().parent
balanced.BINS = {'turbo': Path(os.environ.get('TURBO_BIN', str(Path(H) / 'attempt-34-v8-tantivy_worker'))),
                 'upstream': OUT / 'upstream-harness/target/release/tantivy_worker'}
balanced.INDICES = {'turbo': Path(os.environ.get('TURBO_INDEX', str(balanced.TINDEX))), 'upstream': OUT / 'upstream-027.idx'}
os.sched_setaffinity(0, {6})
work = [q for q in CASES if 'port-style' in q['tags'] or 'wiki20' in q['tags']]
def run(w, q, mode, n):
    return w.request({'op': 'run', 'id': q['id'], 'mode': mode, 'iterations': n})['ns']
import subprocess
class UpstreamWorker(Worker):
    # Same protocol; upstream reports its own statistics (all 1M docs, its own token total),
    # so record them instead of asserting turbo's Lucene-aligned field statistics.
    def __init__(self, engine, folder, label, cpu=4, deadline=None):
        self.engine = engine; self.expected = None; self.deadline = deadline
        self.stderr = (folder / f'{label}.stderr').open('w'); self.log = (folder / f'{label}.protocol.jsonl').open('w')
        self.p = subprocess.Popen([str(balanced.BINS[engine]), str(balanced.INDICES[engine]), str(balanced.QUERIES)],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=self.stderr, text=True, bufsize=1,
            preexec_fn=lambda: os.sched_setaffinity(0, {cpu}))
        self.ready = self.read(120)
        assert self.ready['queries'] == len(CASES) and self.ready['num_docs'] == 1000000, self.ready
folder = OUT / 'run'; folder.mkdir(exist_ok=True)
workers = {'turbo': Worker('turbo', folder, 'turbo', deadline=time.time() + 7200),
           'upstream': UpstreamWorker('upstream', folder, 'upstream', deadline=time.time() + 7200)}
(folder / 'ready.json').write_text(json.dumps({e: w.ready for e, w in workers.items()}, indent=1))
raw = []; dumps = {}
try:
    for q in work:  # overlap check + warmup
        for e, w in workers.items():
            dumps.setdefault(q['id'], {})[e] = w.request({'op': 'dump', 'id': q['id']}, timeout=60)
            run(w, q, 'top10', 4); run(w, q, 'count', 2)
    for order in ('AB', 'BA'):
        cases = work.copy(); random.Random(20261017).shuffle(cases)
        for q in cases:
            for mode in ('top10', 'count'):
                for side in order:
                    e = 'turbo' if side == 'A' else 'upstream'
                    raw.append({'order': order, 'engine': e, 'mode': mode, 'id': q['id'], 'ns': run(workers[e], q, mode, 12)})
    rss = {e: w.rss() for e, w in workers.items()}
finally:
    for w in workers.values(): w.close()
(folder / 'samples.json').write_text(json.dumps(raw))
(folder / 'dumps.json').write_text(json.dumps(dumps))
med = {(r['order'], r['engine'], r['mode'], r['id']): statistics.median(r['ns']) for r in raw}
qs = {q['id']: q for q in work}
def group(q):
    if 'wiki20' in q['tags'] and 'port-style' not in q['tags']: return 'wiki20'
    if q['kind'] == 'OR': return 'OR3+' if len(q['terms']) >= 3 else 'OR2'
    return q['kind']
summary = {}
for mode in ('top10', 'count'):
    for g in ('ALL', 'TERM', 'AND', 'OR2', 'OR3+', 'PHRASE', 'wiki20'):
        ids = [i for i in qs if (g == 'ALL' and 'port-style' in qs[i]['tags']) or group(qs[i]) == g]
        if not ids: continue
        res = {}
        for o in ('AB', 'BA'):
            t = sum(med[o, 'turbo', mode, i] for i in ids); u = sum(med[o, 'upstream', mode, i] for i in ids)
            gm = math.exp(statistics.mean(math.log(med[o, 'turbo', mode, i] / med[o, 'upstream', mode, i]) for i in ids))
            wins = sum(med[o, 'turbo', mode, i] < med[o, 'upstream', mode, i] for i in ids)
            res[o] = {'turbo_mean_us': t / len(ids) / 1e3, 'upstream_mean_us': u / len(ids) / 1e3, 'sum_ratio': t / u, 'geomean_ratio': gm, 'turbo_faster': wins}
        summary[f'{mode}/{g}'] = {'queries': len(ids), **res}
overlap = []
for i, d in dumps.items():
    a = {h[0] for h in d['turbo']['hits']}; b = {h[0] for h in d['upstream']['hits']}
    overlap.append(len(a & b) / max(1, max(len(a), len(b))))
summary['top10_overlap_mean'] = statistics.mean(overlap)
summary['count_equal'] = sum(d['turbo']['count'] == d['upstream']['count'] for d in dumps.values()) / len(dumps)
summary['rss'] = rss
(OUT / 'summary.json').write_text(json.dumps(summary, indent=1))
print(json.dumps(summary, indent=1))
