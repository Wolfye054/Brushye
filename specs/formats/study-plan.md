# Format study plan: .sut and .clip

Status: plan (no samples studied yet). CSP version: 1.13.2. Written 2026-11-11.

## Clean-room rules for this work (from CLAUDE.md)
- The only inputs are **files the maintainer created in CSP 1.13.2 containing their own work**,
  plus published specs for any standard sub-formats found inside (e.g. SQLite, zlib, PNG).
- Not used: decompiled/disassembled CSP code, files from a CSP installation (including the
  default brushes CSP ships), leaked material, CSP's installed settings or databases,
  **and third-party reverse-engineering write-ups of .clip/.sut**. Those aren't on the
  allowed-source list. If the maintainer wants to allow published community docs, that
  needs an explicit decision in DECISIONS.md first.
- Every finding goes into `specs/formats/sut.md` / `specs/formats/clip.md`, tied to the sample
  file(s) it came from and how it was found (e.g. "differential: S-03 vs S-04, only `size` changed").
- Sample files live in a private fixtures repo or folder that the maintainer controls (git-lfs or
  outside the repo); `specs/formats/samples.md` lists their hashes and what each contains.
  Committing them to the public repo is the maintainer's call, since they contain only their own work.
- Brushye import code runs only on files the user supplies at runtime. Nothing CSP-made is bundled.

## Method
1. **Black-box differential analysis.** Make pairs of files that differ in exactly one
   setting, then compare them structurally (container listing, per-record diff, byte diff of
   changed records). Each finding = "setting X lives at Y, encoded as Z", backed by ≥ 2 pairs.
2. **Container first.** Identify the outer container with standard tools (`file`, magic
   numbers, `sqlite3` if it's SQLite, zip listing, etc.) and document only what the samples
   show.
3. **Value sweeps.** For numeric settings, export the same sub tool at several values
   (min, max, default, two in between) to recover encoding and scaling.
4. **Behaviour cross-check.** For each imported brush, draw a reference stroke in CSP
   (automated observation, or by the maintainer for pressure) and compare it with Brushye's
   rendering of the imported brush. The format spec is only "done" when behaviour matches.
5. **Robustness.** The importer is fuzzed. Unknown fields are kept and reported, never a crash.

Tooling (to be written in `tools/format-study/`, Python or Rust; no CSP code involved):
container dump, record differ, value-sweep tabulator.

## Sample files for the maintainer to create in CSP 1.13.2

Create **new** sub tools each time with **File > … / Sub Tool palette > Create custom sub tool**
or by duplicating *your own* custom sub tool. Don't export CSP's default sub tools, because they
are CELSYS materials. Export with the Sub Tool palette menu's export command to a `.sut`.
Name each file exactly as listed. Write any extra notes in `samples.md`.

### .sut set A — baseline and identity
| ID | File | How to make it |
|---|---|---|
| S-01 | `S-01_base.sut` | New custom brush sub tool "Base": circle tip, size 10 px, opacity 100, hardness default, AA Middle, everything else default |
| S-02 | `S-02_base_again.sut` | Export S-01's sub tool a second time without changes (shows nondeterministic fields like timestamps/UUIDs) |
| S-03 | `S-03_renamed.sut` | S-01 renamed to "Base Renamed" |
| S-04 | `S-04_icon.sut` | S-01 with a different sub tool icon/colour if the UI allows it |

### .sut set B — single-setting sweeps (each made from S-01, one change only)
| ID | Change |
|---|---|
| S-10..S-14 | Brush size 1, 2.5, 50, 500, 2000 |
| S-15..S-17 | Opacity 0, 37, 99 |
| S-18..S-21 | Anti-aliasing None / Weak / Strong (Middle = S-01) |
| S-22..S-25 | Hardness min, 25 %, 75 %, max |
| S-26..S-28 | Thickness 10, 50, 100 (+ direction vertical in S-28) |
| S-29..S-31 | Angle/direction 0, 45, 359 |
| S-32..S-35 | Gap: Narrow / Normal / Wide / Fixed (value 7) |
| S-36 | Brush density 50 |
| S-37 | Pressure → size ON with minimum 20 % |
| S-38 | Same as S-37 with a custom 3-point pressure curve (draw a clear S-curve; screenshot the graph) |
| S-39 | Pressure → opacity ON, minimum 0 |
| S-40 | Pressure → density ON |
| S-41 | Velocity → size ON |
| S-42 | Random → size ON |
| S-43 | Tilt → thickness ON (if available) |
| S-44..S-47 | Ink blending mode: Multiply, Compare density, Background, Replace alpha |
| S-48..S-50 | Colour mixing: Blend (paint amount 60, density 40, stretch 30), Running colour, Smear |
| S-51..S-52 | Stabilization 0 and 30; S-53 adjust-by-speed on |
| S-54 | Starting and ending: brush size, in 20 px out 40 px |
| S-55 | Texture: your own texture image registered as a material (made by you), density 50, scale 120 % |
| S-56 | Spray effect on (size 10, density 20, bias 50) |
| S-57 | Watercolor edge on (width 5, opacity effect 50, darkness 50, blur 3) |
| S-58 | Colour jitter on (hue 10, sat 10, val 10) |
| S-59 | Material tip: your own image (e.g. a 64×64 star you drew) as the tip |
| S-60 | Two material tips + repeat method "Random" |
| S-61 | 2-brush shape (dual brush) enabled with your own tip |
| S-62 | Ribbon on |

For each B-file, also save a screenshot of the Sub Tool Detail category you changed
(`S-xx.png`), so the findings can be tied to what the UI showed.

### .sut set C — other tool kinds (later)
Eraser sub tool, airbrush sub tool, a blend (blur) tool, fill tool, gradient tool, each made
as a new custom sub tool.

### .clip set — minimal and incremental
All 256×256 px, 72 dpi unless stated, RGB "Color" expression, made from **File > New**.
| ID | File | Content |
|---|---|---|
| C-01 | `C-01_empty.clip` | New canvas, paper white, one empty raster layer, saved immediately |
| C-02 | `C-02_empty_again.clip` | Same steps as C-01 again (nondeterministic fields) |
| C-03 | `C-03_size.clip` | As C-01 but 300×200, 350 dpi |
| C-04 | `C-04_paper_color.clip` | As C-01 with paper colour #336699 |
| C-05 | `C-05_one_pixel.clip` | C-01 + one pixel #ff0000 at (10,20) drawn with a 1 px pen, AA None |
| C-06 | `C-06_full_fill.clip` | C-01 with the layer filled #123456 |
| C-07 | `C-07_two_layers.clip` | Two raster layers with simple content, named "A" and "B" |
| C-08 | `C-08_opacity.clip` | C-07 with B at opacity 37 % |
| C-09 | `C-09_blend.clip` | C-07 with B in Multiply (+ C-09b Add (Glow), C-09c Divide) |
| C-10 | `C-10_hidden.clip` | C-07 with A hidden |
| C-11 | `C-11_folder.clip` | A and B in a folder; C-11b folder set to Through |
| C-12 | `C-12_clip.clip` | B clipped to A |
| C-13 | `C-13_lock_alpha.clip` | B with lock transparent pixels |
| C-14 | `C-14_mask.clip` | A with a layer mask (half masked) |
| C-15 | `C-15_large.clip` | 4000×3000 with a big painting of yours (tiling/compression) |
| C-16 | `C-16_semi_alpha.clip` | Soft airbrush strokes with partial alpha (alpha precision) |
| C-17 | `C-17_vector.clip` | One vector layer with 2 strokes (later) |
| C-18 | `C-18_text.clip` | One text layer "Brushye" (later) |
| C-19 | `C-19_gray_mono.clip` | Layers with Gray and Monochrome expression colour |
| C-20 | `C-20_ref_draft.clip` | A as reference layer, B as draft layer |
| C-21 | `C-21_reopened.clip` | Open C-07, move one pixel, save (does re-saving keep stale data?) |

For every .clip also export a PNG (File > Export (Single Layer) > .png, RGB, no scaling)
so pixel content can be checked against Brushye's import.

## Deliverables
- `specs/formats/sut.md`, `specs/formats/clip.md`: findings, each tied to sample IDs.
- `specs/formats/samples.md`: sample registry (ID, file name, SHA-256, CSP version, how made).
- Importer fuzz corpus made from the samples (stays wherever the samples live).
