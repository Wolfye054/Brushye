# M6 report: app shell and GPU canvas (2026-11-13)

**Status: ready for the maintainer's pen test.** M6's exit criterion is that the maintainer
draws comfortably with pressure on their main OS (Linux, Wayland). It has been tested here with
the virtual stylus on X11, not yet with a real pen on Wayland.

Run it with `cargo app` (release build), or `cargo app path/to/file.brsh`.

## What works
- **Window** (`brushye-app`, eframe/egui on wgpu): menus (File, Edit, View), tool and colour
  palette on the left, layer palette on the right, status bar (zoom, angle, cursor position,
  pen pressure, tablet backend, GPU), dialogs.
- **Canvas display, as D-012 4 specifies:**
  - the CPU composites only the 256² tiles that changed (`display.rs`), found by comparing
    shared-tile identity, with no dirty flags;
  - a small wgpu pipeline of our own (`canvas_gpu.rs`) draws them with pan, zoom and rotation:
    nearest sampling at ≥ 100 %, linear below, and a checkerboard under transparent areas.
- **Display test** (`tests/display.rs`): offscreen on a software GPU, at 100 % the shown pixels
  equal the CPU renderer's exactly (blend modes, strokes, clipped edge tiles, an offset view).
  CI installs lavapipe on Linux and requires an adapter on all three OSes
  (`BRUSHYE_REQUIRE_GPU`).
- **Drawing:**
  - pen (tablet backend from M1) and mouse; the pen's eraser end erases;
  - a live preview while drawing, then one `brush.stroke` command when the pen lifts. The
    preview is incremental and exact (see Engine below);
  - samples are quantized as they arrive, so the preview uses the same values the command
    records;
  - in pressure-only delivery (X11) the mouse is ignored while the pen is near, so a pen
    delivered as the mouse isn't drawn twice. With full delivery (Wayland) the pen also drives
    the UI.
- **Tools:**
  - Pen and Eraser, each with its own brush settings: size, opacity, density, hardness, AA,
    gap (incl. density by gap), stabilization, tip thickness/direction/angle/follow line, ink
    mode, and pressure for size, opacity and density with minimums;
  - colour picker;
  - brush library: load and save `.brshbrush` in a folder (per-user config dir by default).
- **Layers:** new, delete, up and down, select, show/hide, rename, blend mode, opacity (slider
  drags merge into one undo step, as in CSP), lock and lock transparency. All are commands.
- **Files:** New (size, dpi, paper), Open, Save, Save As with optional embedded journal (the
  journal is also written next to the file as `.bcmd`), Export PNG. All writes are crash-safe.
  The window title marks unsaved changes, and New/Open/Quit ask before discarding them.
- **Navigation:**
  - wheel zoom around the pointer; Space+drag or middle-drag to pan;
    Shift+Space+drag to rotate;
  - Ctrl+0 fits, Ctrl+Alt+0 shows 100 %, Ctrl+± zooms; View menu for rotation;
  - shortcuts: Ctrl+Z/Y/Shift+Z, Ctrl+N/O/S/Shift+S, P (pen), E (eraser), [ ] (size).

## Engine changes for live drawing
- **`LiveStroke` (brushye-core):**
  - the dabs on the final part of the path are painted once into the stroke buffer; the few
    tail dabs that can still move are redrawn each update on copies of the chunks they touch;
  - only changed chunks are committed to the preview;
  - property test: the preview equals the finished stroke exactly for random strokes fed in
    random pieces (all ink modes, pressure, thin tips, existing layer content), and final dabs
    never change.
  - Updates take 1–6 ms each on a 2000-sample stroke, even with a 300 px brush.
- **Faster dabs:** the dab loop now looks up its chunk once per chunk instead of per pixel, so
  full strokes render about 2.5× faster.
- **`render::flatten_rect`:** composites any rectangle with exactly the arithmetic of `flatten`
  (golden hashes unchanged).

## Verified here (automated, isolated pen display :78, virtual stylus over XInput2)
- Pressure strokes appear live and are committed. Menu undo/redo and keyboard shortcuts work
  (with focus, which the bare test display doesn't give a window by itself).
- Save As writes the `.brsh` plus `.bcmd`. **`brushye open` on the saved file and `brushye
  replay` of the journal give the same hash**, so the app's commands reproduce the document.
- Bugs found and fixed on the way:
  - a short-circuited `||` left a GPU reset pending, and it wiped the canvas one frame later;
  - re-sending the window title every frame kept the app repainting at ~260 fps while idle.

## Open / not done
- **Real-pen test on Wayland** (the maintainer's setup): the full-delivery path is untested in
  the app.
- **Windows/macOS tablet backends** are still not written (no test hardware; D-015). The app
  runs there with the mouse.
- **File dialogs are a path text field**: native dialogs need a new dependency
  (e.g. `rfd`, D-016 rules). Autosave is not done.
- **Colour:** egui's HSV picker, not a colour wheel. Main/sub colour, the brush-size palette
  and the pressure curve editor come with the brush editor (M7).
- **Zoomed-out display:** at < 100 % without mipmaps, fine detail can shimmer.

## A decision to settle: where the global pen pressure curve applies
D-015 (accepted) item 5 says the global pressure curve is *a journaled setting applied in the
brush engine*. In M4 I instead made `brush.stroke` samples carry pressure **after** the global
curve (noted in B-006 and the M4 report), which contradicts D-015. Nothing depends on it yet,
because there is no global curve setting in the app so far (identity). Options:
- **(a) Follow D-015 as written:** add a journaled `input.pressure_curve` app-state command;
  strokes record raw pressure and the engine applies the curve. Replay needs the setting in the
  journal.
- **(b) Keep the M4 behaviour:** the app applies the curve at capture; strokes record
  calibrated pressure, so each stroke replays on its own. Update D-015 to say so.

I lean to (b): a stroke stays self-contained and calibration stays a property of the input
device. Either way it needs your call before the pressure-adjustment UI (M7) is built.

## A license note to confirm
egui's bundled UI fonts (Ubuntu font, Hack, Noto Emoji, Emoji Icon Font) are under OFL-1.1 and
the Ubuntu Font Licence. They now ship in the app, so cargo-deny flagged them. I allowed those
two licenses for that one crate only, with a note in D-016. Both are standard font licenses that
allow bundling with MIT/Apache software. If you'd rather pick the UI font yourself, that's a
small change.

## Proposed next step
After the pen test and that decision: M7, the brush editor and alpha polish (curve editor,
main/sub colours, brush-size palette, global pressure adjustment, autosave).
