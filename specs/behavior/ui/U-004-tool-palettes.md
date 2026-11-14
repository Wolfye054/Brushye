# U-004 Tool, Sub Tool, Tool property and Brush size palettes
- Status: **observed**
- See README.

## Tool palette (tool column, x 4–44)
- No title bar in the default layout; it starts at the column's header (y 52–72) and lists
  tools as single icons on a ~25 px pitch (y 108 … 665), each button ~24×22, icon ~20 px,
  centred. The current tool has an accent background (96,106,128) of about 22×22.
- Order (tooltips; letter = shortcut), with 1 px separators (114,114,114) where marked ‖:
  Zoom (/), Move, Operation, Move layer (K), Selection area (M),
  Auto select (W), Eyedropper (I) ‖ Pen (P), Pencil (P), Brush (B), Airbrush (B),
  Decoration (B), Eraser (E), Blend (J), Liquify (J) ‖ Fill (G), Gradient (G), Figure (U),
  Frame Border (U), Ruler (U), Text (T), Balloon (T), Correct line (Y).
  Separators at y 272 (before Pen) and y 475 (before Fill).
- Each tool is one button; a tool's variants live in the Sub Tool palette (no flyouts in the
  tool column).
- Below the tools (y ~683–726): **colour chips**: main colour square (~18×18, top-left,
  selected = framed) overlapping the sub colour square (bottom-right), and a transparent-colour
  chip (checker strip) under them. Same chips appear in the Color Wheel palette (U-005).

## Sub Tool palette (left dock group 1, y 73–361)
- Title bar "Sub Tool: <tool name>" (palette icon + text) with the "≡" menu button.
- **Group tabs** row (y 96–121, 26 px): sub tool groups of the current tool as tabs, two per
  row (each ~100 px wide: icon + label), selected tab in accent (103,113,135) with rounded
  2 px corners. The Pen tool has two groups (Pen, Marker).
- **List** (y 123–339): one row per sub tool, 31 px pitch (30 px + 1 px dark separator
  (34,34,34)). Each row shows a white stroke preview drawn across the row and the sub tool name
  right-aligned near the bottom; the selected row is accent (96,106,128) with a lighter name
  plate behind its label. Vertical scroll bar (9 px, track (90,90,90), thumb (67,67,67)) at the
  right.
- **Bottom bar** (y 342–361, 20 px, (78,78,78)): three icon buttons right-aligned:
  Add sub tool, Create copy of currently selected sub tool, Delete selected sub tool.

## Tool property palette (left dock group 2, y 364–606)
- Title "Tool property: <sub tool name>".
- **Preview strip** (y 381–424, 44 px): the sub tool name in a small label at top-left, a
  preview of the brush's dab trail across the strip, and a lock toggle at the right ("Lock sub
  tool and do not save changes").
- **Setting rows** (y 426–586) for the Pen sub tool, in order:
  1. Brush Size: label at top-left; value "40.0" right-aligned with an up/down spinner; a
     square **dynamics button** at the far right (pressure etc.); a thin slider bar under the
     label (8 px tall, fill (88,88,88) with a 1 px lighter top edge (111) and bottom edge (123);
     unfilled part = baseline only), fill length ∝ value.
  2. Opacity: same row type, value "50".
  3. Anti-aliasing: label + a 4-step segmented control (none / weak / middle / strong shown as
     dots of increasing softness, selected step in accent) + a ▾ dropdown.
  4. Stabilization: label + thin slider + value "0" with spinner (no dynamics button).
  5. Adjust by speed: a "+" expander and checkbox.
  6. Vector magnet: greyed row with a checkbox and a segmented control.
  Rows are ~27 px tall, separated by 1 px lines; labels in the normal text colour at ~12 px.
- **Bottom bar** (y 587–606, 20 px): two buttons right-aligned: "Reset selected sub tool to
  default settings" and "Show [Sub Tool Detail palette] and edit…" (wrench).
- Which settings appear is per sub tool (a subset of the Sub Tool Detail settings).

## Brush size palette (left dock group 3, y 609–850)
- Title "Brush size: <sub tool name>".
- A grid of preset sizes: **6 columns × 32 px**, rows **40 px**, 1 px dividers. Each cell
  shows a white dot whose diameter grows with the size (clipped for large sizes) and the size
  number centred below it. The current size's cell is accent (96,106,128).
- Presets (px), row by row: 0.7, 1, 1.5, 2, 2.5, 3 / 4, 5, 6, 7, 8, 10 / 12, 15, 17, 20, 25, 30 /
  40, 50, 60, 70, 80, 100 / 120, 150, 170, 200, 250, 300 / 400, 500, 600, 700, 800, 1000 /
  1200, 1500, 1700, 2000. The list scrolls (scroll bar at the right).

## What Brushye should match
- Tool column: icon strip with the same tool order and the two separators; Brushye's tools in
  their CSP positions (Zoom, Move, Move layer, Selection, Eyedropper, Pen, Pencil, Brush,
  Airbrush, Eraser, Fill, Gradient, Figure, Text, …) and not-yet-built tools disabled or
  omitted; shortcuts as listed; colour chips at the bottom.
- Sub Tool palette: group tabs + list of sub tools with stroke preview and right-aligned name,
  31 px rows, add/copy/delete buttons at the bottom. Brushye's sub tools are its own presets
  (`.brshbrush`), never CSP's defaults.
- Tool property: preview strip, rows with label/thin bar/value/spinner/dynamics button, the AA
  segmented control, bottom bar with reset and "detail" buttons.
- Brush size: the same preset sizes in a 6-column grid with dots.
