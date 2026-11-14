#!/usr/bin/env python3
"""Dab centres from an export of isolated hard dabs (AA None, density 100, gap large enough
that dabs don't touch). Connected components of alpha > 0 (8-connected); centre = mean of
pixel centres, area = pixel count. Pure Python + Pillow.

usage: dab_centres.py PNG [MIN_AREA]
Importable: centres(path, min_area) -> [(cx, cy, area)] in stroke-independent raster order.
"""
import sys

from PIL import Image


def centres(path, min_area=1):
    im = Image.open(path).getchannel("A")
    w, h = im.size
    a = im.load()
    seen = bytearray(w * h)
    out = []
    for y in range(h):
        for x in range(w):
            if a[x, y] == 0 or seen[y * w + x]:
                continue
            stack = [(x, y)]
            seen[y * w + x] = 1
            sx = sy = n = 0
            while stack:
                px, py = stack.pop()
                sx += px + 0.5
                sy += py + 0.5
                n += 1
                for qx in (px - 1, px, px + 1):
                    for qy in (py - 1, py, py + 1):
                        if 0 <= qx < w and 0 <= qy < h and not seen[qy * w + qx] and a[qx, qy] > 0:
                            seen[qy * w + qx] = 1
                            stack.append((qx, qy))
            if n >= min_area:
                out.append((sx / n, sy / n, n))
    return out


if __name__ == "__main__":
    for cx, cy, n in centres(sys.argv[1], int(sys.argv[2]) if len(sys.argv) > 2 else 1):
        print(f"{cx:9.3f} {cy:9.3f} {n:6}")
