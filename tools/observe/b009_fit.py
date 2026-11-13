#!/usr/bin/env python3
"""B-009: fit candidate falloff models to the size-101 hardness dabs (all pixels).
Models (d = distance pen->pixel centre, R = 50.5):
  erf:    A * 0.5 * erfc((d - c) / (s * sqrt 2))          (hard disc blurred by a Gaussian)
  gauss:  A * exp(-d^2 / (2 s^2)) cut at d > c
  smooth: 255 * smoothstep from c+s down to c-s
Prints the best parameters, exact-match %, mean and max abs error over pixels with data > 0.
usage: b009_fit.py DIR"""
import sys, math, numpy as np
from PIL import Image
d0 = sys.argv[1]
vec_erfc = np.vectorize(math.erfc)
for lvl in (1, 2, 3, 4):
    a = np.asarray(Image.open(f"{d0}/B009_h{lvl}_aa0.png").convert("RGBA")).astype(float)[..., 3]
    ys, xs = np.mgrid[100:301, 200:401]
    d = np.hypot(xs + 0.5 - 300.5, ys + 0.5 - 200.5)
    p = a[100:301, 200:401]
    m = p > 0
    dd, pp = d[m], p[m]
    best = {}
    A0 = p.max()
    for c in np.arange(20, 55.01, 0.25):
        for s in np.arange(0.5, 25.01, 0.25):
            for name in ("erf", "gauss"):
                if name == "erf":
                    A = 255.0
                    pred = A * 0.5 * vec_erfc((dd - c) / (s * math.sqrt(2)))
                    # amplitude so the centre matches
                    A = 255.0 if A0 >= 255 else A0 / (0.5 * math.erfc((0 - c) / (s * math.sqrt(2))))
                    pred = np.minimum(255, A * 0.5 * vec_erfc((dd - c) / (s * math.sqrt(2))))
                else:
                    if c < 40: continue
                    pred = np.where(dd <= c, A0 * np.exp(-dd**2 / (2 * (s * 2.5) ** 2)), 0)
                e = np.abs(np.rint(pred) - pp)
                sc = (e.mean(), e.max())
                if name not in best or sc < best[name][0]:
                    best[name] = (sc, c, s, (e == 0).mean())
    for name, (sc, c, s, ex) in best.items():
        print(f"h{lvl} {name:6s}: c={c:6.2f} s={s:5.2f} exact {ex*100:5.1f}% mean {sc[0]:5.2f} max {sc[1]:.0f}")
