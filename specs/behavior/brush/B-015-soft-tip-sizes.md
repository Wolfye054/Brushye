# B-015 Soft-tip single dabs across sizes (AA Middle)
- Status: **implemented** (interim where marked)
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Date / method / observer: 2026-11-14 / automated-Wine + virtual stylus (D-003) / Claude
- Registry: `~/brushye-samples/b014_b015/registry.md`.

## Setup
- **Sub tool:** new custom sub tool "B015": size pressure off, stabilization 0, density 100,
  opacity 100, AA Middle, black.
- **Input:** `brush_run2.py softdabs`: single dabs at (200.5, 150.5), (600.8, 150.2),
  (200.3, 450.7) and (600.5, 450.5) on an 800×600 canvas (origin (543, 263)).
- **Exports:** `data/B015_h{1..4}_s{8,16,32,64,128}.png` and the hard control
  `B015_h5_s32.png` (Brushye matches it exactly).

## Findings (`crates/brushye-core/tests/b015_soft_profiles.rs`)
1. **Levels 1–3 scale with R** at sizes 32–128: the radial profile, as a function of d/R, is
   the same at every size. Brushye's size-101 table scaled by R matches CSP's ink within 0.4 %.
   Below that, size 16 is within 3.3 % and size 8 runs heavy (+6, +10, +24 % at levels 1–3):
   open.
2. **The +0.5 px soft-dab offset depends on AA and size.** Dab centroids minus the press point,
   averaged over the 4 dabs:

   | Level | s8 | s16 | s32 | s64 | s128 |
   |---|---|---|---|---|---|
   | 1 | 0.00 | 0.01 | 0.01 | **0.52** | **0.55** |
   | 2 | 0.00 | 0.00 | 0.01 | 0.02 | **0.54** |
   | 3 | 0.00 | 0.00 | 0.01 | 0.02 | **0.54** |
   | 4 | 0.00 | 0.00 | 0.00 | 0.01 | 0.02 |

   The cross-check (B-012 X-04: level 1, size 60, offset present) narrows level 1's switch to
   between 32 and 60. With AA None every soft dab is offset (B-009 run 2: sizes 11 and 41).
   - Brushye (`SOFT_OFFSET_FROM_SIZE`): with AA on, the offset applies from size 60 (level 1)
     or 128 (levels 2–3) on, and never at level 4. The switch points between the sampled sizes
     are interim.
3. **Level 4 keeps a hard edge, and anti-aliasing applies to it.**
   - B-009 measured level 4 with AA None. With AA Middle its profile is capped by the round
     tip's AA ramp, ending 0.25 px beyond R (`LEVEL4_AA_EDGE`, fitted).
   - Ink went from +90 % / +31 % / +11 % (sizes 8 / 16 / 32) to −1 … +2 % at sizes 16–128
     (+10 % at size 8). B-013's level-4 strokes are now within 3 %.
4. **CSP's level 1 never reaches full alpha:** the centre peaks at 216–230 at every size.

## Effect on the cross-check (B-012)
X-12 (level 2): 58 % → 2.8 % of pixels off. X-03 (level 3, AA Strong): 31 % → 17 %, worst pixel
16 levels. X-04 is unchanged at 0.6 %. These change journal replay for soft brushes, accepted
before 1.0 (D-021).

## Open
- Size 8 at levels 1–3.
- The exact size at which the offset switches on, and why.
- AA Weak and Strong on level 4 (only Middle was sampled; the code uses each level's ramp).
