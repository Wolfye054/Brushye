# B-001 Round tip: single-dab profile vs size and anti-aliasing
- Status: **observed** (AA Weak/Middle/Strong exact; AA None edge-tie rule open)
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Date / method / observer: 2026-11-13 / automated-Wine + virtual stylus (`vpen.py`, Xorg :78
  libinput → Wine wintab) / Claude. Flag `verify-native` only for the AA None tie rule.

## Setup (as run; deviations from the draft marked ▲)
1. ▲ Canvas 600×400 px, 72 dpi, paper off (exports carry alpha), one raster layer, 100 % zoom,
   canvas origin at screen (649, 362). ▲ Main colour reddish (analysis uses alpha only).
2. Sub tool "Obs pen" (duplicate of Pen > G-pen), settings read from Sub Tool Detail
   (screenshots in `obs-out/brush-settings/`, not committed): Circle tip, hardness max
   (5 of 5), thickness 100, angle 0, density 100 and opacity 100 with no dynamics, size with
   no dynamics, ink Normal, no colour mixing/jitter, spray/texture/dual brush/watercolor edge
   off, gap Normal, Correct velocity input ON, stabilization 0, adjust by speed off,
   starting/ending none, "At least 1 pixel" off.
3. ▲ One dab = pen `in`, move, `down`, move at pressure 1, `up`, `out` with no motion while down
   (`dabs_script` in the run notes). This places exactly one dab.
4. ▲ Grid, 8 sizes {0.7, 1, 1.5, 2, 3, 4.5, 10, 25.3} in rows y = 25 + 46·r. Three columns
   with the pen sent to (60.5, y+0.5) (pixel centre), (200.75, y+0.5) (quarter offset), and
   (341.0, y+1.0) (pixel corner). One export per AA level: `data/B001_aa0..3.png`.
   vpen resolves positions to 0.1 px, so column 1 actually lands at x = 200.8.

## Results
Let d be the distance from the dab centre to the pixel centre, R = size/2, and
8-bit alpha = ⌊65535·c⌋ >> 8 (**truncation**, see B-003).
- **AA Weak/Middle/Strong:** c = min(R,1) · clamp((max(R,1) − d)/W, 0, 1),
  W = max(1, min(W_L, R)), W_L = 1.5 / 2.5 / 3.5.
  - For size ≥ 2 this is a linear ramp of width W ending at R.
  - Below 2 px, the dab keeps radius 1 and scales its amplitude by R. Example: size 1 at a
    pixel corner gives 4 × 37.
  - Sizes ≤ 3 are identical across the three levels.
- **AA None:** alpha 255 if d ≤ R (closed), else 0. There is no minimum pixel: size 0.7 at a
  pixel corner draws nothing.
- Exactness, from `b001_check.py data`, with no free parameters:

  | AA | px exact | dabs with any error |
  |---|---|---|
  | Weak | 1824/1824 (100 %) | 0/24 |
  | Middle | 1824/1824 (100 %) | 0/24 |
  | Strong | 99.6 %, max error 1 | 2/24 (size 25.3, 7 px) |
  | None | 1836/1860 (98.7 %) | 9/24 |

  All AA None errors are pixels at or very near d = R.
- Rounding instead of truncating drops the AA levels to 55–95 % exact (the earlier fit,
  `b001_fit_centre.py`).

## Open questions
- AA None tie rule. At exactly d = R CSP covers some neighbours but not others, depending on
  direction. Examples: size 2 at a pixel centre lights the top, left and right neighbours but
  not the bottom one. Size 1 at a pixel corner lights one pixel. B-002 also shows strokes at
  sizes 16 and 63 covering pixels up to ~0.1 px beyond R. Needs a dedicated sub-pixel sweep
  (sizes 2, 16, 63; offsets in 0.1 px steps).
- AA Strong ±1 at size 25.3 (7 px).
