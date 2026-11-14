# L-008 Layer masks: values, alpha, opacity
- Status: **observed** (raster-layer masks: exact; folder masks: see L-004 run 2)
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK
- Date / method / observer: 2026-11-14 / automated-Wine (Xvfb :77, XTEST mouse drags) / Claude
- Data: `data/observed3/mask*.png` (re-encoded without the sRGB iCCP chunk; pixels unchanged).
  Inputs: `data/inputs/{base,top,top_alpha}.png` (L-001/L-002).

## Setup (exact)
**Run 1 (opaque layer):**
1. File > Open `base.png`, then File > Open `top.png`, Select All, Copy; switch to the base.png
   tab and Edit > Paste. This gives "top Copy" (opaque) over "base", both Normal 100 %.
2. Layer palette "Create layer mask" button (no selection), which makes a reveal-all mask that
   is the paint target.
3. Mask strokes, as horizontal XTEST drags across the full canvas width (screen x 794→1090,
   canvas origin (814, 306), 100 % zoom; 40 steps, 10 ms):
   - Eraser > Hard, size 30, at canvas y 40 and 330.
   - Eraser > Soft, size 60, at y 150 and 420.
   - Pen "B014c" (size 41, AA middle, black) on the mask from canvas x −20 to 128 at y 40.
4. Exports (File > Export (Single Layer) > .png, RGB):
   - the composite: `mask1_normal`, `mask1_multiply`, and `mask1_normal_op50` (layer opacity 50)
   - top Copy alone, base hidden: `mask1_top_alone`, so alpha = mask value M
   - after Layer Mask > Apply Mask to Layer, at opacity 50: `mask1_applied_op50`
   - Also exported but not kept, since they're identical to kept files: the mask disabled
     (= top.png), and applied composite / applied alone / applied Multiply (each identical to
     its masked counterpart).

**Run 2 (semi-transparent layer):**
1. Same base, with `top_alpha.png` pasted as "top_alpha Copy". A new raster layer "Layer 1"
   above it, Edit > Fill with black, which makes it opaque.
2. Reveal-all masks on both layers, then the same eraser strokes from run 3.1 (no pen stroke)
   on each mask. Replayed XTEST drags give identical masks: run 2's M equals run 1's M
   everywhere outside run 1's pen stroke (rows 33–44).
3. Exports:
   - `mask2_readout_alone`: Layer 1 alone, alpha = M
   - `mask2_top_alpha_alone`: top_alpha Copy alone
   - `mask2_top_alpha_normal`: top_alpha Copy over base, Normal 100 %
   - `mask2_top_alpha_normal_op50`: the same at opacity 50

## Observations (all bit-exact, 131 072 pixels each)
- **Masks are continuous** (Layer Property > Mask expression > Show gradients: Yes, the
  default). M takes 211 distinct values in run 1. The soft eraser leaves about 26–30 at its
  centre.
- **Painting on a mask:** the eraser lowers M. The pen with **black** ink (opaque) raises M to
  255. So any opaque colour reveals and the colour is ignored, as the manual says.
- **Mask × pixel alpha:** a masked layer behaves exactly like a layer whose alpha is
  `f = floor(a · M / 255)` (a = pixel alpha). Colour is unchanged. Apply Mask to Layer writes
  exactly f.
- **Composite:** at 100 % opacity, Normal, the result is the usual L-002 lerp with alpha f.
  Multiply with mask = Multiply of the applied layer (bit-exact). Brushye's own model for
  Multiply with partial alpha is still ±2 (L-002/L-003 open, not a mask issue).
- **Opacity with a mask:** opacity first acts on the pixel alpha (L-003:
  `(a · (o8 + 1)) >> 8`, o8 = ⌊op · 255 / 100⌋). Then the mask multiplies with a floor:
  **`eff = floor(((a · (o8 + 1)) >> 8) · M / 255)`**.
  - This is **not** the same as applying the mask and then setting opacity. Applied plus 50 %
    differs from masked plus 50 % on 10 002 pixels (±1).
  - Checked on the opaque layer (a = 255) and on top_alpha (all a).
- **Enable Mask off:** the layer renders as if it had no mask (bit-exact).

## Models tried (for the record)
- f: `round(a·M/255)` (22 273 off), `(a·(M+1))>>8` (10 728), a 15-bit product (36 861).
  Only `floor(a·M/255)` is exact.
- Opacity: `floor(f·o8/255)` (27 232 off), `(f·(o8+1))>>8` (2 949), and mask-then-opacity
  orders. Only opacity-then-mask with a floor is exact.

## Open questions
- Masks on folders: observed in L-004 run 2. A Normal folder follows this spec's alpha rule;
  a Through folder lerps the backdrop with weight `floor(M·32768/255)`.
- A masked layer as a clip base or as a clipped layer: expected to follow the alpha rule above
  (not run).
- Mask expression > Show gradients other than Yes (threshold masks).
- Painting a mask with partially transparent ink or lower brush opacity: expected to follow
  the brush's alpha output. Not run.
