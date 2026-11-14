# U-009 Visual style: colours and metrics
- Status: **observed** (sampled from screenshots of the default workspace, the open menus, the
  Preferences dialog, and the Light theme for comparison; `tools/observe/ui_scan.py`)
- See README. All colours are sRGB 8-bit values read from exported screenshots (no colour
  management in the X server; values are as rendered).

## Theme setting
- File > Preferences > **Interface** > Color: **Interface color** dropdown with **Light color**
  and **Dark color**, plus an **Adjust density** slider (knob at the middle of its track).
- In this prefix the setting is **Dark color** with density in the middle; all values below
  are for that. Whether Dark is CSP 1.13.2's factory default could not be confirmed (the
  application settings were not reset). Switching to Light and back was done in this run and
  the default workspace screenshot was pixel-identical afterwards.
- CSP's UI is flat grey with one blue-grey accent; there are no gradients except bevels on
  dropdown buttons.

## Dark theme colours (default here)
| Element | RGB |
|---|---|
| Window caption / dock header bars / palette title bar background / tab strip / timeline dock | (56,56,56) |
| Menu bar, command bar, palette tab (inactive and title tab), bottom button bars, colour-palette content, navigator controls, layer top bar & toolbars, menu pop-up background | (78,78,78) |
| Lists and palette bodies (sub tool list, tool property rows, brush size grid, tool column, layer list, layer property body) | (63,63,63) |
| Canvas surround, navigator preview background | (45,45,45) |
| Canvas status strip, scroll bar track (vertical), right edge strip | (67,67,67) |
| Splitters / thin dark borders between docks and tab groups | (49,49,49) |
| Row separators in lists (sub tool rows, brush grid) | (34,34,34) |
| Accent: active tab, active group tab, active menu title, hovered menu item, toggles that are on, active document tab | (103,113,135) |
| Accent (selection): selected list row, selected layer row, current tool, current brush size | (96,106,128) |
| Selected sub tool name plate | (135,143,159) |
| Default dialog button (OK) | (104,121,160), focus text (219,235,240) |
| Normal button, dropdown, input field, checkbox box | (103,103,103); dropdown top bevel (126,126,126) |
| Dark value fields in the status strip | (49,49,49) |
| Slider bar fill / top edge / bottom edge | (88,88,88) / (111,111,111) / (123,123,123); unfilled = 1 px baseline (99) |
| Scroll bar track / thumb (palettes) | (90,90,90) / (67,67,67) |
| Tool column separator | (114,114,114) |
| Dialog list selection (Preferences categories) | (115,128,154) |
| Icons (line icons) | (178,178,178); dimmed icons ~(109–128) |
| Text, normal (palettes, layer names, menu items) | (218,218,218) (anti-aliased glyph cores 207–218) |
| Text, dim (inactive tab labels) | ~(146,146,146) |
| Text, disabled (greyed menu items, disabled rows) | (147–152) — use (150,150,150) |
| Text on accent (active document tab) | light grey-blue ~(190,192,198) |
| Canvas checkerboard | (255,255,255) / (204,204,204), 8 px squares |
| Window caption text (Wine) | (161,161,161) |

Hover for menu items = the accent (103,113,135) full-row highlight; pressed/hover states of
palette buttons were not captured (verify-native).

## Light theme (alternative), same elements
| Element | Dark | Light |
|---|---|---|
| Caption / dock headers / title bars / tab strip | (56,56,56) | (224,224,224) |
| Menu bar, command bar, tabs, bottom bars | (78,78,78) | (240,240,240) |
| Lists / palette bodies | (63,63,63) | (230,230,230) |
| Canvas surround | (45,45,45) | (199,199,199) |
| Status strip | (67,67,67) | (239,239,239) |
| Splitters | (49,49,49) | (184,184,184) |
| Accent (active tab / toggles) | (103,113,135) | (184,208,229) |
| Accent (selected row / tool) | (96,106,128) | (179,203,224) |
| Dropdown | (103,103,103) | (255,255,255) |
| Icons and text | light | dark (icons ~black) |

## Metrics (px at 1920×1080, 100 % UI scale)
| Item | Value |
|---|---|
| Menu bar height | 24 |
| Menu pop-up item row | 19; separator block ~8; pop-up left gutter for check marks ~30 |
| Command bar height / button pitch / icon | 24 / ~24 / ~20; toggle-on background 24×24 |
| Dock header bar | 21 |
| Palette title bar (tabs) | 17; "≡" menu button at the left ~9 px wide |
| Sub tool group tabs | 26 tall, ~100 wide, 2 px rounded corners |
| Sub tool list row | 31 (30 + 1 separator) |
| Tool property preview strip / row | 44 / ~27 |
| Bottom button bars | 20 |
| Brush size grid cell | 32 × 40 (1 px dividers) |
| Tool button pitch / highlight | 25 / ~22×22; icon ~20 |
| Icon column button | ~32×32 on a 36 px pitch, 1 px dark border, 4 px gap |
| Layer row | 40 (+1); thumbnail 38 + 1 px frame; eye column ~20, editing column ~19 |
| Layer top bar / toolbar rows | 26 / ~22 each |
| Document tab height / truncated width | 19 / ~99, gap 2 |
| Status strip | 13 |
| Scroll bars | 12 (canvas), 9 (palette lists) |
| Dialog buttons | 80 × 29 |
| Dropdown height (dialogs / layer bar) | 30 / 20 |
| Input field | 38 × 23 (dialog) |
| Checkbox | ~14 × 14 |
| Corner radii | tabs, buttons, fields: square (0); group tabs 2; dropdowns 1 px chamfer |
| Font | UI text cap height 8 px, digits ~9 px → about 12 px (9 pt at 96 dpi); same size in menus, palettes and dialogs; no bold except none observed. Under Wine the face is a substitute: on Windows CSP uses the system UI font (verify-native). |
| Text line height | menu rows 19; layer row text lines ~19 each (two lines in 40 px) |
| Palette padding | content inset ~2–6 px from palette edges; group frames ("Effect", "Expression color") with a 1 px line and the label breaking it |

## What Brushye should match
- Offer Dark (default) and Light interface colours with these exact values, and an
  interface-density adjustment later.
- Use the two accents exactly: (103,113,135) for active tabs / toggles / hovered menu rows and
  (96,106,128) for selected rows / current tool / current preset; the dialog default button
  (104,121,160).
- Flat panels in the four greys (56 / 63 / 78 and 45 for the canvas surround), 1 px (49)
  splitters, (34) row separators.
- Text (218), dim (146), disabled (150); icons (178) — Phosphor icons tinted to this grey.
- The metrics table above (bar heights, row heights, cell sizes, square corners) and a ~12 px
  UI font.
