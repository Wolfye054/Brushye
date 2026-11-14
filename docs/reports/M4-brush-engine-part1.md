# M4 report, part 1: round-tip brush engine (2026-11-13)

> Superseded by `M4-brush-engine.md`. Part 2 replaced the polyline path, the AA None disc and
> the 8-bit commit formula described below.

M4 is not finished. This part covers the engine core that the observed specs pin down.
Stabilization, ink combine modes, tip thickness/angle, tilt and colour mixing still need their
specs observed first (B-007, B-008, B-010, B-011 are drafts).

## What works
- **Observations** (specs B-001, B-002, B-003, B-009, automated-Wine with the virtual stylus;
  commit c74a28b). CSP truncates to 8 bits rather than rounding.
- **Brush engine** (`brushye-core::brush`):
  - round tip: AA None / Weak / Middle / Strong (B-001); hardness 1–5 (B-009, interim table);
  - dab placement along the input polyline: gap presets Narrow / Normal / Wide and Fixed %
    (B-002); first dab on the press point, none added at the release point;
  - stroke buffer: dabs of one stroke build up as 1 − ∏(1 − c·D), opacity scales the result,
    8-bit truncated commit (B-003);
  - pressure dynamics for size, opacity and density, each with a minimum and a curve (B-004,
    B-005); curve points are Bézier control points;
  - lock transparent pixels honoured; locked layers rejected.
- **`brush.stroke` command** (Undoable, one undo step per stroke): all settings as optional
  keys with defaults, plus the recorded samples. Scripts may write samples as `[x,y,p,t;…]`;
  the journal always stores the compact blob. Replay is exact; a golden hash for brush output
  runs on all three CI platforms.

## Reference-image results (`crates/brushye-core/tests/observed_brush.rs`)
| Spec | Result |
|---|---|
| B-001 AA Weak, Middle | every pixel exact (48 dabs) |
| B-001 AA Strong | 7 px off by 1 (same as the spec's model) |
| B-001 AA None | exact except 24 edge pixels (open edge-tie rule) |
| B-002 spacing, 8 exports | **every pixel exact away from dab edges** |
| B-003 build-up, H+V, 4 exports | **every pixel exact away from dab edges**, incl. crossings |
| B-003 polyline P | pinned: CSP cuts the corners and the end of a 5-point polyline |
| B-004/B-005 pressure grids | within 3 levels (opacity, density); 9–13 on the outermost edge pixel (size) |
| B-009 hardness at size 101 | within 1–2 levels (h1–h3), 16 (h4); other sizes unobserved |

All tolerances are pinned in the tests, so a change in either direction shows up.

## Findings while implementing
- **Stroke-over-stroke is an 8-bit operation** that differs from layer compositing:
  A' = A + a − ⌊A·a/255⌋ matches every observed level (L-005's layer path gives 110 where
  CSP's stroke gives 111).
- **Soft dabs sit half a pixel off the press point** (hard dabs don't). Brushye applies the
  offset; whether it is a fixed offset or grid rounding is open (B-009).
- **CSP doesn't follow sparse polylines exactly**: corners and the stroke end fall short.
  Brushye follows the straight polyline; needs its own spec.

## Design choices to confirm (within D-011, easy to change before anything depends on them)
- `brush.stroke` carries every brush setting inline (defaults omitted), so a stroke replays
  without any tool state. When brush presets exist (M5/M7) a stroke can reference a preset plus
  overrides in a version 2 of the command, via the normal migration path.
- Samples hold pressure **after** the global pen pressure curve (B-006), like device
  calibration. The global curve is applied at input capture (M6).

## Not done / open
- From M4's scope: stabilization (B-007), ink combine modes Compare density / Erase (B-008),
  tip thickness and angle, tilt and direction (B-010), colour mixing (B-011). Each needs its
  spec observed first.
- Open rules: AA None edge ties; exact soft-tip curve and size scaling; spacing on curves and
  with pressure-driven size; multi-point pressure curves; stroke colour over different colours.

## Proposed next step
One more observation run: B-007, B-008, B-010 and B-011, plus the follow-ups above (AA None
sub-pixel sweep, hardness at sizes 11 and 41 with sub-pixel offsets, a curved stroke, sparse
polylines, a size-pressure ramp, a 2- and 3-point curve). Then finish M4 against it.
