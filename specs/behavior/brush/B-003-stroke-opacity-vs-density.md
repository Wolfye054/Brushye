# B-003 Stroke accumulation: Opacity cap vs Brush density
- Status: **implemented**
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

## Follow-up 2026-11-13 (run 2): how a stroke is committed to an 8-bit layer
Method: automated-Wine + virtual stylus, observer Claude. Size 40, AA None, hardness 5,
density 100, **opacity 10**, gap Normal, adjust density by gap off, ▲ 800×800 transparent
canvas. `tools/observe/brush_run2.py layers`: 25 separate horizontal strokes on y = 200.5;
stroke k runs from x = 40.5 + 20k to 560.5, so the column x = 10 + 20n lies under exactly n strokes.
Export `data/B003_layers_op10.png`, analysis `tools/observe/b003_layers_analyze.py
data/B003_layers_op10.png`.

Observed levels for n = 1 … 25: 25, 48, 69, 88, 105, 120, 134, 146, 157, 167, 176, 184, 191, 197,
203, 208, 213, 217, 221, 224, 227, 230, 233, 235, 237.

| model | matches (of 25) |
|---|---|
| A' = A + a8 − ⌊A·a8/255⌋ (8-bit, run-1 implementation) | 13 (fails from n = 14) |
| float layer, ⌊65535·(1 − 0.9ⁿ)⌋ >> 8 | 16 |
| ⌊65535·(1 − (1 − A/255)(1 − a))⌋ >> 8 per stroke | 4 |
| **A' = A + ⌊(255 − A)·a + 0.5⌋ for A > 0; an empty pixel gets ⌊65535·a⌋ >> 8** | **25** |

So **the layer stores 8 bits, and each stroke is composited with its unquantised alpha a = O·B**:
the increment (255 − A)·a is rounded half up, except on a fully transparent pixel, where the
stroke alpha is truncated as in run 1 (25, not 26, for a = 0.1). The same rule gives the run-1
stroke-over-stroke levels (63 → 111, 95 → 135, 127 → 191) and B-008's opacity-50 alpha over
every backdrop level 1 … 255.

## Open questions
- AA Weak/Middle/Strong edges and soft tips: presumably c·D per dab with the same buffer.
  Not yet verified.
- (Answered in run 2: the layer is 8-bit; strokes composite with their unquantised alpha,
  increment rounded half up; see the follow-up.)
- Other ink modes (B-008) and layer opacity/blend interaction are not covered.

## Implementation (2026-11-13, M4)
`brushye_core::brush::StrokeBuffer`: per pixel T = ∏(1 − c·D), stroke alpha a = O·(1 − T),
committed as a8 = ⌊65535·a⌋ >> 8. Where a stroke lands on existing ink the commit is 8-bit:

    A' = A + a8 − ⌊A·a8 / 255⌋

That reproduces every observed stroke-over-stroke level (63 over 63 → 111, 95 under 63 → 135,
127 over 127 → 191). The float "over" used in the analysis gives the same values here; the
8-bit form is what the 8-bit layer model can store, and it is exact on all H+V pixels. Note
it differs from the layer compositing path (L-005), which gives 110 for 63 over 63.

Test `b003_accumulation_exact_away_from_dab_edges`: H and V exact away from dab edges in all
four exports; P pinned (see B-002 implementation note on polylines).

Not observed, chosen for now: stroke colour where the layer already has a different colour
(alpha-weighted, like Normal), and per-dab varying opacity (each pixel keeps the highest dab
opacity, so a constant opacity behaves exactly as observed).
