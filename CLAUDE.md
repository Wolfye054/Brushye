# Brushye — CLAUDE.md

## What we're building
Brushye is a free, open-source painting, illustration and comics app for
Windows, macOS and Linux. It follows the workflow and panel organization of
Clip Studio Paint (CSP) 1.13.2, so CSP users feel at home. Icons (Phosphor),
text and branding are original.

**HUGE PRIORITY (maintainer, 2026-11-13): the layout and visuals must match
CSP 1.13.2** — every menu, palette, dock position, panel arrangement, control
order, size and look, as observed in specs `specs/behavior/ui/`. The only
deliberate difference is the icons: Brushye uses Phosphor icons (MIT), never
CSP's, to avoid copyright problems. Never copy CSP's icon images or UI
artwork; reproduce layout and look from observation.

Priorities, in order:
1. A complete command layer: everything scriptable and replayable.
2. A brush engine that matches CSP's behavior and feel.
3. Import of CSP materials the user owns, starting with sub tools/brushes
   (.sut) right after the alpha.
4. Opening .clip files. Saving to .clip is a later goal.

License: MIT OR Apache-2.0 (dual-licensed)

## Clean-room rules (non-negotiable)
Allowed sources:
- The public CSP user manual
- Observations of CSP 1.13.2's behavior, made by the maintainer or by
  Claude under the automated observation rules below
- Published specs (PSD, ORA, ICC/color management, image formats, etc.)
- General graphics and software knowledge

Not allowed:
- Decompiled or disassembled CSP code, or leaked material
- Copying CSP's icons, UI artwork, text, or default brushes and materials
- Extracting, converting or bundling anything from a CSP installation.
  Import code runs only on files the user supplies at runtime.
- Test fixtures for CSP formats other than files the maintainer created in
  CSP, containing their own work
  (files Claude makes by automated observation in the maintainer's CSP, from new
  custom sub tools or documents and never from CSP's defaults or materials, count as the
  maintainer's; DECISIONS D-020)

Match behavior, workflow, layout and visuals (see the priority above; icons
excepted). Don't speculate about how CSP
works internally. If behavior is unclear, write a behavior spec and
implement from it.

### Specs
- `specs/behavior/<area>/<name>.md`: test steps, observations, expected
  results, open questions, CSP version, date, observation method.
  Status: draft → observed → implemented.
- `specs/formats/<format>.md`: findings about .clip, .sut and others, each
  tied to the sample file it came from.
- Observations are never invented. A spec stays a draft until someone
  (the maintainer, or Claude via automated observation) has actually run
  it in CSP and recorded the results.

### Automated observation (CSP under Wine)
Claude may run CSP 1.13.2 under Wine in an isolated display (Xvfb/Xephyr)
and drive it through its UI to make observations.
- The only allowed interfaces are the UI, screenshots, and files CSP
  exports. Use exported images for exact pixel values, not screenshots.
- Forbidden: Wine debug/relay tracing, debuggers, memory inspection, API
  hooking, and reading CSP's installed files, settings or databases.
- Every observation records its method (automated-Wine or manual-native)
  and the exact steps used. Results that look odd under Wine get flagged
  for checking on native Windows.
- Pressure/tilt specs need the maintainer with a real pen, unless virtual
  tablet input has been proven to work under Wine.
- Respect CSP's EULA for automated use.

## Tech
- Rust (stable), cargo workspace. Keep the document model, brush engine,
  command layer and file formats in crates with no UI dependencies,
  testable headless.
- UI: egui/eframe on wgpu. Render the canvas directly with wgpu.
- Platforms: Windows, macOS, Linux.
- Cross-platform tablet input (pressure, tilt, eraser end) is the top
  technical risk. Validate it early.
- Each new dependency needs a stated reason in DECISIONS.md.

## Command architecture (core requirement)
Every action that changes document or application state is a command: an
isolated, serializable, versioned string that reproduces the action when
replayed. The UI, scripts and the MCP server are all clients of the same
command layer. Nothing changes state outside it.
- Undo/redo and a session journal are built on commands.
- Brush strokes are commands carrying the full input stream (position,
  pressure, tilt, timestamps).
- Every state can be inspected through queries (layers, selection, pixels,
  tool settings), not only changed.
- Brushye runs headless (no window) for scripting and tests.
- An MCP server crate exposes commands and queries to AI clients.
- Replaying a journal must reproduce the document. A CPU reference
  renderer defines correct output; GPU output is tested against it within
  a stated tolerance.

## Working agreement
- Claude drives the work and reports back at each milestone.
- Ask before big architectural changes: crate layout, document model,
  command format, rendering approach, undo design, native file format.
- Keep this file current. Log decisions in DECISIONS.md (date, decision,
  alternatives considered, reason).
- Tests: document model, blending math (exact values from observed specs),
  command replay, and brush output (reference-image tests).
- Small, focused commits. CI builds and tests on Windows, macOS and Linux.
- A milestone report covers what works, what doesn't, specs waiting on
  observation, and the proposed next step.

## Repo map
- `FEATURES.md`: CSP 1.13.2 feature inventory, tagged alpha / post-alpha / later.
- `ROADMAP.md`: milestones to the alpha and beyond.
- `DECISIONS.md`: decision log; PROPOSAL entries need maintainer approval.
- `specs/`: behaviour specs (`behavior/<area>/`) and format notes (`formats/`).
  Observed data lives next to its spec (`specs/behavior/<area>/data/`).
- `docs/observation/wine-harness.md`: how to run CSP 1.13.2 under Wine/Xvfb.
- `docs/research/`: manual study notes (paraphrased, with source URLs).
- `tools/observe/`: UI driver and analysis scripts for automated observation.
- `crates/`: Rust workspace. Layout per DECISIONS D-010 (accepted): brushye-core, brushye-command,
  brushye-format, brushye-app, brushye-cli, plus brushye-tablet (M1). Only brushye-app and
  brushye-tablet may use UI/GPU/OS-input crates. Crates are created as their milestone starts
  (now: brushye-core, brushye-command, brushye-format, brushye-cli, brushye-tablet, brushye-app,
  brushye-sut, brushye-mcp;
  run the app with `cargo app`).
- `docs/command-format.md`: the exact command text grammar (D-011).
- `docs/native-format.md`: the exact `.brsh` / `.brshbrush` layout (D-014).
- `docs/mcp.md`: the MCP server's tools and setup (M9).
- `examples/scripts/`: example command scripts (a test keeps them runnable).
- `docs/reports/`: milestone reports.

## Current status
Phase 1 (study & planning) is done, and all Phase 1 proposals (D-010 to D-017: crate layout,
command format, determinism, undo/redo, .brsh format, tablet input, dependencies, blend-mode
strategy) are ACCEPTED. M1 (tablet spike) is done on Linux; Windows/macOS backends are deferred
(no test hardware). Pre-M3 blending observations are done (L-003..L-007). M2 (document model +
command layer) is done, verified on all three CI platforms. M3 (CPU compositor, exact against
the observed blend data) is done. M4 (brush engine) is done: path, spacing, tips,
ink modes and pressure dynamics, built against two observation runs (open items in the M4
report). M5 (undo/redo fuzzing, the .brsh format and .brshbrush user
brushes) is done. M6 (app shell, GPU canvas display, live strokes, CSP 1.13.2 layout and look) is done;
the maintainer's pen test passed. M7 (brush editor + alpha polish) is done, including D-018
(tool state as commands) and D-019 (native file dialogs); the maintainer tested it and
v0.1.0-alpha is tagged. M8 (.sut import, crate brushye-sut) is done; open engine item: anti-aliased
thin tips (B-014). D-021: before 1.0, brush fidelity fixes may change journal replay. M9 (MCP server) is in
progress: the headless stdio server `brushye-mcp` and script examples are in; the journal viewer
and attaching to the running app are open. New architectural decisions still
need maintainer approval first (see Working agreement). Never copy code from the maintainer's
other projects (photocraft, clipcraft); design ideas only.

## Alpha goal
Basic drawing with custom brushes:
- Canvas with pan/zoom/rotate and tablet pressure
- Raster layers with blend modes and opacity
- Undo/redo, built on the command layer
- Brush engine with a brush editor; user-created brushes saved in
  Brushye's own format
- Save and load in a native format (.brsh), plus PNG export
- Every alpha feature reachable through commands, headless

.sut import comes right after the alpha.
