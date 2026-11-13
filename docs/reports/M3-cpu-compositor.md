# M3 report: CPU reference compositor (2026-11-13)

## What works
- **Blend math** (`brushye-core::blend::math`), per DECISIONS D-017, checked against every observed
  CSP 1.13.2 export (`tests/observed_blend.rs`):
  - opaque layers (L-001): **22 of 22 separable modes bit-exact** on all 131,072 pixels (19 integer
    formulas, observed tables for Overlay/Hard light/Vivid light, Soft light as W3C float);
  - source alpha (L-002): bit-exact for every mode except Multiply (±2) and the six colour modes;
  - layer opacity (L-003): bit-exact for Normal, Screen, Add, Add (Glow) at 1/37/50/99/100 %;
  - backdrop alpha (L-005): bit-exact for all 28 modes; Normal colour 99.6 % exact (±1).
- **Renderer** (`render::flatten`): paper, visible layers bottom to top, blend + opacity; whole
  documents rebuilt from the observed data render bit-exactly (`tests/observed_render.rs`).
  Finding: without paper, CSP copies the bottom layer instead of compositing it (noted in L-005).
- **Pixels in/out headless**: `layer.fill`, `layer.pixels` (raw RGBA in the command, so journals
  stay self-contained), `?pixel`, `?layer.pixel`; locks honoured (L-006: lock transparent pixels
  keeps alpha and leaves transparent pixels untouched).
- **PNG export** (`brushye-format`): straight RGBA, no colour chunks, deterministic bytes;
  `brushye render JOURNAL OUT.png` and `brushye run SCRIPT --png OUT.png` (identical bytes).

## Stated tolerances (open; only ever tighten — pinned in tests/observed_blend.rs)
| Case | Exact | Max error | Closes with |
|---|---|---|---|
| Darker/Lighter color (opaque) | 99.998 % | x.5 luma ties | L-007 tie rule |
| Hue/Saturation/Color/Brightness (opaque) | 95.7–97.1 % | 4–7 | L-007 clip rule |
| Multiply with partial alpha / opacity < 100 % | 49–58 % | 2 | L-002/L-003 open question |
| Colour over a semi-transparent backdrop, non-Normal modes | 12–79 % | up to 191 | L-005 |

The last row is the biggest gap: it only matters for layers drawn over transparent areas of a
canvas without paper, in modes other than Normal.

## Not in M3
Folders/Through and clipping (observed in L-004/L-006) belong to M10 in the roadmap.
Rendering is single-threaded and whole-document for queries; the app (M6) will render
incrementally per dirty tile, with per-tile parallelism where needed (D-012).

## Next step
M4: brush engine v1 (headless), starting with the mouse-only brush specs that still need
observation (B-001 dab profile, B-002 spacing, B-003 accumulation).
