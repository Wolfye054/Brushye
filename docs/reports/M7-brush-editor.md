# M7 report: brush editor and alpha polish (2026-11-13)

**Status: feature work done. The alpha waits on two proposals (D-018, D-019) and the
maintainer's test.** CI is green on Windows, macOS and Linux (run 43550266044).

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
- **Tool state isn't on the command layer.** Sub tool edits, tool selection and colour chips
  live in the app and its settings file, not in commands. Proposal **D-018** fixes this. Under
  CLAUDE.md I treat this as an alpha blocker.
- **File dialogs are an in-app path field.** Proposal **D-019** adds native dialogs (`rfd`).
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

## Proposed next step
1. The maintainer approves (or changes) D-018 and D-019.
2. I implement both (about 2 days).
3. The maintainer tests the brush editor with their pen.
4. Tag `v0.1.0-alpha` and publish the draft release. Then M8, `.sut` import.
