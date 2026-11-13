#!/usr/bin/env python3
"""B-002 stroke gap analysis (pure Python + Pillow). Reproduces the numbers in
specs/behavior/brush/B-002-stroke-spacing.md from the stored exports.

Brush: round tip, hardness max, AA None, density 30, opacity 100. Every dab is a hard disc;
within one stroke n overlapping dabs give alpha LEV(n) = floor(65535*(1-0.7^n) / 256)
(observed table 76,130,168,194,212,225,234,241,245,248,250,252,253,254,254,255; see B-003).

Model tested on the centre row of each horizontal stroke:
    spacing      s   = max(1, pct * size)      (pct: Narrow .04, Normal .08, Wide .12, Fixed v -> v/100)
    dab centres  c_k = press + k*s,  k = 0..floor((release-press)/s)   (computed directly, no
                 accumulation; no extra dab at the release point)
    coverage     rule "closed":  |p - c| <= R               (R = size/2)
                 rule "ss4":     >= 8 of 16 sub-samples (4x4 grid at (i+.5)/4) inside the disc
A free scan of s (closed rule) is also reported, so the spacing is measured, not assumed.

usage: b002_analyze.py PNG ROW [ROW ...]
  ROW = "label:y:size:press_x:release_x:pct"  (y = pixel row of the stroke axis, canvas px;
        the stroke axis is at y+0.5)
"""
import math
import sys

from PIL import Image

EPS = 1e-9


def lev(n):
    return min(255, int(65535 * (1 - 0.7 ** n)) // 256)


def centres(press, release, s, extra_end=False):
    N = math.floor((release - press) / s + EPS) + 1
    cs = [press + k * s for k in range(N)]
    if extra_end and release - cs[-1] > EPS:
        cs.append(release)
    return cs


def covered(px, py, c, cy, R, rule):
    if rule == "closed":
        return math.hypot(px - c, py - cy) <= R + EPS
    hits = 0
    for i in range(4):
        for j in range(4):
            sx = px - 0.5 + (i + 0.5) / 4
            sy = py - 0.5 + (j + 0.5) / 4
            hits += (sx - c) ** 2 + (sy - cy) ** 2 <= R * R
    return hits >= 8


def mismatches(alpha, y, R, cs, rule):
    bad = []
    for x, a in enumerate(alpha):
        px = x + 0.5
        n = sum(covered(px, y + 0.5, c, y + 0.5, R, rule) for c in cs if abs(px - c) <= R + 1)
        if lev(n) != a:
            bad.append(x)
    return bad


def closed_count_row(width, R, press, release, s):
    """Fast closed-rule dab count per pixel of the centre row (no end dab)."""
    N = math.floor((release - press) / s + EPS) + 1
    out = []
    for x in range(width):
        px = x + 0.5
        lo = max(0, math.ceil((px - R - press) / s - EPS))
        hi = min(N - 1, math.floor((px + R - press) / s + EPS))
        out.append(max(0, hi - lo + 1))
    return out


def scan_spacing(alpha, R, press, release, guess):
    """All s (step 0.0005) with the minimum closed-rule mismatch, around the guess."""
    res = []
    s = max(0.5, guess * 0.85)
    while s <= guess * 1.15 + 0.01:
        n = closed_count_row(len(alpha), R, press, release, s)
        res.append((sum(lev(v) != a for v, a in zip(n, alpha)), s))
        s += 0.0005
    b = min(r[0] for r in res)
    ok = [r[1] for r in res if r[0] == b]
    return b, min(ok), max(ok)


def main():
    img = Image.open(sys.argv[1]).getchannel("A")
    print(f"# {sys.argv[1].split('/')[-1]}")
    for spec in sys.argv[2:]:
        label, y, size, press, release, pct = spec.split(":")
        y = int(y); size = float(size); press = float(press); release = float(release)
        pct = float(pct)
        R = size / 2
        alpha = [img.getpixel((x, y)) for x in range(img.width)]
        s = max(1.0, pct * size)
        cs = centres(press, release, s)
        b_closed = mismatches(alpha, y, R, cs, "closed")
        b_ss4 = mismatches(alpha, y, R, cs, "ss4")
        b_end = mismatches(alpha, y, R, centres(press, release, s, extra_end=True), "ss4")
        fb, smin, smax = scan_spacing(alpha, R, press, release, s)
        print(f"{label:>8} size {size:5.1f} model s={s:.4f} dabs={len(cs)} last={cs[-1]:.3f}"
              f" | mismatch closed={len(b_closed)} ss4={len(b_ss4)} ss4+release-dab={len(b_end)}"
              f" | free scan: s in [{smin:.4f},{smax:.4f}] ({fb} mismatch)"
              + (f" | ss4 mismatches at x={b_ss4[:8]}" if b_ss4 else ""))


if __name__ == "__main__":
    main()
