# U-002 Menu bar
- Status: **observed** (top level and every submenu one level deep)
- See README. Labels are recorded as identifiers (short UI labels, CSP's spelling and
  capitalisation as shown); Brushye uses its own wording. "(X)" letters are CSP's access keys.
  Greyed = disabled with the observation document open (800×800, one raster layer, nothing
  selected, no undo history).

Top level, left to right: **File, Edit, Animation, Layer, Select, View, Filter, Window, Help**.
Menus drop down from the menu bar; item rows are 19 px, separators ~8 px; shortcuts are
right-aligned; submenus are marked with a chevron at the right; checked items show a check mark
in the left gutter.

## File
New… (Ctrl+N) · Create New from Clipboard [greyed] · Open… (Ctrl+O) · Recent(F) ▸
— Close (Ctrl+W) · Save (Ctrl+S) · Save as… (Shift+Alt+S) · Save Duplicate(R) ▸ · Revert(G)… [greyed]
— Export (Single Layer) ▸ · Export animation(Y) ▸ · Export webtoon… · Export vectors… [greyed]
— Timelapse ▸ — Import ▸ — Print Settings(D)… · Print… (Ctrl+P)
— Preferences… (Ctrl+K) · Privacy Settings… · Command Bar Settings… · Shortcut Settings… (Ctrl+Shift+Alt+K) · Modifier Key Settings… (Ctrl+Shift+Alt+Y)
— Manage fonts… — Connect to smartphone… · Tab-Mate Controller ▸ · CLIP STUDIO TABMATE ▸ · Pen Pressure Settings(J)… · QUMARION(T) ▸
— Open CLIP STUDIO… — Quit CLIP STUDIO PAINT(X) (Ctrl+Q)

Submenus:
- Recent: numbered list of recent files with full paths.
- Save Duplicate: .clip, .bmp, .jpg, .png, .tif, .tga, .psb, .psd.
- Export (Single Layer): .bmp, .jpg, .png, .tif, .tga, .psb, .psd.
- Export animation: Image sequence…, Animated GIF…, Animated sticker (APNG)…, Movie… — Audio… (all greyed: no timeline).
- Timelapse: Record timelapse, Export timelapse… [greyed].
- Import: Image…, Pattern from Image…, Create file object…, Movie…, 3D Data(M)…, Vector…, Audio… [greyed] — Pose Scanner (image) (Technology preview)(B)… — Scan…, Select Scan Device….
- Tab-Mate Controller: Tab-Mate Controller Settings…, Quick Menu Settings… (greyed).
- CLIP STUDIO TABMATE: CLIP STUDIO TABMATE Settings…, Tool Rotation Settings….
- QUMARION: Use QUMARION — Import Calibration Information, Export Calibration Information(S) (greyed).

## Edit
Undo (Ctrl+Z) · Redo (Ctrl+Y) [both greyed: no history]
— Cut (Ctrl+X) · Copy (Ctrl+C) · Copy vectors as SVG(K) [greyed] · Paste (Ctrl+V) [greyed] · Paste to shown position (Ctrl+Shift+V) [greyed]
— Delete (Del) · Delete Outside Selection (Shift+Del) [greyed] · Smart Smoothing… · Fill (Alt+Del) · Advanced Fill… · Colorize (Technology preview) ▸ · Convert to drawing color(H) · Outline Selection(G)… [greyed] · Convert brightness to opacity
— Register Material(J) ▸ · Tonal Correction(D) ▸ · Transform ▸
— Change Image Resolution… · Change Canvas Size… · Crop(Z) [greyed] · Rotate/Flip canvas ▸ · Canvas Properties…
— Clear Memory ▸ — Pick screen color(X)… · Hide windows and pick screen color(Y)…

Submenus:
- Colorize: Colorize all(F), Use hint image and colorize, Use more advanced settings….
- Register Material: Image…, Template…, Animation… [greyed].
- Tonal Correction: Brightness/Contrast…, Hue/Saturation/Luminosity… (Ctrl+U), Posterization…, Reverse Gradient (Ctrl+I), Level Correction…, Tone Curve…, Color balance…, Binarization…, Gradient map….
- Transform: Scale/Rotate… (Ctrl+T), Scale…, Rotate… — Free Transform(D)… (Ctrl+Shift+T), Distort…, Skew(P)…, Perspective… — Flip Horizontal…, Flip Vertical… — Mesh Transformation… — OK(F)…, Cancel… (greyed).
- Rotate/Flip canvas: Rotate by 90 Degrees Clockwise, Rotate 180, Rotate by 90 Degrees Counterclockwise, Flip Horizontal, Flip Vertical.
- Clear Memory: History.

## Animation
New animation layer ▸ · New animation cel [greyed] · Edit track ▸ · Animation curve(F) ▸
— Play/Stop [greyed] · Playback settings ▸ · Move frame ▸ — Label ▸ · Timeline ▸
— Show animation cels ▸ · Light table ▸ · Show 2D camera frames · Show camera path · Show all field guides (last three greyed)

Submenus (mostly greyed without a timeline):
- New animation layer: Animation folder, 2D camera folder, Audio.
- Edit track: Delete, Cut, Copy, Paste — Enable keyframes on this layer, Add keyframe, Switch keyframe to hold interpolation(Q), Switch keyframe to linear interpolation, Switch keyframe to smooth interpolation, Edit layers with active keyframes, Delete all keyframes — Assign cel to frame…, Assign multiple cels(B)…, Create all supported cels(N), Rename in timeline order(X), Rename in layer order, Select previous cel, Select next cel — Combine clips, Divide clip, Set as first displayed frame, Set as last displayed frame.
- Animation curve: Graph Editor(F), Snap to X axis, Snap to Y axis, Unpair handles(B).
- Playback settings: Play in real time, Play all frames — Loop play, Render 2D camera, Render before starting playback — Prefer speed, Prefer quality.
- Move frame: Go to start, Go to end, Go to previous frame, Go to next frame, Previous keyframe, Next keyframe, Go to specified frame…, Go to timeline label….
- Label: Create timeline label…, Delete timeline label(R), Create track label(D)…, Delete track label, Create inbetween track label "○", Create inbetween track label "●" (J).
- Timeline: Enable timeline — New timeline…, Go to previous timeline, Go to next timeline, Change settings…, Change frame rate…, Manage timeline(O)… — Insert frame…, Delete frame….
- Show animation cels: Show edited cel only, Enable onion skin, Onion skin settings… — Check cel motion by key input, Check surrounding cels by key input(A).
- Light table: Enable light table(V), Lock current animation cel as editing target(F) — Register selected layer, Select and register file, Register layer copied from another canvas, Register onion skin images — Deregister selected image from light table, Deregister all images from light table — Move canvas to center….

## Layer
New Raster Layer (Ctrl+Shift+N) · New Layer ▸ · New Correction Layer(J) ▸ · New Layer Folder · Create folder and insert layer (Ctrl+G) · Ungroup Layer Folder(S) (Ctrl+Shift+G) [greyed] · Duplicate Layer · Delete Layer [greyed]
— Layer Mask ▸ · Ruler/Frame ▸ · File object(X) ▸ · Layer Settings(P) ▸ · Selection from Layer ▸ · Rasterize [greyed] · Convert Layer(H)…
— Transfer To Lower Layer(C) · Merge with layer below (Ctrl+E) · Merge selected layers (Shift+Alt+E) · Merge visible layers (Ctrl+Shift+E) · Merge visible to new layer(X) [all greyed: one layer] · Flatten image
— Layer Order ▸ · Change Selected Layer(I) ▸

Submenus:
- New Layer: Raster layer…, Vector layer…, Gradient, Fill(M)…, Tone…, Frame Border folder(C)…, 3D layer(H) — Paper….
- New Correction Layer: Brightness/Contrast…, Hue/Saturation/Luminosity…, Posterization…, Reverse Gradient, Level Correction…, Tone Curve…, Color balance…, Binarization…, Gradient map….
- Layer Mask: Mask Outside Selection(V), Mask Selection(D) — Delete Mask, Apply Mask to Layer — Enable Mask, Show Mask Area, Link Mask to Layer.
- Ruler/Frame: Selection from Ruler, Draw along ruler…, Ruler from vector — Delete Ruler, Show Ruler(B), Link Ruler to Layer — Create Perspective Ruler… — Divide frame border equally(T)…, Combine frames(U)… — Show in All Layers, Show in Same Folder, Show Only When Editing Target — Link guide to ruler.
- File object: Convert layer to file object…, Paste as file object, Change file of file object…, Open file of file object…, Open folder of file object…, Update file object, Update all file objects.
- Layer Settings: Set as Reference Layer, Set as Draft Layer, Clip to Layer Below (Ctrl+Alt+G), Lock Layer, Lock transparent pixels — ✓Show Layer(V), Change Layer Name… — Display palette color in parent folder, Display palette color on canvas — Advanced Settings….
- Selection from Layer: Create Selection, Add Selection, Delete Selection, Select Overlapping Area(M).
- Layer Order: Top, Up, Down, Bottom.
- Change Selected Layer: Layer Above(U) (Alt+]), Layer Below (Alt+[).

The layer list's right-click menu repeats the Layer menu from **New Raster Layer** down to
**Flatten image** (same groups and separators, without shortcuts shown).

## Select
Select All (Ctrl+A) · Deselect (Ctrl+D) · Reselect (Ctrl+Shift+D) · Invert selected area (Ctrl+Shift+I)
— Expand selected area… · Shrink selected area(U)… · Blur border… — Select Color Gamut…
— Select Overlapping Vectors · Select Vectors Within Area — Quick Mask · Convert to Selection Layer · Convert Selection Layer to Selection(F)
(Deselect … Blur border and the vector/selection-layer items greyed: nothing selected.)

## View
Rotate/Flip(V) ▸ · Zoom In (Ctrl+Num+) · Zoom Out (Ctrl+Num−) · 100% (Ctrl+Alt+0) · 200% · Fit To Screen(W) (Ctrl+0) · Reset Display (Ctrl+`) · Fit to Navigator · Print size
— ✓Scroll Bar · Selection Border [greyed] · ✓Selection Launcher · Selection Launcher Settings… [greyed] · ✓Transform launcher · ✓Text launcher(K) · ✓Object Launcher(3)
— Grid · Ruler Bar (Ctrl+R) · Grid/Ruler Bar Settings… · Crop mark/Default border [greyed] · Crop mark/Default border settings… · Safety margin(4) [greyed] · On-screen area (webtoon)(5) · On-screen area settings (webtoon)(6)… · Paper(Y) [greyed] · Show Tone Area(J) ▸ · Show vector paths(4) ▸
— Show playback fps [greyed] — ✓Snap to Ruler (Ctrl+1) · ✓Snap to Special Ruler (Ctrl+2) · Snap to Grid (Ctrl+3) · Toggle special ruler snapping (Ctrl+4) [greyed] — Color profile(Q) ▸

Submenus:
- Rotate/Flip: Reset Rotation/Inversion, Reset Rotation, Rotate 90, Rotate 180, Rotate 270 — Flip Horizontal, Flip Vertical — Rotate Left (−), Rotate Right (^).
- Show Tone Area: Show Selected Tone Area, Show All Tone Areas — Include Image Material Area.
- Show vector paths: Show vector paths on selected layers, Show all vector paths on vector layers — Show control points at beginning and end, Show all control points.
- Color profile: Preview Settings…, Preview.

## Filter
Blur ▸ · Correction(L) ▸ · Distort ▸ · Effect ▸ · Render ▸ · Sharpen ▸
- Blur: Blur, Blur (strong)(F), Gaussian blur…, Motion blur…, Radial blur…, Smoothing.
- Correction: Adjust line width…, Remove dust….
- Distort: Curved surface…, Fish-eye lens(D)…, Pinch…, Polar coordinates…, Ripple(Z)…, Twirl(S)…, Wave(F)…, ZigZag(W)….
- Effect: Artistic…, Mosaic…, Remove jpeg noise.
- Render: Perlin noise….
- Sharpen: Sharpen, Sharpen more, Unsharp mask….

## Window
Canvas ▸ — Workspace ▸ — ✓Command Bar · Quick Access
— ✓Tool · ✓Sub Tool: <tool> · ✓Tool property: <sub tool> · ✓Brush size: <sub tool> · Sub Tool Detail
— ✓Color Wheel · Color Slider · Color Set · Intermediate Color(Z) · Approximate Color(F) · Color History(Q) · Color Mixing
— ✓Layer · Search Layer(F) · ✓Layer Property — Animation cels · Timeline(X)
— ✓Navigator · Sub View — History · Auto Action · Information(Y) · Item bank — Material ▸
— Hide All Palettes (Tab) · Hide Title Bar(J) (Shift+Tab) · ✓Always Show Tab in Canvas(K) · Palette dock ▸
(A check mark = palette currently visible; tabbed palettes behind another tab are unchecked.)

Submenus:
- Canvas: New Window, Cascade [greyed], Tiled — Next canvas (Ctrl+Tab), Previous Canvas (Ctrl+Shift+Tab) — numbered list of open canvases with ✓ on the active one.
- Workspace: Reset to Default(I) — Register Workspace…, Register workspace as material…, Search for workspace material(s), Manage Workspace… [greyed], Workspace import settings…, Reload workspace(T) [greyed] — Illustration, Comic, By category.
- Material: Material: Color pattern, Background, Monochromatic pattern, Basic, Manga material, Effect line, Image material, Illustration, 3D, Pose.
- Palette dock: Fix the width of palette dock, Lock palette height, Lock palette position(D) — Display Tool as pop-up, Display Sub Tool / Tool property / … as pop-up, Display Timeline as pop-up, Display Quick Access / Material … as pop-up, Display Navigator / Sub View / … as pop-up.

## Help
Training and Tutorials(H) (F1) · Getting Started… · Instruction Manual · Support — Get License(R)… · Review/Change License… — File Associations… — About <app>… · Version information….

## What Brushye should match
- The nine top-level menus in this order, with Brushye's equivalents placed in the same menu
  and the same group (separator) as CSP's item, and the same shortcuts where Brushye has the
  feature (Ctrl+N/O/W/S, Shift+Alt+S, Ctrl+Z/Y, Ctrl+X/C/V, Del, Alt+Del, Ctrl+T, Ctrl+Shift+N,
  Ctrl+G, Ctrl+E, Ctrl+Shift+E, Ctrl+A/D, Ctrl+Shift+D/I, Ctrl+Num+/−, Ctrl+Alt+0, Ctrl+0,
  Ctrl+`, Ctrl+R, Ctrl+1/2/3, Ctrl+K, Ctrl+Q, Ctrl+U, Ctrl+I, Ctrl+Tab, Tab, Shift+Tab, F1).
- Items for features Brushye doesn't have yet are either omitted or shown disabled; they never
  appear in a different menu. Disabled items use the disabled text colour (U-009).
- Window menu lists every palette with a check mark when it's visible, and has the Workspace
  submenu with Reset to Default.
- The layer list context menu mirrors the Layer menu's first four groups.
