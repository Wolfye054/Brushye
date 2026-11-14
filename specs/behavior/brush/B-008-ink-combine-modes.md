# B-008 Ink blending (combine) modes of the brush itself
- Status: **observed** (exact models for 28 of 35 modes at opacity 100 and 50; the rest within ±1–3 or the L-007 tolerances)
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Date / method / observer: 2026-11-13 / automated-Wine + virtual stylus / Claude

## Purpose
Brush-level modes (Sub Tool Detail > Ink > Blending mode): Normal, the layer modes, and the
tool-only ones. Are they the layer blend math (L-001/L-002/L-005), and what happens to alpha?

## The list (1.13.2 UI, in order)
Normal, Darken, Multiply, Color burn, Linear burn, **Black burn**, Subtract, Lighten, Screen,
Color dodge, Glow dodge, **White burn**, Add, Add (Glow), Overlay, Soft light, Hard light,
Difference, **Erase**, **Background**, **Replace alpha**, **Compare density**, **Erase (compare)**,
Vivid light, Linear light, Pin light, Hard mix, Exclusion, Darker color, Lighter color, Divide,
Hue, Saturation, Color, Brightness (35; bold = brush-only). Export files are numbered 00–34 in
this order.

## Setup (as run; deviations from the draft marked ▲)
- Base layer: `tools/observe/make_b008_input.py` (256×256: R = x, G = y, B = 255 − x; alpha
  255 for y < 128, 2(y − 128) below), opened in CSP as the document. Opening and re-exporting
  it is lossless (alpha and every visible colour).
- Sub tool "Obs pen" (B-001 setup): ▲ size 400 (covers the canvas from the centre), AA None,
  hardness 5, density 100, gap Normal, main colour S = (64,128,192) (= #4080c0).
- Per mode × opacity (100, 50): one dab at canvas (128.5, 128.5), export, undo
  (`tools/observe/b008_modes.sh`, which also screenshots the selected mode's label and checks
  that the canvas is back to the base before and after).
- Data: `data/B008_op100_NN_<mode>.png`, `data/B008_op50_NN_<mode>.png`.
- ▲ Second brush colour (200,40,120) for Black/White burn only: `data/B008_c2_op{100,50}_*.png`.
- The base image itself: `data/B008_base.png` (= `make_b008_input.py` output).
- Analysis: `b008_analyze.py` (opaque rows vs CSP's own layer exports, pure Pillow) and
  `b008_alpha.py` (every model below, whole image, numpy; reproduces the table).

## Results
Notation: b = backdrop colour, ab = backdrop alpha (0..255), S = brush colour, O = opacity as an
exact fraction (0.5, 1), A = 255·O, a8 = ⌊65535·O⌋ >> 8, B(b, S) = the layer blend result for the
mode (L-001 for separable modes, the L-007 integer model for the others),
round(v) = ⌊v + 0.5⌋.

1. **On opaque pixels every layer-type mode is the layer blend:** the 22 separable layer modes
   at opacity 100 are bit-exact against CSP's own L-001 layer exports. The colour modes
   (Darker/Lighter color, Hue, Saturation, Color, Brightness) match Brushye's L-007 model within
   the same open tolerances as layers.
2. **Stroke alpha is not quantised before compositing.** At opacity 50 the result is computed
   with O = 0.5 exactly and only the output is quantised: alpha truncated,
   A' = ⌊65535·(1 − (1 − ab/255)(1 − O))⌋ >> 8 (exact on all 65,536 px for every "paint" mode),
   colour rounded half up (Normal on an opaque pixel: round(b + (S − b)·O); 32.5 → 33). This
   also reproduces B-003 (the layer still stores 8 bits, see the B-003 follow-up below).
3. **Two alpha families.**
   - **Keep alpha** (the layer's alpha is unchanged, transparent pixels stay transparent):
     Color burn, Lighten, Screen, Color dodge, Add, Overlay, Soft light, Vivid light,
     Linear light, Pin light, Hard mix, Divide, Darker color, Lighter color, Hue, Saturation,
     Color; plus Glow dodge, Add (Glow), Black burn, White burn.
     Colour = round(b + (B(b, S) − b)·O). Screen uses its unrounded value b + S − bS/255.
   - **Paint** (alpha A'; the stroke also paints transparent pixels): Normal, Darken, Multiply,
     Linear burn, Subtract, Hard light, Difference, Exclusion, Brightness.
     Colour = round((r·A + b·(A' − A)) / A'), where r is the blend against **the backdrop
     composited over white**, bw = (b·ab + 255·(255 − ab) + 127) // 255, r = B(bw, S).
     So over a transparent pixel Subtract/Difference/Exclusion give 255 − S and Hard light
     gives B(255, S). Multiply uses r = S·(65025 − ab·(255 − b)) / 65025 unrounded.
     Brightness uses the raw backdrop, r = B(b, S).
4. **Brush-only and premultiplied modes** (all exact at both opacities unless noted):
   - Add (Glow): round(b + S·O), clamped; Glow dodge: ⌊b·255 / (255 − S·O)⌋ (L-001 dodge
     against the premultiplied colour). Alpha kept.
   - **Black burn / White burn: b − k / b + k with k = 64·O² (64 at 100 %, 16 at 50 %),
     independent of the brush colour** (same k with S = (64,128,192) and (200,40,120)). Alpha kept.
   - Erase: alpha round(ab·(1 − O)). Erase (compare): alpha min(ab, round(255·(1 − O))).
   - Replace alpha: colour S, alpha a8 everywhere the dab covers.
   - Compare density: where ab ≤ a8 the pixel becomes (S, a8), otherwise it is unchanged.
   - Background (paints behind): alpha A'; colour
     round((b·ab/255 + S·O·(1 − ab/255)) / (1 − (1 − ab/255)(1 − O))). Exact at 100 % except
     53 px (±1); at 50 % 58,287 / 65,536 (±1).

### Exactness (`b008_alpha.py data ../blending/data/observed 100 50`, all 65,536 px)
| Mode family | Opacity 100 | Opacity 50 |
|---|---|---|
| 12 keep-alpha separable modes, Add (Glow), Glow dodge, Black/White burn | all exact | all exact |
| Normal, Darken, Linear burn, Subtract, Hard light, Difference, Exclusion | all exact | opaque rows exact; 99.0–99.7 % overall, ±1 (semi-transparent rows) |
| Multiply | exact | 17 % (±2) — same open question as L-002 (Multiply with partial alpha) |
| Erase, Erase (compare), Replace alpha, Compare density | exact | exact |
| Background | 99.9 % (±1) | 88.9 % (±1) |
| Darker/Lighter color | 2 px off (luma ties, L-007) | same |
| Hue, Saturation, Color, Brightness | 71–99 % (±1–3, L-007 clip rule) | 85–99 % (±1–2) |

## Open questions
- Paint-family colour over semi-transparent pixels at opacity < 100 (±1 on ~1 % of px), and
  Multiply at opacity < 100 (±2): the exact rounding is open.
- Black/White burn: why 64·O²? (Only O = 0.5 and 1 measured; check 25 % and 75 %.) Does
  density change k?
- Brush modes inside one stroke: do overlapping dabs of one stroke combine before the mode
  is applied (B-003 buffer), as assumed? Only single dabs were used here.
