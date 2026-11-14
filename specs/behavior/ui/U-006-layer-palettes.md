# U-006 Layer palette and Layer Property palette
- Status: **observed** (tooltips by hovering; right-click menu opened on the layer row)
- See README.

## Layer palette (right dock group 3, y 555–1079)
Tabs: **Layer** (active, icon + label), History, Auto Action.

**Top bar** (y 572–597, 26 px):
1. Palette colour button (square swatch) with ▾ — "Change palette color".
2. **Blending mode** dropdown ("Normal"), ~74 px wide, button style (103,103,103) with a 1 px
   bevel.
3. **Opacity**: a horizontal slider showing a checker-to-solid ramp, with the value "100" and a
   spinner at the right.

**Toolbar row 1** (y ~599–616), right-aligned icon buttons, in order:
Clip to Layer Below · Set as Reference Layer · Set as Draft Layer · Lock Layer ·
Lock transparent pixels · Enable Mask (▾) · Set ruler range (▾) · Change layer color (colour
swatch ▾).

**Toolbar row 2** (y ~618–643): at the left "Show layer in 2 panes"; right-aligned:
New Raster Layer · New vector layer · New Layer Folder · Transfer To Lower Layer ·
Merge with layer below · Create layer mask · Apply Mask to Layer · Delete Layer.
Unavailable actions are drawn dimmed.

**Layer list** (from y 644, background (63,63,63)):
- Row height **40 px** (+1 px separator). Columns left to right:
  1. **eye** (visibility), ~20 px column;
  2. **editing-target pen** column, ~19 px (shows a pen icon on the layer being drawn on);
  3. **thumbnail** 38×38 inside a 1 px frame (transparent shown as checker);
  4. text, two lines: first "100 % Normal" (opacity and blend mode, small), second the layer
     name ("Layer 1").
- Selected row: accent background (96,106,128) across the text area and around the thumbnail;
  the eye/pen columns keep the list background.
- Folders and the paper layer were not present in the observation document (not observed).
- Right-click on a row: the Layer menu's first four groups (U-002), from New Raster Layer to
  Flatten image.

## History tab
A plain list of history steps (first entry = document name), selected step in accent.

## Auto Action tab
A set dropdown ("Default") with buttons, then a list of actions with a checkbox, a second
checkbox column and an expander per row; play/add/delete buttons at the bottom.

## Layer Property palette (right dock group 2, y 314–552)
Tabs: **Layer Property** (active), Search Layer, Animation cels.
- Group "Effect": a framed group label, then a row of three effect toggle buttons plus a ▾
  (icons not identified: tooltips not read).
- Group "Expression color": a dropdown showing "Color" with a small RGB icon.
- The rest of the palette is empty list background for a plain raster layer.
- Search Layer tab: a "Show these layers" filter dropdown and buttons. Animation cels tab:
  cel controls (greyed without a timeline).

## What Brushye should match
- Layer palette at the bottom of the right dock, tabbed with History (Brushye's undo history
  view) and Auto Action (later).
- Top bar: palette colour, blend mode dropdown, opacity slider + value.
- Two toolbar rows of icon buttons in the same order (Brushye's available ones enabled, the
  rest disabled), "new / merge / delete" on the second row.
- 40 px layer rows with eye, editing-target, 38 px thumbnail, "opacity % blend" line above the
  name, accent selection; right-click menu mirroring the Layer menu.
- Layer Property palette above it with Effect and Expression color groups.
