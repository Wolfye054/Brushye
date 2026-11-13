#!/usr/bin/env python3
"""Measure stroke widths in a pen_grid.py export.

usage: analyze_pen_grid.py EXPORT.png P1,P2,...  [--size S]
Per stroke: 'coverage width' = mean over the central 40 columns of sum(ink)/255 per column,
where ink = 255 - R on a white background with black ink (= stroke width in px, AA-aware).
Also the peak alpha (darkest pixel) and the extent of rows with any ink.
"""
import sys, numpy as np
from PIL import Image
img = np.asarray(Image.open(sys.argv[1]).convert("RGB")).astype(np.int64)
Ps = [float(v) for v in sys.argv[2].split(",")]
size = float(sys.argv[sys.argv.index("--size") + 1]) if "--size" in sys.argv else None
ink = 255 - img[..., 0]
print(f"{'P':>6} {'width_px':>9} {'w/size':>7} {'peak':>5} {'rows':>5}")
for i, P in enumerate(Ps):
    c, r = i % 7, i // 7
    x0, y = 20 + c * 170 + 25, 150 + r * 300
    xs = slice(x0 + 40, x0 + 80); ys = slice(y - 140, y + 140)
    cell = ink[ys, xs]
    w = cell.sum(0).mean() / 255
    rows = int((cell.max(1) > 0).sum())
    rel = f"{w/size:7.4f}" if size else ""
    print(f"{P:6.3f} {w:9.3f} {rel:>7} {int(cell.max()):5d} {rows:5d}")
