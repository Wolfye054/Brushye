#!/usr/bin/env python3
"""B-006 follow-up: vpen strokes for CSP's pen pressure wizard (File > Pen Pressure Settings).

usage: b006_wizard.py PROFILE PEAK [Y ...]     -> vpen commands on stdout
  PROFILE: "sine" (half-sine pressure 0 -> PEAK -> 0) or "plateau" (ramp up over the first
  15 % of events, hold PEAK, ramp down over the last 15 %).
  One stroke per Y (screen px), from x 660 to 1000, 120 events 8 ms apart (as in B-006).
  PEAK is CSP's pressure P; vpen receives p = 0.0099 + 0.9901 P (B-004 input definition).
"""
import math
import sys


def vpen_p(P):
    return 0.0 if P <= 0 else min(1.0, 0.0099 + 0.9901 * P)


def profile(kind, peak, t):
    if kind == "sine":
        return peak * math.sin(math.pi * t)
    ramp = 0.15
    if t < ramp:
        return peak * t / ramp
    if t > 1 - ramp:
        return peak * (1 - t) / ramp
    return peak


def main():
    kind, peak = sys.argv[1], float(sys.argv[2])
    ys = [float(y) for y in sys.argv[3:]] or [600.0, 650.0, 700.0]
    n = 120
    out = []
    for y in ys:
        out += ["in", f"move 660.0 {y} 0", "wait 80", "down"]
        for i in range(n + 1):
            t = i / n
            out += [f"move {660 + 340 * t:.2f} {y} {vpen_p(profile(kind, peak, t)):.5f}", "wait 8"]
        out += ["wait 40", "up", "wait 80", "out", "wait 200"]
    print("\n".join(out))


if __name__ == "__main__":
    main()
