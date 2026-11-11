# B-001 Round tip: single-dab profile vs size and anti-aliasing
- Status: draft (automation path demonstrated, see Observations)
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK (mouse; no pressure dynamics involved)

## Purpose
Pin down the exact alpha profile of one dab of a circle tip: edge position, how
fractional sizes rasterize, and what each AA level (None/Weak/Middle/Strong) does to the edge.

## Setup (exact)
1. File > New: 128×128 px, 72 dpi, paper **off/transparent** (or delete the paper layer),
   expression colour Color.
2. One raster layer. Main colour #000000.
3. Make a new custom sub tool (Pen group, "Create custom sub tool", raster). In Sub Tool Detail:
   Brush tip = Circle, Hardness = max, Thickness 100, Angle 0, Brush density 100, Opacity 100,
   Ink blending mode Normal, every dynamics OFF (size, opacity, density), Stabilization 0,
   Starting and ending off, Texture off, Spray off.

## Steps (exact)
For each size in {1, 1.5, 2, 3, 4.5, 10, 25.3} px and each AA in {None, Weak, Middle, Strong}:
1. Set size and AA.
2. Canvas at 100 % zoom. Mouse-click once (press and release, no motion) at canvas pixel
   centre (64.5, 64.5) (screen → canvas mapping verified by a 1 px test dot first).
3. Export (Single Layer) .png, RGB, no scaling, transparency kept. File `B001_s<size>_aa<level>.png`.
4. Undo (Ctrl+Z) before the next one.
Repeat step 2 at sub-pixel offsets (64.25, 64.5) and (64.0, 64.0) for sizes 1, 3, 10.

## Expected result / what to measure
Alpha per pixel along the dab's radius, for each (size, AA, offset). Derive: radius definition
(diameter = size?), coverage model (area coverage vs distance threshold), AA ramp width per level.

## Observations
- 2026-11-11, automated-Wine: the harness can select tools, click on the canvas and export
  PNGs (see wine-harness.md). Mouse clicks under Xvfb arrive as normal pointer input with no
  pressure. Demo: the default pen sub tool, a 60-step scripted drag; the stroke appeared in the
  exported PNG at the expected canvas position (canvas = screen − (821, 307) at 100 % zoom on a
  256×512 canvas) with constant width. The full sweep above has **not** been run yet.

## Open questions
- Does a click with zero motion place exactly one dab?
- Does mouse input apply an implied pressure (Preferences "reduce pressure for mouse")?
  If so, turn that preference off for all mouse-driven specs and record its state.
