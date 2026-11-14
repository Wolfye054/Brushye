# B-010 Tilt and direction dynamics (thickness, angle)
- Status: **observed** (partial: tip thickness/direction/angle exact; tilt reaches CSP under Wine; "Direction of line" observed; pen-tilt angle not run)
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine with virtual stylus *probably* OK. Tilt axes reach X (D-003), but CSP's use of them is unverified. Otherwise maintainer + real pen

## Steps
Thickness 30, Direction horizontal, Angle source = "Direction of pen", then = "Pen tilt".
Draw strokes at a fixed pen orientation, then rotate the pen. Log tilt values in the M1 spike app.

## What to measure
Dab orientation vs logged tilt azimuth/altitude.

## Observations (2026-11-13, run 2)
Method: automated-Wine + virtual stylus, observer Claude. ▲ The steps above were replaced by
scripted strokes and single dabs (no M1 app logging). 800×800 transparent canvas, sub tool
"Obs pen" (B-001 setup), AA None, hardness 5, density 100.

### Tip thickness, direction and angle (no dynamics)
Size 41, single dabs at (200.5, 200.5) and (500.3, 200.7) (`brush_run2.py tip`). Exports
`data/B010_tip_t<T>_a<A>_<vert|horiz>.png`: T30 A0 Vertical, T30 A0 Horizontal, T30 A30
Vertical, T50 A90 Vertical. Analysis `tools/observe/b010_tip.py data`.

**The dab is a closed ellipse with semi-axes R and R·T/100**:
- Direction **Vertical** shortens the vertical axis (a flat, wide dab); **Horizontal**
  shortens the horizontal axis;
- **Angle A rotates it clockwise on screen** (A = 30 puts the long axis at −30° counter-clockwise);
- its **centre is 0.5 px left of the press point** (x − 0.5, y), for both dab positions and
  all four settings (centroid offsets −0.43 … −0.56 in x, ±0.1 in y).

Exactness (AA None, pixel (x, y) inside when its centre is in the ellipse):
788/788, 792/796, 796/796 and 1314/1314 px (3690/3694 overall; the 4 misses are edge ties).
The round tip (T = 100) uses the scanline rule of B-001 instead.

### Angle "Direction of line"
Export `data/B010_dirs_line.png`: `brush_run2.py dirs2` (8 strokes of 120 px in directions
k·45°, events every 2 px), size 41, thickness 30, Vertical, angle 0 with angle dynamics
"Direction of line", gap Fixed 150 % (61.5 px). Analysis `tools/observe/b010_dirs.py data`.
- Every dab's long axis lies along the stroke direction (to 0.1°), **including the dab at the
  press point**.
- In 3 of 8 strokes (directions 135°, 180°, 45° counter-clockwise; strokes 4, 5, 7) the first
  dab sits ≈ 10 px along the stroke instead of at the press point, and the second 61.5 px
  after it. Unexplained.

### Tilt reaches CSP (size dynamics "Tilt", minimum 100 %, maximum 150 %, default curve)
Size 40, gap Narrow, straight strokes with constant tilt (`brush_run2.py tilt`, `tiltsweep`).
vpen tilt units are degrees (−64 … 63).
- `data/B010_tilt_size.png`: tilt (0,0) → 39 ink rows; (50°, 0), (0, 50°) and (−50°, −50°) → 55 rows
  each. The diagonal giving the same width as one axis suggests CSP (or Wine's wintab
  translation) does not use the combined tilt magnitude: **verify on native Windows**.
- `data/B010_tiltsweep_size.png`, tilt_x = 0, 9, 18, 27, 36, 45, 54, 63°: rows 39, 41, 41, 41, 45, 51,
  57, 59 (size ≈ rows + 1, i.e. 100 % … 150 %). The flat step from 9° to 27° is unexplained.
  The mapping from device tilt to CSP's tilt input goes through Wine and is not reliable for
  exact modelling.

## Open questions
- Angle dynamics "Pen tilt" and "Direction of pen" (azimuth) were not run.
- The ≈ 10 px late first dab with "Direction of line" in some directions.
- Exact tilt input mapping on native Windows; how tilt_x and tilt_y combine.
- Thickness with AA levels other than None, and with soft tips.

