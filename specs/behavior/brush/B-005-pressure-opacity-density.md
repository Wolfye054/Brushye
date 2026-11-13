# B-005 Pen pressure → opacity and → density
- Status: draft
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine with virtual stylus OK (D-003, verified 2026-11-13). Flag `verify-native`: one spot check with a real pen on native Windows (Wintab vs Windows Ink may scale pressure differently)

## Steps
As in B-004, but with size fixed, pressure → opacity ON (min 0) in run 1, and pressure →
density ON (min 0) in run 2. Use a constant-pressure ramp stroke and three discrete-pressure taps.
Export each.

## What to measure
Alpha vs pressure; whether the opacity dynamic changes the per-stroke ceiling or per-dab alpha.
