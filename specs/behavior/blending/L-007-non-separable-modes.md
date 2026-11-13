# L-007 Non-separable modes: Hue, Saturation, Color, Brightness, Darker color, Lighter color
- Status: **observed** (Darker/Lighter: rule found, tie rounding open; Hue/Sat/Color/Brightness:
  integer pipeline found for unclipped cases, clipping rule open)
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK
- Date / method / observer: 2026-11-13 / automated-Wine (Xvfb :77) / Claude (background run)

## Setup (exact)
Same document as L-003 (`combo_base.png` + `combo_top.png`, opacity 100). Rows used:
- 0–191: one-channel sweeps. Block `row//64` is the swept channel. In rows 0–31 of a block the
  top channel sweeps 0–255 over a fixed base; in rows 32–63 the base channel sweeps over a fixed
  top. 16 fixed colours (`FIXED` in the generator).
- 192–351: 40 960 random pairs with equal `30R+59G+11B`; 352–511: pairs with equal `77R+151G+28B`.
Also the L-001 random-colour data. Exports: `data/observed2/combo_{hue,saturation,color,
brightness,darker_color,lighter_color}.png` (rows 0–511 stored). Inputs: `data/inputs2/`. Models: `tools/observe/nonsep_models.py`.

## Observations (2026-11-13)
**Luma** is `0.30R + 0.59G + 0.11B` (W3C weights): a grid search over weights peaks sharply
there (54 % exact vs ≤ 27 % for any neighbour at ±0.01). Rec.601 and Rec.709 are clearly
wrong. CSP works on 8-bit **integers**, not floats:

1. **Darker color / Lighter color** pick the whole top or base pixel (every output pixel equals
   one of the two inputs). The comparison is on **integer-rounded luma**
   `L = round((30R+59G+11B)/100)`, and **ties go to the top layer**. This is 99.77 % exact on the
   designed data and 99.998 % on the L-001 random data (W3C float: 54 %/99.7 %). The remaining
   errors are all pixels whose luma is exactly x.5; neither round-half-up/down nor float32/64
   evaluation resolves them all (open).
2. **Hue / Saturation / Color / Brightness** use an integer version of W3C SetSat/SetLum:
   - SetSat in integers: `c' = ((c - min)·sat + r/2) // r` (r = the colour's max − min), i.e. rounded;
   - SetLum: `c'' = c' + (round(Lum(target)) − round(Lum(c')))`, both lumas rounded as in 1.
   Without clipping (result inside 0..255) this is 99.0 % exact for Hue (W3C float: 64 %);
   the remaining 1 % are again x.5 luma ties.
   Overall exact match (all pixels): Hue 86.5 % (W3C 54.7 %), Saturation 90.2 % (56.6 %), Color
   93.5 % (82.5 %), Brightness 93.5 % (82.5 %); max error 4–6 (W3C 5–8).
3. **Clipping differs from W3C ClipColor.** When the shifted colour leaves 0..255, CSP keeps the
   hue ratio and clamps the extreme channel, but the result is not luma-preserving. Example:
   base (255,0,0), top (0,30,200), Hue → CSP (35,68,255) (luma 78.7, target 77); W3C (32,66,255).
   Rounding variants of W3C's ClipColor were tried; none fit.

## Expected result (for Brushye)
Implement items 1–2 as integer code; until the clip rule and the tie rule are found, the D-017
test tolerance for these modes is: Darker/Lighter exact except x.5-luma ties; Hue/Saturation/
Color/Brightness exact for unclipped pixels except ties, ≤ 6 for clipped pixels.

## Open questions
- Tie rounding of `Lum` at exactly x.5 (affects ~0.2–1 % of pixels).
- The clip rule for results outside 0..255 (designed data: rows where the shifted colour
  exceeds 255 or goes below 0 are identified by the model; fit a clip rule there).
