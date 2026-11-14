# U-008 Canvas window: tabs, status strip, scroll bars
- Status: **observed**
- See README.

## Document tabs (y 78–96, 19 px, canvas column)
- Strip background (56,56,56). One tab per open canvas, square corners, ~99 px wide when the
  title is truncated (longer for the active one here: 135 px), separated by 2 px gaps.
- Inactive tab (78,78,78) with dim text; active tab accent (103,113,135) with light text.
  Each tab ends with a "●" (shown on modified documents here; its hover/close behaviour was not checked).
- A ▾ button at the far right of the strip (function not checked).

## Canvas area (y 98–1028)
- Surround colour (45,45,45). Transparent canvas pixels shown as an **8 px checkerboard of
  white (255) and light grey (204)**.
- Vertical scroll bar at x 1629–1640 (12 px) with arrow buttons; horizontal scroll bar in the
  status strip.

## Status strip (y 1029–1041, 13 px, background (67,67,67))
Left to right:
1. Zoom value field "100.0" (dark field (49,49,49)).
2. Zoom slider (wedge track).
3. Zoom out (−), Zoom in (+), and a "■" button (100% / fit).
4. Rotation value field "0.0".
5. Rotation slider (wedge).
6. Rotate left (↶), rotate right (↷), reset rotation.
7. "<" scroll arrow, then the **horizontal scroll bar** (thumb (90,90,90)) to the right edge,
   ">" arrow.

## Timeline dock (y 1042–1079)
Collapsed bottom dock under the canvas column (see U-007).

## What Brushye should match
- Tabs above the canvas with this styling and the modified-document dot.
- Surround (45,45,45) and an 8 px white / (204,204,204) checkerboard for transparency.
- The thin status strip under the canvas with zoom field + slider + −/+ and rotation field +
  slider + rotate/reset, followed by the horizontal scroll bar; a vertical scroll bar at the
  right of the canvas.
