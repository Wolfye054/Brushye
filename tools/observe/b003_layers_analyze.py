#!/usr/bin/env python3
"""B-003 follow-up: is a stroke committed to the layer in 8 bits? (pure Python + Pillow)

Export B003_layers_op10.png: 25 horizontal strokes (size 40, AA None, density 100, opacity 10,
gap Normal) on y = 200.5; stroke k runs from x = 40.5 + 20k to 560.5 (brush_run2.py "layers"),
so the column at x = 10 + 20n is covered by exactly n strokes (strokes 0..n-1).
One stroke alone gives a8 = floor(65535 * 0.1) >> 8 = 25.

Models for n strokes:
  int8    A <- A + a8 - floor(A * a8 / 255)        (8-bit layer, Brushye's current model)
  float   A = 1 - (1 - 0.1)^n, then floor(65535 A) >> 8   (higher-precision layer)
  int8r   A <- A + a8 - round(A * a8 / 255)
  f8      8-bit layer, each stroke composited in float with the unquantised stroke alpha and
          truncated: A <- floor(65535 (1 - (1 - A/255)(1 - 0.1))) >> 8   (B-008 opacity-50 finding)
  inc     8-bit layer, A <- A + floor((255 - A) * 0.1 + 0.5) for A > 0 (increment rounded half
          up, with the unquantised stroke alpha); an empty pixel gets floor(65535 * 0.1) >> 8
usage: b003_layers_analyze.py PNG
"""
import sys

from PIL import Image


def main():
    a = Image.open(sys.argv[1]).getchannel("A").load()
    a8 = int(65535 * 0.1) >> 8
    i8 = i8r = f8 = inc = 0
    rows = []
    for n in range(1, 26):
        i8 = i8 + a8 - (i8 * a8) // 255
        i8r = i8r + a8 - round(i8r * a8 / 255)
        f8 = int(65535 * (1 - (1 - f8 / 255) * 0.9) + 1e-7) >> 8
        fl = int(65535 * (1 - 0.9 ** n) + 1e-7) >> 8
        inc = a8 if inc == 0 else inc + int((255 - inc) * 0.1 + 0.5)
        x = 10 + 20 * n  # 10 px inside stroke n-1, 10 px before stroke n starts
        obs = {a[x, y] for y in range(195, 206)}
        rows.append((n, x, sorted(obs), i8, i8r, fl, f8, inc))
    ok = {"int8": 0, "int8r": 0, "float": 0, "f8": 0, "inc": 0}
    print(" n    x  observed   int8 int8r float   f8  inc")
    for n, x, obs, i8v, i8rv, flv, f8v, incv in rows:
        o = obs[0] if len(obs) == 1 else None
        ok["int8"] += o == i8v
        ok["int8r"] += o == i8rv
        ok["float"] += o == flv
        ok["f8"] += o == f8v
        ok["inc"] += o == incv
        print(f"{n:2} {x:4}  {str(obs):9}  {i8v:4} {i8rv:5} {flv:5} {f8v:4} {incv:4}")
    print("matches out of 25:", ok)


if __name__ == "__main__":
    main()
