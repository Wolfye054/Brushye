#!/usr/bin/env python3
"""B-011 basic colour mixing: print the colour along the mixing stroke (pure Python + Pillow).

Export B011_blend_default.png (tools/observe/b011_mix.sh): stroke A vertical at x = 400.5 in
colour 1 (mixing off), a reference dab at (100.5, 700.5) in colour 2 (mixing off), then stroke B
horizontal at y = 400.5 from x = 100.5 to 700.5 in colour 2 with Color mixing ON (defaults).
usage: b011_profile.py PNG
"""
import sys

from PIL import Image


def main():
    im = Image.open(sys.argv[1]).convert("RGBA")
    c1, c2 = im.getpixel((400, 200)), im.getpixel((100, 700))
    print(f"colour 1 (stroke A) {c1}, colour 2 (reference dab) {c2}")
    for x in list(range(90, 380, 30)) + list(range(380, 440, 5)) + list(range(440, 720, 20)):
        p = im.getpixel((x, 400))
        print(f"x {x:3d}: {p}   distance from colour 2 (R) {c2[0] - p[0]:+d}")


if __name__ == "__main__":
    main()
