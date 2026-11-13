# B-002 Stroke gap (spacing): Narrow / Normal / Wide / Fixed
- Status: **implemented (straight strokes; curves open)**
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

## Open questions
- **AA None edge rule.** A 4×4 sub-sample rule (≥ 8 of 16 inside) fits these horizontal
  centre rows, but it over-covers the vertical caps in B-003, so it is not the true rule.
  See B-001.
- Spacing along curves: arc length vs chord between events. Only straight strokes were
  measured here.
- Whether s follows the per-dab size when size has pressure dynamics.
- "Adjust brush density by gap" ON, and "Correct velocity input" OFF, were not tested.
- The dropdown next to the presets could not be opened under Wine. Its contents and the
  Fixed range are unknown (10 to 90 used successfully).

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
