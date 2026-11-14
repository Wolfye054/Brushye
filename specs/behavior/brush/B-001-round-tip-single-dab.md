# B-001 Round tip: single-dab profile vs size and anti-aliasing
- Status: **implemented (AA None scanline rule; ties open)**
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

## Follow-up 2026-11-13 (run 2): the AA None edge rule
Method: automated-Wine + virtual stylus, same sub tool (AA None, hardness 5, density 100,
opacity 100), ▲ 800×800 transparent canvas (origin screen (549, 163)). `tools/observe/
brush_run2.py sweep SIZE`: 10×10 single dabs per size, press point = origin + pitch·(i, j) +
(i/10, j/10), so every 0.1 px sub-pixel offset in x and y is covered. Sizes 1, 2, 3, 5, 16, 63:
`data/B001_sweep_s<SIZE>.png`. ▲ Plus the size-16 sweep at AA Weak:
`data/B001_sweep_aa1_s16.png`.

1. **The dabs land exactly where the pen was sent.** Fitting each AA Weak dab with the exact
   model above (`b001_centres.py data 16 1`, 1/100 px grid search) gives offset (0.00, 0.00)
   for all 100 dabs, residual ≤ 2 alpha levels per dab. So the AA None misses are not input
   error.
2. **AA None is a scanline fill, not a closed disc.** A row whose centre is at dy from the dab
   centre is drawn when |dy| ≤ R; in it, a pixel is lit when **|dx| ≤ √(R² − dy²) + 0.1**.
   `b001_sweep_analyze.py data` (edge band = pixels within 1.5 px of the circle):

   | size | span + 0.1 | closed disc d ≤ R | 4×4 supersample |
   |---|---|---|---|
   | 1 | 1237/1253 (16 wrong) | 26 wrong | 52 |
   | 2 | 1943/1957 (14) | 44 | 68 |
   | 3 | 2802/2817 (15) | 63 | 71 |
   | 5 | 4686/4704 (18) | 100 | 76 |
   | 16 | 15080/15096 (16) | 322 | 242 |
   | 63 | 59379/59394 (15) | 1263 | 887 |

   The constant 0.1 px is the same at every size (0.09 and 0.11 are worse; adding a row
   offset does not help). It explains why run 1 saw "pixels up to ~0.1 px beyond R" in the
   horizontal direction only.
3. The 14–18 remaining pixels per size are exact ties:
   - tangent rows (|dy| = R exactly): the **top** row (dy = −R) lights the pixel under the
     centre, the bottom row (dy = +R) lights nothing;
   - pixels at exactly |dx| = √(R² − dy²) + 0.1 go either way.
   Size 1 at a pixel corner lighting the top-left pixel (run 1) is the tangent-row case.

Status of the open question: **answered** (rule above), apart from the tie handling.

## Open questions
- (Answered 2026-11-13, see the follow-up: scanline rule with +0.1 px in x.) AA None tie rule. At exactly d = R CSP covers some neighbours but not others, depending on
  direction. Examples: size 2 at a pixel centre lights the top, left and right neighbours but
  not the bottom one. Size 1 at a pixel corner lights one pixel. B-002 also shows strokes at
  sizes 16 and 63 covering pixels up to ~0.1 px beyond R. Needs a dedicated sub-pixel sweep
  (sizes 2, 16, 63; offsets in 0.1 px steps).
- AA Strong ±1 at size 25.3 (7 px).

## Implementation (2026-11-13, M4)
`brushye_core::brush::tip::coverage`. Test `observed_brush.rs` (`b001_*`) replays the grid:
- Weak and Middle: every pixel exact; Strong: 7 px off by 1 (as in the spec).
- AA None, closed disc: 24 px wrong, all on the open edge-tie rule. One of them is 0.21 px
  outside the edge (size 1 at a pixel corner lights one pixel).

## Implementation, M4 part 2 (2026-11-13)
`tip::coverage` implements the scanline rule (rows −R ≤ dy < R, |dx| ≤ √(R² − dy²) + 0.1).
Run-1 grid: 4 of 1858 px wrong (was 24 with the closed disc). Run-2 sweeps
(`b001_run2_scanline_edge_rule`): 9–20 px wrong per 100 dabs, all exact ties; AA Weak sweep
59 px off by 1.
