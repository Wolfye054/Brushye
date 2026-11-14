# B-002 Stroke gap (spacing): Narrow / Normal / Wide / Fixed
- Status: **implemented (B-spline path, density by gap)**
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Date / method / observer: 2026-11-13 / automated-Wine + virtual stylus / Claude

## Purpose
Dab interval as a function of brush size and gap setting. Whether Fixed is in px, and
whether the first dab is placed at the press point.

## UI (1.13.2, as found)
Sub Tool Detail > Stroke > **Gap** has four preset buttons. Their tooltips, left to right,
are **Fixed, Wide, Normal, Narrow**, followed by a dropdown arrow. Under Wine, opening the
dropdown closed the palette, so it was not used. The value field is enabled only for Fixed,
which defaults to 10.0.

Other settings: Correct velocity input ON, Continuous spraying off, Blend brush tips with
Darken off. Brush tip > "Adjust brush density by gap" off.

## Setup (as run; deviations marked ▲)
- B-001 setup, AA None, hardness 5, density 30, opacity 100.
- ▲ Canvas 600×400 instead of 512×128.
- Strokes come from `tools/observe/b002_strokes.py`: straight horizontal lines at constant
  pressure, evenly spaced motion events. The pixel centre of canvas pixel i is i + 0.5.
  The canvas was undone to clean, and verified by screenshot diff, between exports.
- `B002_<preset>.png` (preset = narrow, normal, wide, fixed10), stroke 40.5 → 472.5:
  - size 40 at y = 40.5: 200 events × 5 ms;
  - size 40 at y = 120.5: 200 events × 20 ms;
  - size 40 at y = 200.5: 50 events × 20 ms;
  - size 10 at y = 270.5: 200 events × 5 ms;
  - size 10 at y = 320.5: 50 events × 20 ms.
- `B002_sweep_<preset>.png` (narrow, normal, wide, fixed37), stroke 60.5 → 472.5, 200 events
  × 5 ms: sizes 2, 4, 7, 16, 25, 63, 100 at y = 15.5, 35.5, 60.5, 95.5, 145.5, 230.5, 340.5.
  The 412 px length is not a multiple of most spacings, which tests the stroke end.

## Results
`b002_analyze.py` was run on each centre row; the output is in the run notes (`obs-out/`).
1. **Spacing s = max(1 px, k · size)**:
   - Narrow k = 0.04, Normal k = 0.08, Wide k = 0.12.
   - **Fixed v: k = v/100**. Fixed is a percentage of size, not pixels. Fixed 10 at size 40
     gives 4 px; Fixed 37 at size 4 gives 1.48.
   - Measured on sizes 2–100. A free scan of s brackets the model value in every row, for
     example Wide size 40: [4.8000, 4.8000].
2. **No dependence on speed or event density.** The 5 ms, 20 ms and 50-event rows put
   every dab at the same x. Their centre rows are identical.
   - Off-axis edge pixels exactly on a disc edge (|d| = R) differ between rows in mirrored
     pairs (above/below). That fits tiny per-row y differences in the input chain, not speed.
3. **The first dab is exactly at the press point**, and dab k is at press + k·s along the
   path.
   - Centres behave as if computed directly, without accumulated rounding: ties at exact
     pixel boundaries (s = 3.2, 4.8) resolve as exact arithmetic. Accumulating in float32 or
     float64 instead gives 16–81 wrong pixels per row.
4. **The stroke ends at the last lattice point at or before the release point. No dab is
   placed at the release point itself.** Adding one makes every non-multiple row worse:
   17–101 wrong pixels, against 0–3.
5. Dabs within one stroke accumulate: alpha(n) = ⌊65535·(1 − 0.7ⁿ)⌋ >> 8, i.e. 76, 130, 168,
   194, 212, 225, 234, 241, 245, 248, 250, 252, 253, 254, 254, 255 (see B-003).
6. **Exactness** (centre rows, 48 rows, 28,800 px), with the spacing model and the
   4×4-subsample coverage rule:
   - 43/48 rows exact; 10 px wrong (99.97 %).
   - The wrong pixels are all one dab count off at a disc edge (max alpha error 38). They
     are in size 63 (all presets) and Fixed 37 size 7.
   - With the plain closed disc (d ≤ R), sizes 4/7/16 at some presets have 11–52 wrong
     pixels, but the spacing found is the same.

## Follow-up 2026-11-13 (run 2): paths, pressure-driven size, stroke options
Method: automated-Wine + virtual stylus, observer Claude, sub tool "Obs pen", AA None,
hardness 5, ▲ 800×800 transparent canvas. Inputs from `tools/observe/brush_run2.py`.

### How the input events become a path
Exports `data/B002_arc_200.png`, `B002_arc_12.png` (semicircle, centre (300.5, 330.5), r 150,
200 or 12 segments), `B002_poly_1.png`, `B002_poly_0.png` (polyline (60.5,60.5) → (300.5,60.5) →
(300.5,300.5) → (540.5,300.5) → (420.5,120.5), with an event every ~2 px or at the corners only).
Brush size 6, density 100, gap Fixed 300 (18 px), so every dab is isolated; dab centre =
centroid of its connected pixels (`dab_centres.py`). Analysis `tools/observe/b002_smoothing.py data`.

**CSP treats the input events as control points of a quadratic B-spline**: the path runs
straight from the first event to the midpoint of the first segment, then for every interior
event P_i follows the quadratic Bézier (m_i, P_i, m_{i+1}) between the neighbouring segment
midpoints, then straight from the last midpoint to the last event. Dabs sit every s px of arc
length along that path from the press point (B-002 rule).

| export | events | polyline: dabs, max dist | **B-spline: dabs, max dist** |
|---|---|---|---|
| arc_200 | 201 | 27/27, 0.25 px | **27/27, 0.25 px** |
| arc_12 | 13 | 27 vs 26 observed, 15.5 px | **26/26, 0.25 px** |
| poly_1 | 2 px apart | 53 vs 52, 16.8 px | **52/52, 0.25 px** |
| poly_0 | corners only | 53 vs 43, 50.2 px | **43/43, 0.26 px** |

0.25 px is the centroid precision of a 6 px AA None dab. With dense events the B-spline
differs from the polyline only by rounding each event's corner, which shortens the path a
little (poly_1: 0.5 px after the first corner); with sparse events it cuts corners heavily
(poly_0: the first 90° corner is replaced by a curve from (180.5, 60.5) to (300.5, 180.5)).
This explains run 1's polyline P (B-003): its corners and end fell short of the polyline.
Arc length vs chord between dabs cannot be told apart at this curvature (r = 150, s = 18: the
two differ by 0.28 px over the whole arc).

### Spacing with pressure-driven size
`data/B002_ramp.png`: size 20, size dynamics pen pressure (minimum 0, linear), gap Fixed 300,
density 100, straight stroke (40.5, 200.5) → (560.5, 200.5) with pressure 0.2 → 1.0 (vpen,
`RAMP`). `b002_options.py` (`ramp_check`): **the gap after a dab follows that dab's own size**
(x_{k+1} = x_k + 3·20·P(x_k)): 19/19 dabs, max 0.77 px, mean 0.33 px (pressure precision).
Using the next dab's size instead is wrong (17 vs 19 dabs).

### Stroke options (size 40, density 30, gap Normal unless noted; rows at 5 ms, 20 ms and 52 events)
Exports `data/B002_rows_*.png`, analysis `tools/observe/b002_options.py data`.
- `rows_base`: the B-002/B-003 model with the AA None scanline rule of B-001: 64518/65020 px exact.
- **"Correct velocity input" OFF** (`rows_novelocity`): byte-identical to the base. No effect on
  evenly timed straight strokes (under Wine; it may matter with real, uneven input).
- **"Adjust brush density by gap" ON** lowers the per-dab density in proportion to the gap.
  Each export gives two constraints on the per-dab density d′ (with CSP's truncation, an 8-bit
  level v means alpha in [256v/65535, 256(v+1)/65535)): the plateau, covered by n dabs, is
  1 − (1 − d′)ⁿ, and the single dab near the press point is d′:

  | export | gap k | density D | plateau (n dabs) | single dab | d′ interval | 3·D·k |
  |---|---|---|---|---|---|---|
  | rows_adjustgap_narrow | 0.04 | 30 | 157 (26) | 9 | [0.03588, 0.03626) | 0.036 |
  | rows_adjustgap | 0.08 | 30 | 159 (13) | 18 | [0.07193, 0.07267) | 0.072 |
  | rows_adjustgap_wide | 0.12 | 30 | 164 (9) | 27 | [0.10749, 0.10857) | 0.108 |
  | rows_adjustgap_d60 | 0.08 | 60 | 222 (13) | 36 | [0.14384, 0.14453) | 0.144 |

  All four fit **d′ = 3·D·k = D·s / (size/3)**: density is normalised to a reference gap of a
  third of the size. Four exports only, and gaps above 33 % of size (is d′ capped at D?)
  are untested.

## Open questions
- (Answered in run 2, see B-001: AA None is a scanline fill with +0.1 px in x.)
- (Answered in run 2: input events are B-spline control points; dabs follow its arc length.)
  Arc length vs chord between dabs on tight curves.
- (Answered in run 2: the gap follows the size of the dab just placed.)
- "Adjust brush density by gap": confirm d′ = 3·D·k at more densities and at gaps above
  33 % of size. "Correct velocity input" with uneven event timing (native Windows).
- The dropdown next to the presets could not be opened under Wine. Its contents and the
  Fixed range are unknown (10 to 300 used successfully).

## Implementation (2026-11-13, M4)
`brushye_core::brush::place_dabs`: t_k = k·gap·size/100 computed directly (constant size), dabs
up to and including t = path length, none added at the release point. With pressure-driven
size each gap follows the current dab's size (open question above).

Test `b002_spacing_exact_away_from_dab_edges` replays all 8 exports: **every pixel away from
a dab edge (> 0.15 px) is exact**. Edge pixels differ only through the open AA None edge rule.

Finding while implementing (B-003 data): on the 5-point polyline P, CSP leaves pixels empty
at the corners and at the end that a dab placed exactly on the input polyline would cover. CSP
seems to smooth or shorten the path between sparse input points. Brushye follows the straight
polyline for now; this needs a spec of its own (sparse events, sharp corners, sub-pixel ends).

## Implementation, M4 part 2 (2026-11-13)
`brush::path` implements the B-spline path; `density_by_gap` the d′ = D·3s/size rule (capped at
full density above a 33 % gap, unobserved). Tests:
- run-1 rows and sweeps, run-2 rows (base, velocity off, 4 × density by gap): every pixel exact
  away from dab edges;
- arcs and polylines: 1–15 edge px wrong per export;
- run-1 polyline P (B-003): exact away from dab edges once its events are taken as 2 px apart
  (corners only does not fit), so it was a dense polyline and the B-spline explains its
  corners;
- pressure ramp: 19 of 19 dabs within 0.9 px.

Correction to the run-2 ramp setup: `brush_run2.py ramp` sends the pressure ramp as raw vpen
values, so CSP received P = (p − 0.0099)/0.9901 (B-004 input definition), i.e. 0.192 → 1.0, not
0.2 → 1.0. With that, the "gap follows the dab's own size" rule matches; the ~0.4 px per-gap
shortfall in `ramp_check` was this offset.
