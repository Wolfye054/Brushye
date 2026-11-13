#!/usr/bin/env python3
"""B-001: alpha vs distance from the pen centre for one dab of the grid.
usage: b001_radial.py EXPORT.png ROW COL   (ROW 0..7 sizes, COL 0..2 offsets; see analyze_b001_dabs.py)"""
import sys, numpy as np
from PIL import Image
SIZES = [0.7, 1, 1.5, 2, 3, 4.5, 10, 25.3]
COLS = [(60.5, 0.5), (200.75, 0.5), (341.0, 1.0)]
a = np.asarray(Image.open(sys.argv[1]).convert("RGBA")).astype(int)[..., 3]
r, c = int(sys.argv[2]), int(sys.argv[3])
cx, dy = COLS[c]; cy = 25 + 46 * r + dy
pts = {}
for y in range(int(cy) - 20, int(cy) + 21):
    for x in range(int(cx) - 20, int(cx) + 21):
        d = round(float(np.hypot(x + 0.5 - cx, y + 0.5 - cy)), 4)
        pts.setdefault(d, set()).add(int(a[y, x]))
print(f"size {SIZES[r]} pen ({cx},{cy}); radius {SIZES[r]/2}")
for d in sorted(pts):
    v = sorted(pts[d])
    if d < SIZES[r] / 2 + 3 and (max(v) > 0 or d < SIZES[r] / 2 + 1.5):
        print(f"  d={d:8.4f}  alpha {v}")
