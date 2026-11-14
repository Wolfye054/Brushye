#!/usr/bin/env python3
"""B-001 follow-up: where do dabs really land? (pure Python + Pillow)

Fits each dab centre of an AA Weak/Middle sweep export (B001_sweep_aa<L>_s<SIZE>.png, layout
brush_run2.sweep_points) with the exact B-001 AA model, by grid search in 1/100 px within
+-0.3 px of the press point we sent, and prints the offset (fitted - sent) per sub-pixel offset.
Then the AA None sweep of the same size is re-scored with the closed disc around the FITTED
centres: if that is exact, the AA None "edge rule" is just input-position error.

usage: b001_centres.py DATA_DIR SIZE LEVEL
"""
import math
import os
import sys

from PIL import Image

sys.path.insert(0, os.path.dirname(__file__))
from brush_run2 import sweep_points  # noqa: E402

W_L = {1: 1.5, 2: 2.5, 3: 3.5}


def model(d, R, level):
    Re, amp = max(R, 1.0), min(R, 1.0)
    W = max(1.0, min(W_L[level], R))
    return int(65535 * amp * min(1.0, max(0.0, (Re - d) / W)) + 1e-7) >> 8


def fit(a, cx, cy, R, level):
    reach = int(R) + 4
    px0, py0 = int(cx) - reach, int(cy) - reach
    obs = [(px, py, a[px, py]) for py in range(py0, py0 + 2 * reach + 1)
           for px in range(px0, px0 + 2 * reach + 1)]
    best = None
    for ix in range(-30, 31):
        for iy in range(-30, 31):
            sx, sy = cx + ix / 100, cy + iy / 100
            err = 0
            for px, py, v in obs:
                p = model(math.hypot(px + 0.5 - sx, py + 0.5 - sy), R, level)
                err += abs(p - v)
                if best and err > best[0]:
                    break
            if best is None or err < best[0]:
                best = (err, ix / 100, iy / 100)
    return best


def main():
    d, size, level = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
    R = size / 2
    a = Image.open(f"{d}/B001_sweep_aa{level}_s{size}.png").getchannel("A").load()
    centres = {}
    print(f"size {size} AA level {level}: fitted offset (fitted - sent), residual sum |err|")
    for i, j, cx, cy in sweep_points(size):
        err, ox, oy = fit(a, cx, cy, R, level)
        centres[(i, j)] = (cx + ox, cy + oy)
        if j in (0, 5) or i in (0, 5):
            print(f"  sent frac ({i / 10:.1f},{j / 10:.1f}): offset ({ox:+.2f},{oy:+.2f}) residual {err}")
    oxs = [centres[k][0] - (sweep_points(size)[k[1] * 10 + k[0]][2]) for k in centres]
    print(f"  mean x offset {sum(oxs) / len(oxs):+.3f}")
    none = f"{d}/B001_sweep_s{size}.png"
    if os.path.exists(none):
        an = Image.open(none).getchannel("A").load()
        for name, rule in (("closed", lambda dd: dd <= R * R + 1e-9), ("open", lambda dd: dd < R * R - 1e-9)):
            bad = tot = 0
            for (i, j), (sx, sy) in centres.items():
                reach = int(R) + 3
                for py in range(int(sy) - reach, int(sy) + reach + 1):
                    for px in range(int(sx) - reach, int(sx) + reach + 1):
                        dd = (px + 0.5 - sx) ** 2 + (py + 0.5 - sy) ** 2
                        if abs(math.sqrt(dd) - R) > 1.5:
                            continue
                        tot += 1
                        bad += rule(dd) != (an[px, py] > 0)
            print(f"  AA None sweep, {name} disc around fitted centres: {tot - bad}/{tot} edge-band px")


if __name__ == "__main__":
    main()
