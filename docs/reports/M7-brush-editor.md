# M7 report: brush editor and alpha polish (2026-11-13)

**Status: done, pending the maintainer's test.** D-018 and D-019 were approved and are
implemented (see the update at the end).

## What works
- **Sub tools as in CSP (U-011):**
  - each tool has sub tool groups (tabs) and sub tools with original names and original
    default settings;
  - each sub tool has a stroke preview, drag reordering, and Stroke or Text view;
  - the ≡ menu and the bottom buttons copy, delete, add defaults, lock, reset, and register
    the current settings as the user default;
  - the eraser end of the pen uses the Eraser sub tool.
- **Tool property palette:**
  - its rows come from the sub tool's shown list;
  - eye toggles in Sub Tool Detail add or remove rows, as in CSP;
  - a reset button and a wrench button open Sub Tool Detail.
- **Sub Tool Detail window (U-010):**
  - a preview strip, the 19 CSP categories in the text list, and the rows for every alpha
    brush setting: size, opacity, AA, hardness, density, density by gap, gap, thickness and
    direction, angle and follow line, stabilization, ink mode, and pressure for size, opacity
    and density;
  - settings Brushye doesn't have yet are shown greyed, so the layout matches CSP.
- **Dynamics pop-ups:** they open next to their button, with pen pressure, the minimum value
  and the curve editor. Tilt, velocity and random are listed and disabled.
- **Brush size palette:** CSP's preset sizes, plus user presets from the ≡ menu.
- **Colour (U-013):**
  - main, sub and transparent chips in sync in both places;
  - X switches the active chip;
  - HSV square or HLS triangle, chosen with the wheel button or the ≡ menu;
  - a history that records colours used in strokes, with the current colour framed red.
- **Palette ≡ menus (U-012):** Navigator, Layer Property, Layer and Color. Each palette can be
  hidden or shown from the Window menu, and the choices persist in `settings.json`.
- **Brush files:** `.brshbrush` import and export, and the brush library folder (D-014,
  already accepted).
- **Headless:**
  - every alpha *document* feature is a command: layers, blend and opacity, fills, strokes with
    every brush setting, the pressure curve, undo and redo;
  - the CLI creates, shows and lists brush files (`brush new/show/list`), runs, replays and
    renders journals, and writes .brsh and PNG;
  - `docs/commands.md` is generated and checked by a test.
- **Release builds:** `v*` tags build Windows, macOS and Linux binaries and create a **draft**
  GitHub release for the maintainer to review.
- **Autosave and recovery:** unsaved work is saved every 2 minutes. After a crash it reopens as
  `<name>[Restore]`. Tested with `kill -9`.

## Checked by hand (automated, Xvfb 1920×1080)
- The default layout, Sub Tool Detail (Brush Size and Brush tip categories), the Brush Size and
  Angle dynamics pop-ups, the colour ≡ menu, and the HLS triangle. Fixed during the check: the
  dynamics pop-up used to open at the top-left of the screen instead of beside its button.

## What doesn't work yet / gaps
- **Pressure wizard (CSP's "Adjust pen pressure"):** not built. B-006 recorded its inputs (taps
  at 1/6, 1/2, 5/6 of the maximum pressure, plus the maximum), but the outputs fit no simple
  rule yet. The manual curve editor (File › Pen pressure settings) works.
- **Hardness bar:** 5 plain steps. CSP's ▸ at its end isn't drawn yet.
- **Not in the alpha:** tilt and velocity dynamics, material tips, texture, spray, dual brush,
  palette splitter resizing, floating palettes, and Windows/macOS tablet backends (no test
  hardware).

## Specs waiting on observation
- B-006 wizard output rule (needs more targeted runs, or the maintainer's pen).
- Tilt dynamics (needs the maintainer's pen; virtual tilt not proven under Wine).

## Update: D-018 and D-019 (2026-11-13)
- **Tool state is on the command layer (D-018).** 15 commands (`tool.select`, `subtool.select`,
  `.set`, `.add`, `.copy`, `.delete`, `.move`, `.lock`, `.reset`, `.save_default`,
  `.show_row`, `.add_defaults`, `.group_set`, `color.set`, `color.active`) and 3 queries
  (`?tools`, `?subtool`, `?color`). The model lives in `brushye-command::tools`. The session
  holds it and keeps it across `doc.new` and file opens. Every palette, the Sub Tool Detail
  window, the brush size palette, shortcuts, import, and the colour palette and chips now
  change it only through commands. Tool commands aren't written to the document journal
  (refinement recorded in D-018).
- **Native file dialogs (D-019):** Open, Save as, Save duplicate, Export PNG, and sub tool
  import and export use the OS dialogs via `rfd` 0.17.
- **Checked:** unit tests for every command, a test that the palettes' edit-to-command
  conversion reproduces an edit, and a run in the app on a 1920×1080 test display (tool and
  sub tool selection, brush size presets, colour ring, copy, a stroke in the chosen colour,
  saved settings).

## Proposed next step
1. The maintainer tests the brush editor and the file dialogs with their pen.
2. Tag `v0.1.0-alpha` and publish the draft release. Then M8, `.sut` import.
