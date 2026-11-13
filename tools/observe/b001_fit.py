#!/usr/bin/env python3
"""B-001: fit the round-tip dab model per AA level and size.
usage: b001_fit.py DIR   (DIR has B001_aa0..3.png)
Model: alpha = R8(255 * clamp((R - d) / W, 0, 1)), R = size/2, d = distance from the pen
position to the pixel centre (canvas coords, pixel centres at i+0.5). For each (level, size)
the best W (0.05 grid) and rounding are searched; AA None is also tested as d < R / d <= R.
"""
import sys, numpy as np
from PIL import Image
SIZES = [0.7, 1, 1.5, 2, 3, 4.5, 10, 25.3]
COLS = [(60.5, 0.5), (200.75, 0.5), (341.0, 1.0)]
d0 = sys.argv[1]
def patches(a, r):
    out = []
    for cx, dy in COLS:
        cy = 25 + 46 * r + dy
        x0, y0 = int(cx) - 20, int(cy) - 20
        ys, xs = np.mgrid[y0:y0 + 41, x0:x0 + 41]
        d = np.hypot(xs + 0.5 - cx, ys + 0.5 - cy)
        out.append((d, a[y0:y0 + 41, x0:x0 + 41]))
    return out
RND = {"round": np.rint, "floor": np.floor, "ceil": np.ceil, "half_up": lambda v: np.floor(v + 0.5)}
for lvl in range(4):
    a = np.asarray(Image.open(f"{d0}/B001_aa{lvl}.png").convert("RGBA")).astype(float)[..., 3]
    print(f"== AA level {lvl}")
    for r, size in enumerate(SIZES):
        R = size / 2
        P = patches(a, r)
        best = None
        cands = []
        if lvl == 0:
            cands += [("d<R", lambda d: np.where(d < R, 255, 0)), ("d<=R", lambda d: np.where(d <= R, 255, 0))]
        for W in np.arange(0.5, 4.01, 0.05):
            for rn, f in RND.items():
                cands.append((f"W={W:.2f} {rn}", lambda d, W=W, f=f: f(255 * np.clip((R - d) / W, 0, 1))))
        for name, fn in cands:
            n = ex = 0; mx = 0
            for d, p in P:
                e = np.abs(fn(d) - p); n += e.size; ex += (e == 0).sum(); mx = max(mx, e.max())
            score = (ex / n, -mx)
            if best is None or score > best[0]:
                best = (score, name, mx)
        print(f"  size {size:5}: best {best[1]:16s} exact {best[0][0]*100:7.3f}%  max {int(best[2])}")
