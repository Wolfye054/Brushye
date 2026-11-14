#!/usr/bin/env python3
"""B-006 follow-up: read the control points from a screenshot of CSP's pen pressure graph.

usage: b006_graph_points.py SCREENSHOT X0 Y0 X1 Y1
  (X0, Y0) = screen pixel of the graph's 0 %/0 % corner, (X1, Y1) = 100 %/100 % corner.
Control points are small squares in a blue/violet accent; their pixel clusters are found
by colour (b > r + 40 and b > g + 40), and each cluster's centre is mapped to (input, output).
"""
import sys

from PIL import Image


def main():
    path = sys.argv[1]
    x0, y0, x1, y1 = (float(v) for v in sys.argv[2:6])
    im = Image.open(path).convert("RGB")
    pts = []
    for y in range(int(y1) - 6, int(y0) + 7):
        for x in range(int(x0) - 6, int(x1) + 7):
            r, g, b = im.getpixel((x, y))
            if b > r + 40 and b > g + 40 and b > 150:
                pts.append((x, y))
    clusters = []
    for p in pts:
        for c in clusters:
            if any(abs(p[0] - q[0]) <= 2 and abs(p[1] - q[1]) <= 2 for q in c):
                c.append(p)
                break
        else:
            clusters.append([p])
    res = []
    for c in clusters:
        if len(c) < 12:  # smaller clusters are curve-line pixels, not markers
            continue
        cx = sum(p[0] for p in c) / len(c)
        cy = sum(p[1] for p in c) / len(c)
        res.append(((cx - x0) / (x1 - x0), (y0 - cy) / (y0 - y1), len(c)))
    for i, o, n in sorted(res):
        print(f"({i:.3f}, {o:.3f})  [{n} px]")


if __name__ == "__main__":
    main()
