# M2 report: document model + command layer (2026-11-13)

Status: **built and tested locally; the cross-platform exit check is in place but has not run yet**
(the repo has no remote, so CI has never executed).

## What works
- `brushye-core`: document (size, dpi, paper), bottom-to-top layer stack with ids, names,
  visibility, CSP-style integer opacity, the 28 blend-mode names, locks; sparse copy-on-write
  256² RGBA8 tiles; a deterministic FNV-1a content hash. Edits share unchanged layers/tiles
  between versions.
- `brushye-command`:
  - the text syntax exactly as `docs/command-format.md` (parse/print, canonical round-trip);
  - typed parameters declared once (`params!`), from which text, JSON and schema derive;
  - registry with command classes (session / undoable / app state / history) and per-command
    versions, plus the migration engine (tested with a synthetic 3-version command);
  - `Session` executor on immutable versions (D-013): undo/redo, redo-branch discard,
    CSP-style merging of consecutive same-property changes recorded as `@n+`, visibility
    journaled but not undoable (H-001), undo limit 1..200, failed commands leave no trace;
  - journal writing and strict replay (header, sequence, migrations, recorded merges only);
  - queries: `?doc.info`, `?layer.list`, `?layer.get`, `?doc.hash`, `?history`, `?commands`.
- `brushye-cli` (binary `brushye`): `run SCRIPT [--journal OUT]`, `replay JOURNAL --query …`,
  `commands`; errors as `FILE:LINE: message`.
- Tests: 45 in the workspace, including property tests (random commands survive text and JSON
  round-trips) and replay = identical document **and** identical journal text.

## Exit criterion
"A journal replays to an identical document hash on all 3 CI OSes": a golden-hash test
(`golden_document_hash_is_platform_independent`) pins the hash of a fixed script. It passes on
Linux; it will check Windows and macOS the first time CI runs, which needs a GitHub remote.

## Not done / deferred
- The optional time-based undo merge interval (D-013 point 3, default 0 ms): needs command
  timestamps; deferred to the app milestone.
- Folders, pixel commands and strokes come with M3/M4.
- Commands carry a short registry description; per-parameter docs are in the schema.

## Next step
M3: CPU reference compositor (blend modes per D-017, against the observed specs).
