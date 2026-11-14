#!/usr/bin/env python3
"""vpen scripts for the second brush observation run (2026-11-13). Every layout is defined
here so the analysis scripts can import the exact inputs.

usage: brush_run2.py EXPERIMENT OX OY [ARGS]   -> vpen commands on stdout
  OX, OY: screen position of canvas (0,0) at 100 % zoom. Canvas coords: pixel centres at i+0.5.
Experiments (canvas layouts):
  sweep SIZE            B-001 AA-None edge sweep, 10x10 press offsets (0.0..0.9 px) per size
  hard                  B-009 sizes 11/41 at a pixel centre, size 41 at x offsets 0.0..0.9
  layers                B-003 truncation: 25 horizontal strokes, stroke k from x0_k to 560
  arc N                 B-002 semicircle (centre 300,330, r 150) with N events
  poly DENSE            B-002 polyline with sharp corners: DENSE=1 -> 2 px events, 0 -> corners only
  ramp                  B-002 pressure ramp 0.2 -> 1.0 along a straight line
  rows                  B-002 straight rows (size set in UI) at 3 speeds, for option toggles
  zigzag MS             B-007 zigzag (10 segments, 40 px amplitude, 2 px events, MS per event)
  tilt                  B-010 4 strokes at tilt (0,0), (0.8,0), (0,0.8), (-0.8,-0.8)
  dirs                  B-010 8 short strokes in 8 directions (dab direction)
  dirs2                 B-010 8 strokes of 120 px in 8 directions (k*45 deg, y down), events
                        every 2 px; cell centres (100 + 200 (k % 4), 200 + 400 (k // 4))
  mixA / mixref / mixB   B-011: vertical stroke x = 400.5, y 100.5 -> 700.5; one dab at (100.5, 700.5);
                        horizontal stroke y = 400.5, x 100.5 -> 700.5 (events every 2 px, 5 ms)
  tip                   B-010 two single dabs: (200.5, 200.5) and (500.3, 200.7)
  tiltsweep             B-010 8 strokes with tilt_x = 0, 1/7 .. 1 (vpen units of 63), tilt_y 0
  pgrid                 B-004 21 constant-pressure strokes P = 0, 0.05 .. 1 on an 800x800 canvas
                        (3 columns x 7 rows, cell 260x110; same event pattern as pen_grid.py)
"""
import math
import sys

# ---- layouts (canvas coordinates) ----
SWEEP_ORIGIN = {1: (20.0, 20.0), 2: (20.0, 20.0), 3: (20.0, 20.0), 5: (20.0, 20.0),
                16: (20.0, 20.0), 63: (40.0, 40.0)}
SWEEP_PITCH = {1: 10, 2: 10, 3: 12, 5: 14, 16: 25, 63: 75}


def sweep_points(size):
    """(i, j, x, y): press at origin + pitch*(i, j) + (i/10, j/10)."""
    ox, oy = SWEEP_ORIGIN[size]
    p = SWEEP_PITCH[size]
    return [(i, j, ox + p * i + i / 10, oy + p * j + j / 10) for j in range(10) for i in range(10)]


HARD_POINTS = [(11, 40.5, 40.5), (41, 120.5, 60.5)] + [
    (41, 60.5 + 60 * k + k / 10, 160.5) for k in range(5)] + [
    (41, 60.5 + 60 * k + (k + 5) / 10, 260.5) for k in range(5)] + [(11, 40.5 + 30 * k + k / 10, 340.5) for k in range(10)]

LAYERS_N = 25


def layers_strokes():
    """25 strokes on y = 200.5, stroke k from x = 40.5 + 20 k to 560.5 (so x gets k+1 strokes)."""
    return [[(40.5 + 20 * k + 2 * i, 200.5) for i in range(int((520 - 20 * k) / 2) + 1)]
            for k in range(LAYERS_N)]


def arc_points(n):
    cx, cy, r = 300.5, 330.5, 150.0
    return [(cx - r * math.cos(math.pi * i / n), cy - r * math.sin(math.pi * i / n)) for i in range(n + 1)]


POLY = [(60.5, 60.5), (300.5, 60.5), (300.5, 300.5), (540.5, 300.5), (420.5, 120.5)]


def poly_points(dense):
    if not dense:
        return list(POLY)
    out = [POLY[0]]
    for (x0, y0), (x1, y1) in zip(POLY, POLY[1:]):
        L = math.hypot(x1 - x0, y1 - y0)
        n = int(L / 2)
        out += [(x0 + (x1 - x0) * i / n, y0 + (y1 - y0) * i / n) for i in range(1, n + 1)]
    return out


RAMP = ((40.5, 200.5), (560.5, 200.5), 0.2, 1.0, 260)  # start, end, p0, p1, events


def ramp_points():
    (x0, y0), (x1, y1), p0, p1, n = RAMP
    return [(x0 + (x1 - x0) * i / n, y0 + (y1 - y0) * i / n, p0 + (p1 - p0) * i / n) for i in range(n + 1)]


ROWS = [((40.5, 80.5), (560.5, 80.5), 260, 5), ((40.5, 200.5), (560.5, 200.5), 260, 20),
        ((40.5, 320.5), (560.5, 320.5), 52, 20)]  # start, end, events, ms


def zigzag_points():
    pts = []
    x0, y0, seg_w, amp = 50.5, 200.5, 50.0, 40.0
    corners = [(x0 + seg_w * k, y0 + (amp if k % 2 else -amp) * (k > 0) * 1.0) for k in range(11)]
    corners[0] = (x0, y0)
    pts = [corners[0]]
    for (xa, ya), (xb, yb) in zip(corners, corners[1:]):
        L = math.hypot(xb - xa, yb - ya)
        n = int(L / 2)
        pts += [(xa + (xb - xa) * i / n, ya + (yb - ya) * i / n) for i in range(1, n + 1)]
    return pts


TILTS = [(0.0, 0.0), (0.8, 0.0), (0.0, 0.8), (-0.8, -0.8)]


def dirs2_strokes():
    out = []
    for k in range(8):
        a = math.pi * k / 4
        cx, cy = 100.5 + 200 * (k % 4), 200.5 + 400 * (k // 4)
        sx, sy = cx - math.cos(a) * 60, cy - math.sin(a) * 60
        out.append([(sx + math.cos(a) * t, sy + math.sin(a) * t) for t in range(0, 121, 2)])
    return out


def dirs_strokes():
    out = []
    for k in range(8):
        a = math.pi * k / 4
        cx, cy = 80.5 + 140 * (k % 4), 110.5 + 180 * (k // 4)
        out.append([(cx + math.cos(a) * t, cy + math.sin(a) * t) for t in range(0, 41, 2)])
    return out


TIP_POINTS = [(200.5, 200.5), (500.3, 200.7)]

PGRID_P = [i / 20 for i in range(21)]


def pgrid_cells():
    """(P, x0, y): stroke from x0 to x0 + 120 at y; press at x0 with p = 0 (pen_grid.py)."""
    return [(P, 20 + (i % 3) * 260 + 50, 55 + (i // 3) * 110) for i, P in enumerate(PGRID_P)]


def vpen_p(P):
    """vpen value for CSP pressure P (libinput chain, B-004): p = 0.0099 + 0.9901 P."""
    return 0.0 if P <= 0 else min(1.0, 0.0099 + 0.9901 * P)


# ---- vpen emission ----
def stroke(ox, oy, pts, ms=5, p=1.0, tilt=None):
    L = []
    def mv(x, y, pp):
        t = f" {tilt[0]} {tilt[1]}" if tilt else ""
        return f"move {x + ox:.3f} {y + oy:.3f} {pp}{t}"
    x, y = pts[0][0], pts[0][1]
    L += ["in", mv(x, y, 0), "wait 80", "down"]
    for q in pts:
        pp = q[2] if len(q) > 2 else p
        L += [mv(q[0], q[1], pp), f"wait {ms}"]
    L += ["wait 60", "up", "wait 80", "out", "wait 250"]
    return L


def dab(ox, oy, x, y):
    return ["in", f"move {x + ox:.3f} {y + oy:.3f} 0", "wait 60", "down",
            f"move {x + ox:.3f} {y + oy:.3f} 1", "wait 60", "up", "wait 60", "out", "wait 120"]


def main():
    ex, ox, oy = sys.argv[1], float(sys.argv[2]), float(sys.argv[3])
    a = sys.argv[4:]
    L = []
    if ex == "sweep":
        for _, _, x, y in sweep_points(int(a[0])):
            L += dab(ox, oy, x, y)
    elif ex == "hard":
        size = int(a[0]) if a else None
        for s, x, y in HARD_POINTS:
            if size is None or s == size:
                L += dab(ox, oy, x, y)
    elif ex == "layers":
        for pts in layers_strokes():
            L += stroke(ox, oy, pts)
    elif ex == "arc":
        L += stroke(ox, oy, arc_points(int(a[0])), ms=8)
    elif ex == "poly":
        L += stroke(ox, oy, poly_points(a[0] == "1"), ms=5)
    elif ex == "ramp":
        L += stroke(ox, oy, ramp_points(), ms=5)
    elif ex == "rows":
        for (x0, y0), (x1, y1), n, ms in ROWS:
            L += stroke(ox, oy, [(x0 + (x1 - x0) * i / n, y0 + (y1 - y0) * i / n) for i in range(n + 1)], ms=ms)
    elif ex == "zigzag":
        L += stroke(ox, oy, zigzag_points(), ms=int(a[0]))
    elif ex == "tilt":
        for k, t in enumerate(TILTS):
            y = 60.5 + 90 * k
            L += stroke(ox, oy, [(60.5 + 4 * i, y) for i in range(121)], ms=5, tilt=t)
    elif ex == "mixA":
        L += stroke(ox, oy, [(400.5, 100.5 + 2 * i) for i in range(301)])
    elif ex == "mixref":
        L += dab(ox, oy, 100.5, 700.5)
    elif ex == "mixB":
        L += stroke(ox, oy, [(100.5 + 2 * i, 400.5) for i in range(301)])
    elif ex == "tip":
        for x, y in TIP_POINTS:
            L += dab(ox, oy, x, y)
    elif ex == "tiltsweep":
        for k in range(8):
            y = 50.5 + 95 * k
            L += stroke(ox, oy, [(60.5 + 4 * i, y) for i in range(121)], ms=5, tilt=(round(k / 7, 4), 0.0))
    elif ex == "dirs2":
        for pts in dirs2_strokes():
            L += stroke(ox, oy, pts, ms=8)
    elif ex == "dirs":
        for pts in dirs_strokes():
            L += stroke(ox, oy, pts, ms=8)
    elif ex == "pgrid":
        for P, x0, y in pgrid_cells():
            p = vpen_p(P)
            sx, sy = x0 + ox, y + oy
            L += ["in", f"move {sx:.2f} {sy:.2f} 0", "wait 80", "down", "wait 20",
                  f"move {sx + 0.5:.2f} {sy:.2f} {p:.5f}", "wait 20"]
            for k in range(1, 61):
                L += [f"move {sx + 2 * k:.2f} {sy:.2f} {p:.5f}", "wait 8"]
            L += ["wait 30", "up", "wait 80", "out", "wait 120"]
    else:
        sys.exit(f"unknown experiment {ex}")
    print("\n".join(L))


if __name__ == "__main__":
    main()
