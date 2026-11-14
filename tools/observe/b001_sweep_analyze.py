#!/usr/bin/env python3
"""B-001 follow-up: AA None edge rule from the sub-pixel sweep (pure Python + Pillow).

Each export B001_sweep_s<SIZE>.png holds 10x10 single dabs (AA None, density 100) pressed at
(origin + pitch*(i, j) + (i/10, j/10)), layout in brush_run2.py. For every dab and pixel, d is
the distance from the press point to the pixel centre. Candidate rules are scored on every
pixel within 1.5 px of the edge:
  closed   d <= R
  open     d <  R
  sub4/sub8  coverage of an NxN supersample grid >= 50 %
  floor    pixel (px, py) inside iff (px + 0.5 - cx)^2 + ... with the centre snapped to 1/8 px (round)
  shiftK   closed disc around (cx - K, cy - K), K in 1/16 steps
  span+E   scanline: row inside iff |dy| <= R, pixel inside iff |dx| <= sqrt(R^2 - dy^2) + E
           (E = 0.1 is the best rule found, 2026-11-13; the centres themselves are exact, see
           b001_centres.py)
usage: b001_sweep_analyze.py DATA_DIR [SIZE ...]
"""
import math
import sys

from PIL import Image

sys.path.insert(0, __import__("os").path.dirname(__file__))
from brush_run2 import sweep_points, SWEEP_PITCH  # noqa: E402


def rules(R):
    def closed(dx, dy):
        return dx * dx + dy * dy <= R * R + 1e-9

    def opn(dx, dy):
        return dx * dx + dy * dy < R * R - 1e-9

    def sub(n):
        def f(dx, dy):
            c = 0
            for a in range(n):
                for b in range(n):
                    x = dx - 0.5 + (a + 0.5) / n
                    y = dy - 0.5 + (b + 0.5) / n
                    c += x * x + y * y <= R * R + 1e-9
            return 2 * c >= n * n
        return f

    def span(e):
        def f(dx, dy):
            if dy * dy > R * R + 1e-12:
                return False
            return abs(dx) <= math.sqrt(max(0.0, R * R - dy * dy)) + e + 1e-12
        return f

    out = {"closed": closed, "open": opn, "sub4": sub(4), "sub8": sub(8),
           "span+0.05": span(0.05), "span+0.1": span(0.1), "span+0.15": span(0.15)}
    for k in range(1, 9):
        K = k / 16
        out[f"shift{k}/16"] = (lambda K: lambda dx, dy: (dx + K) ** 2 + (dy + K) ** 2 <= R * R + 1e-9)(K)
    for q in (2, 4, 8):
        out[f"snap1/{q}"] = (lambda q: None)(q)
    return out


def main():
    d = sys.argv[1]
    sizes = [int(s) for s in sys.argv[2:]] or [1, 2, 3, 5, 16, 63]
    for size in sizes:
        try:
            a = Image.open(f"{d}/B001_sweep_s{size}.png").getchannel("A").load()
        except FileNotFoundError:
            continue
        R = size / 2
        rs = rules(R)
        score = {k: [0, 0] for k in rs if rs[k] is not None}
        score.update({f"snap1/{q}": [0, 0] for q in (2, 4, 8)})
        reach = int(R) + 3
        examples = []
        for i, j, cx, cy in sweep_points(size):
            for py in range(int(cy) - reach, int(cy) + reach + 1):
                for px in range(int(cx) - reach, int(cx) + reach + 1):
                    dx, dy = px + 0.5 - cx, py + 0.5 - cy
                    dist = math.hypot(dx, dy)
                    if abs(dist - R) > 1.5:
                        continue
                    obs = a[px, py] > 0
                    for k, f in rs.items():
                        if f is None:
                            continue
                        score[k][0] += 1
                        score[k][1] += f(dx, dy) != obs
                    for q in (2, 4, 8):
                        sx, sy = round(cx * q) / q, round(cy * q) / q
                        ddx, ddy = px + 0.5 - sx, py + 0.5 - sy
                        score[f"snap1/{q}"][0] += 1
                        score[f"snap1/{q}"][1] += (ddx * ddx + ddy * ddy <= R * R + 1e-9) != obs
                    if obs != (dx * dx + dy * dy <= R * R + 1e-9) and len(examples) < 12:
                        examples.append(f"off({i/10:.1f},{j/10:.1f}) px({px - int(cx):+d},{py - int(cy):+d}) d-R={dist - R:+.3f} obs={int(obs)}")
        print(f"size {size} (R={R}):")
        for k, (t, b) in sorted(score.items(), key=lambda kv: kv[1][1]):
            print(f"  {k:10} {t - b}/{t} edge-band px correct ({b} wrong)")
        print("  closed-disc misses, e.g.: " + "; ".join(examples))


if __name__ == "__main__":
    main()
