# B-005 Pen pressure → opacity and → brush density
- Status: **implemented (linear, minimum 0)**
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine with virtual stylus (D-003). Flag `verify-native` (real-pen spot check)
- Date / method / observer: 2026-11-13 / automated-Wine + virtual stylus / Claude

## Setup (exact)
Same as B-004 (Obs pen, canvas, input definition), except:
- Brush Size **40 px**, size dynamics all OFF.
- Run 1: **Opacity dynamics: Pen pressure ON**, minimum 0, linear (Tool Property > Opacity's dynamics button).
- Run 2: Opacity dynamics OFF; **Brush density dynamics: Pen pressure ON**, minimum 0, linear
  (Sub Tool Detail > Brush tip > Brush density's dynamics button).
- Global pen pressure: default (identity). It was reset and confirmed (Reset to default → Check
  adjusted settings → Done), then checked by reopening the dialog.

## Steps
`pen_grid.py` 21 constant-pressure strokes (P = 0 … 1 step 0.05), export,
`analyze_pen_grid_alpha.py`: centre-row ink (255 − R) over the central 40 columns.

## Observations (2026-11-13)
Data: `data/B005_opacity_min0.png`, `data/B005_density_min0.png`.

1. **Pressure → opacity: stroke alpha ≈ P, perfectly uniform along the stroke.**
   Centre ink = 12, 25, 38, 51, 63, 76, 89, 102, 115, 127, 140, 153, 166, 179, 191, 204, 217, 230, 243, 255
   for P = 0.05 … 1.0 (min = max within each stroke). This is about round or floor of 255·P,
   within ±1. Overlapping dabs (spacing ≈ 3.4 px on a 40 px tip) do **not** build up, so the
   opacity dynamic sets a per-stroke ceiling, matching the manual's description of Ink > Opacity.
2. **Pressure → brush density: per-dab alpha = P, and dabs accumulate within the stroke.**
   Centre alpha follows 1 − (1 − P)^N with N ≈ 11.7. Solving for N at P = 0.05, 0.10, 0.15,
   0.20, 0.25 gives 11.64, 11.69, 11.77, 11.75, 11.81. It saturates to 255 from P = 0.40 up. There is
   slight periodic variation along the stroke (min/max differ by ≤ 2), consistent with discrete
   dab positions.
3. Lead for B-002 (spacing): N ≈ 11.7 dabs over a centre pixel of a 40 px circle implies a dab
   interval of ≈ 3.4 px (≈ 8.5 % of size) for gap preset 3/4 with these settings. Not yet measured
   directly.

## Expected result (for Brushye)
- Opacity dynamic: the stroke buffer's alpha is capped at opacity(P). Overlapping dabs within
  one stroke combine by max (or equivalent), not "over".
- Density dynamic: each dab's alpha = density(P), and dabs are composited "over" each other
  in the stroke.

## Open questions
- Exact rounding of the opacity value (the input chain fixes P only to about ±0.5/2047). Next
  step: B-003 with fixed opacity values typed into the UI, no pressure involved.
- Where the opacity cap applies when P varies within a stroke: per dab or per pixel max?
  Run a pressure ramp with the opacity dynamic.
- Does "Adjust brush density by gap" change N?

## Implementation (2026-11-13, M4)
Opacity scales the stroke buffer (B-003), density scales each dab. Test
`b004_b005_pressure_dynamics_match_within_tolerance`: both grids within 3 levels.
