# L-001 Layer blending modes, opaque layers (all 28 modes)
- Status: **observed** (19 of 22 separable modes have an exact formula; the rest have exact
  observed tables and a closest-known formula)
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK
- Date / method / observer: 2026-11-11 / automated-Wine (Xvfb :77, Wine 11.4) / Claude

## Purpose
The exact 8-bit result of every raster-layer blending mode, for an opaque layer over an
opaque layer, including rounding.

## Setup (exact)
1. Generate inputs with `tools/observe/make_blend_inputs.py <dir>` (no ICC/gAMA chunks):
   - `base.png` 256×512 RGB. Rows 0–255: pixel (x,y) = gray x. Rows 256–511: random RGB (seed 1132).
   - `top.png` 256×512 RGB. Rows 0–255: pixel (x,y) = gray y. Rows 256–511: independent random RGB.
   The gray half covers **every** (base, top) pair of 8-bit values. The colour half covers
   cross-channel behaviour.
2. In CSP: File > Open `base.png` (one raster layer "base", no paper layer).
3. File > Open `top.png`, Select > All, Edit > Copy. Switch to the base.png tab, Edit > Paste.
   This makes layer "top Copy" above "base", aligned at (0,0).
4. With "top Copy" selected, set its blending mode in the Layer palette dropdown.
5. File > Export (Single Layer) > .png. Settings: Preview off, Expression color **RGB color**,
   Embed ICC on (the default; pixel values unaffected, see D-002), scale 100 %.
6. Repeat 4–5 for all 28 modes (`tools/observe/blend_export_loop.sh`).

Check: opening `base.png` and exporting it straight away is bit-identical to the input.

## Observations
**The 1.13.2 raster-layer blending modes, in dropdown order:** Normal, Darken, Multiply,
Color burn, Linear burn, Subtract, Lighten, Screen, Color dodge, Glow dodge, Add, Add (Glow),
Overlay, Soft light, Hard light, Difference, Vivid light, Linear light, Pin light, Hard mix,
Exclusion, Darker color, Lighter color, Divide, Hue, Saturation, Color, Brightness
(28 modes, with no Through on a raster layer).

**Data:** `data/observed/opaque_NN_<mode>.png` (iCCP chunk stripped; alpha was 255 everywhere,
so the files are stored as RGB). Inputs are in `data/inputs/`.

Notation: b = base (backdrop) 8-bit value, s = top (source) 8-bit value, per channel.
`//` is integer floor division. Results were checked by `tools/observe/check_blend_model.py`
on all 131,072 pixels × 3 channels.

| Mode | Exact formula (8-bit integer) | Match |
|---|---|---|
| Normal | s | exact |
| Darken | min(b,s) | exact |
| Multiply | (b·s + 127) // 255 | exact |
| Color burn | b=255 → 255; s=0 → 0; else clamp(255 − ((255−b)·255) // s) | exact |
| Linear burn | clamp(b + s − 255) | exact |
| Subtract | clamp(b − s) | exact |
| Lighten | max(b,s) | exact |
| Screen | b + s − (b·s + 127)//255 | exact |
| Color dodge | b=0 → 0; s=255 → 255; else min(255, (b·255) // (255−s)) | exact |
| Glow dodge | same as Color dodge **for opaque layers** (differs with alpha, see L-002) | exact |
| Add | min(255, b+s) | exact |
| Add (Glow) | same as Add **for opaque layers** (differs with alpha, see L-002) | exact |
| Soft light | W3C soft-light formula in float, rounded to nearest | exact |
| Difference | abs(b − s) | exact |
| Linear light | clamp(b + 2s − 255) | exact |
| Pin light | s<128 → min(b, 2s); else max(b, 2s−255) | exact |
| Hard mix | s<128 → (b+s ≥ 255 ? 255 : 0); else (b+s > 255 ? 255 : 0) | exact |
| Exclusion | b + s − (2·b·s + 127)//255 | exact |
| Divide | s=0 → (b>0 ? 255 : 0); else min(255, (b·255 + s//2) // s) | exact |
| Overlay | Hard light with b and s swapped. Multiply half (b<128) exact with (2bs+127)//255; screen half differs by ±1 on ~49 % of cells | **±1, open** |
| Hard light | W3C, same pattern as Overlay (screen half s≥128 ±1) | **±1, open** |
| Vivid light | W3C formula rounded: 99.04 % exact, ±1 elsewhere | **±1, open** |
| Darker color | W3C with luma .30/.59/.11: gray exact, colour 99.72 % (rest = the other pixel chosen; tie/luma rule differs) | **open** |
| Lighter color | as Darker color: 99.71 % | **open** |
| Hue / Saturation / Color / Brightness | W3C non-separable (SetLum/SetSat/ClipColor) with .30/.59/.11 is the closest model tried: gray exact, colour ~71–72 % exact, max error 6–8 | **open** |

Notes:
- Rounding differs between modes: Color burn, Color dodge and Divide-type divisions use
  floor or half-up integer division, while multiplications round to nearest. This is the main
  reason a float W3C implementation is off by one on ~24 % of cells.
- Luma weights: Rec.601 (.299/.587/.114), Rec.709, plain average, HSL and HSV variants all fit
  worse than .30/.59/.11 (tested 2026-11-11).
- Fixed-point internal models (8/10/12/15/16-bit scales with round/floor in and out) didn't
  reproduce the Overlay/Hard light screen half.

## Expected result (for Brushye)
The Brushye CPU compositor must reproduce `data/observed/opaque_*` **bit-exactly**. Until the
open formulas are found, use the observed 256×256 gray table as a LUT for Overlay, Hard light
and Vivid light (DECISIONS D-017). The non-separable modes need more observation (below).

## Open questions
1. Overlay/Hard light screen half and Vivid light: what is the exact integer formula?
   Try asymmetric operand order, e.g. 255 − ((255−b)·(255−(2s−255))) with different
   rounding at each step. The full observed table is ground truth either way.
2. Non-separable modes: needs a targeted test. Sweep one channel with the others fixed,
   saturated/unsaturated pairs, and pairs with exactly equal luma (Darker/Lighter tie rule).
3. Should any result here be re-checked on native Windows? The blend math runs inside CSP and
   isn't affected by Wine, but a single native spot check (e.g. 3 modes) would rule out
   Wine-specific differences. Flagged `verify-native` (low priority).
