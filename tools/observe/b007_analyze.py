#!/usr/bin/env python3
"""B-007 stabilization: fit per-event smoothing models to zigzag strokes (pure Python + Pillow).

Exports B007_zigzag_s<S>_ms<MS>[_speed].png: brush_run2.py "zigzag" (10 segments 50 px wide,
+-40 px, events every ~2 px, MS ms apart), size 4, AA None, density 100, gap Narrow, so the
stroke is a thin continuous line. The observed path is the alpha-weighted centre y of each
column (x is monotone along the zigzag), plus the stroke's x extent.

Models (the smoothed points are then drawn as before; with 2 px events the path smoothing of
B-002 is negligible):
  ema N     q_k = q_{k-1} + (p_k - q_{k-1}) / N            (first point = press point)
  mean N    q_k = mean of the last N events (fewer at the start)
  rope N    lazy string of length N/4 px
  mean2 N   moving mean applied twice
For each export the best N of each family is reported with the RMS and max |dy| over the
columns, and where the modelled stroke ends vs the observed end.
usage: b007_analyze.py DATA_DIR
"""
import glob
import math
import os
import sys

from PIL import Image

sys.path.insert(0, os.path.dirname(__file__))
from brush_run2 import zigzag_points  # noqa: E402


def centreline(path):
    a = Image.open(path).getchannel("A")
    w, h = a.size
    px = a.load()
    out = {}
    for x in range(w):
        s = sy = 0
        for y in range(h):
            v = px[x, y]
            if v:
                s += v
                sy += v * (y + 0.5)
        if s:
            out[x] = sy / s
    return out


def ema(pts, n):
    q = [pts[0]]
    for p in pts[1:]:
        l = q[-1]
        q.append((l[0] + (p[0] - l[0]) / n, l[1] + (p[1] - l[1]) / n))
    return q


def mean(pts, n):
    q = []
    for k in range(len(pts)):
        w = pts[max(0, k - n + 1):k + 1]
        q.append((sum(p[0] for p in w) / len(w), sum(p[1] for p in w) / len(w)))
    return q


def rope(pts, n):
    """Lazy string of length n/4 px: q follows p only when p is farther than the string."""
    L = n / 4
    q = [pts[0]]
    for p in pts[1:]:
        l = q[-1]
        d = math.hypot(p[0] - l[0], p[1] - l[1])
        q.append(l if d <= L else (p[0] - (p[0] - l[0]) * L / d, p[1] - (p[1] - l[1]) * L / d))
    return q


def mean2(pts, n):
    """Moving mean applied twice (window n each)."""
    return mean(mean(pts, n), n)


def path_y(q):
    """y at each integer column centre, by linear interpolation along the smoothed polyline."""
    out = {}
    for (x0, y0), (x1, y1) in zip(q, q[1:]):
        if x1 == x0:
            continue
        lo, hi = sorted((x0, x1))
        for x in range(math.ceil(lo - 0.5), math.floor(hi - 0.5) + 1):
            c = x + 0.5
            out.setdefault(x, y0 + (y1 - y0) * (c - x0) / (x1 - x0))
    return out


def score(obs, q):
    m = path_y(q)
    d = [obs[x] - m[x] for x in obs if x in m]
    if not d:
        return 1e9, 1e9
    return math.sqrt(sum(v * v for v in d) / len(d)), max(abs(v) for v in d)


def main():
    pts = zigzag_points()
    for f in sorted(glob.glob(f"{sys.argv[1]}/B007_zigzag_*.png")):
        obs = centreline(f)
        xs = sorted(obs)
        line = f"{os.path.basename(f)}: ink x {xs[0]}..{xs[-1]} (input ends at x {pts[-1][0]:.1f})"
        for name, fn in (("ema", ema), ("mean", mean), ("rope", rope), ("mean2", mean2)):
            best = min((score(obs, fn(pts, n)) + (n,) for n in range(1, 201)), key=lambda t: t[0])
            q = fn(pts, best[2])
            line += f"\n   {name:5s} N={best[2]:3d}: rms {best[0]:.3f} px, max {best[1]:.2f}; model ends x {q[-1][0]:.1f}"
        print(line)


if __name__ == "__main__":
    main()
