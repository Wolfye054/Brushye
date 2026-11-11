# CSP feature inventory and version deltas against 1.13.2

Researched 2026-11-11. Sources: only the official CELSYS manual and release notes (help.clip-studio.com, clipstudio.net, support.clip-studio.com).
Wording is my own paraphrase. Pages were read through a summarizing fetch tool, so a few long pages were read in chunks (offsets).
Where a page did not list something, I fill in from general knowledge and mark it **[GK]**. Treat [GK] items as leads to check, not facts.

**Naming caveat:** the current online manual covers **Ver. 5.0**, and Ver. 5.0.0 renamed several palettes. When reading it against 1.13.2, translate the names:
- "Tool Group palette" is the old **Sub Tool palette**.
- "Tool Settings palette" is the old **Tool Property palette**.
- "Advanced Tool Settings palette" is the old **Sub Tool Detail palette**.
- "Tool Properties" (menu item) is the old "Tool Settings".

## Sources

Manual (current, Ver. 5.0):
- https://help.clip-studio.com/en-us/First_Topic.htm (top page and chapter list; also links the older PDF manuals)
- https://help.clip-studio.com/en-us/manual_en/180_layers/180_layers.htm
- https://help.clip-studio.com/en-us/manual_en/180_layers/Blending_modes.htm
- https://help.clip-studio.com/en-us/manual_en/180_layers/Using_layers.htm
- https://help.clip-studio.com/en-us/manual_en/180_layers/Layer_properties.htm
- https://help.clip-studio.com/en-us/manual_en/180_layers/Other_layer_settings.htm
- https://help.clip-studio.com/en-us/manual_en/240_brushes/240_brushes.htm
- https://help.clip-studio.com/en-us/manual_en/240_brushes/Customizing_brush_tools.htm
- https://help.clip-studio.com/en-us/manual_en/240_brushes/Drawing_and_painting.htm
- https://help.clip-studio.com/en-us/manual_en/240_brushes/Adjusting_pen_pressure.htm
- https://help.clip-studio.com/en-us/manual_en/240_brushes/Special_brushes.htm
- https://help.clip-studio.com/en-us/manual_en/240_brushes/Blending_tools.htm
- https://help.clip-studio.com/en-us/manual_en/240_brushes/Eraser_tools.htm
- https://help.clip-studio.com/en-us/manual_en/150_tools/150_tools.htm
- https://help.clip-studio.com/en-us/manual_en/210_file/210_file.htm
- https://help.clip-studio.com/en-us/manual_en/210_file/Exporting_files.htm
- https://help.clip-studio.com/en-us/manual_en/210_file/Open_file.htm
- https://help.clip-studio.com/en-us/manual_en/690_interface/690_interface.htm
- https://help.clip-studio.com/en-us/manual_en/270_canvas/270_canvas.htm
- https://help.clip-studio.com/en-us/manual_en/270_canvas/Navigating_the_canvas.htm
- https://help.clip-studio.com/en-us/manual_en/300_color/300_color.htm
- https://help.clip-studio.com/en-us/manual_en/300_color/Color_Slider_palette.htm
- https://help.clip-studio.com/en-us/manual_en/330_selection/330_selection.htm
- https://help.clip-studio.com/en-us/manual_en/330_selection/Basic_selection_functions.htm
- https://help.clip-studio.com/en-us/manual_en/360_transform/360_transform.htm
- https://help.clip-studio.com/en-us/manual_en/360_transform/Types_of_transformations.htm
- https://help.clip-studio.com/en-us/manual_en/390_filters/390_filters.htm
- https://help.clip-studio.com/en-us/manual_en/390_filters/Filters.htm
- https://help.clip-studio.com/en-us/manual_en/390_filters/Smart_Tools.htm
- https://help.clip-studio.com/en-us/manual_en/390_filters/Tonal_Correction.htm
- https://help.clip-studio.com/en-us/manual_en/420_fill/420_fill.htm
- https://help.clip-studio.com/en-us/manual_en/420_fill/Fill_Tool.htm
- https://help.clip-studio.com/en-us/manual_en/450_figure/450_figure.htm
- https://help.clip-studio.com/en-us/manual_en/480_text/480_text.htm
- https://help.clip-studio.com/en-us/manual_en/510_ruler/510_ruler.htm
- https://help.clip-studio.com/en-us/manual_en/510_ruler/Basics_of_creating_rulers.htm
- https://help.clip-studio.com/en-us/manual_en/540_comic/540_comic.htm
- https://help.clip-studio.com/en-us/manual_en/600_animation/600_animation.htm
- https://help.clip-studio.com/en-us/manual_en/630_material/630_material.htm
- https://help.clip-studio.com/en-us/manual_en/660_3d/660_3d.htm
- https://help.clip-studio.com/en-us/manual_en/780_shortcuts/780_shortcuts.htm
- https://help.clip-studio.com/en-us/manual_en/810_subtools/810_subtools.htm (settings glossary index, A–Z)
- https://www.clip-studio.com/site/gd_en/csp/toolguide/csp_toolguide/100_reference/Freeformgradient.htm (older Tool Setting Guide, (c) 2020)
- https://www.clipstudio.net/en/manuals/

Release notes and news:
- https://www.clipstudio.net/en/dl/release_note/latest/ (5.0.0 to 5.1.5)
- https://www.clipstudio.net/en/dl/release_note/v4/ (4.0.0 to 4.2.7; read in two chunks)
- https://www.clipstudio.net/en/dl/release_note/v3/ (3.0.0 to 3.2.3; two chunks)
- https://www.clipstudio.net/en/dl/release_note/v2/ (2.0.0 to 2.3.4; two chunks)
- https://www.clipstudio.net/en/dl/release_note (Ver. 1 notes, 1.6.x to 1.13.2; read 0 to 300k of 360k chars)
- https://www.clipstudio.net/en/news/202303/14_01/ (Ver. 2.0 announcement)
- https://support.clip-studio.com/en-us/faq/articles/20230004 (Ver. 1 download FAQ)

Note: https://help.clip-studio.com/en-us/manual_en/ itself returns 404. The entry point is First_Topic.htm.

## Feature inventory by area

Each line ends with an edition tag where the manual gives one (DEBUT/PRO/EX). Items added after 1.13.2 are marked **(new: x.y)**, from the delta table below.

### Interface, workspace, palettes, preferences
- Dockable palettes and palette docks, with hide/show of the title bar and menu bar.
- Workspace registration and management (saved layouts, and workspace materials since 1.6.7).
- Command Bar, with registerable commands. 2.1 added 80+ new shortcut and command-bar options.
- Quick Access palette (since 1.7.1). "Duplicate set" was added in 5.1.
- Navigator palette and Sub View palette (reference images). The Sub View image list arrived in 2.2 (EX/PRO), and Import from clipboard in 5.0.
- Information palette. It records work time since 2.1.
- Keyboard shortcut settings, Modifier Key Settings and touch gesture settings.
- Auto Actions palette. Its "Edit set" dialog arrived in 1.13.0 and became resizable in 1.13.2 (EX/PRO).
- Preferences categories: Tool, Canvas (rotation angle step), Pen/Touch, File, Color conversion, 3D, Animation, Interface, and others.
- Tablet service selection between Wintab and TabletPC/Windows Ink [GK]. The manual page read here doesn't cover it. Windows defaulted Input frequency to "Prefer quality" in 1.13.0.
- Pen pressure adjustment wizard (File > Pen Pressure Settings). This is a single global curve applied to all tools.
- Companion mode, which uses a smartphone as a shortcut remote (1.11.9; all platforms in 1.12.0).
- Tool Slider palette (new: 3.1). Pop-up related palettes per tool (new: 5.0).
- Interface Light and Dark modes. Simple Mode is a separate simplified UI on tablets and phones (new: 2.0.5/2.1).

### Canvas and view
- Zoom: menu, icons, Ctrl+NumPad, mouse wheel, 100%/200%/Print size presets, and the Navigator zoom bar.
- Pan with the Hand tool, plus scroll bars.
- Rotate with the Rotate tool or Navigator bar, in steps (5° by default, configurable). Also 90/180/270° and a two-finger touch rotate.
- Flip horizontal and vertical. These change the view only.
- Fit to Screen, Fit to Navigator, Reset Rotation, Reset Rotation/Inversion and Reset Display.
- Undo/Redo, Cut/Copy/Paste and Delete. The undo-level count is configurable (mobile default 30 since 2.3.4).
- Canvas margins, ruler bars and grid, plus Change canvas size and Change image resolution. Smart Smoothing interpolation for resolution change arrived in 2.2.

### File operations
- New canvas with presets for illustration, comic, webtoon and animation. Vertical and square video presets were added in 4.0. Setting the color profile in the New dialog is (new: 4.2).
- Open and import: .clip, .lip, BMP, JPEG, PNG, TIFF, TGA, PSD, PSB, and ibisPaint .ipv (1.12.0).
  - IllustStudio .xpg and ComicStudio .cpg (PRO/EX, Win/macOS).
  - .cst, .cmc, .csnf (EX) and PDF (EX).
  - SVG (1.10.0) and WebP (new: 3.0).
  - Movie import (Win/macOS earlier; iPad/Android new: 3.2).
- Save .clip, with background saving (new: 2.0), canvas recovery and "Save recovery data for every operation" (new: 5.0).
- Export (single layer): BMP, JPEG (quality), PNG (transparency; ICC profile EX/PRO since 1.12.0), WebP (new: 3.0), TIFF, TGA, PSD and PSB. Text layers export to PSD (EX/PRO, 1.12.0).
- Export settings: output size, expression color (RGB, Gray, Duotone, CMYK), embed ICC profile, and "process when scaling" for illustration or comic.
  - Export presets (new: 3.0).
  - Watermark (new: 2.2) and noise pattern (new: 3.1).
- Batch export of multiple pages (EX), PDF export (since 1.10.5) and webtoon export (embed ICC since 1.12.7).
- Animated GIF, APNG, MP4/AVI, image sequences and animation cels.
- Timelapse recording and export (since 1.10.5).
- Print, Manage files in Clip Studio, and cloud sync.

### Layers
- Types: raster, vector, folder, paper (one per canvas), fill, gradient, tone, text and balloon.
  - Frame border folder, 3D layer and image material layer.
  - Tonal correction layers, light table layers, animation folders and 2D camera folder.
  - File objects. File objects can reference .clip files with 3D layers since 3.2.
- Blending modes (28): Normal, Darken, Multiply, Color burn, Linear burn, Subtract, Lighten, Screen, Color dodge, Glow dodge, Add, Add (Glow), Overlay, Soft light, Hard light, Difference, Vivid light, Linear light, Pin light, Hard mix, Exclusion, Darker color, Lighter color, Divide, Hue, Saturation, Color, Brightness.
  - Folders also have **Through**. The manual mentions it under clipping-to-folder.
  - Twelve of the modes were added in 1.6.2.
- Opacity, Clip to Layer Below (including clipping to folders unless the folder is Through), Lock layer and Lock transparent pixels.
- Reference layer (lighthouse icon) and draft layer. "Hide all visible draft layers" is (new: 5.0, EX/PRO).
- Layer color (main and sub color swap), palette color marks, expression color (Color, Gray, Monochrome) and display-decrease color.
- Layer masks: create, apply, show area, and mask expression (semi-transparent masking threshold).
- Layer Property effects: Border effect Edge and Watercolor edge, Extract lines (EX), Tone (frequency, density, dot shape, angle, posterization), and Overlay texture for image materials.
  - "Vivid" style for watercolor edge and "Reflect opacity" for edge are (new: 5.1).
- Convert layer, rasterize, merge down, transfer to lower layer, and Delete empty/hidden layers (new: 4.1).
- Search Layer palette. Keyword search is (new: 2.0, EX).
- Layer comps (new: 3.0, EX).

### Selection
- Select All, Reselect, Deselect and Invert. Selection Launcher, selection layers, and selection from layer (Ctrl-click a thumbnail).
- Sub tools [GK for exact list]: rectangle, ellipse, lasso, polyline, selection pen, selection eraser, shrink selection, and Auto select (refer editing layer or other layers).
  - Magnetic lasso is (new: 4.0, EX/PRO), as is lasso selection mode (single stroke or multiple input).
- Quick mask [GK], plus expand, shrink and border selection [GK].

### Drawing tools and sub tools
- Pen group: G-pen and other pens, Marker, and Rough pen (new: 3.1).
- Pencil/Sketch group: pencil and pastel.
- Brush group: watercolor and thick paint (Real watercolor set since 1.8.6).
- Airbrush (soft, dense, scattered droplets), Decoration (effect, background, clothing, hatching, ruled line), dot pen and Copy stamp.
- Eraser: normal and kneaded eraser (1.8.6). Dynamic Eraser is (new: 4.1).
  - Vector eraser has three modes: touched area, up to intersection (optionally referring to all layers), and whole line.
  - "Erase on all layers" option.
- Blend: blur, finger and others, plus Liquify. Liquify is a Blend sub tool in 1.11.6 and its own tool in 1.12.0.
- Fill: refer editing layer, refer other layers, enclose and fill, lasso fill, and leftover pen (paint unfilled area). Settings include close gap, tolerance, area scaling and multiple referencing [GK for the last three].
- Gradient tool, contour line paint, and Freeform gradient (not in DEBUT). Freeform gradient appears in the ©2020 Tool Setting Guide, so it is likely pre-2.0. Version not verified.
- Figure tool: direct draw (straight line, curve, polyline, rectangle, ellipse, polygon) and continuous curve [GK]. The Bezier sub tool arrived in 1.9.9.
  - Smart Shape is (new: 5.0; menu commands new: 5.1).
- Rulers: linear, curve (spline, polyline, Bezier), figure ruler, ruler pen, special rulers, guide, symmetrical ruler and perspective ruler (1/2/3-point).
  - Special ruler types: parallel line, parallel curve, multiple curve, radial line, radial curve, concentric circle.
  - Fisheye perspective is (new: 2.0, EX/PRO).
  - Draw along ruler and ruler from vector arrived in 1.10.0.
- Text, Balloon, Frame border, Correct line (vector width, simplify and similar) [GK detail], Operation/Object, Move layer and Eyedropper (redesigned in 1.12.7).
- Snapping to rulers, grid and special rulers.
  - Snap to objects and the View > Snap submenu are (new: 4.0, EX/PRO).

### Sub tool detail (Advanced Tool Settings) categories
- Brush size, with minimum size and dynamics.
- Ink: opacity, combine mode, and color mixing.
  - Mixing modes: Blend, Running color and Smear. Smear arrived in 1.11.0; dual-brush mixing in 1.11.0.
  - Amount of paint, density of paint, color stretch, and blur intensity (automatic or fixed).
  - Mixing mode "Perceptual" with brightness correction is (new: 2.0).
  - High-quality running-color blending is (new: 4.2).
- Tool-only ink combine modes: Black burn, White burn, Erase, Erase (Compare), Compare density, Background and Replace alpha.
- Color Jitter (since 1.10.5).
- Anti-aliasing levels. The "Speed and Quality" option is (new: 4.2).
- Brush shape (presets), and brush tip (circle or material; angle, hardness, thickness, density, and flip, which arrived in 1.10.5).
- Spraying effect.
- Stroke: spacing, repeat method, and ribbon [GK].
- Texture: modes and brightness/contrast (1.10.5).
- 2-Brush shape / dual brush (1.10.10).
- Watercolor edge. The Vivid style is (new: 5.1).
- Erase category (vector eraser, all layers).
- Correction: stabilization (including by speed, 1.9.4), post-correction, taper and snap.
- Starting and ending, and Anti-overflow (reference-line stop).
- Brush dynamics: pressure, tilt, velocity and random, each with min/max and curve.
  - The velocity settings graph is (new: 4.1).
  - Android tilt is (new: 4.2), and Apple Pencil Pro barrel roll is (new: 3.0.5).
- Photoshop .abr import (1.10.5) and .sut import/export.
  - .sutg group export/import and multi-select in the Sub Tool palette are (new: 4.0).

### Color
- Color Wheel (HSV/HLS), Color Slider (RGB, HSV, HLS, CMYK not in DEBUT), Color Set, Color History, Intermediate Color and Approximate Color.
- Lab sliders are (new: 5.1).
- Color Mixing palette (1.12.0) and the Color Settings dialog.
- Eyedropper, pick screen color, and hide windows to pick screen color.
- Color profile preview (View menu, CMYK proof), and color conversion profiles in Preferences.
  - Per-canvas color profile is (new: 4.2).
  - Wide color gamut on mobile is (new: 4.2).

### Filters, correction and smart tools
- Blur: Blur, Blur (Strong), Gaussian, Smoothing, Radial and Motion. Spin blur is (new: 2.0) and Lens blur is (new: 4.0).
- Sharpen: Unsharp mask, Sharpen and Sharpen More.
- Effect: Artistic (1.9.4), Mosaic and Remove JPEG noise.
  - Chromatic aberration, Noise, Normal map, Pencil drawing and Retro film are (new: 3.0).
  - Crystallize is (new: 4.0).
  - Glow, Inflate and Drop shadow are (new: 5.1).
- Distort: Pinch, Ripple, Curved surface, Polar coordinates, ZigZag, Wave, Twirl and Fish-eye lens. Geometric distortion and Convert to Panorama are (new: 2.0).
- Render: Perlin noise. Correction: Remove dust and Adjust line width.
- Quick Filters and Quick Effects (new: 3.2 in the Studio Mode menu).
- Tonal correction (menu and correction layers): Brightness/Contrast, Levels, Tone curve, Hue/Saturation/Luminosity, Color balance, Reverse gradient, Posterization, Binarization and Gradient map. Posterization, Binarization and Color balance are [GK]; the page read doesn't list them.
  - Tonal correction across multiple layers is (new: 5.0).
  - Color Match is (new: 3.0, EX/PRO).
- Smart tools: Smart Smoothing (1.9.1 preview, standard in 1.9.4), Colorize technology preview (1.8.4), and Delete or grayscale tones.
  - Shading Assist is (new: 2.0).
- Convert layer to lines and tones (EX).

### Transform
- Scale, rotate, skew, free transform, distort, perspective and flip.
- Mesh transformation (PRO/EX, pre-2.0). Multi-point selection is (new: 2.3), as are later speedups.
- Puppet warp is (new: 4.0, EX/PRO).
- Interpolation choices [GK]: hard edges (nearest neighbor), bilinear, bicubic, high accuracy, and Smart Smoothing in resolution change (2.2).
- Align and Distribute is (new: 2.0, EX/PRO). The orientation-aligned bounding box is (new: 4.1).
- Liquify tool.

### Text and balloons
- Text tool: fonts, size, vertical text and ruby ("readings"), and managing and adding font files (1.11.6).
- Text boxes, wrap at frame, ligatures and multi-select are (new: 2.0).
- Kerning and Justify are (new: 2.3).
- Circular text and OpenType features are (new: 3.0).
- Font search and recent fonts are (new: 2.1).
- Balloon tool: balloon shapes, tails and flash. Rectangle balloon and Rough balloons are (new: 2.3/3.1).
- Story Editor (EX).

### Comics and webtoon
- Frame border folders, divide frames and frame templates.
- Speed lines and focus lines (special ruler types and Decoration), and screentones (tone layers and tone materials).
- Multi-page projects: Page Manager, story folder, crop marks, inner border and safety margins (PRO/EX for many; multipage EX).
- Teamwork (EX, 1.10.10). The webtoon creation tools arrived in 1.10.5.
  - Change canvas height (Webtoon) is (new: 2.0, EX).

### Animation
- Animation folders and cels, the Timeline palette, frames, onion skin and the Animation Cels palette.
- Light table layers, clips, tracks, keyframes and Graph Editor (1.8.4), 2D camera folders and audio (1.8.4).
- Label operations and timeline import/export (EX; xdts in 1.13.0).
- Export animation: GIF, APNG, video and image sequence.
- Edition limits: DEBUT and PRO open read-only when the file has more than 24 frames.
- Track color labels and file prefix/suffix are (new: 3.1). Audio scrubbing is (new: 3.0).

### 3D
- 3D layers, 3D drawing figures (Ver.2 body since 1.7.4) and 3D character materials.
- 3D primitives (1.11.6), panorama and skydome (1.11.6), lights and shadows, and cameras (switch camera since 1.12.7).
- FBX, OBJ and glTF import (glTF since 1.12.7), the Pose Scanner tech preview (1.8.6), and the All Sides View palette (EX, 1.6.7).
- Head model and Hand Scanner are (new: 2.0). VRM is (new: 2.2), BVH is (new: 3.1), shape keys and Posemaniacs are (new: 3.0), and the Hand model is (new: 5.0).
- Painting on 3D objects is (new: 4.0/5.0/5.1).
- Fog is (new: 2.0/2.1), and lens shift and tilt correction are (new: 2.0).

### Materials and CLIP STUDIO ASSETS
- Material palette: image, brush and 3D materials, Favorites (1.9.1), and Item bank palette.
- Register your own materials and material properties. Download from ASSETS (since 1.6.2).
- Auto-install downloads is (new: 4.2).
- Recently used and frequency sort are (new: 5.0), and search and pin options are (new: 5.1).

## Features newer than 1.13.2

Baseline: 1.13.2 was released 2023-01-31 and is the last 1.x version. Every version listed below came after it.
"Confidence" is **high** when the item is stated in the official release notes or the official announcement page.

| Feature | Version added | Source URL | Confidence |
|---|---|---|---|
| Perceptual color-mixing mode for brushes + brightness correction | 2.0.0 (2023-03-14) | https://www.clipstudio.net/en/dl/release_note/v2/ | high |
| Shading Assist (Edit menu) | 2.0.0 | v2 notes; https://www.clipstudio.net/en/news/202303/14_01/ | high |
| 3D head model (adjustable facial features) | 2.0.0 | v2 notes; news 202303 | high |
| Hand Scanner (hand pose from camera or photo) | 2.0.0 | v2 notes; news 202303 | high |
| Fisheye perspective ruler (PRO/EX) | 2.0.0 | v2 notes | high |
| Align/Distribute palette and commands (PRO/EX) | 2.0.0 | v2 notes | high |
| Liquify on multiple layers, masks and selections (Liquify itself predates this, 1.11.6) | 2.0.0 | v2 notes | high |
| Spin blur; Geometric distortion; Convert to Panorama (PRO/EX) | 2.0.0 | v2 notes | high |
| 3D fog; lens shift and tilt correction (PRO/EX); touch camera control | 2.0.0 | v2 notes | high |
| Text: text boxes, wrap at frame, ligatures, multi-select | 2.0.0 | v2 notes | high |
| Background saving of .clip | 2.0.0 | v2 notes | high |
| Keyword layer search; Change canvas height (EX) | 2.0.0 | v2 notes | high |
| Simple Mode UI (phones 2.0.5, tablets 2.1.0) | 2.0.5 / 2.1.0 | v2 notes | high |
| Transparent animated GIF export; font search and recents; 3D mirror/straighten pose | 2.1.0 (2023-07-27) | v2 notes | high |
| Parent/child 3D hierarchy; VRM import; Sub View image list (EX/PRO); Watermarks on export; Smart Smoothing as resample method | 2.2.0 (2023-09-26) | v2 notes | high |
| Perceptual gradient mixing; text kerning and justify; mesh transform multi-point selection; Parallel projection 3D; Rectangle balloon (EX) | 2.3.0 (2023-12-05) | v2 notes | high |
| Color Match (EX/PRO) | 3.0.0 (2024-03-14) | https://www.clipstudio.net/en/dl/release_note/v3/ | high |
| Layer Comps (EX) | 3.0.0 | v3 notes | high |
| Filters: Chromatic aberration, Noise, Retro film, Pencil drawing, Normal map | 3.0.0 | v3 notes | high |
| Circular text, OpenType features, text compatibility version | 3.0.0 | v3 notes | high |
| WebP import/export; export presets; audio scrubbing; 3D shape keys; Posemaniacs import | 3.0.0 | v3 notes | high |
| Templates for gradient map and tonal correction layers | 3.0.0 | v3 notes | high |
| Tool Slider palette; Rough pen, Rough rectangle, Rough balloon; BVH import (EX/PRO); 3D shadows onto other objects; timelapse quality/size mode | 3.1.0 (2024-07-30) | v3 notes | high |
| Quick Filters and Quick Effects in the Studio Mode Filter menu; mobile movie import; VRM expressions | 3.2.0 (2024-12-03) | v3 notes | high |
| Puppet warp (EX/PRO) | 4.0.0 (2025-03-12) | https://www.clipstudio.net/en/dl/release_note/v4/ | high |
| Snap to objects and View > Snap submenu (EX/PRO) | 4.0.0 | v4 notes | high |
| Magnetic lasso (EX/PRO); lasso selection mode | 4.0.0 | v4 notes | high |
| Lens blur; Crystallize | 4.0.0 | v4 notes | high |
| Drawing on 3D primitives and figures | 4.0.0 | v4 notes | high |
| Sub Tool palette multi-select; .sutg group export/import (EX/PRO) | 4.0.0 | v4 notes | high |
| Create new from movie | 4.0.0 | v4 notes | high |
| Velocity settings graph in brush dynamics | 4.1.0 (2025-07-29) | v4 notes | high |
| Dynamic Eraser sub tool | 4.1.0 | v4 notes | high |
| Delete empty or hidden layers; object-aligned bounding box; timeline multi-keyframe edit | 4.1.0 | v4 notes | high |
| Anti-aliasing "Speed and Quality"; High-quality running-color mixing; stroke-preview (lag reduction) preference | 4.2.0 (2025-12-02) | v4 notes | high |
| Per-canvas color profile in New and Canvas Properties; wide color gamut (mobile) | 4.2.0 | v4 notes | high |
| Auto-install of downloaded ASSETS materials | 4.2.0 | v4 notes | high |
| Smart Shape | 5.0.0 (2026-03-11) | https://www.clipstudio.net/en/dl/release_note/latest/ | high |
| 3D hand model; Height fog | 5.0.0 | latest notes | high |
| Tonal correction across multiple layers | 5.0.0 | latest notes | high |
| Save recovery data for every operation | 5.0.0 | latest notes | high |
| Hide all visible draft layers (EX/PRO); batch export textures (EX); CMYK and ICC in PDF export (EX) | 5.0.0 | latest notes | high |
| Palette renames (Tool Group, Tool Settings, Advanced Tool Settings); pop-up related palettes | 5.0.0 | latest notes | high |
| Watercolor edge "Vivid" style (brush and layer border effect) | 5.1.0 (2026-03-13) | latest notes | high |
| Glow, Inflate, Drop shadow filters | 5.1.0 | latest notes | high |
| Lab color sliders | 5.1.0 | latest notes | high |
| Edge border effect "Reflect opacity" (EX/PRO); Smart Shape menu commands | 5.1.0 | latest notes | high |
| Painting directly on 3D characters; Follow-camera lights | 5.1.0 | latest notes | high |
| Freeform gradient (PRO/EX) | not found in any release notes read; appears in the ©2020 Tool Setting Guide, so **probably in 1.x** | toolguide Freeformgradient.htm | unverified |
| "Dynamic brushes" as a Ver. 3.0 feature | not found in v3 notes | v3 notes | unverified / likely false |
| Picking color from tonal-correction-adjusted result as a 2.0 feature | not mentioned in v2 notes or the 2.0 announcement | news 202303 | unverified / not found |
| "Face/body posing from photo" | Pose Scanner (body) was a 1.8.6 tech preview, so it is **not** new. The 2.0 addition is the Hand Scanner. | v1 notes; v2 notes | high (for this correction) |

How the prompt's examples turned out:
- **Liquify is not post-1.13.2.** Liquify itself is 1.x (1.11.6). Only its multi-layer use came in 2.0.
- **The Color Mixing palette is 1.x** (1.12.0).

## Notes on 1.13.2 / archived manual

**1.13.2 release:**
- Released **2023-01-31**. It is the final Ver. 1 release, and the Ver. 1 notes page tops out there (https://www.clipstudio.net/en/dl/release_note).
- New in 1.13.2: the Auto Action "Edit set" dialog became resizable (EX/PRO), and license wording changed in Copy Diagnostics.
- Fixes in 1.13.2: 3D-layer compatibility-mode saves, sluggishness with the Object sub tool, JPEG export preview, iPad pen/gesture issues, and Teamwork/Story Editor issues (EX).

**1.13.1** (2023-01-18): fixed a Windows launch crash.

**1.13.0** (2022-12-06):
- Compatibility-mode save (Preferences > File) so 3D-layer files open in Ver. 1.5 on mobile.
- Copy Diagnostics button.
- Apple Pencil hover.
- Auto Action "Edit set" (EX/PRO).
- PDF "import annotations as image" (EX).
- xdts export of 2D camera info and a Toei exposure-sheet camera spec (EX).
- Windows Input frequency default changed to "Prefer quality".

**Most recent large 1.x feature drops before that:**
- 1.12.0 (2022-05-26): companion mode on all platforms, Color Mixing palette, PSD text-layer export, and faster Liquify.
- 1.12.7 (2022-09-28): redesigned eyedropper, 3D camera switching, glTF import, and Embed ICC on webtoon export.

**Archived 1.x manual:**
- CELSYS hosts a **Ver. 1 user guide PDF** (84.3 MB): https://vd.clipstudio.net/clipcontent/paint/manual/EN_CSP_UserGuide_Ver1.pdf. It is linked from https://help.clip-studio.com/en-us/First_Topic.htm.
- PDFs also exist for Ver. 2, 3 and 4 (…/EN_CSP_UserGuide_Ver2.pdf, …_Ver3.pdf, …_Ver4.pdf).
- I did not download the PDF (it is large), so I have not confirmed which 1.x point release it describes. **That PDF should be the primary reference for 1.13.2-era UI names**, for example Sub Tool Detail rather than Advanced Tool Settings.
- The older HTML Tool Setting Guide (clip-studio.com/site/gd_en/csp/toolguide/…, ©2020) is still online and predates 2.0 for at least some pages.

**Installer:** 1.13.2 is still downloadable for Ver. 1 perpetual licenses on Windows and macOS (https://support.clip-studio.com/en-us/faq/articles/20230004). This matters for the Wine observation plan; only the maintainer's licensed copy should be used.

**Gaps:**
- The tail of the Ver. 1 notes (versions before 1.6.2, roughly the last 60k characters) was not read.
- Several manual detail pages were not opened: selection sub tools, gradient settings, text detail, frame borders, the preferences list, and tablet driver settings. Items from those areas are marked [GK].
