# B-011 Colour mixing: Blend and Running colour
- Status: **observed** (basic only: one setting, default Blend; the parameter sweep above was not run)
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK

## Setup
Canvas 512×128 with a generated background: left half #ff0000, right half #0000ff, and a
transparent strip at the bottom. Brush size 40, colour #ffff00, density 100.

## Steps
For Blend and Running colour, with amount of paint {0, 30, 70, 100} × density of paint {100, 50}
× colour stretch {0, 50}: one scripted horizontal stroke across the boundary (fixed timing).
Export each.

## What to measure
Colour along the stroke centre: pickup rate, carry distance, how transparency is picked up.
This is the hardest brush behaviour to match, so expect several rounds.

## Observations (2026-11-13, run 2: basic behaviour only)
Method: automated-Wine + virtual stylus, observer Claude. ▲ Instead of the generated background:
800×800 transparent canvas, sub tool "Obs pen" (B-001 setup), size 40, AA None, hardness 5,
density 100, opacity 100, gap Normal. `tools/observe/b011_mix.sh` (inputs `brush_run2.py
mixA/mixref/mixB`):
1. stroke A, vertical at x = 400.5 (y 100.5 → 700.5), colour 1 = (78, 42, 60), mixing off;
2. colour 2 = (217, 182, 200) picked in the colour square; reference dab at (100.5, 700.5), mixing off;
3. Ink > **Color mixing ON with the defaults**: first mode button (of three), amount of paint 50,
   density of paint 50, colour stretch 10; stroke B horizontal at y = 400.5 from x = 100.5 to 700.5.
Export `data/B011_blend_default.png`; profile `tools/observe/b011_profile.py data/B011_blend_default.png`.

Along stroke B's centre row:
- **Over transparent canvas the stroke picks up transparency:** alpha 160 at the press point
  (a cap pixel), rising to a plateau of **232** (not 255) from x ≈ 180; the colour stays exactly
  colour 2.
- **Crossing stroke A it picks up colour 1:** the darkest point is x ≈ 390 (183, 148, 165), about
  25 % of the way from colour 2 to colour 1, and alpha rises to 251–252 there.
- **After A the picked-up colour fades out over ≈ 150 px:** the red channel's distance from
  colour 2 is 22, 14, 10, 6, 4, 3, 2, 1 at x = 420, 440, 460, 480, 500, 520, 540, 560. That is
  roughly halving every 25–30 px (≈ ×0.92 per dab at 3.2 px spacing), back to colour 2 by x ≈ 590.
  Alpha falls back to 232 alongside.

## Open questions
- The mixing rule (per dab: sample under the dab, blend with the carried colour by amount/density
  of paint?), and the role of colour stretch: the sweep in Steps.
- The three mode buttons (Blend, Running colour, …): only the first (default) was used.
- Why the plateau over transparent canvas is 232.

