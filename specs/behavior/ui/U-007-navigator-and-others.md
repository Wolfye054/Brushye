# U-007 Navigator, Sub View and other right-dock palettes
- Status: **observed**
- See README.

## Navigator (right dock group 1, active tab)
Tabs: **Navigator** (active), Sub View, Item bank, Information.
- **Preview** (y 96–267, 172 px tall, full dock width): the whole canvas scaled to fit, centred
  on the canvas-surround colour (45,45,45).
- **Controls** (y 269–311, two rows of ~20 px on (78,78,78)):
  - Row 1: zoom slider (wedge-shaped track ~85 px with a vertical knob), value field "100.0",
    then buttons: Zoom Out (−), Zoom In (+), 100%, Fit to Navigator, Fit To Screen.
  - Row 2: rotation slider (wedge, centre = 0), value field "0.0", then: Rotate Left,
    Rotate Right, Reset rotation, Flip Horizontal, Flip Vertical.

## Sub View
An empty image area (dark) with the same zoom/rotation rows, plus an eyedropper button and
previous/next/import/open/clear buttons at the bottom.

## Item bank
Empty list with add/settings buttons at the bottom (CSP feature; no Brushye counterpart).

## Information
"System: 49% Application: 100%" (memory use), then X / Y and H / V / L readouts with selection
icons.

## Quick Access and Material palettes (icon column)
Collapsed as square icon buttons (U-001). Not opened in this run.

## Timeline (bottom dock)
Collapsed; one icon button "Timeline" plus up/down arrows at the left to expand the dock.

## What Brushye should match
- Navigator as the first tab of the top-right group: fitted canvas preview, zoom slider + value
  + −/+/100%/fit-to-navigator/fit-to-screen, rotation slider + value + rotate left/right/reset
  + flip H/V. These are view-only (not undoable), like the canvas status strip.
- Sub View / Information tabs can come later; Item bank, Quick Access and the Material
  palettes are CSP-ecosystem features (Brushye decides whether to keep their slots).
