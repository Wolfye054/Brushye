# L-002 Layer blending modes with per-pixel source alpha (opaque backdrop)
- Status: **observed** (27/28 modes have an exact model; Multiply ±2 open)
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK
- Date / method / observer: 2026-11-11 / automated-Wine / Claude

## Purpose
How a semi-transparent pixel in a blended layer mixes into an opaque backdrop, per mode.
This includes what makes the "Glow" variants different.

## Setup (exact)
As L-001, except the top layer comes from `top_alpha.png`: same RGB as `top.png`, alpha =
(7x + 13y) mod 256 (every alpha value occurs many times, uncorrelated with RGB). Steps:
open `top_alpha.png`, Select All, Copy; open `base.png` in a new tab; Paste → layer
"top_alpha Copy"; then the L-001 export loop with prefix `alpha`. Layer opacity stays 100 %.

## Observations
Data: `data/observed/alpha_NN_<mode>.png`. Output alpha is 255 everywhere (opaque backdrop).

Let B = the mode's opaque result for (b, s) (L-001), a = source alpha (0–255).

1. **Alpha is used at 15-bit precision.** `a15 = (a·32768 + 127) // 255`, then
   `out = (b·(32768 − a15) + B·a15 + 16384) >> 15`.
   This is **bit-exact on all pixels** for every mode except Multiply and the two Glow modes,
   given the observed opaque B.
   For comparison, the common 8-bit form `(b·(255−a) + B·a + 127)//255` is off by one on
   0.005–0.02 % of pixels, and floor variants on ~50 %.
2. **Add (Glow):** `out = min(255, b + ((s·a15 + 16384) >> 15))`. The source is
   premultiplied by alpha and added, with no lerp. Bit-exact.
3. **Glow dodge:** Color dodge (L-001 integer form) with the premultiplied source
   `s' = (s·a15 + 16384) >> 15` and no lerp. Bit-exact.
   - So "Glow" modes treat transparent source pixels as black (adding nothing), while
     Add / Color dodge fade the *result* toward the backdrop. With a = 255 they're identical.
4. **Multiply:** neither the lerp above nor any of the variants tried is exact: ~58–65 %
   exact, max error 2. Tried: lerp of the rounded product; Cb·lerp(255, s, a); a single combined
   division; 15-bit internal colour.
5. Modes with an open opaque formula (Overlay, Hard light, Vivid light, non-separable) are exact
   under rule 1 once the *observed* opaque B is used. Their error comes only from the opaque formula.

## Expected result (for Brushye)
Reproduce `data/observed/alpha_*` bit-exactly, Multiply aside. The 15-bit alpha path suggests
storing or using alpha at 15-bit internally (DECISIONS D-012 point 1).

## Open questions
1. Multiply with partial alpha: exact formula. Next test: a dedicated sweep with b fixed,
   s ∈ 0..255, a ∈ 0..255 (one 256×256 image per few b values).
2. Transparent or semi-transparent **backdrop** (L-005): output alpha and colour.
3. Layer **opacity** vs per-pixel alpha: are they multiplied at 15 bits? (L-003)
4. Do brush ink modes (B-008) use the same rules?
