# B-003 Stroke accumulation: Opacity cap vs Brush density
- Status: draft
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK

## Purpose
CSP describes Ink > Opacity as a per-stroke ceiling and Brush density as a per-dab value that
builds up. Verify this and measure the exact accumulation formula.

## Setup
B-001 setup, size 40, AA Middle, gap Narrow.

## Steps
1. Opacity 50, density 100: one straight stroke, then a second stroke crossing it
   (a separate press). Export.
2. Opacity 100, density 50: same. Export.
3. Opacity 50, density 50: same. Export.
4. One stroke that crosses itself (a figure-8 in one press) for each case. Export.

## Expected result / what to measure
Alpha in the overlap regions, within one stroke and between strokes. Model: stroke buffer
clamped by opacity, then composited with the ink mode? Measure it.
