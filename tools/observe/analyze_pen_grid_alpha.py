#!/usr/bin/env python3
"""Measure stroke opacity in a pen_grid.py export (black ink on white paper).

usage: analyze_pen_grid_alpha.py EXPORT.png P1,P2,...
Per stroke: centre-line alpha = mean over the central 40 columns of (255 - R) on the stroke's
centre row, as 0..255 and 0..1, plus min/max over those pixels (uniformity).
"""
import sys, numpy as np
from PIL import Image
img = np.asarray(Image.open(sys.argv[1]).convert("RGB")).astype(np.int64)
Ps = [float(v) for v in sys.argv[2].split(",")]
ink = 255 - img[..., 0]
print(f"{'P':>6} {'mean':>7} {'alpha':>7} {'min':>4} {'max':>4}")
for i, P in enumerate(Ps):
    c, r = i % 7, i // 7
    x0, y = 20 + c * 170 + 25, 150 + r * 300
    row = ink[y, x0 + 40:x0 + 80]
    print(f"{P:6.3f} {row.mean():7.2f} {row.mean()/255:7.4f} {row.min():4d} {row.max():4d}")
