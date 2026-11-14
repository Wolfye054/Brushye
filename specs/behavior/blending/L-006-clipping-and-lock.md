# L-006 Clip to layer below; lock transparent pixels
- Status: **observed** (clipping exact for Normal/Multiply; run 2 2026-11-14: semi-transparent clipped layer, Multiply base, folder bases; lock: alpha exact, AA edge rule open)
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

## Run 2 (2026-11-14): semi-transparent clipped layer, clip base modes, folders
Method: automated-Wine (Xvfb :77). Data: `data/observed3/clip_*.png`. Checks:
`tools/observe/l006b_check.py`.

Setup:
1. base.png, then `top_alpha Copy` (clip base, top_alpha.png), then `top Copy` (top.png) with
   Clip to Layer Below.
2. Model (L-006 run 1): the clipped layers composite onto the clip base's colour, treated as
   opaque. The group keeps the base's alpha, then composites onto the backdrop with the base's
   blend mode and opacity.

Results, 131 072 px each:
- `clip_normal`: **exact** (the run 1 result again).
- `clip_clipped_op50`: the clipped layer at 50 % opacity. **Exact**: it composites onto the
  base colour at its own effective alpha (L-003).
- `clip_base_multiply`: the clip base in Multiply. CSP's output is **identical** to its own
  plain Multiply layer with the base's alpha (L-002 `alpha_02_multiply`, 0 px differ).
  - So the group model holds.
  - Brushye's prediction is ±2 on the partial-alpha pixels only, which is L-002's open
    Multiply rule (the same 79 841 px).
- `clip_onto_folder_normal`: a Normal folder as clip base works like a layer: the folder's
  composited result is the base. **Exact**, identical to `clip_normal`.
- `clip_onto_folder_through`: with a **Through** folder as clip base, the clipped layer is
  **not clipped at all**. The output equals the unclipped stack exactly.
  - The Layer palette shows the clip bar **grey instead of red** (U-014).
  - This corrects the manual note "treated as Normal" (docs/research).

## Run 3 (2026-11-14): hidden base, no base
Method: automated-Wine (Xvfb :77). Data: `data/observed3/clip_{hidden_base,bottom_layer,
bottom_of_folder}.png`. Setup: base, `top_alpha Copy`, `top Copy` (clipped), as in run 2.

- `clip_hidden_base`: the clip base (top_alpha Copy) is hidden. The clipped layer is **not
  drawn either**: the export equals base alone (0 px differ).
  - In the Layer palette the clipped layer's eye is dimmed. While it is selected, the blend
    and opacity controls are greyed out.
- `clip_bottom_layer`: Clip to Layer Below is **available on the bottom layer**. With no layer
  below it, the layer draws **normally**: the export equals the unclipped stack.
  - Its clip bar is **grey**.
- `clip_bottom_of_folder`: a clipped layer at the bottom of a folder (top Copy alone in
  Folder 1, with top_alpha below the folder) is **not clipped**: the export equals the
  unclipped stack. The base is looked for only among its siblings in the same folder.
  - Its bar is grey.
  - Create folder and insert layer (Ctrl+G) keeps the layer's clip setting.

## Open questions
- Lock transparent pixels: the colour rule at the anti-aliased stroke edge (expected lerp by
  coverage; not verified because the coverage isn't known independently).
- Several clipped layers on one base, and clip bases in modes other than Normal/Multiply
  (expected to follow the group model).
