#!/usr/bin/env python3
"""B-004 follow-up: pressure curves with 2 and 3 control points (pure Python + Pillow).

Exports (brush_run2.py "pgrid": 21 constant-pressure strokes, P = 0 .. 1 step 0.05, size 100,
AA Middle, density 100, size dynamics pen pressure, minimum 0):
  B004_pgrid_linear.png  linear curve (calibration)
  B004_pgrid_2pt.png     control points (0.25, 0.75), (0.75, 0.25)
  B004_pgrid_3pt.png     control points (0.25, 0.75), (0.5, 0.25), (0.75, 0.25)
Width of a stroke = mean over 40 central columns of sum(alpha)/255 across the stroke. The
effective curve output C(P) is the linear-run P whose width equals the measured width
(linear interpolation in the calibration table). Models compared with C(P):
  bspline   quadratic B-spline on the control polygon (0,0), points…, (1,1): Bezier pieces
            joined at the midpoints between interior points (Brushye's current model)
  bezier    one Bezier curve of degree n+1 through all control points
  polyline  straight lines through the points
usage: b004_curves.py DATA_DIR
"""
import math
import os
import sys

from PIL import Image

sys.path.insert(0, os.path.dirname(__file__))
from brush_run2 import pgrid_cells  # noqa: E402

CURVES = {"2pt": [(0.25, 0.75), (0.75, 0.25)],
          "3pt": [(0.25, 0.75), (0.5, 0.25), (0.75, 0.25)]}


def widths(path):
    a = Image.open(path).getchannel("A").load()
    out = []
    for P, x0, y in pgrid_cells():
        cols = [sum(a[x, yy] for yy in range(y - 54, y + 55)) / 255 for x in range(x0 + 40, x0 + 80)]
        out.append((P, sum(cols) / len(cols)))
    return out


def invert(cal, w):
    for (p0, w0), (p1, w1) in zip(cal, cal[1:]):
        if w0 <= w <= w1 and w1 > w0:
            return p0 + (p1 - p0) * (w - w0) / (w1 - w0)
    return 0.0 if w <= cal[0][1] else 1.0


def quad_y(a, b, c, x):
    qa, qb, dx = a[0] - 2 * b[0] + c[0], 2 * (b[0] - a[0]), x - a[0]
    den = qb + math.sqrt(max(qb * qb + 4 * qa * dx, 0.0))
    u = min(max(2 * dx / den, 0.0), 1.0) if den > 0 else 0.0
    v = 1 - u
    return v * v * a[1] + 2 * u * v * b[1] + u * u * c[1]


def bspline(pts, x):
    ctrl = [(0.0, 0.0)] + pts + [(1.0, 1.0)]
    n = len(pts)
    mid = lambda p, q: ((p[0] + q[0]) / 2, (p[1] + q[1]) / 2)
    for j in range(1, n + 1):
        s = ctrl[0] if j == 1 else mid(ctrl[j - 1], ctrl[j])
        e = ctrl[n + 1] if j == n else mid(ctrl[j], ctrl[j + 1])
        if x <= e[0] or j == n:
            return quad_y(s, ctrl[j], e, x)


def bezier(pts, x):
    ctrl = [(0.0, 0.0)] + pts + [(1.0, 1.0)]
    def at(u):
        c = list(ctrl)
        while len(c) > 1:
            c = [((1 - u) * p[0] + u * q[0], (1 - u) * p[1] + u * q[1]) for p, q in zip(c, c[1:])]
        return c[0]
    lo, hi = 0.0, 1.0
    for _ in range(60):
        m = (lo + hi) / 2
        lo, hi = (m, hi) if at(m)[0] < x else (lo, m)
    return at(lo)[1]


def polyline(pts, x):
    ctrl = [(0.0, 0.0)] + pts + [(1.0, 1.0)]
    for p, q in zip(ctrl, ctrl[1:]):
        if x <= q[0]:
            return p[1] + (q[1] - p[1]) * (x - p[0]) / (q[0] - p[0])
    return 1.0


def main():
    d = sys.argv[1]
    cal = widths(f"{d}/B004_pgrid_linear.png")
    for name, pts in CURVES.items():
        meas = [(P, invert(cal, w)) for P, w in widths(f"{d}/B004_pgrid_{name}.png")]
        print(f"{name}: measured C(P) = " + " ".join(f"{c:.3f}" for _, c in meas))
        for model in (bspline, bezier, polyline):
            err = [abs(model(pts, P) - c) for P, c in meas if 0 < P < 1]
            print(f"   {model.__name__:9s} max |error| {max(err):.4f}, mean {sum(err) / len(err):.4f}")


if __name__ == "__main__":
    main()
