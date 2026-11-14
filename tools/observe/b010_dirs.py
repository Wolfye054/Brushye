#!/usr/bin/env python3
"""B-010 follow-up: tip angle "Direction of line" (pure Python + Pillow).

Export B010_dirs_line.png: brush_run2.py "dirs2" (8 strokes of 120 px in directions k*45 deg,
screen y down, events every 2 px), size 41, AA None, thickness 30, direction Vertical, angle 0
with angle dynamics "Direction of line", gap Fixed 150 % (61.5 px), so each stroke leaves two
isolated dabs: one at the press point and one 61.5 px along the stroke.
For every dab (connected component): centroid and the major-axis angle from second moments,
reported counter-clockwise on screen (y up), next to the stroke direction in the same sense.
usage: b010_dirs.py DATA_DIR
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from brush_run2 import dirs2_strokes  # noqa: E402
from b010_tip import moments  # noqa: E402
from dab_centres import centres  # noqa: E402
from PIL import Image  # noqa: E402


def main():
    f = f"{sys.argv[1]}/B010_dirs_line.png"
    a = Image.open(f).getchannel("A").load()
    found = centres(f, 20)
    for k, pts in enumerate(dirs2_strokes()):
        d_ccw = (-45 * k) % 360  # stroke direction, counter-clockwise with y up
        sx, sy = pts[0]
        mine = sorted((math.hypot(cx - sx, cy - sy), cx, cy) for cx, cy, _ in found
                      if math.hypot(cx - sx, cy - sy) < 130)
        desc = []
        for dist, cx, cy in mine:
            _, _, ang, l1, l2 = moments(a, cx, cy)
            desc.append(f"dab at {dist:5.1f} px: major axis {ang:+6.1f} deg ({l1:.1f} x {l2:.1f})")
        print(f"stroke {k} direction {d_ccw:3d} deg (ccw): " + "; ".join(desc))


if __name__ == "__main__":
    main()
