# L-006 Clip to layer below; lock transparent pixels
- Status: draft
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK

## Steps
1. base (opaque gradient), mid (partial-alpha shape), top (Normal and Multiply) clipped to mid.
   Export.
2. Lock transparent pixels on mid, paint over it with a scripted stroke, export.

## What to measure
Clipped layer alpha = top alpha × mid alpha (rounding?); lock behaviour at partial alpha.
