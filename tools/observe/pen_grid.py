#!/usr/bin/env python3
"""Write a grid of constant-pressure strokes to the vpen FIFO (pressure specs B-004/B-005).

usage: pen_grid.py FIFO OFFSET_X OFFSET_Y P1,P2,...   (P = target normalized X pressure 0..1)
Canvas cell i: column i%7, row i//7; cell 170x300 px; stroke horizontal from x0+25 to x0+145
at y = 150 + row*300 (canvas px). OFFSET = screen position of canvas (0,0) at 100 % zoom.
libinput maps vpen p to X pressure as X/2047 = (p - 0.0099)/0.9901 (measured 2026-11-13),
so we send p = 0.0099 + 0.9901*P (P=0 -> p=0).
"""
import sys
fifo, ox, oy, plist = sys.argv[1], float(sys.argv[2]), float(sys.argv[3]), sys.argv[4]
Ps = [float(v) for v in plist.split(",")]
def p_of(P): return 0.0 if P <= 0 else min(1.0, 0.0099 + 0.9901 * P)
L = ["in"]
for i, P in enumerate(Ps):
    c, r = i % 7, i // 7
    x0, y = 20 + c * 170 + 25, 150 + r * 300
    sx, sy, p = ox + x0, oy + y, p_of(P)
    L += [f"move {sx:.2f} {sy:.2f} 0", "wait 80", "down", "wait 20", f"move {sx+0.5:.2f} {sy:.2f} {p:.5f}", "wait 20"]
    for k in range(1, 61):
        L += [f"move {sx + 2*k:.2f} {sy:.2f} {p:.5f}", "wait 8"]
    L += ["wait 30", "up", "wait 120"]
L += ["out", "wait 100"]
open(fifo, "w").write("\n".join(L) + "\n")
