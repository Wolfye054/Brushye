# U-003 Command bar
- Status: **observed** (tooltips read by hovering each button)
- See README.

## Placement and size
- One row above the canvas column only (x 257–~760 used, the rest of the row up to x 1640 is
  empty bar), y 53–76 (24 px). Background = menu bar colour (78,78,78).
- Icon buttons on a ~24 px pitch, icons ~20 px, drawn in the light icon grey (178,178,178).
  Toggle buttons that are on get a 24×24 accent background (103,113,135); adjacent "on" toggles
  touch. Groups are separated by a 1 px vertical separator with ~6 px padding on each side.
- A drag grip "||" at the left end (x ~250) belongs to the dock header row.

## Buttons in order (tooltip → function)
| # | Tooltip | Function | Default state |
|---|---|---|---|
| 1 | Open CLIP STUDIO | opens the companion launcher app | |
| — | separator | | |
| 2 | New | File > New | |
| 3 | Open | File > Open | |
| 4 | Save | File > Save; a small ▾ next to it has the same tooltip (not opened) | |
| — | separator | | |
| 5 | Undo | Edit > Undo | |
| 6 | Redo | Edit > Redo | |
| — | separator | | |
| 7 | Delete | Edit > Delete (clears selection/layer content) | |
| 8 | Delete Outside Selection | Edit > Delete Outside Selection | greyed (no selection) |
| 9 | Fill | Edit > Fill | |
| 10 | Scale/Rotate | Edit > Transform > Scale/Rotate | |
| — | separator | | |
| 11 | Deselect | Select > Deselect | greyed |
| 12 | Invert selected area | Select > Invert | greyed |
| 13 | Show border of selected area | toggle | |
| — | separator | | |
| 14 | Snap to Ruler | toggle (View > Snap to Ruler) | **on** |
| 15 | Snap to Special Ruler | toggle | **on** |
| 16 | Snap to Grid | toggle | off |
| — | separator | | |
| 17 | Connect to smartphone | companion-app pairing | |
| — | separator | | |
| 18 | CLIP STUDIO PAINT support | opens support pages | |

The bar is user-configurable (File > Command Bar Settings…); the above is the default set.

## What Brushye should match
- A command bar above the canvas column only, 24 px tall, menu-bar colour, with Brushye's
  equivalents of these buttons in the same order and groups: New, Open, Save(▾) | Undo, Redo |
  Delete, Delete outside selection, Fill, Scale/Rotate | Deselect, Invert selection, Show
  selection border | Snap to ruler, Snap to special ruler, Snap to grid. CSP-ecosystem buttons
  (CLIP STUDIO launcher, smartphone, support) have no Brushye counterpart: leave them out or
  replace with Brushye's own help button at the end.
- Toggle buttons show the accent background when on; buttons for unavailable actions are drawn
  disabled, not hidden.
- Phosphor icons at ~20 px in the icon grey.
