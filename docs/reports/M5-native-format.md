# M5 report: undo/redo hardening and the native format (2026-11-13)

## What works
- **`.brsh` documents** (`brushye-format::brsh`, D-014; layout in `docs/native-format.md`):
  - ZIP with a manifest, the document JSON, one zstd tile per non-empty 256² tile, a full-size
    preview PNG and a 256 px thumbnail;
  - the session journal is embedded only on request;
  - output is deterministic, so a document always saves to the same bytes;
  - crash-safe saves (temporary file, sync, rename);
  - a tile cache, so a save after a small stroke recompresses one tile instead of all of them;
  - a newer format version is refused with a clear message, and older versions go through a
    migration chain (empty for now; the engine is tested with a fake chain);
  - unknown content is an error when loading strictly (tests) and a warning otherwise.
- **Journals that continue an opened document:** a session started from a file records the
  file's content hash as `base=` in the journal header. Replay refuses any other base, and
  undo stops at the opened document.
- **User brushes** (`.brshbrush`): a ZIP with `brush.json`, holding the `brush.stroke` settings
  without layer, colour and samples, plus room for tip/texture images. Validated by a shared
  `BrushPreset` type, which `brush.stroke` now also uses for its settings. A brush library is
  a folder of these files.
- **CLI:**
  - `run --open FILE.brsh --brsh OUT [--embed-journal]`;
  - `open FILE.brsh --query … --png …`;
  - `replay JOURNAL --base FILE.brsh`;
  - `render` on a journal or a `.brsh`;
  - `dump FILE.brsh [--layers DIR]` (manifest, layers, tile statistics, entries, layer PNGs);
  - `brush new / show / list`.

## Tests (M5 exit criteria)
- **Save → load → replay gives the same hash:**
  - `.brsh` round trip of a document with every layer property, strokes and a hidden layer;
  - the embedded journal replays to the same hash;
  - 24 random documents per run (property test);
  - CLI end to end: run → save → open/render/dump → continue → replay on the base, with
    byte-identical PNGs from the file and from the journal.
- **Undo/redo fuzz tests pass:** random sequences of up to 60 commands (layers, properties,
  locks, fills, pixels, strokes in every ink mode, visibility, selection, undo, redo) checked
  against a model of the history:
  - a failed command changes nothing and is not journaled;
  - undo and redo return exactly the state recorded at that history position;
  - visibility applies across history without adding a step;
  - undoing and redoing everything round-trips;
  - the journal replays to the same document, history and active layer.

  48 cases per CI run; a 5,000-case run passed locally (`BRUSHYE_FUZZ_CASES=5000`).
- **Golden files:** `golden_v1.brsh` and `golden_v1.brshbrush` (made from `golden_v1.bcmd`) must
  keep opening with the same hash and settings.
- **Hostile files rejected:** wrong tile sizes, tiles outside the canvas, missing entries, a
  bad active layer, wrong format ids and non-ZIP input.

## Dependencies
`zip` pinned at `=7.3.0` with only the pure-Rust `deflate` feature (zip 9 needs Rust 1.88,
above our MSRV); `zstd` 0.14 (the one C library, as approved in D-016). All licenses are on the
cargo-deny allow list.

## Design notes to confirm
- **Opening a file is not a command.** A journal that continues a document records the base
  hash instead of a `doc.open` line, so a journal never depends on a file path. Replay takes
  the base document explicitly (`--base`).
- **The active layer is saved in `document.json`** (it's UI state, but reopening on the same
  layer is what CSP does).
- **Brushes aren't referenced from `brush.stroke` yet.** A stroke still carries its full
  settings (self-contained replay). `brushye brush show` prints them ready to paste. Tool state
  that selects a library brush belongs with the brush editor and tool palette (M6/M7).

## Not done / open
- Autosave and the app's save flow (M6, with the window).
- Embedding a document's brushes (D-014: later, optional).
- 16-bit layers (the `layer.json` format field leaves room).

## Proposed next step
M6: the app shell. An eframe window with the wgpu canvas display, tablet input wired to
`brush.stroke`, a layer palette and colour and brush-size controls, and Open/Save using `.brsh`.
That's where the maintainer starts drawing with the pen.
