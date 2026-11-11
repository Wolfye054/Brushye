# B-008 Ink blending (combine) modes of the brush itself
- Status: draft
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK

## Purpose
Brush-level modes: Normal, the layer modes, and the tool-only ones (Compare density, Background,
Replace alpha, Erase, Erase (Compare), Black burn, White burn). First read the exact list and
order from the 1.13.2 UI.

## Setup
Canvas 256×256, one raster layer pre-filled by opening a generated test PNG (same technique as
L-001: gradients plus per-pixel alpha). Brush: B-001 tip, size 300 (covers the canvas in one
click), AA None, density 100, main colour #4080c0, opacity 100 and 50.

## Steps
For each mode × opacity: one click at the canvas centre, export, undo.

## What to measure
Result vs (existing pixel, brush colour, brush alpha). Compare against layer blending results
(L-001/L-002) to see whether brush modes share the same math.
