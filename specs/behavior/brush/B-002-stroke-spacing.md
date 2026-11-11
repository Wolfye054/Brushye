# B-002 Stroke gap (spacing): Narrow / Normal / Wide / Fixed
- Status: draft
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK (mouse, constant speed via scripted XTEST motion)

## Purpose
Dab interval as a function of brush size and gap setting. Also: does Fixed take px,
and is the first dab placed at the press point?

## Setup
B-001 setup, size 40 px, AA None, **Brush density 30** (so overlapping dabs are visible), canvas
512×128, transparent.

## Steps
1. For each gap in {Narrow, Normal, Wide, Fixed=10, Fixed=37}: a horizontal stroke from
   (40,64) to (472,64), 200 motion events, 5 ms apart (scripted, recorded in the result file).
2. Export each as `B002_<gap>.png`.
3. Repeat Normal with sizes 10 and 100.

## Expected result / what to measure
Alpha along the centre row. Periodic maxima → dab positions → spacing in px; spacing/size ratio
per setting; whether spacing depends on mouse speed (rerun Normal with 20 ms intervals).

## Open questions
- Is spacing measured along the path (arc length) or per input event?
- "Adjust brush density by gap": run with it on and off.
