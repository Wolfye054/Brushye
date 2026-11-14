# U-001 Default workspace layout
- Status: **observed**
- See README for version, method and workspace reset.

## Overall structure (top to bottom)
| Band | y (px) | Height | Contents |
|---|---|---|---|
| Window caption | 4–27 | 24 | Document title centred ("<name>* (<w> x <h>px <dpi>dpi <zoom>%) - <app>"), min/max/close at the right. Wine-drawn here. |
| Menu bar | 28–51 | 24 | Nine menus, see U-002. Full window width. |
| Dock headers + command bar | 52–76 | ~24 | Left and right dock columns start here (each with a thin header bar); the **command bar sits only above the canvas column** (x 257–1640), see U-003. |
| Document tabs | 78–96 | 19 | One tab per open canvas (canvas column only). |
| Canvas | 98–1028 | 931 | Canvas on a flat surround; vertical scroll bar at the right edge (x 1629–1640). |
| Canvas status strip | 1029–1041 | 13 | Zoom and rotation controls + horizontal scroll bar, see U-008. |
| Timeline dock (collapsed) | 1042–1079 | 38 | A collapsed bottom dock under the canvas column holding the Timeline palette as an icon button, with expand arrows at its left end. |

The left and right dock columns run the full height from y 52 to the bottom of the window; the
command bar, tabs, canvas, status strip and timeline dock are all between them.

## Columns (left to right)
| Column | x (px) | Width | Contents |
|---|---|---|---|
| Tool column | 4–44 | 41 | **Tool** palette: one vertical strip of tool icons, then main/sub/transparent colour chips (U-004). |
| splitter | 45–46 | 2 | 1–2 px dark line. |
| Left dock | 47–254 | 208 | Four stacked palette groups (below). |
| splitter | 255–256 | 2 | |
| Canvas column | 257–1640 | 1384 | command bar, tabs, canvas, status strip, timeline dock. |
| Icon column | 1642–1681 | 40 | Collapsed palettes shown as icon buttons: **Quick Access** and ten **Material** palettes. |
| Right dock | 1683–1909 | 227 | Three stacked palette groups (below), plus a 10 px strip at 1910–1919 used by scroll bars. |

## Left dock (x 47–254), top to bottom
| # | Palette group (tabs) | y range | Height | Notes |
|---|---|---|---|---|
| – | dock header | 52–72 | 21 | Collapse arrows/grips (U-009). |
| 1 | **Sub Tool** (single palette, title "Sub Tool: <tool>") | 73–361 | 289 | Title 73–89; group tabs 96–121; list 123–339; bottom button bar 342–361. |
| 2 | **Tool property** (title "Tool property: <sub tool>") | 364–606 | 243 | Title 364–380; preview strip 381–424; settings rows 426–586; bottom bar 587–606. |
| 3 | **Brush size** (title "Brush size: <sub tool>") | 609–850 | 242 | Title 609–625; 6-column preset grid from 631, scrolls. |
| 4 | **Colour** group: tabs Color Wheel (active), Color Set, Color Slider, Color Mixing, Color History, Approximate Color, and Intermediate Color reachable from the tab overflow arrow | 854–1079 | 226 | Title/tab strip 854–870; content 871–1079. |

## Right dock (x 1683–1909), top to bottom
| # | Palette group (tabs) | y range | Height | Notes |
|---|---|---|---|---|
| – | dock header | 52–72 | 21 | |
| 1 | **Navigator** (active), Sub View, Item bank, Information | 73–311 | 239 | Title 73–89; preview 96–267; two control rows 269–311. |
| 2 | **Layer Property** (active), Search Layer, Animation cels | 314–552 | 239 | Title 314–330; content 336–552. |
| 3 | **Layer** (active), History, Auto Action | 555–1079 | 525 | Title 555–571; top bar 572–597; two toolbar rows 599–643; layer list from 644. |

## Icon column (x 1642–1681)
Eleven square buttons, 36 px pitch from y 75: Quick Access, Material: Color pattern,
Material: Background, Material: Monochromatic pattern, Material: Basic, Material: Manga material,
Material: Effect line, Material: Image material, Material: Illustration, Material: 3D,
Material: Pose. Clicking one opens that palette as a pop-up next to the column.

## Tabs and title bars
- Every palette group has a 17 px title bar: a "≡" menu button at the left, then one tab per
  palette. The active tab shows an icon + its name on a lighter tab; inactive tabs show only
  their icon. The colour group shows only icons (tab names appear in its overflow list).
- No palette is collapsed or floating in the default layout; only the icon column and the
  timeline dock are in collapsed (icon) form.

## ASCII layout (1920×1080)
```
y   x:0  4   45 47                254 257                                      1640 1642 1681 1683         1909 1919
4   +---------------------------------------- window caption --------------------------------------------------+
28  | File Edit Animation Layer Select View Filter Window Help                                                   |
52  |[hdr][ left dock header         ][ command bar (U-003)                          ][hdr ][ right dock header   ]|
73  | T  |[Sub Tool: Pen            ]|[ tab | tab | tab* ]                             | Q  |[Navigator|SV|IB|In]|
96  | o  | Pen | Marker  (group tabs)|                                                 | M1 |  canvas preview    |
123 | o  | sub tool list (31px rows) |                                                 | M2 |                    |
269 | l  |                           |              canvas                             | M3 | zoom row / rot row |
314 |    | [bottom buttons]          |              (surround 45,45,45)                | M4 |[Layer Property|SL|AC]
364 | c  |[Tool property: <sub>]     |                                                 | .. |  Effect /          |
381 | o  | preview strip             |                                                 | M10|  Expression color  |
426 | l  | setting rows              |                                                 |    |                    |
555 | u  |                           |                                                 |    |[Layer|History|AA]  |
587 | m  | [bottom buttons]          |                                                 |    | blend | opacity    |
609 | n  |[Brush size: <sub>]        |                                                 |    | toolbar row 1      |
631 |    | 6×N size grid             |                                                 |    | toolbar row 2      |
683 |chip|                           |                                                 |    | layer rows (40px)  |
854 |    |[colour tabs ......... v]  |                                                 |    |                    |
871 |    | colour wheel + chips      |                                                 |    |                    |
1029|    |                           |[ zoom | rot | ...      status strip     scroll  ]|    |                    |
1042|    |                           |[^ Timeline (collapsed dock)                     ]|    |                    |
1079+----+---------------------------+-------------------------------------------------+----+--------------------+
```

## What Brushye should match
- The same seven vertical regions in the same order: tool column (~41 px), left dock (~208 px)
  with Sub Tool / Tool property / Brush size / Colour group stacked in that order, canvas column,
  an icon column (~40 px) of collapsed palettes, and a right dock (~227 px) with Navigator group /
  Layer Property group / Layer group.
- Menu bar across the full width; the command bar only above the canvas column; document tabs
  above the canvas; the zoom/rotation status strip under it; a collapsed timeline dock at the
  bottom of the canvas column.
- Group heights at 1080 px as in the tables (Sub Tool ≈ 289, Tool property ≈ 243, Brush size
  ≈ 242, Colour ≈ 226; Navigator ≈ 239, Layer Property ≈ 239, Layer = the rest). Docks resize
  with the window; the Layer group takes the remaining height.
- Tabbed palette groups with icon-only inactive tabs, a "≡" palette menu on each title bar, and
  the tab sets listed above. Palettes Brushye doesn't have yet keep their place as disabled
  tabs/buttons or are omitted until built (Brushye's call per palette).
- A reset-to-default-layout command that restores exactly this arrangement.
