# B-004 Pen pressure → brush size (minimum value + curve)
- Status: **implemented** (linear, minimum, 1-, 2- and 3-point curves observed; run 2 confirmed the B-spline)
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine with virtual stylus (D-003). Flag `verify-native`: one spot check with a
  real pen on native Windows (Wintab vs Windows Ink may scale raw pressure differently)
- Date / method / observer: 2026-11-13 / automated-Wine + virtual stylus (`vpen.py`, Xorg libinput) / Claude

## Purpose
How raw pen pressure maps to dab size: minimum value, curve shape, and how the curve
combines with the global pen pressure adjustment (see B-006).

## Input pressure definition
P = the value Wine's wintab hands to CSP, normalized: the X stylus pressure axis (0..2047)
divided by 2047. libinput maps the uinput value p (0..1) to X/2047 = (p − 0.0099)/0.9901,
clamped at 0. This was measured with XI2 raw events on 41 evenly spaced steps (linear above
the ~1 % dead zone). `pen_grid.py` sends p = 0.0099 + 0.9901·P.

## Setup (exact)
1. Global pen pressure setting = default (identity curve). Checked in File > Pen Pressure
   Settings > "Show pen pressure graph"; 2 points, (0,0) and (1,1).
2. Observation sub tool "Obs pen": a duplicate of the Pen > G-pen sub tool, then changed as listed.
   Final settings, each read from the Sub Tool Detail palette:
   - Brush Size 100 px; **size dynamics: Pen pressure ON, Tilt OFF** (G-pen default has Tilt ON,
     100→150 %), Velocity/Random OFF; minimum value per run.
   - Ink: Opacity 100 (no dynamics), Blending mode Normal, Color mixing off.
   - Anti-aliasing: Middle (3rd of 4).
   - Brush tip: Circle, hardness max (5/5), thickness 100, direction vertical, angle 0,
     brush density 100, adjust density by gap off.
   - Stroke: gap preset 3 of 4 (G-pen default), continuous spraying off, correct velocity input on.
   - Correction: stabilization 0, adjust by speed off, post correction off.
   - Starting and ending: None.
3. Canvas: File > New 1200×900 px, 72 dpi, Color, white paper. View at 100 % zoom;
   canvas (0,0) = screen (348,113). One raster layer, main colour black.

## Steps (exact)
For each run, `tools/observe/pen_grid.py FIFO 348 113 0,0.05,...,1.0` draws 21 strokes,
one per P value. Each is a constant-pressure horizontal stroke, 120 px long, made of
61 events 8 ms apart, in a 7×3 grid of 170×300 px cells. Then `csp_export_png.sh` exports it
and `analyze_pen_grid.py … --size 100` measures it. Between runs, undo ×21.
- Run 1: minimum value 0, linear curve.
- Run 2: minimum value 20, linear curve.
- Run 3: minimum value 0, curve with one added point: the midpoint dragged from (0.5,0.5) to (0.5,0.25).

Measures: `rows` = number of rows containing ink in the stroke's middle (= dab diameter);
`width` = mean per-column ink coverage (Σ(255−R)/255) over the central 40 columns.

## Observations (2026-11-13)
Data: `data/B004_min0_linear.png`, `data/B004_min20_linear.png`, `data/B004_curve1pt.png`.

1. **Linear, minimum 0: diameter = size × P.** Rows = 4, 10, 14, 20, 24, 30 … 94, 100 for
   P = 0.05 … 1.0, i.e. 100·P rounded to an even count, which suggests a dab centred on the
   pixel grid. Coverage width = diameter − ≈1.2…1.9 px at every P (the Middle AA edge).
   **P = 0 draws nothing** (no ink at all for a stroke at zero pressure).
2. **Minimum value m: diameter = size × (m + (1 − m) × P).** With m = 20 %, rows =
   24, 28, 32 … 96, 100 for P = 0.05 … 1.0, exactly 100·(0.2 + 0.8·P). P = 0 still draws nothing.
3. **The curve editor's points are Bézier control points, not interpolation points.**
   With one point at (0.5, 0.25), the effective output (width inverted through run 1's
   width↔P table) equals the quadratic Bézier through (0,0) → (0.5,0.25) → (1,1), i.e.
   y = 0.5P + 0.5P², to within 0.0044 at all 20 steps. A straight-line polyline through the point
   is off by up to 0.125. The UI also draws the curve passing above the point.
4. A mouse click with the same sub tool gives a full-size dab: mouse input counts as pressure 1.
   (Seen on a throwaway canvas; not measured.)
5. Global and per-setting curves compose as **sub(global(P))**; see B-006.

## Follow-up 2026-11-13 (run 2): curves with 2 and 3 points
Method: automated-Wine + virtual stylus, observer Claude. ▲ 800×800 transparent canvas, size 100,
AA Middle, hardness 5, density 100, gap Normal, size dynamics pen pressure (minimum 0).
`tools/observe/brush_run2.py pgrid`: 21 constant-pressure strokes P = 0, 0.05 … 1 (same event
pattern as `pen_grid.py`, 3 columns × 7 rows, 260×110 px cells). Curve points were dragged onto
grid-line intersections of the curve editor (graph (0,0) at screen (256, 593), (1,1) at
(407.5, 430); screenshots in `obs-out/run2-ui/`, not committed):
- `data/B004_pgrid_linear.png`: linear (calibration; width ≈ 100·P − 1.9 px);
- `data/B004_pgrid_2pt.png`: points (0.25, 0.75), (0.75, 0.25);
- `data/B004_pgrid_3pt.png`: points (0.25, 0.75), (0.5, 0.25), (0.75, 0.25).

Analysis `tools/observe/b004_curves.py data` (stroke width inverted through the linear run):

| curve | quadratic B-spline | one Bézier of degree n+1 | polyline |
|---|---|---|---|
| 2 points | **max 0.0015**, mean 0.0008 | 0.122 | 0.251 |
| 3 points | **max 0.0024**, mean 0.0010 | 0.177 | 0.215 |

**Multi-point curves are the quadratic B-spline on the control polygon** (0,0), points…, (1,1),
pieces joined at the midpoints between interior points. The editor draws the curve touching
the polygon at those midpoints (e.g. at (0.625, 0.25) for the 3-point curve).

## Expected result (for Brushye)
dab_diameter = size × (m + (1 − m) × C_sub(C_global(P))), with C = the curve through the
editor's points (quadratic Bézier for 1 interior point). P = 0 produces no dab.

## Open questions
- (Answered in run 2: quadratic B-spline on the control polygon.)
- Exact rounding of the diameter (the even row counts) needs a sub-pixel analysis of single dabs
  (B-001).
- Is there a pressure threshold below which no dab is drawn (CSP side, besides P = 0)?
  Sweep P = 0.001…0.02.

## Implementation (2026-11-13, M4)
`brushye_core::brush::{Dynamics, Curve}`. Multi-point curves use the quadratic B-spline
through the control points (equal to the observed Bézier for one point; confirmed for 2 and 3 points in run 2).
Test `b004_b005_pressure_dynamics_match_within_tolerance` replays the 21-stroke grids: errors
only on the outermost anti-aliased pixel (max 9 / 13 / 11 levels for the three runs), about
0.05 px of diameter, within the precision with which pressure reaches CSP.
