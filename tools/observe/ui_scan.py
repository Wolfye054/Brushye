#!/usr/bin/env python3
"""Run-length colour scans of a screenshot (UI layout observation, specs U-001/U-009).
usage: ui_scan.py PNG row Y [X0 X1] | col X [Y0 Y1] ...  (min run length 3 px shown)"""
import sys
from PIL import Image
im = Image.open(sys.argv[1]).convert('RGB')
a = sys.argv[2:]
i = 0
while i < len(a):
    kind, v = a[i], int(a[i + 1]); i += 2
    lo, hi = 0, (im.width if kind == 'row' else im.height)
    if i + 1 < len(a) and a[i].lstrip('-').isdigit():
        lo, hi = int(a[i]), int(a[i + 1]); i += 2
    px = [im.getpixel((t, v) if kind == 'row' else (v, t)) for t in range(lo, hi)]
    runs, s = [], 0
    for t in range(1, len(px) + 1):
        if t == len(px) or px[t] != px[s]:
            if t - s >= 3:
                runs.append(f"{lo+s}-{lo+t-1}:{px[s]}")
            s = t
    print(f"{kind} {v}: " + "  ".join(runs))
