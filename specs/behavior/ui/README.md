# UI behaviour specs (U-xxx)

How CSP 1.13.2 organises its window. Brushye matches the organisation and the look (palette
placement, order, sizes, colours, metrics); icons are Brushye's own (Phosphor) and wording is
Brushye's own. Screenshots used for these specs stay in `obs-out/ui/` on the observation machine
and are never committed (CSP UI artwork).

Common header for U-001 … U-009:
- CSP version: 1.13.2 (PRO, 64-bit), observation prefix `~/.wine-csp-observe`.
- Date / method / observer: 2026-11-13 / automated-Wine (Xorg :78, 1920×1080 virtual desktop,
  XTEST mouse/keys, `import` screenshots) / Claude.
- Workspace: restored with **Window > Workspace > Reset to Default(I)**, confirmed with **Yes**
  in the "layout of palette will be reset" dialog. Doing it twice gives pixel-identical
  screenshots, so the layout below is CSP's default palette layout (not a customised one).
- Open documents at the time: three restored tabs from earlier observation runs, the active one
  800×800 px with one raster layer. Document content doesn't affect the layout.
- Coordinates are screen pixels at 1920×1080; the CSP window is maximised inside the Wine
  desktop. Analysis helper: `tools/observe/ui_scan.py` (run-length colour scans).
- Wine caveats: the window caption (title bar with min/max/close, y 4–27) is drawn under Wine
  without a window manager, and fonts are Wine substitutes. On Windows the caption is the OS's
  and the UI font is the system font (verify-native).

## Implementation in Brushye (2026-11-13, `crates/brushye-app/src/ui/`)
Status of U-001 … U-009: **implemented** (layout and look), with the gaps below.
- `ui/mod.rs`: the U-001 regions and palette group heights at 1080 px; the Layer group takes
  the rest. `ui/theme.rs`: the U-009 Dark and Light colours and metrics (Dark default;
  File > Preferences > Interface color). Phosphor icons in the icon grey.
- Menus (U-002): nine menus in CSP's order and groups. Unbuilt features are disabled in place;
  CSP-ecosystem items (CLIP STUDIO launcher, TABMATE, QUMARION, smartphone) are omitted.
- Command bar (U-003), tool column with separators and colour chips (U-004), Sub Tool (group
  tabs, stroke-preview rows rendered by the brush engine, add/copy/delete), Tool property (preview
  strip, slider rows with dynamics buttons, AA segmented control, reset/detail buttons), Brush
  size grid, colour group (wheel, set, slider, history, approximate) (U-005), Navigator and
  Information (U-007), Layer Property, Layer palette (top bar, two toolbar rows, 40 px rows,
  paper row, context menu, drag reorder, rename) and History (U-006), document tab, status strip,
  scroll bars, timeline dock and icon column (U-008).
- Gaps:
  - **Layout:** no splitter dragging between palette groups or docks; no floating or undocked
    palettes; the palette "≡" menus are empty.
  - **Palettes not built:** Color Mixing, Intermediate Color, Sub View, Item bank, Search
    Layer, Animation cels, Auto Action, Quick Access, materials, timeline. Their tabs and slots
    are kept and disabled.
  - **Navigator:** the flip buttons are disabled (view flipping isn't built).
  - **Light theme:** row separator, name plate and slider colours are estimates; only the
    U-009 table's elements were measured.
  - **Platform differences:** the window caption is the OS's, and the UI font is Go Sans; CSP
    uses the system UI font.
