# M8 report: .sut import (2026-11-14)

**Status: DONE.** The maintainer marked M8 done on 2026-11-14. One brush-engine item stays
open and tracked: the anti-aliased thin tip (B-014). CI is green on Windows, macOS and Linux.

## What works
- **Format study** (`specs/formats/sut.md`, samples `specs/formats/samples.md`):
  - 98 samples made by automated observation in the maintainer's CSP (D-020), from new custom
    sub tools only, and kept outside the repo.
  - A `.sut` is an SQLite database. Settings are named columns of `Variant`.
  - Pressure dynamics are big-endian "effector" blobs: input mask, minimum, curve points.
  - Mapped: size, opacity, AA, hardness (bar segments 1/30/60/95/100 = levels 1..5),
    thickness and direction, angle and Direction of line, gap (presets and fixed), density and
    density by gap, all 35 ink modes, stabilization, and pen pressure for size, opacity and
    density with curves.
- **Importer** (`crates/brushye-sut`):
  - It loads the file from bytes into a read-only, in-memory SQLite (`rusqlite` with bundled
    SQLite, approved in D-016) with SQLite's defensive settings.
  - It produces a brush preset. Anything Brushye lacks is reported as "approximated" or "not
    imported": colour mixing, spray, watercolour edge, colour jitter, material tips, dual
    brush, ribbon, texture, tilt/velocity/random dynamics, size on screen, at least 1 px, and
    adjust by speed.
  - All 98 samples import. 9,000 corrupted files fail cleanly; this fuzzing found and fixed a
    panic inside rusqlite on damaged column names.
- **Where it's available:**
  - CLI: `brushye brush import-sut FILE.sut [OUT_DIR]` (also writes `.brshbrush`).
  - App: Import sub tool accepts `.sut`. Each sub tool is added through the `subtool.add`
    command, and a report lists what wasn't imported.
- **Cross-check** (B-012, the exit check): 12 brushes made in CSP, exported and drawn with the
  virtual stylus, then imported and replayed in Brushye.
  - Every setting imports exactly.
  - Hard brushes match: total ink within 0.5 %, lone dabs pixel-identical.
  - Soft brushes match after the engine fixes below.
- **Brush engine fixes from M8 observations:**
  - B-013: the preset gaps space soft dabs further apart (×3, ×2.75, ×2, ×1 for levels 1–4).
    Soft stroke ink was up to +123 % off and is now within 0.6 %.
  - B-015: the +0.5 px soft-dab offset depends on AA and size. Level 4 keeps an AA edge (its
    ink at size 8 was +90 %, now about +10 %).
  - Cross-check results: X-04 93 % → 0.6 % of pixels off, X-12 58 % → 2.8 %, X-03 31 % → 17 %.
- **Decisions:** D-020 (sample creation and storage), D-021 (before 1.0, fidelity fixes may
  change journal replay; saved documents keep their pixels).
- **Releases:** v0.1.0-alpha, and v0.1.0-alpha.2 with a Linux AppImage (embedded update
  information for Gear Lever from the next release on), the maintainer's app icon, and no
  console window on Windows.

## What doesn't work yet
- **Anti-aliased thin tips (B-014).**
  - Observed in three runs: single dabs, a 600-dab sub-pixel sweep, and strokes.
  - AA None thin strokes match CSP.
  - With AA on, no exact dab rule has been found yet. The tip looks like a 12-gon inscribed in
    the ellipse, with mixed row and column anti-aliasing (all models tried are in the spec).
  - The engine keeps its interim model: stroke ink is right within ~1–2 %, but the edge
    softness differs.
- **Soft tips at very small sizes** (size 8, levels 1–3, +6 … +24 % ink), and the exact sizes
  at which the soft-dab offset switches on (B-015).
- **Not imported:** the features listed under the importer above. They need engine support
  first.

## Specs waiting on observation or analysis
- **B-014:** the exact thin-tip AA rule. Analysis only; the data is in.
- **B-015:** the soft-tip offset switch sizes, and size 8.
- **Native Windows checks:** Direction of line at a motionless press (vpen nudge), and a 1-level
  difference between two identical H3/H4 dab positions.

## Proposed next step
**M9:** the MCP server (`brushye-mcp`), exposing the command and query registry to AI clients,
plus script examples and a journal viewer.
