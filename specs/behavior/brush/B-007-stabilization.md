# B-007 Correction > Stabilization and "adjust by speed"
- Status: **implemented (moving mean; S ≥ 50 open)**
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

## Observations (2026-11-13, run 2)
Method: automated-Wine + virtual stylus (`vpen.py`, Xorg :78), observer Claude. Flag
`verify-native`: event timestamps may not reach CSP from Wine (see 3).

Setup (▲ = deviation from the steps above): sub tool "Obs pen" (B-001 setup), ▲ size 4, AA None,
hardness 5, density 100, opacity 100, gap Narrow, 800×800 transparent canvas.
`tools/observe/brush_run2.py zigzag MS`: ▲ zig-zag from (50.5, 200.5), 10 segments 50 px wide
alternating ±40 px, events every ~2 px along it (input path = `zigzag_points()`), MS ms apart.
Exports (`data/B007_zigzag_s<S>_ms<MS>[_speed].png`):
- ▲ stabilization S = 0, 6, 20, 50, 100 at 5 ms;
- S = 20 at 20 ms;
- S = 20 with "Adjust by speed" ON at 5 ms and 20 ms.
Analysis: `tools/observe/b007_analyze.py data` (centre line = alpha-weighted y per column).

1. **The stroke always reaches the release point.** Ink extends to x = 551 (input ends at
   550.5) at every setting, so the lagging, smoothed path catches up when the pen lifts.
2. **Small settings act as a moving mean over the last ≈ S events.** Best fits:

   | S | moving mean N (rms, max) | EMA 1/N (rms) | lazy string (rms) |
   |---|---|---|---|
   | 0 | N = 3: 0.21 px, 0.40 (≈ unsmoothed) | 0.24 | — |
   | 6 | N = 7: 0.23 px, 0.70 | 0.31 | 0.30 |
   | 20 | **N = 20: 0.25 px, 0.98** | 0.92 | 0.97 |
   | 50 | N = 47: 2.1 px, 9.0 | 4.0 | 3.9 |
   | 100 | N = 85: 5.8 px, 19.8 | 10.0 | 10.2 |

   No tested family (EMA, moving mean, mean applied twice, lazy string) fits S = 50 or 100.
3. **Stabilization counts events, not time (under Wine).** S = 20 at 5 ms and at 20 ms are
   byte-identical exports. With "Adjust by speed" ON the result changes (rms vs the moving mean
   0.25 → 0.58 px) but is again identical at 5 ms and 20 ms. So "speed" acts per event (distance
   per event) here. Wine may not pass usable event timestamps to CSP: check on native Windows.
4. Stabilization is deterministic: S = 100 run twice gave byte-identical exports.

## Open questions
- The smoothing for large S (≥ 50), and how the catch-up at release is drawn.
- "Adjust by speed": its exact rule; and whether native Windows (with real timestamps) makes
  either option time-dependent.
- Post correction (Correction page) was not tested.

## Implementation, M4 part 2 (2026-11-13)
`path::stabilize`: each event becomes the mean of the last **S** events (positions only), and
the last event is repeated S − 1 times at release so the path catches up; the result then goes
through the B-spline (B-002). With the spline in the chain, N = S fits better than the N ≈ S + 1
fitted above without it: centre-line rms 0.03 / 0.06 / 0.16 px at S = 0 / 6 / 20 (N = S + 1:
0.15 and 0.31). S = 50 and 100 use the same rule and are off (rms 2.2 and 6.4 px), pinned as open.
"Adjust by speed" and post correction are not implemented.
