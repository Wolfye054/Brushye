#!/usr/bin/env python3
"""B-010 follow-up: tip thickness, direction and angle from single dabs (pure Python + Pillow).

Exports B010_tip_t<T>_a<A>_<vert|horiz>.png: brush size 41 (R = 20.5), AA None, hardness 5,
density 100, thickness T %, angle A degrees, direction Vertical/Horizontal; two dabs per
export at brush_run2.TIP_POINTS (200.5, 200.5) and (500.3, 200.7).
For each dab: area, second-moment orientation (degrees, counter-clockwise on screen, i.e.
y up) and the two axis half-lengths. Then a closed ellipse test, pixel by pixel, with the
centre at the press point or 0.5 px to its left:
semi-axes R and R*T/100, the short axis along y ("vert") or x ("horiz") before rotating by
+A or -A degrees; the best convention is reported with its exactness.
usage: b010_tip.py DATA_DIR
"""
import glob
import math
import os
import re
import sys

from PIL import Image

sys.path.insert(0, os.path.dirname(__file__))
from brush_run2 import TIP_POINTS  # noqa: E402

R = 20.5


def moments(a, cx, cy):
    n = sx = sy = 0
    pts = []
    for y in range(int(cy) - 25, int(cy) + 26):
        for x in range(int(cx) - 25, int(cx) + 26):
            if a[x, y]:
                pts.append((x + 0.5, y + 0.5))
    n = len(pts)
    mx = sum(p[0] for p in pts) / n
    my = sum(p[1] for p in pts) / n
    sxx = sum((p[0] - mx) ** 2 for p in pts) / n
    syy = sum((p[1] - my) ** 2 for p in pts) / n
    sxy = sum((p[0] - mx) * (p[1] - my) for p in pts) / n
    ang = 0.5 * math.degrees(math.atan2(2 * -sxy, sxx - syy))  # major axis, CCW with y up
    tr, det = sxx + syy, sxx * syy - sxy * sxy
    l1 = tr / 2 + math.sqrt(max(tr * tr / 4 - det, 0))
    l2 = tr / 2 - math.sqrt(max(tr * tr / 4 - det, 0))
    return n, (mx, my), ang, 2 * math.sqrt(l1), 2 * math.sqrt(l2)  # uniform ellipse: a = 2 sqrt(l)


def inside(dx, dy, t, ang, short):
    c, s = math.cos(math.radians(ang)), math.sin(math.radians(ang))
    # rotate the pixel offset into the tip frame (screen y down; +ang = counter-clockwise on screen)
    u = c * dx - s * dy
    v = s * dx + c * dy
    a, b = (R, R * t) if short == "y" else (R * t, R)
    return (u / a) ** 2 + (v / b) ** 2 <= 1 + 1e-12


def main():
    for f in sorted(glob.glob(f"{sys.argv[1]}/B010_tip_*.png")):
        m = re.search(r"t(\d+)_a(\d+)_(vert|horiz)", f)
        t, A, d = int(m[1]) / 100, int(m[2]), m[3]
        a = Image.open(f).getchannel("A").load()
        print(os.path.basename(f))
        for cx, cy in TIP_POINTS:
            n, c, ang, l1, l2 = moments(a, cx, cy)
            print(f"   dab ({cx}, {cy}): area {n}, centroid offset ({c[0] - cx:+.2f}, {c[1] - cy:+.2f}), "
                  f"major axis at {ang:+.1f} deg, half-axes {l1:.2f} x {l2:.2f}")
        best = None
        for short in ("y", "x"):
            for sign in (1, -1):
              for off in (0.0, -0.5):
                bad = tot = 0
                for cx, cy in TIP_POINTS:
                    for y in range(int(cy) - 25, int(cy) + 26):
                        for x in range(int(cx) - 25, int(cx) + 26):
                            p = inside(x + 0.5 - cx - off, y + 0.5 - cy, t, sign * A, short)
                            o = a[x, y] > 0
                            if p or o:
                                tot += 1
                                bad += p != o
                if best is None or bad < best[0]:
                    best = (bad, tot, short, sign, off)
        print(f"   closed ellipse: short axis along {best[2]} (before rotation), angle "
              f"{'+' if best[3] > 0 else '-'}A, centre x offset {best[4]:+.1f}: "
              f"{best[1] - best[0]}/{best[1]} px exact")


if __name__ == "__main__":
    main()
