#!/usr/bin/env python3
"""B-002 follow-up: how CSP turns the input events into a path (pure Python + Pillow).

Exports B002_arc_<N>.png / B002_poly_<0|1>.png (layouts in brush_run2.py), brush size 6,
AA None, density 100, gap Fixed 300 (= 18 px between dabs), so each dab is isolated and its
centre is the centroid of its connected component (dab_centres.py).

Path models, dabs every S px of arc length from the press point, none past the path end:
  poly      the straight polyline through the events (Brushye run 1)
  bspline   quadratic B-spline with the events as control points: straight from P0 to the
            midpoint m1 of P0P1, then for each interior event Pi the quadratic Bezier
            (m_i, P_i, m_{i+1}), then straight from the last midpoint to the last event
  bspline_m same, but the path ends at the last midpoint (no final straight piece)
Each model dab is matched to the nearest observed centre; reports dab counts and distances.
usage: b002_smoothing.py DATA_DIR [S]
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from brush_run2 import arc_points, poly_points  # noqa: E402
from dab_centres import centres  # noqa: E402

SUB = 2000  # Bezier subdivision for arc length


def mid(a, b):
    return ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2)


def path(pts, model):
    """Dense polyline approximating the model path."""
    if model == "poly" or len(pts) < 3:
        return list(pts)
    out = [pts[0], mid(pts[0], pts[1])]
    for i in range(1, len(pts) - 1):
        a, b, c = mid(pts[i - 1], pts[i]), pts[i], mid(pts[i], pts[i + 1])
        for k in range(1, SUB + 1):
            u = k / SUB
            v = 1 - u
            out.append((v * v * a[0] + 2 * u * v * b[0] + u * u * c[0],
                        v * v * a[1] + 2 * u * v * b[1] + u * u * c[1]))
    if model == "bspline":
        out.append(pts[-1])
    return out


def lattice(poly, s):
    out, total, k = [], 0.0, 0
    cum = [0.0]
    for p, q in zip(poly, poly[1:]):
        total += math.hypot(q[0] - p[0], q[1] - p[1])
        cum.append(total)
    j = 0
    while k * s <= total + 1e-9:
        t = k * s
        while j + 1 < len(cum) - 1 and cum[j + 1] < t:
            j += 1
        L = cum[j + 1] - cum[j] if len(cum) > 1 else 0
        u = (t - cum[j]) / L if L else 0
        p, q = poly[j], poly[min(j + 1, len(poly) - 1)]
        out.append((p[0] + (q[0] - p[0]) * u, p[1] + (q[1] - p[1]) * u))
        k += 1
    return out, total


def main():
    d = sys.argv[1]
    s = float(sys.argv[2]) if len(sys.argv) > 2 else 18.0
    for name, pts in (("arc_200", arc_points(200)), ("arc_12", arc_points(12)),
                      ("poly_1", poly_points(True)), ("poly_0", poly_points(False))):
        f = f"{d}/B002_{name}.png"
        if not os.path.exists(f):
            continue
        obs = [(x, y) for x, y, _ in centres(f, 4)]
        for model in ("poly", "bspline", "bspline_m"):
            dabs, total = lattice(path(pts, model), s)
            dist = [min(math.hypot(mx - ox, my - oy) for ox, oy in obs) for mx, my in dabs]
            back = [min(math.hypot(mx - ox, my - oy) for mx, my in dabs) for ox, oy in obs]
            print(f"{name:8s} {model:9s}: length {total:7.2f}, dabs model {len(dabs):3d} / observed "
                  f"{len(obs):3d}; model->obs max {max(dist):6.2f} mean {sum(dist) / len(dist):.3f}; "
                  f"obs->model max {max(back):6.2f}")


if __name__ == "__main__":
    main()
