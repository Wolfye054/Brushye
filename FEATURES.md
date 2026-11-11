# FEATURES — CSP 1.13.2 feature inventory for Brushye

Scope: what CLIP STUDIO PAINT **1.13.2** offers (PRO/EX), grouped by area and tagged for
Brushye:

- **alpha**: part of the alpha goal in CLAUDE.md
- **post-alpha**: next after the alpha (roughly v0.2–v0.5)
- **later**: valuable, but after the core painting/comics workflow is solid
- **n/a**: not planned (out of scope, or depends on CELSYS services)

Sources: the public CSP manual and CELSYS release notes (details and URLs in
`docs/research/csp-manual-inventory.md`), plus what we've seen in 1.13.2 itself. The current online
manual describes Ver. 5.x and uses newer palette names (Tool Group = 1.x *Sub Tool*,
Tool Settings = *Tool Property*, Advanced Tool Settings = *Sub Tool Detail*). Brushye
follows the **1.13.2** names and workflow.

## Features in the online manual that 1.13.2 does NOT have
These are out of scope for parity. Each could be considered later as a Brushye-original idea.
Each entry was checked against the official release notes (version added in parentheses):

- Perceptual colour mixing + brightness correction (2.0), Shading Assist (2.0),
  3D head model and Hand Scanner (2.0), fisheye perspective ruler (2.0), Align/Distribute (2.0),
  Liquify across multiple layers (2.0; Liquify itself is 1.11.6), Spin blur, Geometric
  distortion, Convert to panorama (2.0), text boxes/ligatures (2.0), background saving (2.0),
  layer keyword search (2.0), Simple Mode (2.0.5/2.1)
- Transparent animated GIF, font search (2.1); VRM import, Sub View image list, export
  watermarks, Smart Smoothing as a resample method (2.2); perceptual gradients, kerning, justify,
  rectangle balloon (2.3)
- Color Match, Layer Comps, WebP, export presets, new filters (Chromatic aberration, Noise,
  Retro film, Pencil drawing, Normal map), circular text, OpenType features (3.0); Tool Slider
  palette, Rough pen (3.1); Quick Filters (3.2)
- Puppet warp, Snap to objects, Magnetic lasso, Lens blur, Crystallize, .sutg groups,
  Sub Tool multi-select (4.0); velocity dynamics graph, Dynamic Eraser (4.1);
  AA "Speed and Quality", high-quality running colour, per-canvas colour profile (4.2)
- Smart Shape, 3D hand model, multi-layer tonal correction, per-operation recovery data,
  palette renames (5.0); Vivid watercolor edge, Glow/Inflate/Drop shadow filters, Lab sliders (5.1)

---

## 1. Canvas & view
| Feature | Tag |
|---|---|
| New canvas: width/height, resolution (dpi), paper colour, units (px/mm/in) | alpha |
| Canvas presets (illustration, comic, webtoon, animation) | post-alpha |
| Pan (Hand tool, Space-drag, scroll bars) | alpha |
| Zoom (tool, wheel, keyboard, 100%/fit, Navigator slider) | alpha |
| Rotate (tool, Navigator, configurable step, reset) | alpha |
| Flip view horizontal/vertical (view only) | post-alpha |
| Navigator palette | post-alpha |
| Sub View palette (reference images, eyedropper from them) | later |
| Multiple open documents (tabs) | post-alpha |
| Multiple views of one canvas (Window > Canvas > New window) | later |
| Change canvas size / image resolution (with resampling choices) | post-alpha |
| Grid, ruler bar, canvas margins | later |

## 2. Files & formats
| Feature | Tag |
|---|---|
| Native save/load (.brsh) | alpha |
| PNG export (single flattened image, transparency option) | alpha |
| PNG/JPEG/BMP/TGA/TIFF open as new document | post-alpha |
| Export JPEG/BMP/TIFF/TGA; export scale; expression colour (RGB/Gray/Mono) | post-alpha |
| PSD/PSB import & export with layers | post-alpha |
| ICC embed on export, colour profile preview, CMYK export | later |
| Import image as layer, Paste from clipboard | post-alpha |
| **.sut import (sub tools/brushes)** | post-alpha (first after alpha) |
| **.clip import** | later (after .sut) |
| .clip export | later |
| Animated GIF/APNG/video export, image sequences | later |
| Timelapse recording/export | later |
| PDF export, batch page export (EX) | later |
| Print | later |
| Auto-recovery / backups | post-alpha |
| CLIP STUDIO cloud sync, ASSETS store | n/a |

## 3. Layers
| Feature | Tag |
|---|---|
| Raster layers (create, delete, duplicate, rename, reorder, show/hide) | alpha |
| Layer opacity | alpha |
| 28 blending modes (Normal … Brightness; see specs) | alpha |
| Layer folders | post-alpha (alpha if cheap) |
| Folder "Through" (pass-through) mode | post-alpha |
| Paper layer | alpha |
| Clip to layer below | post-alpha |
| Lock layer, lock transparent pixels | post-alpha |
| Layer masks (create/apply/show) | post-alpha |
| Reference layer, draft layer | post-alpha |
| Merge down, merge visible, merge selected, rasterize | post-alpha |
| Layer colour (main/sub), expression colour (Color/Gray/Monochrome) | later |
| Layer property effects: Border (Edge), Watercolor edge, Tone, Extract lines | later |
| Fill layer, gradient layer | post-alpha |
| Tonal correction layers (levels, curves, HSL, colour balance, gradient map, …) | later |
| Vector layers (vector strokes, vector eraser, correct line) | later |
| Text layers, balloon layers | later |
| Tone layers | later |
| Frame border folders | later |
| Image material layers, file objects | later |
| 3D layers | later (maybe n/a) |
| Animation folders / cels | later |
| Layer search (EX) | later |

## 4. Brush engine (Sub Tool Detail)
| Feature | Tag |
|---|---|
| Brush size (with min size, pressure/velocity/random dynamics) | alpha |
| Opacity with dynamics | alpha |
| Pressure curve per setting (graph) | alpha |
| Global pen-pressure adjustment (File > pen pressure settings) | alpha |
| Anti-aliasing: None/Weak/Middle/Strong | alpha |
| Brush tip: circle (hardness, thickness, direction/angle) | alpha |
| Brush tip: material (image tips, multiple tips, flip, repeat method) | alpha (single image tip), post-alpha (multi-tip) |
| Brush density, adjust density by gap | alpha |
| Stroke: gap (narrow/normal/wide/fixed), continuous spray | alpha |
| Stroke: ribbon, repeat methods | post-alpha |
| Ink combine modes (normal + 28 layer modes + Compare density, Background, Replace alpha, Erase, Erase compare, Black/White burn) | alpha (Normal, Compare density, Erase); post-alpha (rest) |
| Colour mixing: Blend, Running colour; amount/density of paint; colour stretch | post-alpha |
| Smear mixing (1.11.0+) | post-alpha |
| Correction: stabilization, adjust by speed | alpha |
| Correction: post correction, vector magnet | post-alpha / later |
| Starting and ending (in/out by length/percentage, size/opacity/thickness) | post-alpha |
| Texture (density, scale, rotation, modes, brightness/contrast) | post-alpha |
| Spray effect (particle size/density/bias) | post-alpha |
| 2-brush shape (dual brush, 1.10.10+) | post-alpha |
| Watercolor edge | post-alpha |
| Colour jitter (1.10.5+) | post-alpha |
| Tilt and direction-of-pen dynamics | post-alpha (alpha if tablet layer gives it for free) |
| Sub tool palette: create, duplicate, delete, reorder, groups | alpha (minimal) |
| Brush editor UI (Sub Tool Detail equivalent) | alpha |
| Save user brushes in Brushye format | alpha |
| Import .abr (Photoshop brushes) | later |
| Import .sut | post-alpha |

## 5. Tools
| Tool | Tag |
|---|---|
| Pen / Pencil / Brush / Airbrush (as brush presets) | alpha |
| Eraser (raster) | alpha |
| Eyedropper | alpha |
| Hand / Zoom / Rotate | alpha |
| Move layer | post-alpha |
| Selection: rectangle, ellipse, lasso, polyline, selection pen/eraser | post-alpha |
| Auto select (magic wand), refer other layers | post-alpha |
| Fill (refer editing / other layers, close gap, tolerance, area scaling) | post-alpha |
| Enclose-and-fill, Paint unfilled area | post-alpha |
| Gradient tool (linear/circle, edge processing) | post-alpha |
| Blend tools (blur, finger, etc.) | post-alpha |
| Liquify (1.11.6+) | later |
| Decoration brushes (effect, hatching, ruled lines) | later |
| Copy stamp | later |
| Figure tool (lines, curves, shapes) | post-alpha |
| Rulers: linear, curve, figure, perspective (1/2/3-point), symmetry, special (parallel/radial/concentric), guides | later |
| Text, Balloon, Frame border tools | later |
| Correct line, Operation/Object | later |

## 6. Selection & transform
| Feature | Tag |
|---|---|
| Select all / deselect / invert / reselect | post-alpha |
| Selection launcher, selection layers, quick mask | later |
| Expand/shrink/border selection | post-alpha |
| Free transform: scale, rotate, skew, distort, perspective, flip | post-alpha |
| Mesh transformation | later |
| Interpolation modes | post-alpha |

## 7. Colour
| Feature | Tag |
|---|---|
| Main/sub colour, transparent colour | alpha |
| Colour wheel (HSV) | alpha |
| Colour slider (RGB/HSV) | alpha |
| Colour set, colour history | post-alpha |
| Intermediate / approximate colour palettes | later |
| Colour Mixing palette (1.12.0+) | later |
| CMYK slider, colour profile preview | later |

## 8. Filters & correction
| Feature | Tag |
|---|---|
| Gaussian blur, blur, motion/radial blur | post-alpha |
| Sharpen, unsharp mask | later |
| Tonal correction (brightness/contrast, levels, tone curve, HSL, colour balance, posterize, binarize, reverse gradient, gradient map) | post-alpha (destructive), later (layers) |
| Distort filters, Perlin noise, artistic, mosaic | later |
| Smart Smoothing, Colorize, Pose scanner | n/a (ML features) |

## 9. Workspace, settings, automation
| Feature | Tag |
|---|---|
| Dockable palettes, saved workspaces | post-alpha |
| Keyboard shortcut settings, modifier key settings | post-alpha |
| Command bar | post-alpha |
| Quick Access palette | later |
| Auto Actions (record/replay) | post-alpha. Maps naturally onto Brushye's command journal |
| Preferences (tool, canvas, pen/touch, file, interface) | alpha (minimal) |
| Companion mode (smartphone) | n/a |
| Undo/redo, history palette | alpha |
| **Scripting, headless mode, MCP server** (Brushye-specific) | alpha (headless + commands), post-alpha (MCP) |

## 10. Comics, animation, 3D, materials
| Area | Tag |
|---|---|
| Multi-page comics (page manager, story, crop marks, bleed) | later |
| Frame borders, speed/focus lines, screentones | later |
| Animation timeline, cels, onion skin, light table | later |
| 3D models/poses | later or n/a |
| Material palette (user materials: brushes, images, tones) | later |
| CLIP STUDIO ASSETS / downloads | n/a |
