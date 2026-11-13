#!/usr/bin/env python3
"""B-001: check the single-dab model against every dab of the stored exports, no free parameters
(pure Python + Pillow). Reproduces the table in specs/behavior/brush/B-001-round-tip-single-dab.md.

Grid (B001_aa<L>.png, 600x400): sizes SIZES in rows y = 25 + 46*r, three columns with the pen
sent to (60.5, y+0.5), (200.75, y+0.5), (341.0, y+1.0) canvas px. vpen.py resolves positions to
0.1 px (10 units per px, rounded), so column 1 actually arrives at x = 200.8; that is a harness
artifact and the centres below use the resolved positions.

Model (d = distance from the dab centre to the pixel centre, R = size/2, A = 65535 * coverage,
8-bit alpha = A >> 8, i.e. truncation):
  AA None (L=0):  coverage = 1 if d <= R (closed; "--open" for d < R)
  AA Weak/Middle/Strong (L=1,2,3): coverage = min(R,1) * clamp((max(R,1) - d) / W, 0, 1),
                  W = max(1, min(W_L, R)),  W_L = 1.5, 2.5, 3.5
                  (for size >= 2 this is clamp((R - d)/W); smaller dabs keep radius 1 and scale)
usage: b001_check.py DATA_DIR [--open]
"""
import math
import sys

from PIL import Image

SIZES = [0.7, 1, 1.5, 2, 3, 4.5, 10, 25.3]
COLS = [(60.5, 0.5), (200.8, 0.5), (341.0, 1.0)]  # resolved pen x, y offset within the row
W_L = {1: 1.5, 2: 2.5, 3: 3.5}


def model(d, R, level, open_rule):
    if level == 0:
        return 255 if (d < R if open_rule else d <= R + 1e-9) else 0
    Re = max(R, 1.0)                     # dabs smaller than 2 px keep a 1 px radius ...
    amp = min(R, 1.0)                    # ... and are scaled down in amplitude instead
    W = max(1.0, min(W_L[level], R))
    t = amp * min(1.0, max(0.0, (Re - d) / W))
    return int(65535 * t + 1e-7) >> 8


def main():
    d0 = sys.argv[1]
    open_rule = "--open" in sys.argv
    grand = [0, 0]
    for level in range(4):
        a = Image.open(f"{d0}/B001_aa{level}.png").getchannel("A")
        line = []
        for r, size in enumerate(SIZES):
            R = size / 2
            for cx, dy in COLS:
                cy = 25 + 46 * r + dy
                tot = bad = worst = 0
                for y in range(int(cy) - 16, int(cy) + 17):
                    for x in range(int(cx) - 16, int(cx) + 17):
                        p = model(math.hypot(x + 0.5 - cx, y + 0.5 - cy), R, level, open_rule)
                        o = a.getpixel((x, y))
                        if p or o:
                            tot += 1
                            if p != o:
                                bad += 1
                                worst = max(worst, abs(p - o))
                grand[0] += tot; grand[1] += bad
                line.append(f"{size}@{cx}: {tot - bad}/{tot}" + (f" (max {worst})" if bad else ""))
        print(f"AA level {level}:")
        for i in range(0, len(line), 3):
            print("   " + " | ".join(line[i:i + 3]))
    print(f"all dabs: {grand[0] - grand[1]}/{grand[0]} px exact "
          f"({100 * (grand[0] - grand[1]) / grand[0]:.2f}%)  edge rule {'open' if open_rule else 'closed'}")


if __name__ == "__main__":
    main()
