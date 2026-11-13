#!/usr/bin/env python3
"""B-001: refit the ramp model with the dab centre free (the input chain quantises the pen
position). For AA levels 1-3, sizes >= 3: search centre offset (+-0.3 px, 0.02 grid) and ramp
width W (0.05 grid), rounding round-half-even. Statistics over pixels where model or data > 0.
usage: b001_fit_centre.py DIR
"""
import sys, numpy as np
from PIL import Image
SIZES = [0.7, 1, 1.5, 2, 3, 4.5, 10, 25.3]
COLS = [(60.5, 0.5), (200.75, 0.5), (341.0, 1.0)]
d0 = sys.argv[1]
offs = np.arange(-0.3, 0.301, 0.02)
for lvl in (1, 2, 3):
    a = np.asarray(Image.open(f"{d0}/B001_aa{lvl}.png").convert("RGBA")).astype(float)[..., 3]
    print(f"== AA level {lvl}")
    for r in (4, 5, 6, 7):
        size = SIZES[r]; R = size / 2
        for c, (cx, dy) in enumerate(COLS):
            cy = 25 + 46 * r + dy
            x0, y0 = int(cx) - 20, int(cy) - 20
            p = a[y0:y0 + 41, x0:x0 + 41]
            ys, xs = np.mgrid[y0:y0 + 41, x0:x0 + 41]
            best = None
            for W in np.arange(1.0, 4.01, 0.05):
                for ox in offs:
                    for oy in offs:
                        d = np.hypot(xs + 0.5 - (cx + ox), ys + 0.5 - (cy + oy))
                        m = np.rint(255 * np.clip((R - d) / W, 0, 1))
                        mask = (m > 0) | (p > 0)
                        e = np.abs(m - p)[mask]
                        s = ((e == 0).mean(), -e.max())
                        if best is None or s > best[0]:
                            best = (s, W, ox, oy, e.max(), mask.sum())
            s, W, ox, oy, mx, n = best
            print(f"  size {size:5} col {c}: W={W:.2f} centre offset ({ox:+.2f},{oy:+.2f}) exact {s[0]*100:6.2f}% of {n} px, max {int(mx)}")
