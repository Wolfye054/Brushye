#!/usr/bin/env python3
"""B-003 stroke accumulation analysis (pure Python + Pillow). Reproduces the numbers in
specs/behavior/brush/B-003-stroke-opacity-vs-density.md from the stored exports.

Layout (canvas px, see b003_strokes in the spec): stroke H from (40.5,100.5) to (280.5,100.5),
then a separate stroke V from (160.5,30.5) to (160.5,370.5), then one self-crossing polyline
stroke P in x > 300. Size 40, hardness max, AA None.

Model checked pixel by pixel on the region x < 300 (H and V):
    dab centres   c_k = press + k*s along each straight stroke (B-002), coverage |d| <= R.
                  The exact AA-None edge rule is open (B-002), so pixels within MARGIN of any dab
                  edge are scored separately; the accumulation model is judged on the others.
    stroke buffer B = 1 - (1 - density)^n          (n = dabs of that stroke covering the pixel;
                  dabs of one stroke accumulate, including where the stroke crosses itself)
    stroke alpha  = opacity * B                    (opacity scales the buffer, it is not a clamp)
    layer alpha   = H then V composited with "over": a = a_H + a_V * (1 - a_H)
    8-bit output  = floor(65535 * a) >> 8          (truncation, not rounding)
For region P (one stroke crossing itself) every observed level is mapped to the single-stroke
set opacity * (1 - (1-density)^n); a level outside that set would mean the self-crossing was
composited like a second stroke.

usage: b003_analyze.py PNG OPACITY DENSITY SPACING      (e.g. ... 0.5 0.5 1.6)
"""
import math
import sys
from collections import Counter

from PIL import Image

R = 20.0
EPS = 1e-9
MARGIN = 0.15  # pixels whose centre is within this distance of a dab edge are "edge" pixels


def q8(a):
    return int(65535 * a + 1e-7) >> 8


def lattice(p0, p1, s):
    (x0, y0), (x1, y1) = p0, p1
    L = math.hypot(x1 - x0, y1 - y0)
    N = math.floor(L / s + EPS) + 1
    return [(x0 + (x1 - x0) * k * s / L, y0 + (y1 - y0) * k * s / L) for k in range(N)]


def count(px, py, dabs):
    """(dab count with the closed rule |d| <= R, robust flag: no dab edge within MARGIN px)."""
    n, robust = 0, True
    for cx, cy in dabs:
        if abs(px - cx) > R + 1 or abs(py - cy) > R + 1:
            continue
        d = math.hypot(px - cx, py - cy)
        n += d <= R + EPS
        robust &= abs(d - R) > MARGIN
    return n, robust


def main():
    png, op, den, s = sys.argv[1], float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4])
    a = Image.open(png).getchannel("A")
    H = lattice((40.5, 100.5), (280.5, 100.5), s)
    V = lattice((160.5, 30.5), (160.5, 370.5), s)
    st = {True: [0, 0, 0], False: [0, 0, 0]}  # robust? -> [total, bad, worst]
    examples = []
    for y in range(400):
        for x in range(300):
            px, py = x + 0.5, y + 0.5
            nH, rH = count(px, py, H) if abs(py - 100.5) <= R + 1 else (0, True)
            nV, rV = count(px, py, V) if abs(px - 160.5) <= R + 1 else (0, True)
            aH = op * (1 - (1 - den) ** nH)
            aV = op * (1 - (1 - den) ** nV)
            pred = q8(aH + aV * (1 - aH))
            obs = a.getpixel((x, y))
            k = rH and rV
            st[k][0] += 1
            if pred != obs:
                st[k][1] += 1
                st[k][2] = max(st[k][2], abs(pred - obs))
                if k and len(examples) < 6:
                    examples.append((x, y, nH, nV, obs, pred))
    print(f"{png.split('/')[-1]}: opacity {op} density {den} spacing {s}")
    for k, name in ((True, "away from dab edges"), (False, f"within {MARGIN} px of a dab edge")):
        tot, bad, worst = st[k]
        print(f"  H+V {name}: {tot - bad}/{tot} px exact ({100 * (tot - bad) / max(tot, 1):.3f}%),"
              f" max |error| {worst}" + (f", e.g. (x,y,nH,nV,obs,pred) {examples}" if k and bad else ""))
    single = {q8(op * (1 - (1 - den) ** n)): n for n in range(0, 60)}
    lv = Counter(a.getpixel((x, y)) for y in range(400) for x in range(300, 600))
    print("  P region levels (alpha: count -> dabs within one stroke):",
          ", ".join(f"{v}:{c}->{single.get(v, 'NOT a single-stroke level')}" for v, c in sorted(lv.items())))


if __name__ == "__main__":
    main()
