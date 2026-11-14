#!/usr/bin/env python3
"""B-009 follow-up: does the soft-tip profile scale with the radius? (pure Python + Pillow)

Predicts every pixel of the size-11 and size-41 dabs of run 2 (B009_run2_h<L>_s<SIZE>.png,
layout brush_run2.HARD_POINTS) from the size-101 export of run 1, exactly like Brushye's
interim model: radial table binned at STEP px around the soft centre (press + 0.5 px),
linear interpolation at d * 50.5 / R, 8-bit value = floor(65535 * v / 255) >> 8 with v the
interpolated alpha. Prints per level and size: px exact, within 1, within 4, max error.
usage: b009_scaling.py DATA_DIR
"""
import math
import os
import sys

from PIL import Image

sys.path.insert(0, os.path.dirname(__file__))
from brush_run2 import HARD_POINTS  # noqa: E402
from make_hardness_tables import table, STEP  # noqa: E402


def main():
    d = sys.argv[1]
    for lvl in range(1, 5):
        t = table(f"{d}/B009_h{lvl}_aa0.png")
        for size in (11, 41):
            a = Image.open(f"{d}/B009_run2_h{lvl}_s{size}.png").getchannel("A").load()
            R = size / 2
            n = ex = w1 = w4 = worst = 0
            for s, x, y in HARD_POINTS:
                if s != size:
                    continue
                cx, cy = x + 0.5, y + 0.5
                reach = int(R * 1.5) + 2
                for py in range(int(cy) - reach, int(cy) + reach + 1):
                    for px in range(int(cx) - reach, int(cx) + reach + 1):
                        dd = math.hypot(px + 0.5 - cx, py + 0.5 - cy)
                        pos = dd * 50.5 / R / STEP
                        i = int(pos)
                        if i + 1 >= len(t):
                            v = 0.0
                        else:
                            v = (t[i] + (t[i + 1] - t[i]) * (pos - i)) / 256
                        p = int(65535 * v / 255 + 1e-9) >> 8
                        o = a[px, py]
                        if p == 0 and o == 0:
                            continue
                        n += 1
                        e = abs(p - o)
                        ex += e == 0
                        w1 += e <= 1
                        w4 += e <= 4
                        worst = max(worst, e)
            print(f"hardness {lvl} size {size}: {n} px, exact {100 * ex / n:.1f} %, "
                  f"within 1 {100 * w1 / n:.1f} %, within 4 {100 * w4 / n:.1f} %, max {worst}")


if __name__ == "__main__":
    main()
