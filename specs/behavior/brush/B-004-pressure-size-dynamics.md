# B-004 Pen pressure → brush size (minimum value + curve)
- Status: draft
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: **maintainer + real pen** (no pressure path in Xvfb; see DECISIONS D-003)

## Purpose
Mapping from raw pressure to dab size: minimum value, curve shape, how it composes with the
global pen pressure adjustment.

## Setup
1. **Reset the global pen pressure setting to default** (File > Adjust pen pressure detection
   level / Pen Pressure Settings → reset). Record the dialog's graph in a screenshot.
2. B-001 tip, size 100 px, gap Narrow, density 100, AA Middle, **pressure → size ON**,
   minimum 0 %, linear default curve.

## Steps
1. Draw a slow stroke left to right while raising pressure from feather-light to full, about 3 s.
2. At the same time, record the pen's raw pressure stream with an independent logger
   (Brushye's M1 tablet spike app or the OS tablet test tool) so stroke width can be paired
   with input pressure. If they can't run at once, draw three repeatable taps at "light / medium /
   full" instead and log them separately.
3. Export PNG. Repeat with minimum 20 %, 50 %, and with a 3-point S-curve (screenshot of the graph).
4. Repeat minimum 0 % after setting the global pressure adjustment to "Lighter" ×3.

## Expected result
Width(x) along the stroke versus logged pressure → size = f(p). Hypothesis to test (from a
CELSYS tips article): size = max × (min + (1 − min) × curve(p)).

## Open questions
- Does width respond to Wintab vs Windows Ink differently? (Record the tablet service setting.)
