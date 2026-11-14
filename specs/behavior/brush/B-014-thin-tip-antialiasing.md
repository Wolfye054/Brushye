# B-014 Anti-aliased thin tips
- Status: **observed** (rule not found yet; the engine keeps its interim model)
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Date / method / observer: 2026-11-14 / automated-Wine + virtual stylus (D-003) / Claude
- Registry: `~/brushye-samples/b013_b014/registry.md`.

## Setup
- **Sub tool:** new custom sub tool "B014": size 41, size pressure off, stabilization 0,
  density 100, hardness 5, Direction Vertical, black.
- **Input:** `brush_run2.py tip`: single dabs at (200.5, 200.5) and (500.3, 200.7).
- **Exports:** `data/B014_t{30,60}_a{0,30}_aa{1,2,3}.png` (aa1 Weak, aa2 Middle, aa3 Strong),
  plus the AA None control `B014_t30_a30_aa0.png`.

## Findings
1. **Along the long axis,** the edge is a linear ramp with steps of 102 / 73 / 57 alpha levels
   per pixel (Weak / Middle / Strong). That is a ramp width of 2.5 / 3.5 / 4.5 px: the round
   tip's widths (B-001) plus 1. Example: `B014_t30_a0_aa2.png` row 200 at the left edge is
   36, 109, 255, and at the right edge 182, 109, 36. The two edges aren't mirror images.
2. **Across the short axis** the ramp is steeper (e.g. 51, 154, 239, 255 for Middle).
3. **Models tried, scored as total |error| over all 24 dabs** (alpha units / 255; worst pixel
   in levels):

   | Model | Weak | Middle | Strong |
   |---|---|---|---|
   | Brushye's current (round ramp in the stretched frame) | 401 (231) | 577 (228) | 742 (223) |
   | Linear ramp on the first-order distance to the ellipse; widths 1.5 / 2.75 / 3.75, inset 0.25, centred on the input point | 104 (107) | 125 (84) | 149 (93) |
   | Box-filtered closed ellipse | 714 (226) | 1112 (239) | 1490 (189) |

4. **The distance-ramp model isn't kept.** It is about 4× closer for single dabs, but it made
   the cross-check strokes worse (X-05: 25 % of pixels off and −6 % ink, against 18.5 % and
   +1.2 %). CSP's stroke edges come out darker (167 against 20 at the top edge of X-05's
   stroke), so the model is missing something structural. The engine keeps the old interim
   model until the rule is found.

## Open
- A dense sub-pixel sweep of thin dabs at each AA level, as B-001 did for the round tip, to
  find the exact rule.
