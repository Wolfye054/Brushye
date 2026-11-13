# B-006 Global pen pressure adjustment and its graph
- Status: draft
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine with virtual stylus OK (D-003, verified 2026-11-13). Flag `verify-native`: one spot check with a real pen on native Windows (Wintab vs Windows Ink may scale pressure differently)

## Steps
1. Screenshot the default graph. Press Stronger ×1, ×3 and Lighter ×1, ×3, screenshotting the
   graph each time (automated-Wine OK for this part).
2. For each state, draw the B-004 ramp stroke with the pen (maintainer).

## What to measure
The control-point positions per state (from screenshots, used only to understand the
behaviour) and the effect on B-004 output.
