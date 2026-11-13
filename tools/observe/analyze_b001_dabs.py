#!/usr/bin/env python3
"""B-001: single-dab alpha profiles from the dab-grid exports.

usage: analyze_b001_dabs.py EXPORT_aaN.png [--dump]
Grid: rows r=0..7 (sizes 0.7 1 1.5 2 3 4.5 10 25.3) at y = 25+46r; columns: pen at
(60.5, y+0.5), (200.75, y+0.5), (341.0, y+1.0) in canvas coordinates (pixel centres at i+0.5).
Per dab prints: alpha sum/255 (area), equivalent diameter, alpha-weighted centroid (in
canvas coordinates, i.e. +0.5 on pixel index), and with --dump the alpha patch.
"""
import sys, numpy as np
from PIL import Image
SIZES = [0.7, 1, 1.5, 2, 3, 4.5, 10, 25.3]
COLS = [(60.5, 0.5), (200.75, 0.5), (341.0, 1.0)]
a = np.asarray(Image.open(sys.argv[1]).convert("RGBA")).astype(float)[..., 3]
for r, size in enumerate(SIZES):
    y0 = 25 + 46 * r
    for cx, dy in COLS:
        cy = y0 + dy
        x0, y1 = int(cx) - 20, int(cy) - 20
        p = a[y1:y1 + 41, x0:x0 + 41]
        w = p / 255
        area = w.sum()
        if area == 0:
            print(f"size {size:5} pen ({cx:7.2f},{cy:6.2f}): EMPTY"); continue
        ys, xs = np.mgrid[0:41, 0:41]
        mx = (w * (xs + 0.5)).sum() / area + x0
        my = (w * (ys + 0.5)).sum() / area + y1
        nz = np.argwhere(p > 0)
        bw = nz[:, 1].max() - nz[:, 1].min() + 1
        print(f"size {size:5} pen ({cx:7.2f},{cy:6.2f}): area {area:8.3f} eqd {2*np.sqrt(area/np.pi):6.3f} "
              f"centroid ({mx:7.3f},{my:7.3f}) bbox_w {bw:2d} max {int(p.max())}")
        if "--dump" in sys.argv and size <= 4.5:
            sub = p[20 - 4:20 + 5, 20 - 4:20 + 5].astype(int) if cx != 341.0 else p[20 - 4:20 + 5, 20 - 4:20 + 5].astype(int)
            for row in sub:
                print("    " + " ".join(f"{v:3d}" for v in row))
