#!/usr/bin/env python3
"""Emit a vpen.py script of straight strokes (B-002 / B-003 observations).

usage: b002_strokes.py STROKE [STROKE ...]
  STROKE = "x0,y0,x1,y1,N,MS[,P]" in canvas px (pixel centres are i+0.5):
  pen down at (x0,y0), then N motion events evenly spaced along the segment to (x1,y1),
  MS milliseconds apart, constant pressure P (default 1), then up.
  A STROKE "path:x,y;x,y;...;MS[;P]" draws a polyline through the given points, one event
  per point (used for the figure-8 in B-003).
Canvas origin on screen: (649, 362) at 100% zoom (B-001 harness layout).
"""
import sys

OX, OY = 649, 362


def emit(points, ms, p):
    x0, y0 = points[0]
    out = ["in", f"move {x0 + OX:.3f} {y0 + OY:.3f} 0", "wait 80", "down",
           f"move {x0 + OX:.3f} {y0 + OY:.3f} {p}", f"wait {ms}"]
    for x, y in points[1:]:
        out += [f"move {x + OX:.3f} {y + OY:.3f} {p}", f"wait {ms}"]
    out += ["wait 60", "up", "wait 80", "out", "wait 250"]
    return out


lines = []
for spec in sys.argv[1:]:
    if spec.startswith("path:"):
        parts = spec[5:].split(";")
        # trailing fields without a comma are MS and optional P
        tail = [q for q in parts if "," not in q]
        pts = [tuple(map(float, q.split(","))) for q in parts if "," in q]
        ms = int(tail[0]); p = float(tail[1]) if len(tail) > 1 else 1.0
        lines += emit(pts, ms, p)
        continue
    f = spec.split(",")
    x0, y0, x1, y1 = map(float, f[:4]); n, ms = int(f[4]), int(f[5])
    p = float(f[6]) if len(f) > 6 else 1.0
    pts = [(x0 + (x1 - x0) * i / n, y0 + (y1 - y0) * i / n) for i in range(n + 1)]
    lines += emit(pts, ms, p)
print("\n".join(lines))
