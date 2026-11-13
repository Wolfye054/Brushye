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
