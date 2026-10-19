"""Render the README's charts (light and dark SVG variants) from measured data.

Inputs
  turbo-vs-tantivy-0.27/summary.json   measured by turbo-vs-tantivy-0.27/compare.py
  turbo-vs-lucene-rs/summary.json      measured by turbo-vs-lucene-rs/compare.py
  doc/performance/lucene-10.4/results/2026-10-04-native/*.json   published Lucene 10.4 report

Outputs: assets/*-{light,dark}.svg. Fills use 4x4 ordered (Bayer) dithering.
"""
import json
import math
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
OUT = ROOT / "assets"
LUCENE = ROOT / "doc/performance/lucene-10.4/results/2026-10-04-native"

SANS = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif"
MONO = "ui-monospace, SFMono-Regular, Menlo, Consolas, 'Liberation Mono', monospace"
THEMES = {
    "dark": dict(bg="#0d1117", card="#11161d", ink="#e6edf3", muted="#8b949e", grid="#21262d",
                 axis="#3d444d", fast="#b8e550", fast2="#4fc3b0", slow="#f08a7e", neutral="#6e7681"),
    "light": dict(bg="#ffffff", card="#f6f8fa", ink="#1f2328", muted="#59636e", grid="#e6e9ec",
                  axis="#b7bdc4", fast="#4d7c0f", fast2="#0f766e", slow="#c2410c", neutral="#8c959f"),
}
BAYER = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]
LEVELS = (6, 10, 13, 16)


def defs(t):
    out = []
    for name in ("fast", "fast2", "slow", "neutral", "ink"):
        for level in LEVELS:
            cells = "".join(f'<rect x="{x*2}" y="{y*2}" width="2" height="2" fill="{t[name]}"/>'
                            for y in range(4) for x in range(4) if BAYER[y][x] < level)
            out.append(f'<pattern id="{name}{level}" width="8" height="8" patternUnits="userSpaceOnUse">{cells}</pattern>')
    return "<defs>" + "".join(out) + "</defs>"


def esc(s):
    return str(s).replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


class Svg:
    def __init__(self, t, w, h):
        self.t, self.w, self.h, self.parts = t, w, h, []

    def add(self, s):
        self.parts.append(s)

    def text(self, x, y, s, size=14, fill=None, anchor="start", weight=400, mono=False, opacity=1):
        self.add(f'<text x="{x:.1f}" y="{y:.1f}" font-size="{size}" font-weight="{weight}" fill="{fill or self.t["ink"]}" '
                 f'text-anchor="{anchor}" font-family="{MONO if mono else SANS}" opacity="{opacity}">{esc(s)}</text>')

    def header(self, title, subtitle):
        self.text(36, 50, title, 22, weight=650)
        self.text(36, 76, subtitle, 13.5, self.t["muted"])

    def save(self, name):
        body = (f'<svg xmlns="http://www.w3.org/2000/svg" width="{self.w}" height="{self.h}" viewBox="0 0 {self.w} {self.h}">'
                f'{defs(self.t)}<rect x="0.5" y="0.5" width="{self.w-1}" height="{self.h-1}" rx="16" fill="{self.t["card"]}" '
                f'stroke="{self.t["grid"]}"/>' + "".join(self.parts) + "</svg>")
        (OUT / name).write_text(body)


def legend(s, x, y, items):
    for label, fill in items:
        s.add(f'<rect x="{x}" y="{y-10}" width="14" height="12" rx="2" fill="{fill}"/>')
        s.text(x + 20, y, label, 12.5, s.t["muted"])
        x += 34 + 7.2 * len(label)


def diverging(theme, name, title, subtitle, rows, series, note, lo=0.5, hi=4.0, ticks=(0.5, 1, 2, 4)):
    """Log2 axis centred on 1x: faster to the right, slower to the left.
    rows: [(label, sublabel, [time ratio per series])], ratio = candidate time / baseline time."""
    t = THEMES[theme]
    per = 24
    row_h = per * len(series) + 22
    left, right, top = 250, 110, 128
    w = 900
    h = top + row_h * len(rows) + 84
    s = Svg(t, w, h)
    s.header(title, subtitle)
    legend(s, 36, 108, [(lab, f"url(#{pat}16)") for lab, pat in series] + [("slower", "url(#slow13)")])
    plot = w - left - right
    clamp = lambda v: min(max(v, lo), hi)
    lx = lambda r: left + (math.log2(clamp(1 / r)) - math.log2(lo)) / (math.log2(hi) - math.log2(lo)) * plot
    x1 = lx(1.0)
    for v in ticks:
        if lo <= v <= hi:
            x = lx(1 / v)
            s.add(f'<line x1="{x:.1f}" y1="{top-6}" x2="{x:.1f}" y2="{h-66}" stroke="{t["axis"] if v == 1 else t["grid"]}" '
                  f'stroke-width="{1.4 if v == 1 else 1}"/>')
            lab = "same" if v == 1 else (f"{v:g}x faster" if v > 1 else f"{1/v:g}x slower")
            s.text(x, h - 46, lab, 12, t["muted"], "middle")
    for i, (label, sub, values) in enumerate(rows):
        y0 = top + i * row_h
        if i % 2 == 0:
            s.add(f'<rect x="16" y="{y0-2}" width="{w-32}" height="{row_h-6}" rx="8" fill="{t["bg"]}" opacity="0.55"/>')
        mid = y0 + (row_h - 6) / 2
        s.text(left - 18, mid - (2 if sub else -4), label, 14.5, anchor="end", weight=550)
        if sub:
            s.text(left - 18, mid + 15, sub, 11.5, t["muted"], "end")
        for j, ((_, pat), r) in enumerate(zip(series, values)):
            y = y0 + 9 + j * per
            x = lx(r)
            slower = r > 1.01
            same = abs(r - 1) <= 0.01
            fill = "url(#slow13)" if slower else f"url(#{pat}16)"
            a, b = sorted((x1, x))
            s.add(f'<rect x="{a:.1f}" y="{y}" width="{max(b-a, 2):.1f}" height="16" rx="2" fill="{fill}"/>')
            fmt = (lambda v: f"{v:.0f}x" if v >= 20 else (f"{v:.1f}x" if v >= 10 else f"{v:.2f}x"))
            txt = "about the same" if same else (fmt(1 / r) if not slower else f"{r:.2f}x slower")
            if slower and a - 8 - 7.6 * len(txt) < left - 6:
                s.text(x1 + 8, y + 12.5, txt, 12.5, t["slow"], mono=True)
            elif slower:
                s.text(a - 8, y + 12.5, txt, 12.5, t["slow"], "end", mono=True)
            else:
                s.text(b + 8, y + 12.5, txt, 12.5, t["ink"], mono=True, weight=600)
    s.text(w - 36, h - 18, note, 11.5, t["muted"], "end")
    s.save(f"{name}-{theme}.svg")


def lucene_per_query(theme):
    t = THEMES[theme]
    top10 = json.loads((LUCENE / "top_10-tantivy-first.json").read_text())
    top10b = json.loads((LUCENE / "top_10-lucene-first.json").read_text())
    count = json.loads((LUCENE / "count-tantivy-first.json").read_text())
    countb = json.loads((LUCENE / "count-lucene-first.json").read_text())
    queries = top10["queries"]

    def speed(d1, d2, q):
        r1 = d1["medians_us"]["lucene"][q] / d1["medians_us"]["tantivy"][q]
        r2 = d2["medians_us"]["lucene"][q] / d2["medians_us"]["tantivy"][q]
        return math.sqrt(r1 * r2)

    rows = sorted(((q, speed(top10, top10b, q), speed(count, countb, q)) for q in queries), key=lambda r: -r[1])
    left, right, top, row = 230, 60, 128, 26
    w = 900
    h = top + row * len(rows) + 80
    s = Svg(t, w, h)
    s.header("Every query, vs Apache Lucene 10.4",
             "Per-query speedup (Lucene median / fork median), geometric mean of both process orders.")
    legend(s, 36, 108, [("TOP-10", t["fast"]), ("COUNT", t["fast2"])])
    lo, hi = 0.8, 8.0
    plot = w - left - right
    sx = lambda v: left + (math.log2(v) - math.log2(lo)) / (math.log2(hi) - math.log2(lo)) * plot
    for v in (1, 2, 4, 8):
        x = sx(v)
        s.add(f'<line x1="{x:.1f}" y1="{top-8}" x2="{x:.1f}" y2="{h-64}" stroke="{t["axis"] if v == 1 else t["grid"]}" '
              f'stroke-width="{1.4 if v == 1 else 1}"/>')
        s.text(x, h - 44, "same" if v == 1 else f"{v}x faster", 12, t["muted"], "middle")
    for i, (q, a, c) in enumerate(rows):
        y = top + i * row + 8
        if i % 2 == 0:
            s.add(f'<rect x="16" y="{y-12}" width="{w-32}" height="{row}" rx="6" fill="{t["bg"]}" opacity="0.55"/>')
        s.text(left - 16, y + 5, q, 13, anchor="end", mono=True)
        xa, xc = sx(a), sx(c)
        s.add(f'<line x1="{min(xa, xc):.1f}" y1="{y}" x2="{max(xa, xc):.1f}" y2="{y}" stroke="{t["axis"]}" stroke-width="2"/>')
        s.add(f'<circle cx="{xc:.1f}" cy="{y}" r="6" fill="{t["fast2"]}" stroke="{t["card"]}" stroke-width="2"/>')
        s.add(f'<circle cx="{xa:.1f}" cy="{y}" r="6.5" fill="{t["fast"]}" stroke="{t["card"]}" stroke-width="2"/>')
    s.text(w - 36, h - 18, "Wikipedia 1M, native BM25 on both engines; report 2026-10-04-native", 11.5, t["muted"], "end")
    s.save(f"lucene-per-query-{theme}.svg")


def resources(theme, lucene_rs_rss_mib):
    t = THEMES[theme]
    mem = json.loads((LUCENE / "memory.json").read_text())["engines"]
    storage = json.loads((LUCENE / "storage.json").read_text())
    kib = lambda v: int(v.split()[0]) / 1024
    groups = [
        ("Resident memory, warmed query process", "MiB", [
            ("tantivy-turbo", kib(mem["tantivy"]["status"]["VmRSS"]), "fast"),
            ("Lucene 10.4 (default JVM heap)", kib(mem["lucene"]["status"]["VmRSS"]), "slow"),
        ]),
        ("Index size on disk", "MB", [
            ("tantivy-turbo", storage["tantivy"]["total_bytes"] / 1e6, "fast"),
            ("Lucene 10.4", storage["lucene"]["total_bytes"] / 1e6, "neutral"),
        ]),
    ]
    w, left, right = 900, 270, 120
    h = 122 + sum(40 + 34 * len(g[2]) for g in groups) + 6
    s = Svg(t, w, h)
    s.header("Memory and disk vs Apache Lucene 10.4", "Same Wikipedia 1M corpus and 20 queries; RSS includes mapped index pages.")
    y = 122
    for title, unit, bars in groups:
        s.text(36, y + 4, title, 14.5, weight=600)
        y += 22
        vmax = max(v for _, v, _ in bars)
        for label, v, pat in bars:
            s.text(left - 16, y + 13, label, 13.5, anchor="end")
            width = max(v / vmax * (w - left - right), 3)
            s.add(f'<rect x="{left}" y="{y}" width="{width:.1f}" height="18" rx="2" fill="url(#{pat}16)"/>')
            s.text(left + width + 8, y + 14, f"{v:,.1f} {unit}", 13, mono=True, weight=600)
            y += 34
        y += 18
    s.save(f"resources-{theme}.svg")


def regions(theme):
    """Schematic of the attempt-34 change on q920 (21 terms): fine regions vs adaptive coarse regions."""
    t = THEMES[theme]
    w, h = 900, 330
    s = Svg(t, w, h)
    s.header("How the OR speedup works", "MaxScore splits the doc-ID space into regions and re-derives score bounds per region.")
    x0, x1 = 36, w - 36
    rows = [("Tantivy 0.27", "30,242 regions on q920: every term's 128-doc block ends one", 220, "neutral", 10),
            ("tantivy-turbo", "362 regions: wide ranges, bounds from cached skip-data maxima", 70, "fast", 16)]
    y = 112
    for label, sub, n, pat, level in rows:
        s.text(36, y + 4, label, 14.5, weight=600)
        s.text(36 + 8.6 * len(label) + 12, y + 4, sub, 12.5, t["muted"])
        y += 16
        if n > 100:
            step = (x1 - x0) / n
            for i in range(n):
                s.add(f'<rect x="{x0 + i*step:.2f}" y="{y}" width="{max(step-1.2, 0.8):.2f}" height="56" '
                      f'fill="url(#{pat}{LEVELS[i*7 % 4]})"/>')
        else:
            edges = [0, 0.11, 0.2, 0.34, 0.43, 0.58, 0.66, 0.79, 0.88, 1.0]
            for i in range(len(edges) - 1):
                a = x0 + edges[i] * (x1 - x0)
                b = x0 + edges[i + 1] * (x1 - x0)
                fill = f"url(#{'fast' if i % 2 == 0 else 'fast2'}16)"
                s.add(f'<rect x="{a:.1f}" y="{y}" width="{b-a-4:.1f}" height="56" rx="3" fill="{fill}"/>')
        y += 84
    s.text(36, h - 26, "Switched on per query only when region bookkeeping outweighs candidate work; results stay bit-identical.",
           12.5, t["muted"])
    s.save(f"regions-{theme}.svg")


def banner(theme, stats):
    t = THEMES[theme]
    w, h = 900, 250
    s = Svg(t, w, h)
    # Dithered gradient strip: coverage steps left to right.
    cols = 36
    for i in range(cols):
        lvl = LEVELS[1 + min(2, i * 3 // cols)]  # denser levels only: the sparsest reads as crosses
        s.add(f'<rect x="{36 + i*(w-72)/cols:.1f}" y="24" width="{(w-72)/cols - 3:.1f}" height="6" rx="1" fill="url(#fast{lvl})"/>')
    s.text(36, 92, "tantivy-turbo", 44, weight=700)
    s.text(36, 124, "A faster Tantivy. Same API, same results, Lucene-equivalent scoring.", 16, t["muted"])
    tile_w = (w - 72 - 3 * 14) / 4
    for i, (value, label) in enumerate(stats):
        x = 36 + i * (tile_w + 14)
        s.add(f'<rect x="{x:.1f}" y="148" width="{tile_w:.1f}" height="76" rx="10" fill="{t["bg"]}" stroke="{t["grid"]}"/>')
        s.text(x + 16, 184, value, 26, t["fast"], weight=700, mono=True)
        s.text(x + 16, 208, label, 12.5, t["muted"])
    s.save(f"banner-{theme}.svg")


def ratio(entry):
    return math.sqrt(entry["AB"]["sum_ratio"] * entry["BA"]["sum_ratio"])


def main():
    OUT.mkdir(exist_ok=True)
    up = json.loads((HERE / "turbo-vs-tantivy-0.27/summary.json").read_text())
    rs = json.loads((HERE / "turbo-vs-lucene-rs/summary.json").read_text())
    groups = [("OR, 3+ terms", "103 queries", "OR3+"), ("OR, 2 terms", "198 queries", "OR2"), ("Phrase", "300 queries", "PHRASE"),
              ("Single term", "300 queries", "TERM"), ("AND", "300 queries", "AND"), ("All 1,201 queries", "port-style suite", "ALL")]
    series = [("TOP-10", "fast"), ("COUNT", "fast2")]
    rss = rs["rss"]["port"]["VmHWM"]
    for theme in THEMES:
        diverging(theme, "vs-tantivy", "tantivy-turbo vs Tantivy 0.27",
                  "Summed query time, mean of both run orders. Same 1M Wikipedia docs, same machine, identical match counts.",
                  [(a, b, [ratio(up[f"top10/{g}"]), ratio(up[f"count/{g}"])]) for a, b, g in groups], series,
                  "bench-results/turbo-vs-tantivy-0.27")
        diverging(theme, "vs-lucene-rs", "TOP-10 vs lucene-rs (Rust port of Lucene 10)",
                  "Summed query time, mean of both run orders. Identical top-10 IDs, score bits and counts on every query.",
                  [(a, b, [ratio(rs[f"top10/{g}"])]) for a, b, g in groups], [("TOP-10", "fast")],
                  "bench-results/turbo-vs-lucene-rs")
        diverging(theme, "vs-lucene-rs-count", "COUNT vs lucene-rs",
                  "lucene-rs counts by visiting every match; tantivy-turbo uses doc frequencies and SIMD-style counting lanes.",
                  [(a, b, [ratio(rs[f"count/{g}"])]) for a, b, g in groups], [("COUNT", "fast2")],
                  "log scale; bench-results/turbo-vs-lucene-rs", lo=0.5, hi=512, ticks=(1, 4, 16, 64, 256))
        lucene_per_query(theme)
        resources(theme, rss)
        regions(theme)
        banner(theme, [("2x", "faster OR vs Tantivy 0.27"), ("1.36x", "faster top-10, all queries"),
                       ("~2x", "faster than Lucene 10.4"), ("0", "changed results")])
    print("charts written:", sorted(p.name for p in OUT.glob("*.svg")))


if __name__ == "__main__":
    main()
