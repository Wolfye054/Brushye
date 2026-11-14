# M4 report: brush engine v1, headless (2026-11-13)

Supersedes the part 1 report (`M4-brush-engine-part1.md`), which stays as a record.

## What works
Everything in M4's scope, built against two automated observation runs (specs B-001 … B-010):
- **Path:** input events are the control points of a quadratic B-spline (B-002), optionally
  stabilized first (moving mean over S events, catching up at release; B-007).
- **Spacing:** gap presets and Fixed %; the gap after a dab follows that dab's size; "Adjust
  brush density by gap" (B-002).
- **Tip:** round tip with AA None (scanline rule), Weak, Middle and Strong; hardness 1–5
  (interim profile table scaled by size, soft dabs offset +0.5 px); thickness, direction and
  angle; angle following the line (B-001, B-009, B-010).
- **Build-up and commit:** dabs of one stroke build up as 1 − ∏(1 − c·D); opacity scales the
  result; the stroke lands on the 8-bit layer with its unquantised alpha (B-003).
- **Ink modes:** all 35, i.e. the 28 layer modes plus Erase, Background, Replace alpha, Compare
  density, Erase (compare), Black burn and White burn (B-008).
- **Pressure** for size, opacity and density, each with a minimum and a multi-point curve
  (quadratic B-spline, confirmed for 1–3 points; B-004/B-005).
- **`brush.stroke`** exposes all of it as optional keys. It's one undo step per stroke and
  replays exactly. The cross-platform golden hash now includes an angled soft tip, so `libm`
  sin/cos is checked on all three CI platforms.

## Reference-image results (`crates/brushye-core/tests/observed_brush.rs`, 16 tests)
| Area | Result |
|---|---|
| Single dabs, AA Weak/Middle | every pixel exact; Strong 7 px off by 1 |
| AA None edges | scanline rule: 4 px of 1858 wrong (run 1), 9–20 per 100-dab sweep, all exact ties |
| Spacing, 14 row/sweep exports incl. density by gap | every pixel exact away from dab edges |
| Curves and polylines (B-spline) | 1–15 edge px per export; run-1 polyline exact away from edges |
| Pressure ramp | 19/19 dabs within 0.9 px |
| Build-up and commit (5 exports, 25 stacked strokes) | every pixel exact away from dab edges |
| Pressure curves (linear, 1–3 points) | outermost AA pixel only, 9–19 levels (pressure precision) |
| Ink modes, opacity 100 | 28 of 35 exact; the rest ±1–3 (Background, colour modes) |
| Ink modes, opacity 50 | keep-alpha and brush-only exact; paint modes ±1; Multiply ±2 |
| Soft tips, sizes 11/41/101 | within 1–4 levels (h1, h2); 2–15 (h3); h4 edge pixels flip |
| Thin tips | 4 px of 3694 wrong (edge ties) |
| Direction of line | 5 of 8 strokes within 4 px; 3 pinned (CSP's late first dab, open) |
| Stabilization 0 / 6 / 20 | centre line rms 0.03 / 0.06 / 0.16 px |

Every tolerance is pinned, so changes in either direction show up.

## Findings while implementing
- The run-1 polyline (B-003 P) was sent with events every 2 px. With the B-spline it now matches
  everywhere away from dab edges, corners included.
- The run-2 pressure ramp went out as raw virtual-pen values. With the input-chain correction
  (B-004), the "gap follows the dab's own size" rule fits to 0.9 px. Noted in B-002.
- Stabilization fits a window of S events (not S + 1) once the B-spline step is included.

## Not done / open
- Stabilization ≥ 50 (rms 2–6 px), "Adjust by speed", post correction (B-007).
- Colour mixing (B-011): basics observed, not implemented. It is outside M4's scope list.
- Tilt dynamics (B-010): tilt reaches CSP under Wine, but its mapping looks Wine-specific.
  Waiting on a native Windows check.
- The late first dab with "direction of line" in 3 of 8 directions (B-010).
- Exact soft-tip formula (table model now); AA None ties; paint-mode colour at partial alpha;
  Multiply at partial alpha; the colour modes (L-007).
- Brush-mode behaviour inside one stroke is assumed: the buffer builds first, then the mode
  applies once.

## Design notes
- `brush.stroke` stays at version 1 although its output changed (B-spline path, new commit
  rule). Brushye is pre-alpha with no journals in use. From the alpha on, a change in rendering
  gets a new command version.
- ~~Pressure in samples is recorded after the global pressure curve~~. Changed in M6 at the
  maintainer's direction (behave like CSP): samples are raw, and the global curve is the
  journaled `input.pressure_curve` setting applied in the engine (D-015 5, B-006).

## Proposed next step
M5: undo/redo hardening and the native `.brsh` format (D-014), with brush settings saved as
user brushes. That gives `brush.stroke` its brush-preset counterpart.
