#!/usr/bin/env python3
"""B-009 follow-up: soft-tip centre and size scaling (pure Python + Pillow).

Exports B009_run2_h<L>_s11.png and B009_run2_h<L>_s41.png (L = 1..5; the size is a brush
setting, so each size is its own export), dabs per brush_run2.HARD_POINTS:
  size 11 at (40.5, 40.5); size 41 at (120.5, 60.5);
  size 41 at x = 60.5 + 60k + k/10 (k = 0..4, y = 160.5) and x = 60.5 + 60k + (k+5)/10 (y = 260.5);
  size 11 at x = 40.5 + 30k + k/10 (k = 0..9, y = 340.5).
For each dab: alpha-weighted centroid minus press point (the dab's centre offset), and its
peak. Then the radial profile vs d/R around the measured centre is compared with the size-101
profile of run 1 (data/B009_h<L>_aa0.png, centre (301, 201)) to test pure scaling by R.
usage: b009_run2_analyze.py DATA_DIR
"""
import math
import os
import sys

from PIL import Image

sys.path.insert(0, os.path.dirname(__file__))
from brush_run2 import HARD_POINTS  # noqa: E402


def centroid(a, x, y, r):
    sw = sx = sy = 0.0
    for py in range(int(y - r), int(y + r) + 1):
        for px in range(int(x - r), int(x + r) + 1):
            v = a[px, py]
            sw += v
            sx += v * (px + 0.5)
            sy += v * (py + 0.5)
    return (sx / sw, sy / sw, sw / 255) if sw else (x, y, 0)


def profile(a, cx, cy, R, reach, nb=40):
    """Mean alpha per d/R bin (width 1/nb), bins up to reach/R."""
    bins = {}
    for py in range(int(cy - reach) - 1, int(cy + reach) + 2):
        for px in range(int(cx - reach) - 1, int(cx + reach) + 2):
            t = math.hypot(px + 0.5 - cx, py + 0.5 - cy) / R
            k = int(t * nb)
            s = bins.setdefault(k, [0, 0])
            s[0] += a[px, py]
            s[1] += 1
    return {k: v[0] / v[1] for k, v in bins.items()}


def main():
    d = sys.argv[1]
    for lvl in range(1, 6):
        paths = [f"{d}/B009_run2_h{lvl}_s{s}.png" for s in (11, 41)]
        if not all(os.path.exists(p) for p in paths):
            continue
        # the two exports hold disjoint dabs: merge them by maximum
        im = Image.open(paths[0]).getchannel("A")
        im = im.point(lambda v: v)
        from PIL import ImageChops
        a = ImageChops.lighter(im, Image.open(paths[1]).getchannel("A")).load()
        print(f"hardness {lvl}:")
        offs = []
        for size, x, y in HARD_POINTS:
            reach = size * 0.75 + 3
            cx, cy, area = centroid(a, x, y, reach)
            peak = max(a[px, py] for py in range(int(y) - 2, int(y) + 3) for px in range(int(x) - 2, int(x) + 3))
            offs.append((size, x, cx - x, cy - y, area, peak))
        for size, x, ox, oy, area, peak in offs:
            print(f"  size {size:3} press x {x:8.2f} (frac {x % 1:.1f}): centre offset ({ox:+.3f}, {oy:+.3f}) area {area:8.2f} peak {peak}")
        if lvl < 5:
            ref = Image.open(f"{d}/B009_h{lvl}_aa0.png").getchannel("A").load()
            pr = profile(ref, 301.0, 201.0, 50.5, 75)
            for size, x, y in HARD_POINTS[:2]:
                cx, cy, _ = centroid(a, x, y, size * 0.75 + 3)
                p = profile(a, cx, cy, size / 2, size * 0.75 + 2)
                diffs = [abs(p[k] - pr[k]) for k in p if k in pr and k < 56]
                print(f"  size {size} profile vs size-101 scaled by R: max |diff| {max(diffs):.1f}, mean {sum(diffs) / len(diffs):.2f}")


if __name__ == "__main__":
    main()
