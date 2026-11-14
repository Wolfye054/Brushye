#!/usr/bin/env python3
"""B-002 follow-up: stroke options (pure Python + Pillow).

Exports (brush_run2.py "rows": three straight strokes 40.5 -> 560.5 at y = 80.5 (260 events,
5 ms), 200.5 (260 events, 20 ms), 320.5 (52 events, 20 ms)); brush size 40, AA None,
hardness 5, density 30, opacity 100, gap Normal:
  B002_rows_base.png        baseline
  B002_rows_adjustgap.png   Brush tip > "Adjust brush density by gap" ON
  B002_rows_novelocity.png  Stroke > "Correct velocity input" OFF
The baseline is checked against the B-002/B-003 model (dabs every 3.2 px from the press point,
B = 1 - 0.7^n, a8 = floor(65535 B) >> 8) with the AA None scanline rule of the B-001 follow-up
(|dy| <= R, |dx| <= sqrt(R^2 - dy^2) + 0.1). The option exports are compared with the baseline,
and the centre-row plateau level is reported.
usage: b002_options.py DATA_DIR
"""
import math
import os
import sys

from PIL import Image

ROWS = (80.5, 200.5, 320.5)
R, S, D = 20.0, 3.2, 0.3


def model_alpha(px, py, y0):
    dy = py + 0.5 - y0
    if dy * dy > R * R:
        return 0
    w = math.sqrt(R * R - dy * dy) + 0.1
    n = 0
    k = 0
    while True:
        cx = 40.5 + (k * 8 * 40) / 100
        if cx > 560.5 + 1e-9:
            break
        if abs(px + 0.5 - cx) <= w + 1e-12:
            n += 1
        k += 1
    return int(65535 * (1 - (1 - D) ** n) + 1e-7) >> 8


def main():
    d = sys.argv[1]
    base = Image.open(f"{d}/B002_rows_base.png").getchannel("A")
    a = base.load()
    tot = bad = 0
    for y0 in ROWS:
        for py in range(int(y0) - 22, int(y0) + 22):
            for px in range(0, 600):
                p = model_alpha(px, py, y0)
                o = a[px, py]
                if p == 0 and o == 0:
                    continue
                tot += 1
                bad += p != o
    print(f"baseline vs model (span rule): {tot - bad}/{tot} px exact")
    for name in ("adjustgap", "novelocity"):
        f = f"{d}/B002_rows_{name}.png"
        if not os.path.exists(f):
            continue
        b = Image.open(f).getchannel("A").load()
        diff = sum(1 for py in range(400) for px in range(600) if a[px, py] != b[px, py])
        plateau = [b[300, int(y0)] for y0 in ROWS]
        edge = [b[30, int(y0)] for y0 in ROWS]
        print(f"{name}: {diff} px differ from the baseline; centre-row level at x=300: {plateau} "
              f"(baseline {[a[300, int(y0)] for y0 in ROWS]}); at x=30 (single dab region?): {edge}")
        if diff:
            ramp = [b[x, int(ROWS[0])] for x in range(20, 80, 3)]
            print(f"   start of row 1, x = 20..77 step 3: {ramp}")


def adjust_fit(d):
    """'Adjust brush density by gap': per-dab density d' implied by each
    B002_rows_adjustgap[_wide|_narrow|_d60].png. Constraints, with CSP's truncation
    (8-bit level v means alpha in [v*256/65535, (v+1)*256/65535)):
      plateau at (300, 80), covered by n dabs of the lattice: 1 - (1 - d')^n in that range;
      single dab near the press point (21, 80): d' in that range.
    Models checked: d' = 3 D k, and d' = (round(255 D) / 255) * 3k (D = density, k = gap fraction)."""
    out = []
    lo_v = lambda v: v * 256 / 65535
    for suffix, k, D in (("_narrow", 0.04, 0.3), ("", 0.08, 0.3), ("_wide", 0.12, 0.3), ("_d60", 0.08, 0.6)):
        f = f"{d}/B002_rows_adjustgap{suffix}.png"
        if not os.path.exists(f):
            continue
        a = Image.open(f).getchannel("A").load()
        s = max(1.0, k * 40)
        n = sum(1 for j in range(400) if abs(300.5 - (40.5 + j * s)) <= 20.1 and 40.5 + j * s <= 560.5)
        lv, one = a[300, 80], a[21, 80]
        lo = max(1 - (1 - lo_v(lv)) ** (1 / n), lo_v(one))
        hi = min(1 - (1 - lo_v(lv + 1)) ** (1 / n), lo_v(one + 1))
        m1 = 3 * D * k
        m2 = int(255 * D + 0.5) / 255 * 3 * k
        tag = lambda m: f"{m:.5f} {'inside' if lo <= m < hi else 'OUTSIDE'}"
        out.append((k, D, n, lv, one, lo, hi, m1, m2))
        print(f"gap k={k} density {D}: plateau {lv} (n={n}), single dab {one} -> d' in [{lo:.5f}, {hi:.5f}); "
              f"3Dk {tag(m1)}; (D8/255)3k {tag(m2)}")
    return out


def ramp_check(d):
    """B002_ramp.png: size 20 with pen-pressure size (min 0, linear), gap Fixed 300, AA None,
    density 100, straight stroke 40.5 -> 560.5 at y 200.5 with vpen pressure p 0.2 -> 1.0
    (brush_run2 RAMP). CSP pressure P = (p - 0.0099) / 0.9901 (B-004 input chain). Dab centres
    = centroids of the isolated dabs (dab_centres.py). Models for the next dab position:
      'current': x_{k+1} = x_k + 3 * 20 * P(x_k)   (gap from the dab just placed)
      'next':    x_{k+1} = x_k + 3 * 20 * P(x_{k+1})"""
    from brush_run2 import RAMP
    from dab_centres import centres
    (x0, _), (x1, _), p0, p1, _ = RAMP
    obs = sorted(c[0] for c in centres(f"{d}/B002_ramp.png", 4))

    def P(x):
        p = p0 + (p1 - p0) * (x - x0) / (x1 - x0)
        return (p - 0.0099) / 0.9901

    for name in ("current", "next"):
        xs = [x0]
        while True:
            x = xs[-1]
            nx = x + 60 * P(x)
            if name == "next":
                for _ in range(50):
                    nx = x + 60 * P(nx)
            if nx > x1 + 1e-9:
                break
            xs.append(nx)
        err = [abs(a - b) for a, b in zip(xs, obs)]
        print(f"ramp {name}: model {len(xs)} dabs, observed {len(obs)}, max |dx| {max(err):.2f}, "
              f"mean {sum(err) / len(err):.3f}")


if __name__ == "__main__":
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    main()
    adjust_fit(sys.argv[1])
    if os.path.exists(f"{sys.argv[1]}/B002_ramp.png"):
        ramp_check(sys.argv[1])
