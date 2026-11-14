# B-006 Global pen pressure adjustment (File > Pen Pressure Settings) and its curve
- Status: **observed** (wizard behaviour, Stronger/Lighter, composition order)
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine with virtual stylus (D-003). Flag `verify-native`
- Date / method / observer: 2026-11-13 / automated-Wine + virtual stylus / Claude

## The dialog (1.13.2 UI)
File > "Pen Pressure Settings(J)…" opens **"Adjust pen pressure"**. It asks you to draw freely on
the canvas (on a temporary layer "For detecting pen pressure"), then press
"Check adjusted settings". Buttons: Check adjusted settings, Cancel, Reset to default, and the
checkbox "Show pen pressure graph". The second page, **"Check adjusted settings"**, has
**Stronger(H)**, **Lighter(S)**, **Done(F)**, Back to previous screen and Cancel.

## Steps (exact)
1. Show the graph in the observation prefix: default = identity (2 points).
2. Draw 3 strokes on the wizard canvas with the virtual stylus. Each is a half-sine pressure
   profile peaking at **P = 0.5** (120 events, 8 ms apart), at canvas y = 487, 587, 687.
   Press "Check adjusted settings".
3. Capture the graph; press Stronger ×1, ×2, ×3 (graph after each); Lighter ×3 (back);
   Lighter ×1, ×2, ×3 (graph after each); Stronger ×3 (back); Done.
4. Measure the applied curve: the B-004 grid with Obs pen (size 100, linear sub curve, min 0).
5. With the global curve still applied, set the sub tool curve to the 1-point Bézier from B-004
   run 3 and run the grid again.
6. Reset: open dialog → Reset to default → Check adjusted settings → Done. Then reopen to verify.

## Observations (2026-11-13)
Data: `data/B006_global_auto_grid.png`, `data/B006_global_auto_plus_sub1pt.png`. The graph
screenshots are kept out of the repo (CSP UI) in `obs-out/pressure-graphs/` on the
observation machine.

1. **The wizard fits the curve to the pressure you actually used.** After strokes peaking at
   P = 0.5, the proposed curve reaches **100 % output at P ≈ 0.5** and stays flat to 1.0. It has
   6 control points, read from the graph to about ±0.01: (0,0), (0.08,0.02), (0.24,0.30),
   (0.41,0.76), (0.50,1.00), (1,1).
2. **Stronger / Lighter move only the interior points' outputs; their inputs and the endpoints
   stay fixed.** Each Stronger press lowers them by ≈0.05 (0.30 → 0.25 → 0.19 → 0.15;
   0.76 → 0.71 → 0.66 → 0.61). Each Lighter press raises them by ≈0.06, compressed near 1.0
   (0.30 → 0.36 → 0.42 → 0.48; 0.76 → 0.80 → 0.84 → 0.86). Stronger×3 then Lighter×3 returns a
   pixel-identical graph.
3. **The applied global curve shapes stroke size** (sub tool linear): measured rows =
   4, 8, 16, 24, 34, 44, 58, 72, 86, 100 for P = 0.05 … 0.50, and **100 for every P ≥ 0.5**.
4. **Composition order: the global curve is applied first, then the sub tool's curve**
   (size ∝ C_sub(C_global(P))). With both curves active, predicting with the measured global
   curve gives max error 0.0026 for sub(global(P)) and 0.35 for global(sub(P)).
5. **Reset to default only takes effect after confirming** (Check adjusted settings → Done).
   Closing the dialog with its X after pressing Reset to default leaves the old curve active.
6. The wizard's temporary layers disappear after Done; the document's own layers are untouched.

## Expected result (for Brushye)
A global input curve C_global (same curve type as per-setting curves) is applied to raw
pressure before any per-setting dynamics. A wizard can propose C_global from sampled
pressure, with Stronger/Lighter as fixed output nudges of the interior points.

## Open questions
- The wizard's fitting rule: how the control points' inputs are chosen (percentiles of the
  observed pressure histogram?) and how the 100 % point relates to the peak. Next: feed
  strokes peaking at 0.3, 0.7 and 1.0, and strokes with a flat plateau.
- The exact Stronger/Lighter step rule (additive? toward a bound?). Next: more presses, and
  read the points from a larger graph.
- Does the global curve also apply to tilt, or only pressure? (B-010.)

## Implementation (2026-11-13, M6; supersedes the M4 note)
As CSP does and D-015 5 states: strokes record **raw** pressure; the global curve is the
application setting `input.pressure_curve` (journaled, not undoable, kept across `doc.new`,
saved per user by the app), applied in the engine before every setting's curve, sub(global(P)).
The app's File > Pen pressure settings edits the curve directly.

Test `b006_global_curve_applies_before_the_setting_curve` replays both B-006 exports with the
wizard curve read from the graph: stroke widths within 2 rows for P ≤ 0.45, and the other order
fits clearly worse. Above P = 0.5 CSP's strokes are already full size, but a quadratic B-spline
through the graph readings ((0.41, 0.76), (0.50, 1.00), (1, 1)) is not.

Open: the wizard (fitting a curve to strokes the user draws) and its Stronger/Lighter rule are
not implemented, because the fitting rule is unknown (open questions above). Also open: the
curve type of wizard curves (the B-spline that the editor's curves use, B-004, doesn't reach 1.0
at the wizard's 0.5 point).

## Follow-up 2026-11-13 (M7 run): the wizard's fitting rule
Method: automated-Wine + virtual stylus, observer Claude.
- File > Pen Pressure Settings(J)… was opened on a throwaway 600 × 400 canvas, with "Show pen
  pressure graph" on.
- Three wizard strokes were drawn at screen y 600 / 650 / 700, x 660 → 1000, 121 events 8 ms
  apart, using `tools/observe/b006_wizard.py PROFILE PEAK`. vpen p = 0.0099 + 0.9901·P.
- After "Check adjusted settings", the proposed curve's control points were read from the
  graph screenshot with `tools/observe/b006_graph_points.py` (marker clusters; graph 0 % at
  screen (1084, 501), 100 % at (1286, 325) on that page; ±0.005).
- Between runs the dialog was cancelled, which left the curve unchanged, and reopened.
- Graph screenshots are in `obs-out/m7/w*.png`, not committed.

| Strokes | Interior control points (input, output) |
|---|---|
| half-sine, peak 0.3 | (0.053, 0.047) (0.145, 0.314) (0.249, 0.764) (0.299, 1.000) |
| half-sine, peak 0.5 (first run above) | (0.08, 0.02) (0.24, 0.30) (0.41, 0.76) (0.50, 1.00) |
| half-sine, peak 0.7 | (0.123, 0.053) (0.351, 0.314) (0.580, 0.764) (0.694, 1.000) |
| half-sine, peak 0.7, repeated | identical to the line above |
| half-sine, peak 1.0 | (0.192, 0.062) (0.495, 0.309) (0.828, 0.765), last point at (1, 1) |
| plateau at 0.5 (15 % ramps) | (0.082, 0.003) (0.253, 0.022) (0.416, 0.475) (0.497, 1.000) |

Findings:
1. **The wizard is deterministic**: the same strokes give the same curve.
2. **Always four interior points plus the fixed ends.**
3. **Inputs scale with the maximum pressure drawn, P_max:** the points sit at about
   **1/6, 1/2 and 5/6 of P_max, plus P_max itself** (ratios 0.16–0.19, 0.48–0.51, 0.82–0.83,
   0.99–1.0). Output reaches 100 % at P_max and stays there.
4. **The outputs depend on the pressure distribution, not on P_max:**
   - half-sine strokes give about **0.05, 0.31, 0.76** at every peak;
   - the plateau (mostly at P_max) gives **0.003, 0.022, 0.475**, so the curve stays low until
     near P_max.

   Simple percentile models don't fit:
   - the fraction of samples at or below the point: sine 0.11 / 0.33 / 0.63;
   - its complement: 0.89 / 0.67 / 0.37.

   **Open:** the rule mapping the pressure distribution to the outputs. Next: strokes with a
   known two-level distribution, and varying the event count, to see whether samples are
   weighted by time or by distance.
5. Afterwards the global curve was reset (Reset to default → Check adjusted settings → Done).
   Reopening the dialog showed the identity curve (2 points).
