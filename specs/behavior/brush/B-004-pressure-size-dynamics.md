# B-004 Pen pressure → brush size (minimum value + curve)
- Status: draft
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine with virtual stylus OK (D-003, verified 2026-11-13). Flag `verify-native`: one spot check with a real pen on native Windows (Wintab vs Windows Ink may scale pressure differently)

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
   (Automated: `vpen.py` drives exact pressure values, e.g. a linear ramp or discrete steps
   0.05, 0.1, …, 1.0, so steps 2's logging isn't needed.)
2. (Manual only) At the same time, record the pen's raw pressure stream with an independent logger
   (Brushye's M1 tablet spike app or the OS tablet test tool) so stroke width can be paired
   with input pressure. If they can't run at once, draw three repeatable taps at "light / medium /
   full" instead and log them separately.
3. Export PNG. Repeat with minimum 20 %, 50 %, and with a 3-point S-curve (screenshot of the graph).
4. Repeat minimum 0 % after setting the global pressure adjustment to "Lighter" ×3.

## Expected result
Width(x) along the stroke versus logged pressure → size = f(p). Hypothesis to test (from a
CELSYS tips article): size = max × (min + (1 − min) × curve(p)).

## Observations
- 2026-11-13, automated-Wine + virtual stylus: harness check only, **not** the spec run.
  Default G-pen (size 10, its default dynamics), linear ramp 0.05→1.0 over 1000 screen px at
  75.9 % zoom. Effective width (ink sum / 255 per column) ranged from ~0.27 px to ~7.8 px and
  increased monotonically. Two runs were bit-identical. The spec run still needs the custom sub
  tool from the Setup section and discrete pressure steps.

## Open questions
- How does vpen's 0..8191 map to Wine's wintab pressure and then to CSP's 0..1? The X
  stylus axis is 0..2047 after libinput normalization. This needs a calibration run: discrete steps
  give the mapping, and its linearity is part of what's measured.
- Does width respond to Wintab vs Windows Ink differently? (Record the tablet service setting.)
