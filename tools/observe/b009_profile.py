#!/usr/bin/env python3
"""B-009: radial alpha profile of a size-101 dab at canvas (300.5, 200.5).
usage: b009_profile.py EXPORT.png [EXPORT2 ...]  -> alpha at integer distances 0..55 along +x,
plus area (sum alpha/255) and the per-distance spread (min..max over all pixels at that distance)."""
import sys, numpy as np
from PIL import Image
for f in sys.argv[1:]:
    a = np.asarray(Image.open(f).convert("RGBA")).astype(int)[..., 3]
    row = a[200, 300:356]
    ys, xs = np.mgrid[0:a.shape[0], 0:a.shape[1]]
    d = np.hypot(xs + 0.5 - 300.5, ys + 0.5 - 200.5)
    print(f"{f.split('/')[-1]}: area {a.sum()/255:9.2f}  max {a.max()}  nonzero radius {d[a>0].max():.2f}")
    print("  +x profile:", " ".join(str(v) for v in row))
