# B-013 Soft-tip strokes: gap presets space soft dabs further apart
- Status: **implemented** (levels 1–3 and 5 exact to < 1 % of stroke ink; level 4's tip profile open)
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Date / method / observer: 2026-11-14 / automated-Wine + virtual stylus (D-003) / Claude
- Registry: `~/brushye-samples/b013_b014/registry.md` (every file's SHA-256 and steps).

## Setup
- **Sub tool:** new custom sub tool "B013": size 40, size pressure off, stabilization 0,
  AA Middle, density 30, opacity 100, black.
- **Canvases:** fresh 800×600 transparent canvas per configuration, origin at screen (543, 263)
  (re-verified with 3 calibration dabs, within 0.003 px).
- **Input:** `brush_run2.py softrow`: a straight full-pressure stroke (100.5, 200.5) →
  (700.5, 200.5), events every 2 px, 5 ms apart, then one dab at (400.5, 420.5).
- **Exports:** `data/B013_h{1..5}_{narrow,normal,wide,fixed300}.png` and
  `B013_h1_normal_d100.png`. Hardness levels are bar segments 1..5 (stored 1, 30, 60, 95, 100).

## Results (`crates/brushye-core/tests/b013_soft_strokes.rs`)
- **A soft dab in a stroke is the lone dab.** With Fixed 300 (isolated dabs), stroke ink = 4 ×
  the lone dab at every level, and Brushye matches it to 0.4 %.
- **The presets space soft dabs further apart.** With Narrow/Normal/Wide, the old model gave
  up to +123 % ink at level 1. Fitting a spacing factor f (gap = f · preset %) to CSP's stroke
  ink gives one value per level that holds for all three presets:

  | Level | f fitted (narrow / normal / wide) | Used |
  |---|---|---|
  | 1 | 2.992 / 2.997 / 2.994 | 3.0 |
  | 2 | 2.751 / 2.752 / 2.752 | 2.75 |
  | 3 | 2.005 / 1.998 / 1.999 | 2.0 |
  | 4 | (not separable from its profile error, see below) | 1.0 |
  | 5 | exact without a factor | 1.0 |

  Level 4 uses f = 1: its stroke excess (+3.6 … +8.2 %) is the same as its lone-dab excess
  (+7.3 %), so its spacing matches a hard tip's. Fitted with the lone-dab ink factored out, as
  printed above for levels 1–3.
- **With the factors** (`Gap::percent_for`, `SOFT_GAP_FACTORS`): stroke ink and centre-line
  level match CSP to < 0.6 % at levels 1–3 and exactly at level 5, for every preset.
  - Cross-check B-012: X-04 (level 1) went from 93 % of pixels off to 0.6 %.
  - X-03 and X-12 now have their total ink right (< 0.7 %), but some pixels still differ by up
    to 46 and 89 levels: the interim soft-tip profile (B-009).

## Open
- **Level 4's tip profile:** its lone dab has 7 % more ink than CSP's at size 40. B-009
  measured it at size 101, and scaling by R doesn't hold for it.
- **Soft-tip profiles at sizes other than 101** (the per-pixel differences above).
- **Hardness values between the bar's steps** (typed values) use the nearest level's factor.
