# L-006 Clip to layer below; lock transparent pixels
- Status: **observed** (clipping exact for Normal/Multiply; lock: alpha exact, AA edge rule open)
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK
- Date / method / observer: 2026-11-13 / automated-Wine (Xvfb :77) / Claude (background run)

## Setup (exact)
1. Stack (bottom to top): `base` (data/inputs/base.png, opaque), "mid" = `top_alpha.png`
   (per-pixel alpha), "top" = `top.png` (opaque). Built by Open + Select All + Copy + Paste. Note:
   paste lands above the selected layer, and *inside* a folder if a folder child is selected.
2. Select "top"; Layer palette button **Clip to Layer Below** (first icon of the second row,
   (1723, 626) in the default 1920×1080 workspace). Export with top in Normal and in Multiply.
3. Hide top and base. Select mid and turn on **Lock transparent pixels** (button at (1803, 626)).
   Pen tool (Obs pen: size 30, opacity 100, AA Middle, colour (179,83,83)); two horizontal
   mouse strokes at canvas y = 93 and 343, x ≈ 9–247. Export (mid alone, transparent background).

## Observations (2026-11-13)
Data: `data/observed2/clip_00_normal.png`, `clip_02_multiply.png`, `lock_mid_strokes.png`.
1. **Clipping is bit-exact** with "the clipped layer is composited onto its clip base, and the
   result keeps the clip base's alpha":
   - Normal top: `out = lerp15(base, top, a_mid)`;
   - Multiply top: `out = lerp15(base, Multiply(mid, top), a_mid)`. The Multiply is computed
     against the **mid layer's colour**, not against what lies below the clip base.
2. **Lock transparent pixels:** alpha is **unchanged on every pixel** (including the 512 fully
   transparent ones). Inside the strokes the colour is replaced by the brush colour exactly, at
   every alpha. 15 227 pixels changed colour.

## Open questions
- Lock transparent pixels: the colour rule at the anti-aliased stroke edge (expected lerp by
  coverage; not verified because the coverage isn't known independently).
- Clipping with a semi-transparent clipped layer, and with the clip base in other modes.
