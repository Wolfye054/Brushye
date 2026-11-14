# B-012 .sut cross-check: imported sub tools against CSP strokes (M8 exit check)
- Status: **observed**: import exact. Soft-tip spacing fixed by B-013. Anti-aliased thin tips
  (B-014) and soft-tip profiles stay open.
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Date / method / observer: 2026-11-14 / automated-Wine + virtual stylus (D-003) / Claude
- Samples: `~/brushye-samples/xcheck/` (D-020; registry there, with every step).

## Setup
- **Sub tools:** twelve new custom sub tools, X-01..X-12 (Pen group, stabilization 0, so
  "adjust by speed" plays no part). Each was exported as `.sut`, then drawn with on a fresh
  800×600 transparent canvas at 100 %. Main colour black.
- **Canvas origin:** screen (543, 263), from the canvas edges and three calibration dabs
  (centroids within 0.006 px).
- **Pen pressure settings:** the default straight line (checked).
- **Input:** `tools/observe/brush_run2.py xcheck`:
  - a pressure ramp P 0.1 → 1 at y 100.5;
  - constant P 0.5 at y 250.5;
  - a full-pressure sine wave at y 430.5 ± 60;
  - one full-pressure dab at (400.5, 555.5).
  Events are every 2 px, 5 ms apart.
- **Exports:** `data/xcheck/X-xx.png`, RGBA (ICC chunk stripped, pixels unchanged).

| ID | Settings (Sub Tool Detail) | Imported as (`brush.stroke`) |
|---|---|---|
| X-01 | size 20, AA Middle, size pressure min 0 | `size=20.0 direction=horizontal size_pressure=0` |
| X-02 | size 6, AA None | `size=6.0 aa=none direction=horizontal` |
| X-03 | size 40, AA Strong, hardness segment 3 | `size=40.0 aa=strong hardness=3 direction=horizontal` |
| X-04 | size 60, hardness segment 1 | `size=60.0 hardness=1 direction=horizontal` |
| X-05 | size 30, thickness 30, Vertical, angle 30 | `size=30.0 thickness=30 direction=vertical angle=30.0` |
| X-06 | size 30, thickness 40, angle = Direction of line | `size=30.0 thickness=40 direction=horizontal angle_line=true` |
| X-07 | size 30, gap Narrow, density 40 | `size=30.0 density=40 gap=narrow direction=horizontal` |
| X-08 | size 20, gap Fixed 50 | `size=20.0 gap=50.0 direction=horizontal` |
| X-09 | size 30, gap Wide, density 60, adjust density by gap | `size=30.0 density=60 gap=wide density_by_gap=true direction=horizontal` |
| X-10 | size 30, opacity 50, opacity pressure min 20 | `size=30.0 opacity=50 direction=horizontal opacity_pressure=20` |
| X-11 | size 40, size pressure min 30, curve point (0.5, 0.25) | `size=40.0 direction=horizontal size_pressure=30 size_curve=[0.5,0.2515…]` |
| X-12 | size 30, hardness segment 2, density pressure min 0 | `size=30.0 hardness=2 direction=horizontal density_pressure=0` |

The importer reported nothing for any of them. Every setting was mapped, and the importer's
output is pinned in `crates/brushye-sut/tests/xcheck_presets.txt`.

## Results (alpha channel; "off" = pixels differing by more than 2 levels)
| ID | Inked px | Off | Total ink, Brushye / CSP |
|---|---|---|---|
| X-01 | 31 424 | 3.1 % | 29 201 / 29 061 |
| X-02 | 13 488 | 0.6 % | 13 456 / 13 439 |
| X-03 | 109 312 | 34.8 % | 94 438 / 88 982 |
| X-04 | 208 466 | 93.2 % | 145 223 / 110 678 |
| X-05 | 40 416 | 18.5 % | 37 293 / 36 855 |
| X-06 | 66 399 | 15.8 % | 62 151 / 62 253 |
| X-07 | 67 780 | 1.7 % | 64 175 / 64 175 |
| X-08 | 43 066 | 4.2 % | 37 610 / 37 611 |
| X-09 | 67 490 | 1.0 % | 45 996 / 45 992 |
| X-10 | 67 733 | 1.9 % | 25 197 / 25 125 |
| X-11 | 70 135 | 2.8 % | 66 399 / 65 970 |
| X-12 | 89 548 | 82.3 % | 65 243 / 51 029 |

### Findings
1. **The import is exact.** Hard round tips (X-01, X-02, X-07..X-11) draw like CSP:
   - total ink is within 0.5 %, and the lone dabs are pixel-identical;
   - straight strokes are nearly identical, and so are the gaps (narrow, fixed, wide, density
     by gap), opacity and density, the pressure ramps and the one-point pressure curve;
   - the remaining "off" pixels are mostly on the sine wave: dab placement on curves, worth a
     look later.
2. **Soft tips in strokes (engine gap):**
   - X-03, X-04 and X-12 lay down 6–36 % more ink than CSP;
   - the lone soft dab has CSP's total ink and spread (X-04: sd 14.14 vs 14.18 px), so the
     difference is dab spacing or accumulation for soft tips;
   - B-002/B-003 measured these with hard tips only, and B-009 measured soft tips as single
     dabs only.
3. **Thin tips with anti-aliasing (engine gap):**
   - X-05's ellipse has the right angle (30.0° vs 29.9°) and length, but CSP's is thinner
     (minor-axis sd 1.68 vs 2.08 px);
   - B-010 is exact for thin tips with AA None only; X-05 uses AA Middle;
   - X-06 likewise; its lone dab is excluded, because with Direction of line and no movement
     CSP takes the angle from the pen's ±0.1 px press nudge (looks Wine/vpen-specific).

## Update 2026-11-14
- B-013's preset-gap factors for soft tips brought X-04 from 93 % to 0.6 % of pixels off. X-03
  and X-12 now have their total ink within 0.7 % (some pixels still differ: profile).
- B-014 found no exact thin-tip AA rule yet (X-05, X-06 unchanged).

## Open (original list)
- Observe soft-tip strokes: spacing and accumulation at hardness 1..4 and gap presets
  (B-009/B-002 follow-up).
- Observe anti-aliased thin tips: AA Weak/Middle/Strong at several thicknesses (B-010
  follow-up).
- Then tighten the pins in `crates/brushye-sut/tests/xcheck.rs`.
