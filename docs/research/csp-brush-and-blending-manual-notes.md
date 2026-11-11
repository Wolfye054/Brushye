# CSP brush engine and layer blending: what the manual says

Research date: 2026-11-11. Every statement about CSP is paraphrased from the official
CELSYS manual (help.clip-studio.com) or an official CELSYS page. Formulas come from
W3C Compositing and Blending Level 1 (https://www.w3.org/TR/compositing-1/) and the
ISO 32000 / Adobe PDF blend-mode definitions. "NSM" means **not stated in manual — needs observation**.

## Important caveat: manual version
The online manual at https://help.clip-studio.com/en-us/manual_en/ documents the
**current** CSP (Ver. 2.x–4.x+), not 1.13.2. The old Ver. 1.x "Tool Setting Guide" and
"User Guide" URLs (www.clip-studio.com/site/gd_en/csp/toolguide/..., .../userguide/...)
now 301-redirect into the current manual, so a frozen 1.x manual isn't available online.
Some settings described below are explicitly from later versions (marked **[post-1.x]**).
Every option list has to be confirmed in the 1.13.2 UI before a spec moves past draft.
The manual gives almost **no numeric ranges or defaults** for Sub Tool Detail settings.
Treat all ranges and defaults as NSM.

Main manual sources:
- Settings glossary index: https://help.clip-studio.com/en-us/manual_en/810_subtools/810_subtools.htm
- Customizing brush tools: https://help.clip-studio.com/en-us/manual_en/240_brushes/Customizing_brush_tools.htm
- Glossary pages are alphabetical: .../810_subtools/A.htm, B.htm, C.htm, D.htm, I.htm, S.htm, T.htm, W.htm, Number.htm

---

## A. Sub Tool Detail settings for raster brushes

Current versions call these palettes "Tool Settings" and "Advanced Tool Settings".
1.x called them "Tool Property" and "Sub Tool Detail".

### A.1 Dynamics (effect source) and pressure graph
Source: Customizing_brush_tools.htm.
- Settings such as Brush size, Opacity, Brush density, Thickness, Particle size, Texture
  density, Amount/Density of paint and Blend with sub color have a **Dynamics button** at
  their right edge. The icon changes when dynamics are on, and a slashed box means off.
- Input sources: **Pen pressure, Tilt, Velocity, Random**. The Angle setting instead uses
  direction-type sources (see A.4).
- The dialog has a **minimum (or maximum) value slider** and a **pen pressure graph**
  ("Pen pressure settings") for fine input→output adjustment.
- NSM: the graph's axes, how control points are added, moved or removed, the maximum
  number of points, the interpolation (linear or spline), how the minimum value maps
  (output = min + (1−min)·curve(p)?), and how several sources combine (multiply?).
  An official CELSYS tips article gives one example: brush size 100 with pressure minimum
  50 gives size 50 at zero pressure (https://tips.clip-studio.com/en-us/articles/563 ;
  confirm it's the official account). That points to a linear min→100% mapping, but the
  formula still needs observation.
- NSM: the "Brush size effect source" wording from 1.x. The current manual uses "Dynamics".
- **Correct velocity input** (Stroke and Dot pen categories) changes how stroke speed is
  computed for Velocity dynamics. The options are NSM.

### A.2 Global pen pressure: File > Pen Pressure Settings / "Adjust pen pressure"
Source: https://help.clip-studio.com/en-us/manual_en/240_brushes/Adjusting_pen_pressure.htm
- A wizard runs on first launch. You draw strokes with varying pressure, then press
  "Check adjusted settings". You can test-draw, nudge with **Stronger / Lighter**, or edit
  the **Pen pressure settings graph** directly. **Done** applies it to **all tools**.
- Reopen it from File > Pen Pressure Settings (1.x: File > "Adjust pen pressure detection
  level"; check the exact 1.13.2 menu name).
- Reset to default: support FAQ https://support.clip-studio.com/en-us/faq/articles/20190226
- NSM: the curve math, and how the global curve composes with per-setting dynamics curves
  (global applied first?).
- Preferences > Tool: "Reduce pen pressure when drawing with finger/mouse" (speed-based
  taper at start/end), "Use stroke preview" [likely post-1.x], and "Disable drawing with
  transparency on transparency locked layers" (transparent color becomes white).
  Source: https://help.clip-studio.com/en-us/manual_en/720_preferences/Preferences.htm

### A.3 Brush size category
Source: B.htm.
- **Brush size**: a slider with Dynamics (pressure, velocity …). Units and range NSM. The
  1.x UI shows px or mm depending on the Preferences > Ruler/Unit setting, so confirm in
  the UI. Max (commonly 2000 px in 1.x) is NSM.
- **Specify by size on screen**: on keeps the on-screen size constant when zooming (size
  matches 100% view).
- **At least 1 pixel**: thin parts (for example from Starting and ending) never drop
  below 1 px.
- **Brush preview**: shows size and tip, including settings from the other categories.
- NSM: subpixel size handling and how a fractional size rasterizes.

### A.4 Brush tip category
Source: B.htm.
- **Tip shape**: **Circle** or **Material** (image). Several materials make a multi-tip
  brush. You can reorder tips by dragging. A **Repeat method** in Stroke sets their order.
- **Hardness** (Circle only): lower values blur the edge more. The falloff profile NSM.
- **Thickness**: squashes the tip along the axis chosen by **Direction**
  (Horizontal/Vertical). Has Dynamics.
- **Angle** (named "Direction" in 1.x UI; confirm): 0–360°. Its dynamics sources are **None,
  Direction of pen** (pen tilt azimuth; the pen must report it), **Pen tilt**
  (barrel rotation, per the manual wording; confirm), **Direction of line, Random**
  (adjustable amount, combinable with the others).
- **Flip horizontal / Flip vertical** (Material only): **Off, Flip, Random, Invert on
  reversed strokes** (the last is for Repeat method = Reverse).
- **Brush density**: per-stamp opacity, with Dynamics. This differs from Ink > Opacity:
  density is per stamp and accumulates, while opacity caps the whole stroke. That
  interpretation is NSM and needs observation.
- **Adjust brush density by gap**: when Stroke gap is narrow, lowers density so apparent
  opacity stays about constant. The formula is NSM.

### A.5 Ink category
Sources: I.htm, https://help.clip-studio.com/en-us/manual_en/180_layers/Blending_modes.htm
- **Opacity**: has Dynamics. You can't set dynamics when Color mixing is Blend or Running color.
- **Blending mode** (brush's own combine mode with existing pixels on the same layer):
  unavailable when Color mixing = Blend/Running color, and not on vector layers.
  The list is all the layer modes (B.1) **plus tool-only modes**:
  - **Black burn**: darkens like underexposure. Not applied where the existing pixel is
    transparent.
  - **White burn**: lightens. Not applied where the existing pixel is transparent.
  - **Erase**: removes the existing color.
  - **Erase (Compare)**: erases where the new color's opacity is less than the original's,
    after subtracting from 100. The wording is ambiguous, so observe it.
  - **Compare density**: paints only where the new opacity exceeds the existing one.
    This is the "max alpha" behavior used by many pens.
  - **Background** ("Behind"): places the new color under the existing pixels.
  - **Replace alpha**: overlays the color and sets the existing pixels' opacity to the
    tool opacity.
  - The order of the list in 1.13.2 and any "Replace" mode name are NSM. Read them from the UI.
- **Color mixing**: off, or **Blend** (mixes the drawing color with all colors the brush
  passes over), **Running color** (mixes only the drawing color with passed-over
  color), **Smear** (mixes and smears). Blend and Running color can paint transparency.
  Whether Smear exists in 1.13.2 is NSM.
- **Amount of paint**: the RGB mixing ratio of the brush color against existing color. Higher
  means more brush color. Has Dynamics.
- **Density of paint**: the alpha mixing ratio. Blend/Running color only. Has Dynamics.
- **Color stretch**: how far the picked-up color is dragged before mixing in.
- **[post-1.x]** Blending quality (Running color; "Normal" = pre-4.2 behavior), Intensity
  of blur, **Mixing mode** (Standard = Ver. 1 method; Perceptual added after Ver. 2),
  Brightness correction (5 levels). For 1.13.2, model **Standard** only.
- Memo: some settings are unavailable on vector or Monochrome layers.
- NSM: every color-mixing formula, the pickup sampling area, and the update rate.
  This is a priority for observation.

### A.6 Anti-aliasing
Source: A.htm.
- Levels: **None, Weak, Middle, Strong**. The default per tool is NSM, and so is what
  each level does to the tip edge (pixel width of the AA ramp).
- **Speed and Quality** (Automatic / High quality / Low quality) applies only when Color mixing
  is Smear [post-1.x].
- Fill and Gradient tools have only on/off. A Monochrome canvas has no anti-aliasing
  (tips article https://tips.clip-studio.com/en-us/articles/576).

### A.7 Stroke
Source: S.htm.
- **Gap / Interval**: **Narrow, Normal, Wide, Fixed** (Fixed shows a numeric slider,
  optionally with Dynamics). The manual lists no "None" option. The spacing math (percent
  of tip size?) and the 1.x label "Gap" are NSM.
- **Continuous spraying**: keeps stamping while the pen is stationary. Slow strokes give
  darker lines. Not available with Post correction.
- **Ribbon**: deforms the tip along the stroke. Not available with Spray or the Circle tip.
- **Repeat method**: **Repeat, Reverse, Do not repeat, Random, One time only,
  One random cycle** (some may be post-1.x).
- **Blend brush tips with Darken**: overlapping stamps show the darker color, like a
  min-combine within a stroke. Presence in 1.13.2 is NSM.
- **Correct velocity input**: see A.1.

### A.8 Spraying effect
Source: S.htm.
- On/off; **Particle size** (Dynamics), **Brush size** link (particle size follows
  brush size), **Particle density** (Dynamics), **Spray deviation** (higher values cluster
  more toward the center), **Direction of particle**: None / Direction of line /
  Direction of whole spray / Spray toward center / Random.
- The distribution function (Gaussian?) and RNG seeding are NSM. Replay determinism
  matters for Brushye.

### A.9 Texture
Source: T.htm.
- **Texture** (paper material picker), **Texture density** (Dynamics), **Invert texture**
  (the "reverse density" control), **Emphasize density** (the "highlight" control),
  **Scale ratio**, **Rotation angle**, **Brightness**, **Contrast** (the last two may be
  post-1.x).
- **Texture mode**: **Normal, Multiply, Subtract, Compare, Outline, Overlay, Color dodge,
  Hard mix, Height**. Outline emphasizes texture outlines at soft edges. Height emphasizes
  texture only at stroke edges. None of these mode formulas is published (NSM).
- **Apply by each plot**: on applies the texture per stamp, so overlap darkens the
  center. Off applies it once per stroke.
- Texture anchoring (canvas-space vs stroke-space) is NSM.

### A.10 2 - Brush shape (dual brush)
Source: Number.htm. **[may be post-1.x: dual brush appeared in a 1.x update, so check
whether 1.13.2 has it]**
- **Dual brush** on/off; **Mode**: Normal, Multiply, Add (Glow), Subtract, Darken,
  Lighten, Screen, Overlay, Color Dodge, Color Burn, Linear Burn, Hard Mix,
  Height (Linear) (like Subtract, but with darker density).
- **Apply RGB value**: on uses the second tip's color. Off uses only its alpha.
- **Brush size** (Dynamics), **Link to main brush size**, **Anti-aliasing** (None/Weak/
  Middle/Strong), presets. Sub-categories 2 - Brush tip / 2 - Spraying effect / 2 - Stroke /
  2 - Texture mirror the primary ones.

### A.11 Watercolor edge
Source: W.htm.
- On/off with a **width** slider; **Opacity** (stronger border); **Darkness** (lower
  saturation, darker edge); **Process after brush stroke** (applies after pen-up);
  **Blurring width** (only with Process after brush stroke).
- Not available on vector or Monochrome layers. The algorithm is NSM.

### A.12 Starting and ending
Source: S.htm.
- Applies to selectable targets with per-target minimum (e.g. **Brush size, Brush
  density**, opacity …; the list varies by tool).
- **How to specify**: **Specify length** / **By percentage** / **Fade** (the value falls to
  the minimum and holds; Starting is unavailable with Fade).
- **Starting** / **Ending** range sliders. The ramp grows from min to 100% at the start and
  shrinks toward min at the end. The ramp shape (linear?) and units of length (px?) are NSM.
- **Starting and ending by speed**: slower strokes give a milder taper.
- How ending works before pen-up (lookahead or post-pass?) is NSM.

### A.13 Correction
Source: C.htm.
- **Sharp angles**; **Stabilization** (higher is smoother. Range in 1.x UI likely 0–100,
  NSM); **Adjust by speed** with **Stabilization mode**: "Increase stabilization when
  drawing slowly" (disabled if Stabilization ≥ 30) / "Reduce stabilization when drawing
  quickly" (mode selector probably post-1.x).
- **Post correction** (strength; sub-options **Adjust by speed**, **Adjust by scale**),
  **Bezier curve** (quadratic Bézier vs spline output) [probably post-1.x], **Taper**
  [post-1.x], **Enable snapping**, **Vector magnet** (vector layers only; strength
  slider), **Snap to default border**.
- All filter algorithms are NSM.

### A.14 Color Jitter (relevant to color determinism)
Source: C.htm. Change brush tip color (hue/sat/brightness, **Blend with sub color** with
Dynamics), Randomize per stroke, Change target. It may be post-1.x in part.

### A.15 Tablet service (Windows) and macOS
Source: Preferences.htm.
- **Using tablet service**: **Wintab** (pen/display tablets; also lets pen-less tablet PCs
  switch touch/tool) vs **TabletPC** (Windows Ink; tablet PCs). Support says to restart after
  changing it (https://support.clip-studio.com/en-us/faq/articles/20190023).
- **Coordinate detection mode**: "Use mouse mode in tablet driver settings", "Use more
  accurate coordinate detection" (tablet PCs).
- **Tablet operation area** (Wintab only): set the area with the app; operation area on
  monitor (all or one); on tablet (keep aspect or whole surface).
- **Input frequency** (speed vs quality) [may be post-1.x]. macOS: **Priority for process**
  1–6 and **Precision** (Prefer quality / Recommended / Prefer speed).
- Pressure resolution and raw→normalized mapping are NSM.

---

## B. Layer blending

Source: https://help.clip-studio.com/en-us/manual_en/180_layers/Blending_modes.htm

### B.1 Mode list in manual order (current manual)
The prompt's assumed list is **mostly right but out of order**. The manual's order is:
1 Normal, 2 Darken, 3 Multiply, 4 Color burn, 5 Linear burn, 6 Subtract, 7 Lighten,
8 Screen, 9 Color dodge, 10 Glow dodge, 11 Add, 12 Add (Glow), 13 Overlay,
14 Soft light, 15 Hard light, 16 Difference, 17 Vivid light, 18 Linear light,
19 Pin light, 20 Hard mix, 21 Exclusion, 22 Darker color, 23 Lighter color,
24 Divide, 25 Hue, 26 Saturation, 27 Color, 28 Brightness.
**Through** (pass-through) is for folders only. It sits at the top of the dropdown when a
folder is selected (Layer_folders.htm; tips 4081).
The 1.13.2 dropdown order is NSM. The manual's text order may not match the
dropdown (Darker/Lighter color may sit next to Darken/Lighten in the UI). Read it from 1.13.2.

### B.2 Per-mode description (manual) and formula (published)
Notation (W3C): Cb = backdrop, Cs = source (upper layer), channels in [0,1].
"W3C" means the formula is defined in Compositing and Blending Level 1 §9 (same as
PDF 32000-1 §11.3.5). NON-SEPARABLE modes use W3C Lum/Sat/ClipColor with
Lum = 0.3R + 0.59G + 0.11B.

| # | CSP mode | Manual says (paraphrase) | Published formula | Status |
|---|---|---|---|---|
|1|Normal|Sits on top of the layer below|B = Cs (W3C)|standard|
|2|Darken|Keeps the darker color|min(Cb,Cs) (W3C)|standard|
|3|Multiply|Multiplies colors; darker|Cb·Cs (W3C)|standard|
|4|Color burn|Darkens base, increases contrast|W3C color-burn: Cb=1→1; Cs=0→0; else 1−min(1,(1−Cb)/Cs)|standard (edge cases: observe)|
|5|Linear burn|Darkens base to blend color|Cb+Cs−1 clamped at 0 (Adobe; not in W3C)|semi-standard: observe|
|6|Subtract|Subtracts upper from base|Cb−Cs clamped at 0 (common definition; not W3C)|**CSP-specific: observe**|
|7|Lighten|Keeps the lighter color|max(Cb,Cs) (W3C)|standard|
|8|Screen|Inverse multiply; brighter|Cb+Cs−Cb·Cs (W3C)|standard|
|9|Color dodge|Lightens base, lowers contrast|W3C color-dodge: Cb=0→0; Cs=1→1; else min(1,Cb/(1−Cs))|standard (alpha behavior: observe)|
|10|Glow dodge|"Stronger effect than Color dodge"|**not published**|**CSP-specific: observe**|
|11|Add|Adds the color values; brighter|min(1,Cb+Cs) (Adobe "Linear Dodge (Add)")|semi-standard: observe alpha handling|
|12|Add (Glow)|"Stronger effect than Add"|**not published**|**CSP-specific: observe**|
|13|Overlay|Screen on bright base, Multiply on dark|HardLight(Cs,Cb) (W3C)|standard|
|14|Soft light|Bright upper dodges, dark upper burns; white stays white|W3C soft-light (Cs≤0.5: Cb−(1−2Cs)Cb(1−Cb); else Cb+(2Cs−1)(D(Cb)−Cb), D = sqrt or poly). Photoshop's soft light differs from W3C|**variant unknown: observe**|
|15|Hard light|Bright upper screens, dark multiplies|W3C hard-light|standard|
|16|Difference|Subtracts, combines with base|\|Cb−Cs\| (W3C)|standard|
|17|Vivid light|Color dodge + color burn by upper|Adobe: Cs≤0.5 → ColorBurn(Cb,2Cs); else ColorDodge(Cb,2(Cs−0.5)) (not in W3C)|semi-standard: observe|
|18|Linear light|Manual says "Multiply and Linear burn" (likely a translation slip: Adobe = linear dodge + linear burn)|Adobe: Cb+2Cs−1 clamped|semi-standard: observe|
|19|Pin light|Lighten where base darker, Darken where brighter|Adobe: Cs≤0.5 → min(Cb,2Cs); else max(Cb,2Cs−1)|semi-standard: observe|
|20|Hard mix|Adds RGB; sum >255 → 255, otherwise 0 (manual's wording, partly garbled)|Adobe: (Cb+Cs ≥ 1) ? 1 : 0 per channel. The threshold (≥ vs >) and alpha are unpublished|**CSP-specific: observe**|
|21|Exclusion|Like Difference, lower contrast|Cb+Cs−2CbCs (W3C)|standard|
|22|Darker color|Compares total brightness, keeps darker color|Adobe: whole color with lower luminance. Brightness metric not published|**observe metric & ties**|
|23|Lighter color|Compares total brightness, keeps lighter|as above, max|**observe**|
|24|Divide|Base×255 ÷ upper, capped at 255|Cb/Cs clamped; behavior at Cs=0 not published. The manual's worked example is internally inconsistent (gives 13 where the formula gives 25.5)|**CSP-specific: observe**|
|25|Hue|Upper hue, base saturation and brightness|W3C SetLum(SetSat(Cs,Sat(Cb)),Lum(Cb))|standard?|
|26|Saturation|Upper saturation|W3C SetLum(SetSat(Cb,Sat(Cs)),Lum(Cb))|standard?|
|27|Color|Upper hue+sat, base brightness|W3C SetLum(Cs,Lum(Cb))|standard?|
|28|Brightness|Upper brightness, base hue+sat|W3C **Luminosity** = SetLum(Cb,Lum(Cs)). Whether CSP's "Brightness" uses the W3C Lum weights, HSV V, or HSL L is **not published**|**observe**|

Formula sources: W3C https://www.w3.org/TR/compositing-1/#blending ; Adobe PDF
Reference / ISO 32000-1 §11.3.5 (Tables 136–137); Adobe Photoshop blend-mode help
(https://helpx.adobe.com/photoshop/using/blending-modes.html) only describes
Linear burn/dodge, Vivid/Linear/Pin light, Hard mix, Darker/Lighter color, Subtract and
Divide in words, so their Adobe "formulas" above are general graphics knowledge, not a spec.

### B.3 Compositing with transparency
- W3C general formula (compositing-1 §5.8, §9): first mix the blend result into the
  source by backdrop alpha, Cs' = (1−αb)·Cs + αb·B(Cb,Cs), then source-over:
  αo = αs + αb(1−αs); Co = (αs·Cs' + αb(1−αs)·Cb) / αo (non-premultiplied output).
  Whether CSP follows this for every mode is NSM.
- **Manual says nothing** about how Add, Add (Glow), Color dodge or Glow dodge treat
  semi-transparent pixels. Community and official tips say the "(Glow)" and "Glow dodge"
  variants look **stronger in semi-transparent areas** (tips 4162, community). The official
  CELSYS tips article (https://tips.clip-studio.com/en-us/articles/1178) adds:
  Add (Glow) is brighter than Glow dodge over near-black. Glow dodge "hides" light over
  near-black. Add (Glow) makes a layer look see-through, like another way of making it
  transparent. A plausible hypothesis is that the Glow variants don't scale the
  source color by its alpha in the same way, but that's speculation. **Observe: test semi-
  transparent source (α = 0, 25, 50, 75, 100%) over a fixed backdrop, and source over a
  transparent backdrop.**
- Black burn / White burn (tool modes): "not applied where the original is transparent".
- Folder compositing: a **Normal** folder composites its children in isolation, then blends
  the result as one layer. Inner modes don't reach layers below the folder.
  **Through** lets the children blend directly with the layers below
  (Layer_folders.htm). Folder opacity and masks under Through are NSM. Clipping to a
  Through folder isn't possible: the folder is treated as Normal and the clip shows red
  (support FAQ https://support.clip-studio.com/en-us/faq/articles/20190044 ;
  Other_layer_settings.htm). Preferences > Layer/Frame can make Through the default
  for new folders.

### B.4 Opacity, clipping, masks, locks, paper
- **Layer opacity**: a slider in the Layer palette/Properties bar (Using_layers.htm).
  Granularity is 0–100 integer %, as far as the UI shows (NSM). How opacity combines with
  each blend mode is NSM. W3C would multiply αs.
- **Clip to layer below**: the clipped layer is visible only where the layer directly
  below has pixels. It works on folders except Through. A new layer inserted inside a
  clipping group is clipped too (Other_layer_settings.htm). NSM: whether the clip uses
  the base's alpha as a continuous mask (expected) and how the clipped layer's blend mode
  applies (to the base only, or to the whole stack below?). Observe.
- **Layer masks** (Layer_masks.htm): CSP uses an **alpha-scale** mask. Painting with
  transparent color or the eraser hides content, and painting with any opaque color
  reveals it. Show/Enable/Link toggles, "Show mask area" in purple.
  **Mask expression > Show gradients** (partial masking) and **Threshold** may be post-1.x.
  In 1.13.2, whether masks are continuous (I believe yes) is NSM: observe.
- **Lock transparent pixels**: painting is restricted to already-painted areas
  (Using_layers.htm, Other_layer_settings.htm). Whether the existing alpha is preserved
  exactly (W3C "source-atop"-like) is NSM: observe.
- **Paper layer**: a single solid-color layer at the bottom of the stack. Its color is set in
  New dialog > **Paper color**, and it can be changed by double-clicking. You can't set a
  blend mode on it. When hidden, transparency shows as a checkerboard. Only one per canvas.
  (Creating_a_New_Canvas.htm, Other_layer_settings.htm)
- **Expression color per layer**: Color / Gray / Monochrome (Layer_properties.htm).
  Converting downward is lossy, and Monochrome uses a color threshold and an alpha threshold.

---

## C. Color depth and color management

Sources: https://help.clip-studio.com/en-us/manual_en/210_file/Creating_a_New_Canvas.htm ,
https://help.clip-studio.com/en-us/manual_en/210_file/Preview_of_color_profile.htm ,
Preferences.htm (Color conversion).
- **Bit depth**: the manual has **no bit-depth option**. "Color" expression handles
  "about 16.77 million colors" (8 bits/channel RGB + alpha). Gray has 256 levels,
  and Monochrome is black/white/transparent. Use 8-bit RGBA for 1.13.2. The internal
  precision of compositing and brush accumulation (any higher-precision intermediate
  buffers) is NSM, and needed to match exact pixel values. Observe it with
  low-opacity repeated strokes.
- **Basic expression color** (New dialog): Color / Gray / Monochrome. It sets the default
  for new layers.
- **Color profile in New dialog** (current manual): Color profile preview toggle, **Working
  profile** (defaults to the display/device profile), **Embedded profile** (RGB name;
  for CMYK you pick the source RGB). Presence in the 1.13.2 New dialog is NSM. In 1.x it
  was View > Color profile > Preview settings.
- **Color profile preview** (View > Color profile > Preview Settings): a **display
  simulation** (soft-proof). Rendering intents: Perceptual, Saturation, Relative
  colorimetric, Absolute colorimetric. Library (Windows): IccLibrary / Microsoft ICM.
  "Save on canvas" stores the settings with the file. Choosing an RGB working profile
  changes the canvas's **embedded** profile, while CMYK changes only the preview.
  Preview settings **apply to Export (Single Layer)** when saved on canvas.
- **Preferences > Color conversion**: default RGB and CMYK profiles, rendering intent,
  library, and an option to embed the Preferences RGB profile in files saved without an
  export dialog.
- **PNG and profiles**: Export dialog **Embed ICC profile** (RGB/CMYK expression only)
  embeds the canvas profile (from New dialog / canvas properties), or else the
  Preferences > Color conversion RGB profile. NSM: whether pixel values are
  **converted** on RGB export (expected: no, RGB numbers pass through unchanged) and
  whether a PNG without that box ticked carries any sRGB/gAMA/iCCP chunk. Check
  exported files with a PNG chunk dump during observation.
- **Recommendation for observation**: export with Embed ICC profile **off**, Expression
  color **RGB**, no preview profile active, scale 100%. Then confirm via chunk listing
  that there's no iCCP, and compare against known paint values (e.g. paint #808080 at
  100% and read it back).

---

## D. Export (Single Layer) dialog: PNG

Source: https://help.clip-studio.com/en-us/manual_en/210_file/Exporting_files.htm
(File > Export (Single Layer) > .png). Options in the current manual; the 1.13.2 dialog
may lack some, so confirm.
1. **Preset** (save/select setting groups) [likely post-1.x].
2. **Preview rendering result on output**: shows a preview dialog before writing.
3. **Output image**: checkboxes for extras: **draft (layers), crop marks, inner (default)
   border, text, story info, folio, camera paths, safety margins**. For pixel-exact tests
   **turn all of them off** (especially Text and Draft). The paper layer isn't an option
   here. It's controlled by its visibility (hide the Paper layer for an alpha export)
   — NSM, so observe.
4. **Export range** (EX only).
5. **Expression color**: **Auto detect appropriate color depth**, **Duotone (Threshold)**
   [1.x label: Monochrome 2 tone], **Duotone (Toning)**, **Gray**, **RGB color**,
   **CMYK** (not for PNG). Use **RGB color** for tests. "Auto" might drop to gray
   when the image is achromatic, so avoid it.
6. **Advanced color settings** (crop-mark/border colors, tone frequency).
7. **Embed ICC profile** (RGB/CMYK only). Keep it off for tests (see C).
8. **Export transparency** (PNG/WebP): **on** writes the alpha channel. **Off** exports
   transparent areas as **white**. Presence and label in 1.13.2 are NSM. Some 1.x versions
   may have exported alpha implicitly whenever the paper was hidden, so observe.
9. **Output size**: **Scale ratio from original data** (%), **Specify output size**
   (W×H + unit), or **Specify resolution** (72–1200 dpi). Use **100%** to avoid resampling.
10. **Process when scaling**: **For illustration** / **For comic** (comic offers a
    Rasterize mode: Fast / Prefer quality). Irrelevant at 100%, but set
    "For illustration" anyway.
11. Watermark / noise pattern: keep off.

Not stated: PNG bit depth (expect 8-bit RGBA or RGB), whether alpha is premultiplied or
straight (PNG requires straight, so check for darkened edges), and whether hidden
layers or reference flags affect output. All need observation.

Other references:
- Save / export overview: https://help.clip-studio.com/en-us/manual_en/210_file/Save_file.htm
- Change canvas settings: https://help.clip-studio.com/en-us/manual_en/210_file/Change_canvas_size_and_settings.htm

---

## Open questions summary (all need observation in 1.13.2)
1. Exact 1.13.2 option lists, labels and order for every category above. The manual is newer.
2. Numeric ranges and defaults for all brush settings. The manual gives almost none.
3. Dynamics curve math: minimum mapping, graph interpolation, global×per-tool composition.
4. Stamp spacing math for Narrow/Normal/Wide/Fixed. Density vs opacity accumulation.
5. Hardness falloff profile. Anti-aliasing levels.
6. Formulas for Glow dodge, Add (Glow), Subtract, Divide (Cs=0), Hard mix (threshold),
   Darker/Lighter color (metric), Brightness (Lum weights), Soft light variant, plus every
   mode's alpha handling, especially semi-transparent source/backdrop.
7. Tool-only Ink modes (Compare density, Erase (Compare), Replace alpha, Background,
   Black/White burn) exact math.
8. Color mixing (Blend/Running color) formulas.
9. Internal precision (8-bit quantization per stamp?) and dithering.
10. PNG export: iCCP/sRGB chunks, alpha, and the paper/transparency toggle in 1.13.2.
