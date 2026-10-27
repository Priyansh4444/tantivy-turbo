"""Render the README's charts and diagrams (light and dark SVG variants) from measured data.

Inputs
  turbo-vs-tantivy-0.27/summary.json   measured by turbo-vs-tantivy-0.27/compare.py
  turbo-vs-lucene-rs/summary.json      measured by turbo-vs-lucene-rs/compare.py
  doc/performance/lucene-10.4/results/2026-10-04-native/*.json   published Lucene 10.4 report

Outputs: assets/*-{light,dark}.svg.

Style: series colors are categorical slots 1 and 3 of a palette validated for
color-vision deficiency in both modes (blue = TOP-10, aqua = COUNT); "slower" is
the red pole of a blue/red diverging pair. Text always uses text tokens, never a
series color. Bars are at most 16px thick with a 4px rounded data end and a
square baseline; grids are 1px hairlines.
"""
import json
import math
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
OUT = ROOT / "assets"
LUCENE = ROOT / "doc/performance/lucene-10.4/results/2026-10-04-native"

SANS = "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans', Helvetica, Arial, sans-serif"
MONO = "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, 'Liberation Mono', monospace"
THEMES = {
    "light": dict(surface="#fcfcfb", band="#f4f3f0", border="#e4e2dd", grid="#e7e5e0", axis="#a9a7a0",
                  ink="#0b0b0b", ink2="#52514e", muted="#696863",
                  top10="#2a78d6", count="#1baf7a", slow="#e34948", neutral="#a9a7a0", track="#ebe9e4"),
    "dark": dict(surface="#1a1a19", band="#222220", border="#30302d", grid="#2c2c29", axis="#5f5e59",
                 ink="#f0efec", ink2="#c3c2b7", muted="#8f8e86",
                 top10="#3987e5", count="#199e70", slow="#e66767", neutral="#5f5e59", track="#2a2a27"),
}


def esc(s):
    return str(s).replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


class Svg:
    def __init__(self, t, w, h):
        self.t, self.w, self.h, self.parts = t, w, h, []

    def add(self, s):
        self.parts.append(s)

    def text(self, x, y, s, size=14, fill=None, anchor="start", weight=400, mono=False, spacing=0):
        ls = f' letter-spacing="{spacing}"' if spacing else ""
        self.add(f'<text x="{x:.1f}" y="{y:.1f}" font-size="{size}" font-weight="{weight}" '
                 f'fill="{fill or self.t["ink"]}" text-anchor="{anchor}"{ls} '
                 f'font-family="{MONO if mono else SANS}">{esc(s)}</text>')

    def header(self, eyebrow, title, subtitle):
        self.text(40, 46, eyebrow.upper(), 11.5, self.t["muted"], weight=600, spacing=1.2)
        self.text(40, 76, title, 24, weight=700)
        self.text(40, 102, subtitle, 13.5, self.t["ink2"])

    def hbar(self, x0, x1, y, h, fill, radius=4):
        """Horizontal bar from the baseline x0 to x1: square at the baseline,
        rounded at the data end (either direction)."""
        a, b = sorted((x0, x1))
        width = max(b - a, 1.5)
        r = min(radius, width / 2, h / 2)
        if x1 >= x0:   # data end on the right
            d = (f"M{a:.1f},{y:.1f} H{a + width - r:.1f} Q{a + width:.1f},{y:.1f} {a + width:.1f},{y + r:.1f} "
                 f"V{y + h - r:.1f} Q{a + width:.1f},{y + h:.1f} {a + width - r:.1f},{y + h:.1f} H{a:.1f} Z")
        else:          # data end on the left
            d = (f"M{b:.1f},{y:.1f} H{a + r:.1f} Q{a:.1f},{y:.1f} {a:.1f},{y + r:.1f} "
                 f"V{y + h - r:.1f} Q{a:.1f},{y + h:.1f} {a + r:.1f},{y + h:.1f} H{b:.1f} Z")
        self.add(f'<path d="{d}" fill="{fill}"/>')

    def save(self, name):
        t = self.t
        body = (f'<svg xmlns="http://www.w3.org/2000/svg" width="{self.w}" height="{self.h}" '
                f'viewBox="0 0 {self.w} {self.h}" role="img">'
                f'<rect x="0.5" y="0.5" width="{self.w - 1}" height="{self.h - 1}" rx="14" '
                f'fill="{t["surface"]}" stroke="{t["border"]}"/>' + "".join(self.parts) + "</svg>")
        (OUT / name).write_text(body)


def legend(s, x, y, items):
    """Swatch + label in text ink (identity never by colored text alone)."""
    for label, fill in items:
        s.add(f'<rect x="{x}" y="{y - 9}" width="12" height="10" rx="2.5" fill="{fill}"/>')
        s.text(x + 18, y, label, 12.5, s.t["ink2"])
        x += 36 + 7.0 * len(label)


def fmt_ratio(v):
    return f"{v:.0f}×" if v >= 20 else (f"{v:.1f}×" if v >= 10 else f"{v:.2f}×")


def diverging(theme, name, eyebrow, title, subtitle, rows, series, note, lo=0.5, hi=4.0, ticks=(0.5, 1, 2, 4)):
    """Log2 axis centred on 1x: faster to the right, slower to the left.
    rows: [(label, sublabel, [time ratio per series])], ratio = candidate time / baseline time."""
    t = THEMES[theme]
    bar, gap = 14, 6
    row_h = (bar + gap) * len(series) + 24
    left, right, top = 236, 120, 150
    w = 900
    h = top + row_h * len(rows) + 70
    s = Svg(t, w, h)
    s.header(eyebrow, title, subtitle)
    items = [(label, t[key]) for label, key in series]
    if len(series) > 1:
        legend(s, 40, 132, items)
    plot = w - left - right
    for label, _, values in rows:
        for r in values:
            if not lo <= 1 / r <= hi:
                raise ValueError(f"{name}: {label} speedup {1 / r:.3g} falls outside [{lo}, {hi}]")
    lx = lambda r: left + (math.log2(1 / r) - math.log2(lo)) / (math.log2(hi) - math.log2(lo)) * plot
    x1 = lx(1.0)
    bottom = top + row_h * len(rows) - 8
    for v in ticks:
        if lo <= v <= hi:
            x = lx(1 / v)
            s.add(f'<line x1="{x:.1f}" y1="{top - 8}" x2="{x:.1f}" y2="{bottom:.1f}" '
                  f'stroke="{t["axis"] if v == 1 else t["grid"]}" stroke-width="1"/>')
            lab = "same speed" if v == 1 else (f"{v:g}× faster" if v > 1 else f"{1 / v:g}× slower")
            s.text(x, bottom + 22, lab, 11.5, t["muted"], "middle")
    for i, (label, sub, values) in enumerate(rows):
        y0 = top + i * row_h
        if i:
            s.add(f'<line x1="40" y1="{y0 - 6:.1f}" x2="{w - 40}" y2="{y0 - 6:.1f}" stroke="{t["grid"]}"/>')
        block = (bar + gap) * len(series) - gap
        mid = y0 + 6 + block / 2
        s.text(left - 20, mid + (-1 if sub else 5), label, 14.5, anchor="end", weight=600)
        if sub:
            s.text(left - 20, mid + 16, sub, 11.5, t["muted"], "end")
        for j, ((_, key), r) in enumerate(zip(series, values)):
            y = y0 + 6 + j * (bar + gap)
            x = lx(r)
            slower = r > 1.005
            same = abs(r - 1) <= 0.005
            s.hbar(x1, x, y, bar, t["slow"] if slower else t[key])
            txt = "same" if same else (fmt_ratio(1 / r) if not slower else f"{r:.2f}× slower")
            if slower:
                s.text(min(x, x1) - 8, y + 11.5, txt, 12.5, t["ink2"], "end", mono=True, weight=600)
            else:
                s.text(max(x, x1) + 8, y + 11.5, txt, 12.5, t["ink"], mono=True, weight=600)
    s.text(w - 40, h - 20, note, 11, t["muted"], "end")
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
    left, right, top, row = 236, 60, 150, 26
    w = 900
    h = top + row * len(rows) + 72
    s = Svg(t, w, h)
    s.header("vs Apache Lucene 10.4 · earlier fork build", "Faster on every query",
             "Per-query speedup (Lucene median ÷ fork median), geometric mean of both process orders.")
    legend(s, 40, 132, [("TOP-10", t["top10"]), ("COUNT", t["count"])])
    lo, hi = 0.8, 8.0
    plot = w - left - right
    sx = lambda v: left + (math.log2(v) - math.log2(lo)) / (math.log2(hi) - math.log2(lo)) * plot
    bottom = top + row * len(rows) - 6
    for i in range(len(rows)):
        y = top + i * row + 6
        if i % 2 == 0:
            s.add(f'<rect x="40" y="{y - 12}" width="{w - 80}" height="{row - 2}" rx="5" fill="{t["band"]}"/>')
    for q, a, c in rows:
        if not (lo <= a <= hi and lo <= c <= hi):
            raise ValueError(f"lucene-per-query: {q} has a speedup outside [{lo}, {hi}]")
    for v in (1, 2, 4, 8):
        x = sx(v)
        s.add(f'<line x1="{x:.1f}" y1="{top - 10}" x2="{x:.1f}" y2="{bottom}" '
              f'stroke="{t["axis"] if v == 1 else t["grid"]}" stroke-width="1"/>')
        s.text(x, bottom + 22, "same speed" if v == 1 else f"{v}× faster", 11.5, t["muted"], "middle")
    for i, (q, a, c) in enumerate(rows):
        y = top + i * row + 6
        s.text(left - 18, y + 4.5, q, 12.5, t["ink2"], "end", mono=True)
        xa, xc = sx(a), sx(c)
        s.add(f'<line x1="{xa:.1f}" y1="{y - 5}" x2="{xc:.1f}" y2="{y + 5}" stroke="{t["axis"]}" '
              f'stroke-width="2" stroke-linecap="round"/>')
        s.add(f'<circle cx="{xa:.1f}" cy="{y - 5}" r="4.5" fill="{t["top10"]}" stroke="{t["surface"]}" stroke-width="1.5"/>')
        s.add(f'<circle cx="{xc:.1f}" cy="{y + 5}" r="4.5" fill="{t["count"]}" stroke="{t["surface"]}" stroke-width="1.5"/>')
    s.text(w - 40, h - 20, "Wikipedia 1M, native BM25 on both engines; report 2026-10-04-native", 11, t["muted"], "end")
    s.save(f"lucene-per-query-{theme}.svg")


def resources(theme, lucene_rs_rss_mib):
    t = THEMES[theme]
    mem = json.loads((LUCENE / "memory.json").read_text())["engines"]
    storage = json.loads((LUCENE / "storage.json").read_text())
    kib = lambda v: int(v.split()[0]) / 1024
    groups = [
        ("Resident memory, warmed query process", "MiB", [
            ("tantivy-turbo", kib(mem["tantivy"]["status"]["VmRSS"]), "top10"),
            ("Lucene 10.4 (default JVM heap)", kib(mem["lucene"]["status"]["VmRSS"]), "neutral"),
        ]),
        ("Index size on disk", "MB", [
            ("tantivy-turbo", storage["tantivy"]["total_bytes"] / 1e6, "top10"),
            ("Lucene 10.4", storage["lucene"]["total_bytes"] / 1e6, "neutral"),
        ]),
    ]
    w, left, right = 900, 270, 130
    h = 140 + sum(44 + 30 * len(g[2]) for g in groups) + 10
    s = Svg(t, w, h)
    s.header("vs Apache Lucene 10.4 · earlier fork build", "Memory and disk",
             "Same Wikipedia 1M corpus and 20 queries; resident memory includes mapped index pages.")
    y = 140
    for title, unit, bars in groups:
        s.text(40, y + 4, title, 13.5, t["ink2"], weight=600)
        y += 20
        vmax = max(v for _, v, _ in bars)
        for label, v, key in bars:
            s.text(left - 18, y + 12, label, 13, t["ink"], "end")
            width = max(v / vmax * (w - left - right), 3)
            s.hbar(left, left + width, y, 16, t[key])
            s.text(left + width + 8, y + 12.5, f"{v:,.1f} {unit}", 12.5, mono=True, weight=600)
            y += 30
        y += 24
    s.save(f"resources-{theme}.svg")


def blocks_lane(s, x0, x1, y, h, cuts, fill, label, note):
    """A doc-ID lane split at `cuts` (fractions), each piece a rounded tile."""
    t = s.t
    s.text(x0, y - 10, label, 13, weight=600)
    s.text(x1, y - 10, note, 12, t["ink2"], "end")
    edges = [0.0] + sorted(cuts) + [1.0]
    for a, b in zip(edges, edges[1:]):
        xa = x0 + a * (x1 - x0)
        xb = x0 + b * (x1 - x0)
        if xb - xa < 1.2:
            continue
        s.add(f'<rect x="{xa + 0.75:.1f}" y="{y}" width="{max(xb - xa - 1.5, 0.6):.1f}" height="{h}" '
              f'rx="{min(3, (xb - xa) / 3):.1f}" fill="{fill}"/>')


def regions(theme):
    """OR: fine MaxScore regions (any term's block boundary) vs coarse regions."""
    t = THEMES[theme]
    w, h = 900, 366
    s = Svg(t, w, h)
    s.header("How it works · OR", "Fewer, wider MaxScore regions",
             "MaxScore re-derives every term's score bound per region. Fewer regions mean less bookkeeping.")
    x0, x1 = 40, w - 40
    fine = [i / 230 + (0.0016 if i % 3 else 0) for i in range(1, 230)]
    blocks_lane(s, x0, x1, 158, 46, fine, t["neutral"], "Earlier turbo · fine regions",
                "a region ends at every term's 128-doc block: 30,242 on the 21-term query")
    coarse = [0.12, 0.21, 0.36, 0.45, 0.6, 0.68, 0.81, 0.9]
    blocks_lane(s, x0, x1, 248, 46, coarse, t["top10"], "tantivy-turbo",
                "regions follow the essential terms; others bounded by cached ranges: 362")
    s.text(40, h - 42, "Chosen per query by measured cost; candidates are scored in Lucene-style bulk windows.", 12, t["muted"])
    s.text(40, h - 20, "Schematic, not to scale; counts measured for one query.", 12, t["muted"])
    s.text(w - 40, h - 20, "Document IDs →", 12, t["muted"], "end")
    s.save(f"regions-{theme}.svg")


def and_windows(theme):
    """AND: block-max windows cut at any term's block vs following the rare term."""
    t = THEMES[theme]
    w, h = 900, 410
    s = Svg(t, w, h)
    s.header("How it works · AND", "Windows follow the rarest term",
             "Block-max conjunctions score a window at a time, but most windows held no rare-term posting at all.")
    x0, x1 = 40, w - 40
    span = x1 - x0
    rare = [0.07, 0.31, 0.335, 0.62, 0.86]   # rare-term postings
    rare_blocks = [0.2, 0.5, 0.75]
    dense_blocks = [i / 34 for i in range(1, 34)]

    def term_rows(y):
        for label, cuts, dots in (("rare term", rare_blocks, rare), ("common term", dense_blocks, None)):
            s.text(x0, y + 11, label, 12, t["ink2"])
            lx0 = x0 + 110
            s.add(f'<line x1="{lx0}" y1="{y + 7}" x2="{x1}" y2="{y + 7}" stroke="{t["axis"]}"/>')
            for c in cuts:
                xc = lx0 + c * (x1 - lx0)
                s.add(f'<line x1="{xc:.1f}" y1="{y + 1}" x2="{xc:.1f}" y2="{y + 13}" stroke="{t["axis"]}"/>')
            if dots:
                for d in dots:
                    xd = lx0 + d * (x1 - lx0)
                    s.add(f'<circle cx="{xd:.1f}" cy="{y + 7}" r="4.5" fill="{t["ink"]}" '
                          f'stroke="{t["surface"]}" stroke-width="2"/>')
            y += 24
        return lx0

    # Before: windows end at every block boundary of any term.
    y = 146
    s.text(x0, y, "Tantivy 0.27 and Lucene", 13, weight=600)
    s.text(x1, y, "a window ends at the nearest block boundary of any term: many tiny windows", 12, t["ink2"], "end")
    lx0 = term_rows(y + 12)
    cuts = sorted(set(rare_blocks + dense_blocks))
    edges = [0.0] + cuts + [1.0]
    for a, b in zip(edges, edges[1:]):
        xa, xb = lx0 + a * (x1 - lx0), lx0 + b * (x1 - lx0)
        s.add(f'<rect x="{xa + 0.75:.1f}" y="{y + 64}" width="{xb - xa - 1.5:.1f}" height="20" rx="2.5" fill="{t["neutral"]}"/>')
    # After: windows are the rare term's blocks, starting at its next posting.
    y = 268
    s.text(x0, y, "tantivy-turbo", 13, weight=600)
    s.text(x1, y, "a window is the rare term's block, starting at its next posting", 12, t["ink2"], "end")
    lx0 = term_rows(y + 12)
    redges = [0.0] + rare_blocks + [1.0]
    for a, b in zip(redges, redges[1:]):
        inside = [d for d in rare if a <= d < b]
        if not inside:
            continue
        start = min(inside)
        xa, xb = lx0 + start * (x1 - lx0), lx0 + b * (x1 - lx0)
        s.add(f'<rect x="{xa - 4:.1f}" y="{y + 64}" width="{xb - xa + 2.5:.1f}" height="20" rx="4" fill="{t["top10"]}"/>')
    s.text(40, h - 22, "Other terms are bounded over each window by cached range maxima. "
           "About a third less time on AND queries, identical results.", 12, t["muted"])
    s.save(f"and-windows-{theme}.svg")


def banner(theme, stats):
    t = THEMES[theme]
    w, h = 900, 260
    s = Svg(t, w, h)
    # Accent rule: the two series hues, side by side.
    s.add(f'<rect x="40" y="32" width="56" height="5" rx="2.5" fill="{t["top10"]}"/>')
    s.add(f'<rect x="100" y="32" width="28" height="5" rx="2.5" fill="{t["count"]}"/>')
    s.text(40, 92, "tantivy-turbo", 46, weight=750)
    s.text(40, 124, "TOP-10: Oct 23 run, 1,201 queries; Lucene: Oct 4, 20 queries.", 16, t["ink2"])
    gap = 14
    tile_w = (w - 80 - 3 * gap) / 4
    for i, (value, label) in enumerate(stats):
        x = 40 + i * (tile_w + gap)
        s.add(f'<rect x="{x:.1f}" y="152" width="{tile_w:.1f}" height="80" rx="12" fill="{t["band"]}"/>')
        s.add(f'<rect x="{x + 18:.1f}" y="168" width="18" height="3" rx="1.5" fill="{t["top10"]}"/>')
        s.text(x + 18, 202, value, 28, t["ink"], weight=700)
        s.text(x + 18, 222, label, 12, t["ink2"])
    s.save(f"banner-{theme}.svg")


def ratio(entry):
    return math.sqrt(entry["AB"]["sum_ratio"] * entry["BA"]["sum_ratio"])


def main():
    OUT.mkdir(exist_ok=True)
    up = json.loads((HERE / "turbo-vs-tantivy-0.27/summary.json").read_text())
    rs = json.loads((HERE / "turbo-vs-lucene-rs/summary.json").read_text())
    groups = [("OR, 3+ terms", "103 queries", "OR3+"), ("OR, 2 terms", "198 queries", "OR2"),
              ("Phrase", "300 queries", "PHRASE"), ("Single term", "300 queries", "TERM"),
              ("AND", "300 queries", "AND"), ("All 1,201 queries", "port-style suite", "ALL")]
    series = [("TOP-10", "top10"), ("COUNT", "count")]
    rss = rs["rss"]["port"]["VmHWM"]
    for theme in THEMES:
        diverging(theme, "vs-tantivy", "vs Tantivy 0.27", "Every query type, faster in total",
                  "Summed query time, geometric mean of both orders. Same 1M Wikipedia docs and machine, identical counts.",
                  [(a, b, [ratio(up[f"top10/{g}"]), ratio(up[f"count/{g}"])]) for a, b, g in groups], series,
                  "Oct 23 run · log scale · bench-results/turbo-vs-tantivy-0.27", lo=0.8, hi=4.5, ticks=(1, 2, 4))
        diverging(theme, "vs-lucene-rs", "vs lucene-rs (Rust port of Lucene 10)", "TOP-10 search",
                  "Summed query time, geometric mean of both orders. Identical top-10 IDs, score bits and counts on checked queries.",
                  [(a, b, [ratio(rs[f"top10/{g}"])]) for a, b, g in groups], [("TOP-10", "top10")],
                  "Oct 23 run · log scale · bench-results/turbo-vs-lucene-rs", lo=0.8, hi=2.5, ticks=(1, 1.5, 2))
        diverging(theme, "vs-lucene-rs-count", "vs lucene-rs (Rust port of Lucene 10)", "COUNT",
                  "lucene-rs counts by visiting every match; tantivy-turbo uses doc frequencies, bitset lanes and probes.",
                  [(a, b, [ratio(rs[f"count/{g}"])]) for a, b, g in groups], [("COUNT", "count")],
                  "Oct 23 run · log scale · bench-results/turbo-vs-lucene-rs", lo=0.5, hi=512, ticks=(1, 4, 16, 64, 256))
        lucene_per_query(theme)
        resources(theme, rss)
        regions(theme)
        and_windows(theme)
        banner(theme, [(f"{1 / ratio(up['top10/ALL']):.1f}×", "TOP-10 vs Tantivy"),
                       (f"{1 / ratio(rs['top10/ALL']):.2f}×", "TOP-10 vs lucene-rs"),
                       ("1.8–2.1×", "TOP-10 vs Lucene"), ("3 × 1,227", "checks vs prior build")])
    print("charts written:", sorted(p.name for p in OUT.glob("*.svg")))


if __name__ == "__main__":
    main()
