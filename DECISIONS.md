# DECISIONS

Each entry: date, decision, alternatives considered, reason.
Entries marked **PROPOSAL** are awaiting maintainer approval. No application code
gets written against a proposal until it's marked **ACCEPTED**.

---

## D-001 Observation prefix is a reflink clone of the maintainer's CSP prefix — ACCEPTED (operational)
- **Date:** 2026-11-11
- **Decision:** Automated observation runs CSP 1.13.2 from `~/.wine-csp-observe`, a btrfs reflink
  copy (`cp -a --reflink`) of the maintainer's own `~/.wine-csp`. It runs in Xvfb `:77` inside a
  1920×1080 Wine virtual desktop (`wine explorer /desktop=csp,1920x1080`) and uses the cspenguin
  Wine 11.4 build already on the machine.
- **Alternatives:** Drive the maintainer's real prefix (rejected: observation changes tool settings,
  workspace and recent files in their daily install). Fresh install from the CELSYS installer
  (no installer on the machine; licence activation would be needed again).
- **Reason:** Isolation from the maintainer's daily settings at zero disk cost. Copying is a
  mechanical file copy; nothing inside the installation is read, as the clean-room rules require.
  The prefix's `drive_c/users/<user>/Documents` and the `z:` drive still map to the real home
  directory, so all exports go to an explicit scratch path.
- **Reversible:** `rm -rf ~/.wine-csp-observe`.

## D-002 Exported PNG is a bit-exact pixel oracle — ACCEPTED (verified)
- **Date:** 2026-11-11
- **Decision:** Observations read exact values from File > Export (Single Layer) > .png with
  Expression color = "RGB color" and preview off. CSP embeds an sRGB2014 ICC profile, but the RGB
  values round-trip bit-exactly (open PNG → export → compare: identical on all 131,072 pixels).
  The ICC chunk is stripped before observation data is committed.
- **Alternatives:** Screenshots (forbidden for pixel values by CLAUDE.md, and colour-managed/zoomed).

## D-003 Virtual tablet pressure under Wine — ACCEPTED (not possible under Xvfb; works since 2026-11-13 via Xorg + libinput, see update)
- **Date:** 2026-11-11
- **Finding:** Xvfb exposes only core and XTEST pointers with relative X/Y valuators, with no
  pressure or tilt axis, so Wine's wintab has nothing to report. A uinput virtual stylus wouldn't
  help: Xvfb has no evdev/libinput input path, and the `/dev/input/event*` nodes uinput creates are
  `root:input 0660`.
- **Not done on purpose:** creating a uinput tablet device. The maintainer's live KDE session
  would hot-plug it as a real tablet.
- **Paths that could work (need maintainer action, see ROADMAP M1):**
  1. Install `xorg-x11-server-Xorg`, `xorg-x11-drv-dummy` and `xorg-x11-drv-libinput`, plus a
     udev rule that gives the user access only to a uinput device with a fixed name. Then run
     Xorg with the dummy video driver, `AutoAddDevices off` and an InputClass matching only that
     device. Wine's wintab then sees a real XInput2 stylus with pressure and tilt valuators.
  2. A headless Wayland compositor (e.g. weston/cage headless) that forwards a virtual
     `zwp_tablet_v2` to Xwayland. More moving parts.
- **Consequence:** pressure/tilt specs are marked `needs: maintainer + real pen` until one of the
  paths above is proven to work.
- **Update 2026-11-13: path 1 WORKS. Pressure observation is automated.**
  - Maintainer setup: Xorg + dummy/libinput drivers, and the udev rule
    `70-brushye-virtual-pen.rules`. It gives the user ownership of devices named
    "Brushye Virtual Pen" and assigns them to `ID_SEAT=seat-brushye`.
  - `tools/observe/start_pen_display.sh` starts three things: `vpen.py`, a pure-Python uinput
    stylus with X/Y at 0.1 px resolution, pressure 0–8191 and tilt ±64; a rootless
    `Xorg -seat seat-brushye -dumbSched` with dummy video and the libinput driver, which turns
    the pen into an XInput STYLUS device "Brushye Virtual Pen Pen (0)"; and CSP under Wine.
  - **Isolation check:** gnome-shell does not open the pen's event node (its fd table was
    checked). The observation Xorg's udev hotplug only sees seat-brushye devices; an InputClass
    ignores everything else as a second safeguard. The log shows only the pen being added.
  - **Verified:** a 5 %→100 % pressure ramp with CSP's default G-pen (size 10) tapers from about
    0.3 px to about 7.8 px wide, with no artifacts. Two runs of the same pen script exported
    **bit-identical** PNGs (1600×1200), so pressure specs are reproducible.
  - **Pitfalls found (all handled by the scripts):**
    1. Wine's wintab (dlls/winex11.drv/wintab.c, Wine 11.4) reads `axis_data[0..4]` of every
       motion event as X, Y, pressure, tiltX, tiltY. It assumes all axes are present from
       axis 0. The evdev Xorg driver sends only the axes that changed, so a pressure-only update
       was read as position (0,0) and drew a line to the screen corner. The libinput driver
       always sends all axes. (Button events without axes are fine; Wine reuses the last packet.)
    2. `LIBINPUT_IGNORE_DEVICE` in the udev rule also blocks Xorg's libinput driver. The seat
       assignment replaces it.
    3. A device declared in xorg.conf can't spawn libinput tool subdevices: the X server
       silently rejects option lists that carry both `Identifier` and `Name`. So the pen is
       hotplugged through udev instead.
    4. The stylus subdevice appears on the first proximity-in, and Wine enumerates tablets when
       wintab loads. The start script creates the stylus before launching CSP.
    5. The first XTEST click after a pen stroke is swallowed. Move the mouse before clicking.
    6. CSP's export settings only persist if CSP exits cleanly; after `wineserver -k` the PNG
       dialog reverts to preview on and "Auto detect" colour.
  - The `inputtest` Xorg driver was ruled out: pressure sits at axis index 4, after two scroll
    axes, but Wine reads axes by position.
  - Still to verify: whether tilt reaches CSP (the axes reach X; B-010 needs a tilt-sensitive brush test).

---

## D-010 Workspace / crate layout — ACCEPTED (2026-11-13)
- **Date:** proposed 2026-11-11; accepted 2026-11-13 by the maintainer, in the lighter form below.
- **Decision:** a cargo workspace under `crates/` with five crates to start. Each one exists
  because it enforces one of CLAUDE.md's rules at compile time. Further splits are added only
  when a milestone needs them.

```
crates/
  brushye-core      document model (Document, LayerTree, layers, tiles, selection, ids),
                    colour + blend-mode math, brush engine, CPU reference renderer.
                    No UI/GPU/OS deps; everything here is headless-testable.
                    Internal modules: model, color, brush, render.
  brushye-command   command definitions, text codec (parse/print), versioning + migrations,
                    executor, undo/redo, session journal, queries.
                    The single gate every client uses to change state.
  brushye-format    .brsh native format (read/write), PNG import/export. Later: PSD, ORA.
  brushye-app       egui/eframe UI, wgpu canvas, tablet input.
                    The only crate allowed to touch UI, GPU or OS input APIs.
  brushye-cli       headless binary: run scripts, replay journals, queries, render to PNG.
```

  **Dependency direction (one-way):** `core ← command ← format ← {cli, app}`.
  `brushye-command` also depends on `brushye-core`; `brushye-format` may read core types.
  Nothing depends on `brushye-app` or `brushye-cli`.

  **Rules this enforces:**
  - Only `brushye-app` may depend on egui, eframe, wgpu, winit or platform input crates.
  - Clients (`app`, `cli`, later `mcp`) change state only through `brushye-command`.
    Mutating document APIs in `brushye-core` are `pub(crate)` or reachable only via command
    execution, so a client can't bypass the command layer.

  **Added later, when their milestone starts (no new decision needed, only a DECISIONS note):**
  - `brushye-tablet`: the M1 tablet spike starts as its own small crate (with a test app); its
    proven parts may later move into `brushye-app` or stay a crate (D-015).
  - `brushye-mcp` (M9), `brushye-sut` (M8), `brushye-clip` (M11). Keeping the importers
    separate makes the code that touches CSP formats easy to review against the clean-room rules.
  - `brushye-gpu`, if the wgpu renderer outgrows `brushye-app`.
  - Splitting `color`, `brush` or `render` out of `brushye-core` once it gets large or slow
    to compile. This is mechanical, because the module dependencies are already one-way.
- **Alternatives considered:** (a) the original 13-crate split (core, color, brush, render,
  command, format, sut, clip, gpu, tablet, app, cli, mcp, plus xtask): the same guarantees,
  but structure for code that doesn't exist yet. (b) One big crate: nothing stops UI code
  creeping into the core.
- **Reason:** CLAUDE.md requires a headless, testable core and a single command layer that every
  client goes through. Five crates make both compile-time guarantees with the least
  upfront structure.

## D-011 Command format — ACCEPTED (2026-11-13)
- **Date:** proposed 2026-11-11; accepted 2026-11-13 by the maintainer, after comparing it with
  photocraft's command layer (see "Alternatives"). The custom text syntax is kept, and four
  ideas from photocraft are added (registry, queries as registry entries, typed params,
  seeded strokes). **No photocraft code is reused**, only the design ideas.
- **Decision:** a line-oriented text format, one command per line, UTF-8.

```
#brushye-journal v1 app=0.1.0
@1 doc.new w=1600 h=1200 dpi=350 paper=#ffffff
@2 layer.new id=L2 kind=raster name="Ink" above=L1
@3 layer.set L2 blend=multiply opacity=0.80
@4 tool.set brush.size=12.5 brush.aa=middle ink.mode=normal color=#1a1a1aff
@5 stroke L2 tool=@current seed=91827 samples=b64:AAEC…
@6 history.undo
```

  - **One command registry.** Every command is registered once (in `brushye-command`, D-010)
    with: its id, its current version, a typed params struct, a `journal` flag, and an
    `enabled(state)` check. The UI, CLI, scripts and the MCP server all dispatch through the
    same registry and the same `execute(command)` entry point. Nothing changes document or
    application state any other way.
  - **Typed params.** Each command's arguments are a Rust struct. The text codec, the JSON form
    and a machine-readable schema (for the MCP server and generated UI) are all derived from
    that struct, so they can't drift apart. Unknown keys, missing required keys, and values out
    of range are errors, never silently ignored.
  - **Grammar:** `@<seq> <namespace>.<verb>[/<cmdver>] <positional>* <key>=<value>*`.
    Values are typed: integers, decimals (canonical shortest round-trip form, `f32`/`f64` as
    declared per field), `#rrggbb[aa]`, quoted strings with `\` escapes, ids (`L2`, `S5`),
    enums (bare words), and `b64:` blobs. The full grammar (escaping, number formatting,
    whitespace, comments starting with `#`) is written down as a spec before the parser is
    implemented. Print → parse → print must be byte-identical; this is fuzz-tested.
  - **Command ids** are Brushye's own neutral names (`layer.set`, `stroke`, `history.undo`),
    not copies of CSP menu names, so CSP's wording doesn't leak into the API. UI labels map to
    commands through the registry.
  - **Versioning:** each command type has an integer version (`layer.set/2`). A missing
    `/n` means v1. The journal header carries the format version and app version. On load,
    old commands go through a chain of pure migration functions (`vN → vN+1`), each with
    golden tests. Printing always emits the current version.
  - **Determinism:** commands carry every input that affects output, including the random
    seed, the full tool state snapshot (or a reference to a prior `tool.set` in the same
    journal), and ids. Ids are assigned by the command, never by the executor, so replay can't
    diverge. The journal keeps explicit ids; turning ids into layer names is only done when a
    journal excerpt is exported as a reusable macro (not when journaling).
  - **Seeded strokes:** the same `stroke` params with the same `seed` produce the same pixels.
    A live stroke previews incrementally and commits as exactly that one command.
  - **Strokes:** `stroke` carries the full raw input stream as a `b64:` blob, so a journal line
    stays one line. The blob holds a little-endian header (sample count, field mask, units)
    followed by delta-encoded, zigzag + LEB128 varint samples:
    `x,y` in 1/256 px (canvas space, i32), `pressure` u16 (0..65535), `tilt_x,tilt_y` i16
    (1/100 °), `rotation` u16 (1/100 °), `t` in µs since stroke start (u32), and flags (eraser,
    button). Raw device samples are stored **before** stabilization, so changing the stabilizer
    replays differently on purpose and the journal stays the truth. Fields that aren't present
    are omitted through the field mask (a mouse stroke has no pressure). A readable
    alternative `samples=[x,y,p,t;…]` is accepted for hand-written scripts and tests;
    journals always write the blob.
  - **Queries** are registry entries with `journal = false`. They are written with a `?` prefix
    (`?layer.list`, `?pixel L2 x=10 y=20`, `?tool.get brush.size`) and return JSON. They never
    change state and never go into the journal.
  - **Equivalent JSON form:** every command maps one-to-one to a JSON object
    (`{"cmd":"layer.set","v":1,"args":{…}}`) for the MCP server and scripting hosts. It is
    generated from the same typed params. The text form is canonical in journals.
- **Alternatives considered:**
  (a) **photocraft's model** (the maintainer's Rust Photoshop rebuild, studied 2026-11-13):
  a registry of ~700 dotted command ids, JSON params passed as untyped `serde_json::Value`
  and parsed by hand per command, strokes as JSON arrays `[x,y,pressure,tiltX,tiltY,rotation,
  time_ms,wheel]` plus a seed, an in-memory journal of `(id, params)`, and only a file-level
  version on saved Actions. Rejected as the canonical format: JSON is noisy to read and diff,
  JSON stroke arrays are roughly 3–5× larger than the blob (tens of MB per long session), and
  there is no per-command versioning. Its registry, queries-as-commands, seeded strokes and
  the goal of schema-checked params are adopted above.
  (b) A binary journal (e.g. postcard): compact, but not a "string" and painful to diff or
  hand-write. (c) Lua/JS script as the journal: Turing-complete scripts are hard to migrate
  and can't be inspected statically.
- **Reason:** readable, diffable, greppable journals; strict, typed parsing; compact strokes (a
  typical 2-second stroke at 200 Hz is roughly 3–5 KB of base64); migrations make old journals
  replayable forever; and the JSON form keeps MCP and scripting easy.

## D-012 Determinism strategy (CPU reference vs GPU tolerance) — ACCEPTED (2026-11-13)
- **Date:** proposed 2026-11-11; accepted 2026-11-13 by the maintainer, with the change that
  the alpha composites on the CPU only (point 4).
- **Decision:**
  1. **Pixel storage:** layers are stored as 8-bit straight (non-premultiplied) RGBA in
     256×256 tiles. Compositing converts alpha to 15-bit fixed point (0..32768), the
     precision CSP observably uses (L-002). This matches what CSP 1.13.2 exposes: an 8-bit
     "Color" canvas whose compositing results are reproduced exactly by integer formulas (L-001).
     The alpha is **8-bit only**. A 16-bit document option may come post-alpha (soft,
     low-density brushes can band at 8 bits, as they do in CSP).
     Strokes accumulate in a stroke buffer and are combined into the layer once, using the ink
     combine mode. The stroke buffer's precision (8-bit, u16 or f32) is **not decided here**: it
     is set by the brush specs once CSP's within-stroke accumulation is measured (B-003/B-005
     show per-dab density building up, which is where it matters).
  2. **The CPU reference renderer (the `render` module of `brushye-core`, D-010) defines
     correct output.** Compositing and blend modes use integer arithmetic only, so they're
     bit-identical on every platform. Brush geometry uses `f32` with these rules: no
     `mul_add`/FMA, no platform `libm` (use the pure-Rust `libm` crate), no SIMD paths whose
     results differ from scalar, no fast-math, a fixed tile iteration order, and per-tile
     parallelism only where tiles are independent. Random numbers come from a seeded, portable
     PRNG (e.g. PCG32) whose seed is stored in the command (D-011).
  3. **Cross-platform test:** CI renders the same journals on Windows, macOS and Linux and
     compares hashes of the CPU output. Any difference fails the build. This is the safety net
     for the `f32` rules in point 2.
  4. **Alpha rendering: CPU composites, GPU only displays.** The CPU composites the tiles a
     command dirtied (incremental, per 256² tile). The GPU (wgpu, in `brushye-app`) uploads
     composited tiles as textures and draws the canvas with pan, zoom and rotate. There is one
     renderer, so the screen shows exactly what gets saved, and replay determinism needs no
     GPU tolerance. Display test: at 100 % zoom, unrotated, the displayed pixels equal the CPU
     tiles exactly (read back from the GPU in a software-adapter CI test).
  5. **GPU compositing (later, only if profiling demands it** — e.g. very large canvases or
     many layers): it is then tested against the CPU reference on the same journals.
     Tolerances, to be confirmed when it's built: compositing ≤ 1/255 per channel on ≥ 99.9 %
     of pixels and never > 2/255; brush dabs ≤ 2/255 on ≥ 99 % and never > 4/255. Enforced in
     CI with a software adapter (lavapipe/WARP), plus a manual run on real GPUs before release.
     Saving and export **always** use the CPU result, so files never depend on the GPU.
  6. **CSP-matching tests:** observed CSP outputs (exported PNGs) are stored under `specs/`
     and the tests use them directly (e.g. the 256×256 per-mode tables).
- **Alternatives considered:** (a) GPU as the source of truth, with no CPU renderer: fast, but
  output differs across vendors and drivers, so replay can't be verified. (b) GPU compositing
  from the start, within tolerance (the 2026-11-11 proposal): two renderers to keep in step
  before we know the CPU path is too slow. (c) f32 linear-light compositing everywhere: doesn't
  match CSP, which observably blends 8-bit sRGB values with integer rounding. (d) Fixed-point
  brush geometry: safest for determinism but slower to write; f32 with strict rules and the CI
  hash check is enough. (e) 16-bit storage from the start: CSP 1.13.2 has no such canvas mode
  to match.
- **Reason:** CLAUDE.md requires replay to reproduce the document, with a CPU reference and a
  stated GPU tolerance. Integer compositing makes the CSP-matching tests exact, and a single
  renderer for the alpha removes a whole class of mismatch bugs.

## D-013 Undo/redo design — PROPOSAL (revised 2026-11-13 after spec H-001)
- **Date:** proposed 2026-11-11; revised 2026-11-13 with the observed CSP behaviour in
  `specs/behavior/history/H-001-undo-granularity.md`.
- **Proposal:**
  1. **Mechanism: immutable snapshots with shared tiles** (replaces the earlier "inverse delta"
     idea). The document is immutable. Each undoable command produces a new document version
     in which unchanged tiles and properties are shared with the previous version (`Arc`
     copy-on-write, 256² tiles, D-012). History is a list of versions; undo and redo move a
     pointer. A version costs only the tiles it changed. No command needs its own "undo half",
     so undo is correct by construction, and the UI, autosave and export can read a stable
     version without locks.
  2. **What is an undo step (matches CSP, H-001):**
     - *Undoable (document state):* strokes, layer create/delete/move, layer properties
       (opacity, blend mode, name, locks, …), selection changes including deselect, and
       transforms and other pixel edits.
     - *Not undoable, but journaled (application/view state):* tool and brush settings, colours,
       zoom/pan/rotate/flip of the view, which layer is selected, and **layer visibility**
       (CSP doesn't record visibility in history).
     - The registry (D-011) carries this as a per-command flag (`undoable`), next to `journal`.
  3. **Grouping and merging (matches CSP, H-001):**
     - A continuous gesture (a stroke, a slider drag, a transform session) commits as **one
       command and one undo step**. Live previews in between are not commands.
     - **Consecutive changes of the same kind merge into one step** (opacity drag then typed
       opacity, lock then unlock). Undoing a merged step restores the state from before the
       whole run.
     - **Optional time merge:** a preference "undo merge interval", 0–2000 ms, default 0. When
       it's above 0, consecutive actions within the interval merge into one step.
     - **Merges are recorded explicitly in the journal**, so replay never depends on timing. A
       merged command is written `@n+ …` (the `+` means "joins the previous undo step"), and
       replay applies exactly that grouping.
     - Commands whose result is empty (e.g. a stroke fully outside the selection) still create
       a step, as in CSP.
  4. **Linear history:** after an undo, a new undoable command discards the redo branch from
     history. The journal still has everything, including `history.undo` / `history.redo` lines,
     so replay reproduces the exact history state (and which entries are redoable).
  5. **Limit:** a step count, configurable **1–200** like CSP's "Undo levels", default 200 until
     CSP's factory default is confirmed. Plus a memory cap as a backstop for very large canvases
     (oldest steps dropped first). The journal is never truncated by the history limit.
- **Alternatives considered:** (a) inverse deltas per command (the 2026-11-11 proposal): the same
  memory cost, but every command needs correct undo code, and bugs hide there. (b) Recompute by
  replaying the journal from a checkpoint: simple, but slow for long sessions. (c) Implicit,
  time-based merging decided again on replay: replay would depend on timing, so merges are
  recorded instead.
- **Reason:** matches CSP's observed undo behaviour, keeps replay exact (D-011/D-012), and
  structural sharing gives correct undo with no per-command undo code.

## D-014 Native format `.brsh` — PROPOSAL
- **Proposal:** a ZIP container (stored/deflate):
  `manifest.json` (format version, app version), `document.json` (layer tree, properties,
  ids), `tiles/<layer>/<tx>_<ty>.png` for non-empty tiles only, `thumbnail.png`, an optional
  `journal.bcmd`, and `brushes/*.brshbrush` (user brushes are a JSON brush definition plus
  tip/texture PNGs). The format is versioned and has forward migration.
- **Alternatives:** an SQLite container (good for big docs and incremental save; revisit
  after the alpha), or a custom binary format.
- **Reason:** easy to inspect, robust tooling everywhere, and the same container style as
  ORA, which makes ORA export trivial later.

## D-015 Tablet input approach — PROPOSAL
- **Proposal:** a `brushye-tablet` crate that produces a uniform `InputSample` stream.
  First check whether current winit pen/pointer events deliver pressure, tilt and the eraser
  end on all three OSes. Where they don't, add native backends: Windows Ink (`WM_POINTER`)
  with Wintab as an option, macOS `NSEvent` tablet events, and Linux XInput2 (X11) plus
  `zwp_tablet_v2` (Wayland). This is validated with a standalone spike app in **M1, before
  the canvas work**.
- **Reason:** CLAUDE.md names tablet input as the top technical risk.

## D-016 Proposed dependencies (each still needs approval when introduced)
| Crate | Use | Why |
|---|---|---|
| `eframe`/`egui`, `wgpu` | UI, canvas | Mandated by CLAUDE.md |
| `serde`, `serde_json` | document.json, query results, JSON command form | De facto standard; derives avoid hand-written codecs |
| `png` | PNG I/O | Pure Rust, gives exact control over chunks (needed to strip/write ICC deterministically) |
| `zip` | .brsh container | Pure Rust, mature |
| `base64` | stroke blobs | Small and standard |
| `libm` | deterministic transcendental functions | Platform libm differs between OSes (D-012) |
| `rand_pcg` (or a hand-written PCG32) | seeded jitter | Portable and reproducible |
| `thiserror` | error types in library crates | Removes boilerplate, no runtime cost |
| `rmcp` | MCP server | Official Rust MCP SDK |
| `insta` (dev) | golden tests for codec/migrations | Snapshot review workflow |

## D-017 Blend-mode implementation strategy — PROPOSAL
- **Proposal:** implement each separable mode as the integer formula that reproduces the
  observed CSP table exactly where one is known (19 of 22 separable modes so far, plus the
  15-bit source-alpha step for 27 of 28 modes; see specs L-001/L-002). Where no
  closed form is known yet (Overlay/Hard light upper half, Vivid light), ship the observed
  256×256 table (64 KiB per mode) as the implementation, with a test that asserts equality,
  and keep looking for the formula. Non-separable modes (Hue, Saturation, Color,
  Brightness, Darker/Lighter color) start from the W3C formulas with .30/.59/.11 weights
  (closest match, ≤ 8/255 off) until further observation pins them down.
- **Reason:** exact CSP parity now, without guessing internals.
