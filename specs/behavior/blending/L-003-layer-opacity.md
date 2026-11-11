# L-003 Layer opacity (× per-pixel alpha)
- Status: draft
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK

## Steps
L-002 setup (top_alpha over base). For Normal, Multiply, Add, Add (Glow), Screen: set layer
opacity to 1, 37, 50, 99 % (type into the Layer palette opacity field) and export each.

## What to measure
Effective alpha = f(opacity %, a). Hypothesis: opacity is quantized to 15 bits and multiplied
with a15 (L-002). Check the rounding of each step.
