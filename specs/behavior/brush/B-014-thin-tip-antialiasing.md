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

## Run 2 (2026-11-14): sub-pixel sweep
- **Setup:** new custom sub tool "B014b": size 41, thickness 30, Vertical, hardness 5,
  density 100. `brush_run2.py thinsweep`: 10×10 dabs, press (70.5 + 70i + i/10,
  70.5 + 70j + j/10), on an 800×800 canvas (origin (543, 163)).
- **Exports:** `data/B014_sweep_a{0,30}_aa{1,2,3}.png` and the AA None control
  `B014_sweep_a30_aa0.png`. All 100 dabs are present in every file.
- **Ink per dab is stable:** standard deviation 0.7–2.2 px² over the 100 sub-pixel positions.
- **Centroid minus press point,** by the press point's fractional position:
  - AA None: x −0.50 (± 0.05), as in B-010; y 0 ± 0.07.
  - AA on, angle 0: x ≈ 0 (−0.07 … +0.01). y is **negative and grows with AA**: about −0.15
    (Weak), −0.18 (Middle), −0.22 (Strong). It follows a **sawtooth over the fractional y**:
    it decreases from frac 0.0 to 0.3, then jumps up at 0.4–0.5 (Middle: −0.18, −0.20, −0.21,
    −0.23, then −0.17, −0.15 …).
  - AA Middle, angle 30: x −0.13, y −0.11 … −0.15, with the jump at frac 0.5.
- **Reading:** a jump at frac 0.5 is a pixel-row snap. CSP seems to rasterise anti-aliased
  thin tips row by row, with rounding, not as a smooth distance ramp. That fits the earlier
  asymmetric row-200 edges, and explains why no smooth model is exact. The round tip's AA None
  scanline rule (B-001) is the closest known analogue.

## Analysis (2026-11-14): the tip is a 12-gon
- **Edge shape:**
  - In `B014_sweep_a0_aa2.png`, dab at (70.5, 70.5), each column's summed alpha gives the edge
    height (a linear ramp's sum is the distance to its edge).
  - The top edge changes height in **straight runs with kinks** near x ≈ 60, 69, 81 and 86,
    not along an ellipse curve.
  - The rows above and below the solid core are **triangles across x**, peaking near x = 69:
    a shallow polygon edge seen through a vertical ramp.
- **Polygon fit** (`polyfit.py` approach: N-gon inscribed in the ellipse, corners at
  θ0 + k·360°/N, linear ramp on the distance to the polygon):
  - N = 12 with θ0 = 0 fits best of 8–64 sides.
  - Its corners fall at x = 70 ± 10.25 and 70 ± 17.75, where the kinks are.
  - On the run-1 dabs the free parameters come out natural: ramp **1.5 / 2.5 / 3.5 px** (the
    round tip's AA widths, B-001), no edge inset, centre y −0.125.
  - Total error is 142 / 152 / 174 against 401 / 577 / 742 for Brushye's current model; worst
    pixel ≈ 80–120.
  - **What's left is row-structured:** the long-axis tips (up to −81) and the fringe rows
    (±50). CSP's AA on the polygon is probably done per row, not by distance.
- **Not adopted.** In the engine the 12-gon (like the earlier ellipse ramp) makes the
  cross-check strokes worse: X-05 goes to 31 % of pixels off and −6 % ink.
  - Every model that matches CSP's single thin dab leaves its strokes too light. The current
    interim model is ~40 % too heavy per dab, which happens to give the right stroke ink.
  - A spacing factor alone doesn't close it (ink matches only near f ≈ 0.4; 22 % of pixels
    still off).
  - So thin dabs in a stroke differ from a lone dab in some way not yet seen.

## Open
- **Run 3:** thin-tip strokes with isolated dabs (gap Fixed 300) and with Normal gap, angle 0 and
  30, AA None and Middle, plus Direction of line. That shows each dab inside a stroke next to
  a lone dab.
- Then: the per-row AA rule on the 12-gon, from the sweep data.
- (Earlier note) a dense sub-pixel sweep of thin dabs at each AA level, as B-001 did for the round tip, to
  find the exact rule.
