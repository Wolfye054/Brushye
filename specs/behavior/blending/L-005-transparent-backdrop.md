# L-005 Blending over a transparent / semi-transparent backdrop
- Status: draft
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK

## Steps
Base layer from a PNG with per-pixel alpha (generated: gray x, alpha = y). Top = top_alpha.
For all 28 modes: export with "Export transparency" ON (check the option name in the 1.13.2 dialog)
so the output alpha can be read.

## What to measure
Output colour and alpha vs (b, ab, s, as). The W3C general formula is the starting hypothesis.
Watch for the manual's warnings about transparency in Add (Glow) / Color dodge.
