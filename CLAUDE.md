# Brushye — CLAUDE.md

## What we're building
Brushye is a free, open-source painting, illustration and comics app for
Windows, macOS and Linux. It follows the workflow and panel organization of
Clip Studio Paint (CSP) 1.13.2, so CSP users feel at home. Its visual design,
icons, text and branding are all original.

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

Match behavior and workflow, not appearance. Don't speculate about how CSP
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
  (now: brushye-core, brushye-command, brushye-cli, brushye-tablet).
- `docs/command-format.md`: the exact command text grammar (D-011).
- `docs/reports/`: milestone reports.

## Current status
Phase 1 (study & planning) is done, and all Phase 1 proposals (D-010 to D-017: crate layout,
command format, determinism, undo/redo, .brsh format, tablet input, dependencies, blend-mode
strategy) are ACCEPTED. M1 (tablet spike) is done on Linux; Windows/macOS backends are deferred
(no test hardware). Pre-M3 blending observations are done (L-003..L-007). M2 (document model +
command layer) is done locally. Next: M3 (CPU compositor). New architectural decisions still
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
