# L-005 Blending over a transparent / semi-transparent backdrop
- Status: **observed** (output alpha exact for all 28 modes; colour exact-ish for Normal only)
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK
- Date / method / observer: 2026-11-13 / automated-Wine (Xvfb :77) / Claude (background run)

## Setup (exact)
1. Inputs (`tools/observe/make_blend_inputs2.py`): `bd_base.png` and `bd_top.png`, 256×512 RGBA.
   Rows 0–255: base gray x, top gray y. Rows 256–511: random colours (seed 2027). Base alpha
   `(7x+13y) mod 256`, top alpha `(11x+5y+91) mod 256`.
2. Open `bd_top.png`, Select All, Copy, Deselect; open `bd_base.png`; Paste. There is no paper
   layer, so the canvas is transparent.
3. For all 28 modes (opacity 100): export PNG. **The PNG export keeps transparency** when the
   document has no paper layer (output alpha 1–255 observed).

## Observations (2026-11-13)
Data: `data/observed2/bd_NN_<mode>.png`. `bd_00_normal` is full height; the other modes store
the gray half (rows 0–255) only. Checker: `tools/observe/check_backdrop_model.py`. Notation:
a = 8-bit alpha, `x15 = (x·32768+127)//255`.

**Output alpha: three paths, each bit-exact:**
| Modes | Output alpha |
|---|---|
| Normal | `(ab·(32768 − at15) + 255·at15) >> 15` (15-bit lerp of alpha toward 255, floor) |
| Screen, Color dodge, Glow dodge, Soft light, Hard light | `min(255, at + ab)` (**additive**) |
| all other modes | `at + round(ab·(255 − at)/255)` (8-bit) |

**Colour, Normal:** base weight is the 8-bit integer `wb = ao − at`, and
`out = (s·at + b·wb + ao/2) // ao` (rounded). This is 99.62 % exact, max error 1. For comparison,
float W3C source-over gives 84.9 %, max 45, because the integer base weight matters at low
alpha: a base pixel with alpha 1 under top alpha 4 contributes nothing.

**Colour, other modes: open.** Neither the W3C rule (mix the source toward B(b,s) by backdrop
alpha, then source-over) nor variants are close: e.g. Darken 49–55 % exact, max error ~55;
Linear burn 14 %. Results are pulled much more strongly toward B(b,s) than W3C predicts.
Variants tried without success: no source mixing, backdrop flattened over black/white first, and
premultiplied blending.

## Expected result (for Brushye)
Alpha paths as above (exact tests). Normal colour as above (tolerance ±1 until the last 0.4 %
is understood). Other modes over a semi-transparent backdrop: tracked as an open gap (D-017
tolerance policy).

## Implementation note (M3, 2026-11-13)
Rendering these documents through `render::flatten` showed that the **bottom layer on a canvas
without paper is copied, not composited**: L-005's alpha rules match CSP when the backdrop alpha
is the base layer's stored alpha. Compositing the base layer onto the empty canvas with the Normal
rule would already lose 1 on some pixels (e.g. alpha 100 → 99). Inferred from this data at 100 %
opacity, Normal mode; the bottom layer at lower opacity or in other modes is an open question.

## Open questions
- The colour rule for non-Normal modes over a semi-transparent backdrop (most important gap).
- Why the five "additive alpha" modes add alpha (and whether their colour rule differs too).
