# B-007 Correction > Stabilization and "adjust by speed"
- Status: draft
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK (scripted XTEST motion with controlled timing). Flag `verify-native`,
  since timing under Wine/Xvfb may differ.

## Steps
1. Size 3, AA None, density 100. Draw a scripted zig-zag (10 segments, 40 px amplitude) with fixed
   event timing (8 ms) at stabilization 0, 5, 15, 30, 60, 100. Export each.
2. Repeat stabilization 30 with 2 ms and 30 ms intervals (speed dependence), with and without
   "adjust by speed".
3. Record the exact input path in the result file (it becomes a Brushye test input).

## What to measure
Rendered centre-line vs input path: lag, corner rounding, whether the stroke end catches up
to the last input point.
