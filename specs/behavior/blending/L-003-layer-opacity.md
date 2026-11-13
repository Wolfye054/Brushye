# L-003 Layer opacity (× per-pixel alpha)
- Status: **observed** (Normal, Screen, Add, Add (Glow) exact; Multiply open)
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK
- Date / method / observer: 2026-11-13 / automated-Wine (Xvfb :77) / Claude (background run)

## Setup (exact)
1. Inputs from `tools/observe/make_blend_inputs2.py`: `combo_base.png` (256×1280 RGB) and
   `combo_top.png` (256×1280 RGBA). Rows 0–511: top fully opaque (L-007 designs, rich colour
   pairs). Rows 512–1279: three blocks, base colour fixed per block (10,64,128), (191,230,255),
   (37,100,170); top = gray x; top alpha = row mod 256.
2. File > Open `combo_top.png`, Select All, Copy, Deselect; File > Open `combo_base.png`; Paste
   → layer "combo_top Copy" over "combo_base".
3. For each mode in {Normal, Multiply, Add, Add (Glow), Screen} and opacity in {100, 99, 50, 37, 1}:
   set the mode in the Layer palette dropdown, type the opacity into the Layer palette opacity
   field + Enter, then export PNG (`tools/observe/blend_export_runs.sh`).

## Observations (2026-11-13)
Data: `data/observed2/combo_<mode>_op<NN>.png`. Stored crop: rows 0–191 (opaque sweeps) followed by rows 512–1279 (alpha sweep), 960 rows. Inputs: `data/inputs2/`. Checker: `tools/observe/check_opacity_model.py`.

**Model (exact for Normal, Screen, Add, Add (Glow) at every opacity, all pixels, opaque and partial alpha):**
```
o8  = floor(opacity% * 255 / 100)        # 99→252, 50→127, 37→94, 1→2
ae8 = (a * (o8 + 1)) >> 8                # per-pixel alpha a (8-bit) × opacity
a15 = (ae8 * 32768 + 127) // 255         # to 15 bit (L-002)
out = (b*(32768 - a15) + B(b,s)*a15 + 16384) >> 15
```
Glow modes use `a15` to premultiply the source instead (L-002). At 100 %, `o8 = 255` gives
`ae8 = a`, so this reduces to L-002 exactly.

- The obvious `opacity * 32768 / 100` is wrong: for each opacity only a narrow range of 15-bit
  values reproduces the opaque rows, and only `floor(op·255/100)` → 15-bit lands inside all
  of them (e.g. 1 % → 257, not 328).
- `ae8 = (a·(o8+1)) >> 8` was the only one of 11 integer ÷255 variants that is exact for all
  four opacities.

**Multiply is the exception:** at opacity < 100 % even fully opaque pixels deviate (max error 2;
51–72 % exact), and with per-pixel alpha (any opacity) 49–55 % exact. A search over every fixed
effective alpha, with lerp, "lerp source toward white then multiply", and different rounding
variants, found no single alpha that fits (best 74 % at 50 % opacity), so Multiply with
partial coverage is not a plain lerp. See L-002 open question 1.

## Open questions
- Multiply with partial alpha/opacity: exact rule (data above is the ground truth).
