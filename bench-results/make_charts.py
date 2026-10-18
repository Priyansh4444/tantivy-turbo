"""Render the README's dithered SVG charts from the measured summaries.

turbo-vs-tantivy-0.27/summary.json : measured here (compare.py)
Lucene 10.4 numbers                : doc/performance/lucene-10.4/results/2026-10-04-native
"""
import json, math
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE.parent / "assets"
BG, INK, MUTED, GRID = "#0c0e12", "#ecebe4", "#8b919c", "#2a2e36"
COLORS = {"lime": "#c6f35a", "teal": "#5cc8b8", "slate": "#5a606b"}
BAYER = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]


def patterns():
    out = []
    for name, color in COLORS.items():
        for level in (5, 8, 12, 16):
            cells = "".join(
                f'<rect x="{x*2}" y="{y*2}" width="2" height="2" fill="{color}"/>'
                for y in range(4) for x in range(4) if BAYER[y][x] < level)
            out.append(f'<pattern id="{name}{level}" width="8" height="8" patternUnits="userSpaceOnUse">{cells}</pattern>')
    return "".join(out)


def speedup(entry):
    # Mean of both run orders, as a time ratio inverted to "x faster".
    return 1 / math.sqrt(entry["AB"]["sum_ratio"] * entry["BA"]["sum_ratio"])


def bar_chart(path, title, subtitle, rows, series, xmax, note):
    """rows: [(label, [value per series])]; series: [(name, pattern)]."""
    w, left, right, top = 960, 220, 90, 96
    row_h = 22 * len(series) + 22
    h = top + row_h * len(rows) + 64
    plot = w - left - right
    sx = lambda v: left + v / xmax * plot
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" '
             f'font-family="DM Sans, Inter, Segoe UI, Helvetica, Arial, sans-serif">',
             f"<defs>{patterns()}</defs>",
             f'<rect width="{w}" height="{h}" rx="14" fill="{BG}"/>',
             f'<text x="24" y="40" font-size="22" font-weight="600" fill="{INK}">{title}</text>',
             f'<text x="24" y="66" font-size="14" fill="{MUTED}">{subtitle}</text>']
    for v in range(0, int(xmax) + 1):
        x = sx(v)
        parts.append(f'<line x1="{x:.1f}" y1="{top-8}" x2="{x:.1f}" y2="{h-50}" stroke="{INK if v == 1 else GRID}" '
                     f'stroke-width="{1.5 if v == 1 else 1}" {"" if v == 1 else "stroke-dasharray=\"2 4\""}/>')
        parts.append(f'<text x="{x:.1f}" y="{h-30}" font-size="13" fill="{MUTED}" text-anchor="middle">{v}x</text>')
    for i, (label, values) in enumerate(rows):
        y0 = top + i * row_h
        parts.append(f'<text x="{left-14}" y="{y0 + row_h/2 - 2:.1f}" font-size="15" fill="{INK}" text-anchor="end">{label}</text>')
        for j, ((sname, pat), v) in enumerate(zip(series, values)):
            if v is None:
                continue
            y = y0 + 6 + j * 22
            same = abs(v - 1) < 0.01
            slower = v < 1 and not same
            fill = "url(#slate12)" if slower else f"url(#{pat})"
            parts.append(f'<rect x="{left}" y="{y}" width="{max(sx(v)-left, 1):.1f}" height="16" fill="{fill}"/>')
            txt = "≈ same" if same else (f"{v:.2f}x" if not slower else f"{1/v:.2f}x slower")
            parts.append(f'<text x="{sx(v)+6:.1f}" y="{y+13}" font-size="13" font-family="JetBrains Mono, Menlo, Consolas, monospace" '
                         f'fill="{MUTED if slower else INK}">{txt}</text>')
    lx = left
    for sname, pat in series:
        parts.append(f'<rect x="{lx}" y="{h-20}" width="14" height="10" fill="url(#{pat})"/>')
        parts.append(f'<text x="{lx+20}" y="{h-11}" font-size="13" fill="{MUTED}">{sname}</text>')
        lx += 20 + 8 * len(sname) + 24
    parts.append(f'<text x="{w-24}" y="{h-11}" font-size="12" fill="{MUTED}" text-anchor="end">{note}</text>')
    parts.append("</svg>")
    path.write_text("".join(parts))


s = json.loads((HERE / "turbo-vs-tantivy-0.27/summary.json").read_text())
groups = [("20 Wikipedia queries", "wiki20"), ("OR, 3+ terms", "OR3+"), ("OR, 2 terms", "OR2"), ("Phrase", "PHRASE"),
          ("Single term", "TERM"), ("AND", "AND"), ("All 1,201 queries", "ALL")]
rows = [(label, [speedup(s[f"top10/{g}"]), speedup(s[f"count/{g}"])]) for label, g in groups]
bar_chart(OUT / "turbo-vs-tantivy.svg", "tantivy-turbo vs Tantivy 0.27",
          "Speedup on summed query time, mean of both run orders. Same 1M Wikipedia docs, same machine.",
          rows, [("TOP-10", "lime16"), ("COUNT", "teal8")], 6, "right of the 1x line = faster")
# Lucene 10.4, matched native BM25 (2026-10-04 report): T/L geometric means in both process orders.
lucene = [("TOP-10, Tantivy first", [1 / 0.47828]), ("TOP-10, Lucene first", [1 / 0.54941]),
          ("COUNT, Tantivy first", [1 / 0.76935]), ("COUNT, Lucene first", [1 / 0.78807])]
bar_chart(OUT / "turbo-vs-lucene.svg", "Fork vs Apache Lucene 10.4",
          "Geometric mean speedup over 20 Wikipedia 1M queries, native BM25 on both, 20/20 queries faster.",
          lucene, [("speedup", "lime12")], 3, "report: doc/performance/lucene-10.4/results/2026-10-04-native")
print("charts written")
