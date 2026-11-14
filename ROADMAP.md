# ROADMAP

Milestones from an empty repo to the alpha, then .sut import, .clip import and beyond.
Each milestone ends with a report (works / doesn't / specs waiting on observation / next step).
Ordering follows CLAUDE.md's priorities: command layer → brush engine → .sut → .clip.

## Phase 1 — Study & planning (this milestone)
- Wine/Xvfb observation harness for CSP 1.13.2 (done; see `docs/observation/wine-harness.md`).
- Manual study, FEATURES.md, ROADMAP.md, first behaviour specs, format study plan.
- DECISIONS.md proposals: crate layout, command format, determinism, undo, native format.
- Repo skeleton + CI on Windows/macOS/Linux.
- **Exit:** maintainer approves or amends the proposals.

## M1 — Tablet input spike (top technical risk) — DONE on Linux (2026-11-13), Windows/macOS deferred
- Report: `docs/reports/M1-tablet-spike.md`. Native Wayland verified with the maintainer's pen;
  X11 verified with the virtual stylus. Windows/macOS backends deferred (no test hardware).
- `brushye-tablet` crate with the uniform pen-event API and both delivery modes (D-015), plus a
  small egui test app that logs the event stream and draws a pressure trace.
- Backends: Wayland (own, same connection, version negotiation), X11/XInput2, Windows Ink
  (+ WinTab option), macOS AppKit. Check: every sample, sub-pixel positions, OS timestamps.
- Check whether eframe has adopted winit 0.31 by then; if so, use its tablet events for
  Wayland and Windows instead.
- X11 tested automatically with the virtual stylus (D-003). The maintainer tests Wayland
  (GNOME), Windows and macOS with a real pen. Decide on automated Wayland testing.
- **Exit:** pressure + tilt + eraser end confirmed on native Wayland and ≥ 1 other OS; a written
  risk report for the rest.

## M2 — Document model + command layer core (headless) — DONE (2026-11-13); golden hash identical on Linux, Windows, macOS (CI run 43488065617)
- Report: `docs/reports/M2-document-and-commands.md`.
- `brushye-core` (model module): document, layer tree (raster + paper), tiles, ids.
- `brushye-command`: text codec (parse/print round-trip, fuzzed), versioning + migration
  framework, executor, queries, session journal.
- Commands: `doc.new`, `layer.new/delete/move/set` (name, visibility, opacity, blend),
  `history.undo/redo`, and the matching queries.
- `brushye-cli`: `run`, `replay`, `query`.
- **Exit:** a journal replays to an identical document hash on all 3 CI OSes.

## M3 — CPU reference compositor — DONE (2026-11-13)
- Report: `docs/reports/M3-cpu-compositor.md`.
- **Before M3 (D-017):** run the remaining blending observations (non-separable sweeps,
  Multiply with alpha, L-003 opacity, L-004 folders/Through, L-005 transparent backdrop,
  L-006 clipping/lock) so the compositor is built against observed behaviour.
- `brushye-core` (color module): all 28 blend modes, opacity, straight-alpha compositing, paper.
- **Exact-match tests** against the observed CSP tables in `specs/behavior/blending/`.
- PNG export (`brushye-format`), `brushye render doc out.png`.
- **Exit:** every *observed* blend spec passes bit-exactly (formulas, or observed tables for
  Overlay/Hard light/Vivid light, D-017). Open cases have stated tolerances, listed in the report.

## M4 — Brush engine v1 (headless) — DONE (docs/reports/M4-brush-engine.md)
- Input processing, stabilization, dynamics + curves, round tip (hardness, AA levels,
  thickness, angle), spacing, density, opacity, ink combine Normal / Compare density / Erase.
- `brush.stroke` command with the encoded sample stream. Strokes are deterministic and replayable.
- Reference-image tests against observed CSP strokes (mouse-driven specs, automated),
  plus pressure specs once the maintainer has recorded them.
- **Exit:** brush specs at status *observed* have passing tests; open differences are listed.

## M5 — Undo/redo + native format — DONE (docs/reports/M5-native-format.md)
- Tile copy-on-write undo, history as commands.
- `.brsh` save/load (D-014): ZIP + JSON + zstd tiles + flattened preview, crash-safe save,
  round-trip and migration tests; `brushye-cli dump`.
- User brushes saved as `.brshbrush` in a brush library (D-014).
- **Exit:** save → load → replay gives the same hash; undo/redo fuzz tests pass.

## M6 — App shell + GPU canvas display — DONE (docs/reports/M6-app-shell.md; maintainer pen test passed 2026-11-13)
- eframe window, wgpu canvas display: CPU-composited tiles uploaded as textures, pan/zoom/rotate
  (D-012: the CPU composites, the GPU only displays).
- Display test on a software adapter in CI: at 100 % zoom the shown pixels equal the CPU tiles.
- GPU compositing (with the D-012 tolerance tests) only later, if profiling shows the need.
- Tablet input wired to `stroke` commands. Live stroke preview.
- Layer palette, colour wheel/sliders, tool property palette, brush size palette.
- **Exit:** the maintainer draws comfortably with pressure on their main OS.

## M7 — Brush editor + alpha polish → **ALPHA**
**Status (2026-11-13): done (D-018 and D-019 implemented); waiting on the maintainer's test.
See docs/reports/M7-brush-editor.md.**
- Brush editor (Sub Tool Detail equivalent) for all alpha brush settings; create, duplicate
  and save brushes; a set of original default brushes.
- Every alpha feature reachable headless via commands; the CLI documents every command.
- Release builds for Windows/macOS/Linux from CI.
- **Alpha checklist** (from CLAUDE.md): canvas pan/zoom/rotate + pressure ✓, raster layers
  with blend modes + opacity ✓, undo/redo on commands ✓, brush engine + editor + own brush
  format ✓, .brsh + PNG export ✓, everything headless ✓.

## M8 — .sut import (first post-alpha milestone)
**Status: DONE (2026-11-14, maintainer).** Report: `docs/reports/M8-sut-import.md`. All 98
samples import, and every setting imports exactly (cross-check B-012). Hard and soft brushes
match CSP. Still open and tracked: the anti-aliased thin-tip rule (B-014) and soft tips at
size 8 (B-015).
- Format study from maintainer-made samples (`specs/formats/study-plan.md`).
- Map .sut settings to Brushye brush settings; report anything that can't be mapped.
- Import only at runtime from user-supplied files; nothing bundled.
- **Exit:** every sample in the .sut sample set imports, and strokes made with imported brushes
  match CSP strokes from the same brush within brush-test tolerance.

## M9 — MCP server + scripting polish
**Status (2026-11-14): in progress.** Done: the headless stdio server (`docs/mcp.md`), script
examples (`examples/scripts/`), and the journal viewer (`brushye journal show`, Window > Journal).
Open: attaching to the running app (D-022, PROPOSAL).
- `brushye-mcp` exposing commands and queries; script examples; journal viewer.

## M10 — Post-alpha painting features
- Folders + Through mode, clipping, masks, locks, selections, fill, gradient, transform,
  colour mixing (Blend/Running/Smear), texture, spray, dual brush, starting/ending,
  watercolor edge, PSD import/export, workspace/shortcut customization.

## M11 — .clip import
- Container/structure study from maintainer samples; raster layers, folders, blend modes,
  opacity, masks first; vector/text/3D later (or flattened fallback where CSP stores a
  rendered cache).
- **Exit:** the sample .clip set opens with matching flattened output.

## Beyond
- .clip save, vector layers, text/balloons, frame borders, tones, comics page management,
  animation, rulers/perspective, tonal correction layers, colour management/CMYK,
  Auto Actions on the command journal, plugin/script hosting.
