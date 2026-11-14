# B-009 Circle tip hardness falloff
- Status: **implemented (interim profile table, scaled by R; offset +0.5 px)**
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Date / method / observer: 2026-11-13 / automated-Wine + virtual stylus / Claude

## Setup (as run; deviations marked ▲)
B-001 setup and single-dab method, size 101, AA None, density 100, opacity 100.
▲ Hardness is a **5-step segmented control**, not a slider. It sits in Sub Tool Detail >
Brush tip, and its levels are numbered 1 (softest) to 5 (hardest). One dab per level at
(300.5, 200.5): `data/B009_h1..5_aa0.png`. Also `B009_h1_aa2.png` and `B009_h3_aa2.png` at
AA Middle.

## Results
- AA has no effect on soft tips: h1 at AA None and at AA Middle are pixel-identical. h3 at
  AA None and AA Middle are also stored, for comparison.
- Level 5 is the hard disc of B-001. Level 4 is still a hard edge, but beyond R it has a
  narrow ramp (…255, 202, 77, 0).

| level | peak | outer radius (last α>0) | area Σα/255 (px²) |
|---|---|---|---|
| 1 | 229 | 71.87 | 3467.4 |
| 2 | 255 | 66.13 | 4830.4 |
| 3 | 255 | 59.91 | 6444.9 |
| 4 | 255 | 51.79 | 7879.9 |
| 5 | 255 | 50.57 | 8033.0 |

So softer tips reach beyond R = 50.5 and carry less total coverage. Each +x profile is listed
by `b009_profile.py data/B009_h?_aa0.png`.

### Models tried (none exact)
- 1D erf edge, A·½erfc((d−c)/(s√2)), from `b009_fit.py`:

  | level | c | s | mean error | max error |
  |---|---|---|---|---|
  | h1 | 27.75 | 20 | 2.0 | 6 |
  | h2 | 37.25 | 13 | 2.67 | 8 |
  | h3 | 44.75 | 6.5 | 3.47 | 14 |
  | h4 | — | — | — | 119 |

- 2D disc convolved with a Gaussian, with truncation (fitted on the +x and diagonal
  profiles, run notes only):

  | level | disc R | σ | max error | mean error |
  |---|---|---|---|---|
  | h1 | 33.5 | 17 | 10 | 3.9 |
  | h2 | 40 | 13 | 5 | 1.5 |
  | h3 | 46 | 6.5 | 5 | 0.8 |
  | h4 | 50.5 | 1 | 36 | — |

- A Gaussian falloff fits poorly (max error 24–170).

## Open questions
- The exact falloff function per level, and how it scales with size. Only size 101 was run.
  Next: the same 5 levels at sizes 11 and 41, then fit a profile in d/R.
- Is level 4's ramp beyond R a fixed pixel width or a fraction of size?

## Follow-up 2026-11-13 (run 2): size scaling and the soft-tip centre
Method: automated-Wine + virtual stylus, B-001 setup (AA None, density 100, opacity 100),
▲ 800×800 transparent canvas. `tools/observe/brush_run2.py hard SIZE` places single dabs
(layout `HARD_POINTS`): size 11 at (40.5, 40.5), size 41 at (120.5, 60.5), size 41 at
x = 60.5 + 60k + f with f = 0.0 … 0.9 in 0.1 steps (y = 160.5 and 260.5), and size 11 at
x = 40.5 + 30k + k/10 (y = 340.5). Hardness 1 … 5, sizes 11 and 41, one export each:
`data/B009_run2_h<L>_s<SIZE>.png`.

1. **The soft-tip centre offset is fixed, +0.5 px on both axes**, not rounding to the pixel
   grid. Alpha-weighted centroids minus the press point (`b009_run2_analyze.py data`):
   - h1, h2, h3 at sizes 11 and 41: (+0.50 ± 0.04, +0.50 ± 0.02) at every sub-pixel press
     position 0.0 … 0.9 (grid rounding would give offsets that change with the position);
   - h4: +0.38 … +0.50 (its ramp is narrower than a pixel at these sizes, so the centroid is
     noisy); h5 (hard): 0.00, within the pixelisation noise of a hard disc.
2. **The profiles scale with the radius.** Predicting every pixel from the size-101 profile
   scaled by R, with the +0.5 offset, exactly like Brushye's interim table
   (`b009_scaling.py data`):

   | level | size 11 | size 41 |
   |---|---|---|
   | 1 | 100 % within 1 (max 1) | 99.7 % within 1 (max 2) |
   | 2 | 99.0 % within 1 (max 4) | 100 % within 1 (max 2) |
   | 3 | 96.9 % within 1 (max 15) | 99.6 % within 1 (max 2) |
   | 4 | 89 % within 1 (max 255) | 93 % within 1 (max 97) |

   Areas Σα/255 divided by R² agree to 3 digits across sizes 11, 41 and 101 (h1: 1.36).
3. **Level 4 also scales with R**: the ramp runs from about R − 0.04R to R + 0.02R at all three
   sizes (≈ 3 px at size 101, 1.25 px at 41, 0.35 px at 11). Its large errors at 11 and 41 are
   single edge pixels flipping where that ramp is narrower than a pixel. It is point-sampled,
   the same kind of error as the AA None edge rule (B-001).

Status of the open questions: centre offset **answered** (fixed +0.5); scaling **answered** for
levels 1–3 (R-relative profile). Still open: an exact formula for the profile (the table is
empirical) and the exact sub-pixel behaviour of level 4.

## Implementation (2026-11-13, M4)
Interim model until the exact falloff is known: the observed radial profile per level, binned
at 0.25 px (`tools/observe/make_hardness_tables.py` → `brush/hardness_tables.rs`), scaled by
50.5/R for other sizes. At size 101: within 1 level (h1, h2), 2 (h3), 16 (h4).

Finding: **soft dabs are centred half a pixel off the press point.** The h1–h4 dabs are centred
on the pixel corner (301, 201), not on (300.5, 200.5) where the pen went down (the +x profile at
pixel i equals the −x profile at pixel i − 1); the hard tip (h5) is centred on the press point.
Brushye offsets soft dabs by (+0.5, +0.5). Open: is this a fixed offset or rounding of the centre
to the pixel grid? Sweep sub-pixel press positions with a soft tip.

## Implementation, M4 part 2 (2026-11-13)
Run-2 sizes (`b009_run2_soft_tip_sizes`): h1 within 1 level at sizes 11 and 41, h2 within 4/2,
h3 within 15/2, h4 single edge pixels; h5 (hard) 1 px (a tie).
