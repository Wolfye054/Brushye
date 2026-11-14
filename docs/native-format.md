# `.brsh` and `.brshbrush`, format version 2

The exact layout implemented by `brushye-format` (DECISIONS D-014). `brushye dump FILE.brsh`
prints all of it; `tests/data/golden_v1.*` in `brushye-format` are reference files that every
later Brushye must keep opening.

## `.brsh` (ZIP)
| Entry | Content | ZIP method |
|---|---|---|
| `manifest.json` | `{"format": "brushye-document", "version": 2, "app": "<version>", "features": [...]}`; `features` lists `"journal"` when one is embedded | deflate |
| `document.json` | `width`, `height`, `dpi`, `paper` (`"#rrggbb[aa]"` or `null`), `active` (`"L<n>"` or `null`), `layers`: bottom to top, each `{id, name, kind, visible, opacity, blend, locked, lock_alpha, parent, clip, expanded}` with the command-syntax names (`kind: "raster"` or `"folder"`, `blend: "multiply"` or `"through"` …). `parent` is the folder's id or `null`; a folder's contents are the layers directly below it (D-023), which the loader checks | deflate |
| `layers/L<n>/layer.json` | `{"format": "rgba8", "tile_size": 256}` (folders too; a folder has no tiles) | deflate |
| `layers/L<n>/tiles/<tx>_<ty>.zst` | one non-empty tile: 256×256 straight RGBA8, row-major, zstd (level 3); tiles outside the canvas are an error | stored |
| `preview.png` | the flattened document (CPU reference renderer), full size, RGBA, no colour chunks | stored |
| `thumbnail.png` | the preview scaled to fit 256 px (alpha-weighted area average) | stored |
| `journal.bcmd` | the session journal, only when the document opts in | deflate |

- Entries are written in this order, with fixed timestamps (1980-01-01) and permissions, so a
  document always gives the same bytes (for the same zstd/zip versions).
- Reading: a newer `version` is refused with a message naming both versions; older versions
  run through the migrations in `brsh::MIGRATIONS`: **v1 → v2** (2026-11-14, D-023) adds
  `parent: null, clip: false, expanded: true` to every layer. Brushye 0.1 (format 1) refuses
  version 2 files with its "update Brushye" message. Unknown entries, keys or
  features are errors in strict mode (tests) and warnings otherwise (the app). Any ZIP
  compression method the `zip` crate's deflate feature reads is accepted, so a re-zipped file
  still opens.
- Saving writes a temporary file next to the target, syncs it, and renames it over the old
  file. A tile cache keyed on tile identity (tiles are shared copy-on-write between document
  versions) recompresses only tiles that changed since the last save.

## `.brshbrush` (ZIP)
| Entry | Content |
|---|---|
| `brush.json` | `{"format": "brushye-brush", "version": 1, "app": "<version>", "name": "...", "settings": {...}}` |
| `images/<name>.png` | tip/texture images (none used yet) |

`settings` holds the `brush.stroke` keys other than `layer`, `color` and `samples`, in the
command's JSON form (`{"size": 14.0, "gap": "narrow", "size_curve": [[0.5, 0.3]]}`); it is
validated like the command. `brushye brush show` prints it as `key=value` text for a
`brush.stroke` line. A brush library is a folder of `.brshbrush` files (`brushye brush list`).
