# B-003 Stroke accumulation: Opacity cap vs Brush density
- Status: **observed**
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Date / method / observer: 2026-11-13 / automated-Wine + virtual stylus / Claude

## Purpose
CSP describes Ink > Opacity as a per-stroke ceiling and Brush density as a per-dab value that
builds up. Verify this and measure the exact accumulation formula.

## Setup (as run; deviations marked ▲)
- B-001 setup, size 40, hardness 5, ink Normal, on a transparent layer.
- ▲ AA None instead of Middle, so that edge dab counts are exact.
- Opacity is in Tool property and density in Sub Tool Detail > Brush tip; both were read
  back from the UI after setting.
- Three strokes per export (`tools/observe/b002_strokes.py`, 5 ms per event):
  - H: (40.5,100.5) → (280.5,100.5);
  - V: (160.5,30.5) → (160.5,370.5), a separate press crossing H;
  - P: one press along the polyline (330.5,200.5) → (570.5,200.5) → (570.5,350.5) →
    (450.5,350.5) → (450.5,40.5), crossing itself at (450.5,200.5).
- ▲ The self-crossing polyline replaces the figure-8.
- Exports:
  - gap Narrow: `B003_narrow_op50_den100.png`, `B003_narrow_op100_den50.png`,
    `B003_narrow_op50_den50.png`;
  - ▲ added gap Fixed 90 (sparse dabs, 36 px apart, so 2-dab overlaps are isolated):
    `B003_fixed90_op50_den50.png`.

## Results
For one stroke with opacity O and density D, at a pixel covered by n of that stroke's dabs:

    stroke buffer  B = 1 − (1 − D)^n      (dabs of one stroke accumulate, also where it crosses itself)
    stroke alpha   a = O · B              (opacity SCALES the buffer; it is not a clamp/min)
    layer          A ← A + a·(1 − A)      (each stroke composited "over" when it ends)
    8-bit value    ⌊65535·A⌋ >> 8         (truncation: one dab at 0.3 → 76, at 0.5 → 127)

- Evidence on the H stroke's leading staircase (n = 1, 2, …):
  - O 50 / D 50: 63, 95, 111, 119, 123, 125, 126, 127. That is O·(1 − 0.5ⁿ); a clamp would
    give 127 at n = 1.
  - O 100 / D 50: 127, 191, 223, 239, 247, 251, 253, 254, 255.
- O 50 / D 100: a stroke is 127, H crossing V is 191, and P crossing itself stays 127.
- Sparse run (O 50 / D 50):
  - one dab = 63;
  - two dabs of the same stroke = 95, both for neighbouring dabs and for dabs from different
    legs of P;
  - three dabs = 111 and four = 119, within P;
  - one dab each from H and V = 111;
  - an H two-dab lens under one V dab = 135.
  - In region P every level falls in the single-stroke set {O·(1 − (1−D)ⁿ)}.
- **Exactness** (`b003_analyze.py`, region x < 300, all 4 exports, per pixel):
  - **100.000 % exact (449,550 px, max error 0)** on pixels with no dab edge within 0.15 px.
  - Pixels within 0.15 px of a dab edge are 88–92 % exact. They depend on the open AA None
    edge rule (B-001/B-002), not on accumulation.

## Open questions
- AA Weak/Middle/Strong edges and soft tips: presumably c·D per dab with the same buffer.
  Not yet verified.
- Whether the truncation to 8 bits happens on export only or in the layer itself. It
  matters for many-stroke builds; test by laying many O = 10 % strokes over one another.
- Other ink modes (B-008) and layer opacity/blend interaction are not covered.
