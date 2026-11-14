# U-014 Layer palette: folders, clipping and layer masks
- Status: **observed** (screen layout from screenshots; the mask-area colour sampled from a
  screenshot, which is display-only)
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK
- Date / method / observer: 2026-11-14 / automated-Wine (Xvfb :77, XTEST) / Claude
- Screenshots: `obs-out/m10/u014_*.png` on the observation machine. They are not committed,
  since they contain CSP's artwork. See `specs/behavior/ui/README.md`.

## Setup (exact)
1. File > New (Ctrl+N): 800×800 px, 72 dpi, Color, **no paper**. Draw one pen stroke on Layer 1.
2. The default workspace (U-001). Layer palette at the right, bottom group (tabs at y 567).
3. Each step below was done through the menu bar or the Layer palette buttons. A screenshot of
   the palette (x 1680–1920, y 555–805) was taken after each step.

## Observations
### Menus (Layer menu, the items for this spec)
- **Layer** menu, in order: New Raster Layer (Ctrl+Shift+N) · New Layer ▸ · New Correction
  Layer ▸ · **New Layer Folder** · **Create folder and insert layer (Ctrl+G)** · **Ungroup Layer
  Folder(S) (Ctrl+Shift+G)** · Duplicate Layer · Delete Layer · ─ · **Layer Mask ▸** ·
  Ruler/Frame ▸ · File object ▸ · **Layer Settings ▸** · Selection from Layer ▸ · Rasterize ·
  Convert Layer… · ─ · Transfer To Lower Layer · Merge with layer below (Ctrl+E) · Merge
  selected layers (Shift+Alt+E) · Merge visible layers (Ctrl+Shift+E) · Merge visible to new
  layer · Flatten image · ─ · Layer Order ▸ · Change Selected Layer ▸.
  - Ungroup is greyed out unless a folder is selected.
  - Delete Layer is greyed out when deleting would leave no layer. This includes a folder
    that contains every layer.
- **Layer Mask ▸** has three groups:
  - Mask Outside Selection (V) · Mask Selection (D)
  - Delete Mask · Apply Mask to Layer
  - Enable Mask · Show Mask Area · Link Mask to Layer. These three are checkable.
  - Without a mask, only the first two items are enabled. With a mask, the first two are
    greyed out.
  - A new mask has Enable ✓, Show Mask Area off and Link ✓.
- **Layer Settings ▸** begins: Set as Reference Layer · Set as Draft Layer · **Clip to Layer Below
  (Ctrl+Alt+G)** · Lock Layer · Lock transparent pixels · ─ · Show Layer (V) ✓ · Change Layer
  Name… · …

### Creating
- **Ctrl+G** wraps the selected layer in a new folder "Folder *n*", selects the folder and
  leaves it expanded.
  - With a folder selected, Ctrl+G wraps that folder in a new outer folder.
  - New folders are **Normal, 100 %**.
- **New layer position** (Layer > New Raster Layer):
  - A layer is selected: the new layer goes directly above it, in the same folder.
  - An **expanded** folder is selected: it goes **inside the folder, at the top**.
  - A **collapsed** folder is selected: it goes **directly above the folder**, in the folder's
    parent.
- **Create layer mask:**
  - The palette button (toolbar row 2, filled-circle icon, tooltip "Create layer mask")
    creates an **all-white mask (reveal all)** when there is no selection.
  - **Mask Outside Selection** with no selection creates an **all-black mask (hide all)**.
  - Masks can be created on raster layers and on folders, including Through folders.
- **Ungroup Layer Folder** removes the folder and leaves its children in place, in its parent.
  The folder's own mask is discarded. A mask that hides something wasn't tried (open).

### Palette rows
- **Folder row:**
  - Columns, left to right:
    1. eye
    2. edit (pen) column
    3. disclosure arrow, "∨" when expanded and "›" when collapsed
    4. folder icon, open or closed to match
    5. two text lines: "100 % Normal" (or "100 % Through") and the name
  - A folder row has **no thumbnail**. It is shorter than a raster row unless the folder has a
    mask.
- **Children** are indented by about **16 px per level** under their folder. Nested folders
  indent again.
- **Clipped layer:** a **red vertical bar** (about 4 px wide) at the left of the thumbnail column,
  with the thumbnail shifted right by the bar's width. While the clipped layer is selected,
  the Clip to Layer Below button in toolbar row 1 is shown pressed.
- **Layer with a mask:** the layer thumbnail, then a small **link mark** (a check-like glyph,
  present only while linked), then the **mask thumbnail**.
  - The mask thumbnail is white where revealed and black where hidden. A thin hidden band
    shows as a grey line at thumbnail scale.
  - The selected target, layer or mask, has a white border around its thumbnail. After
    Create layer mask, the mask is the target.
  - **Disabled mask:** a red **X** across the mask thumbnail.
  - The row text moves right to make room.
- **Folder with a mask:** the row grows to raster-row height and shows arrow, folder icon, link
  mark, mask thumbnail and text.
- **Toolbar** with a mask on the selected layer:
  - Row 1: the Enable Mask button (circle with a check, plus ▾) shows the state, with an X
    when disabled.
  - Row 2: "Create layer mask" is greyed out and "Apply Mask to Layer" is enabled.
- **Blend list for a folder:** **Through** is the first entry, above Normal. Raster layers
  don't have it.

### Canvas
- **Show Mask Area:** hidden areas are drawn in **(180,180,255)**, opaque over both checker
  colours, with anti-aliased edges blending to the checker. This is display only.
  - With the mask disabled, the overlay isn't drawn and the layer shows unmasked.
- **Layer Property palette:** a layer with a mask shows a **"Mask expression"** section, with
  **Show gradients: Yes** as the default for a new mask. Gradient masks (continuous values) are
  therefore on by default in 1.13.2. The other values are in L-008.

## Open questions
- Ungroup of a folder whose mask hides something: is the mask applied first, or discarded?
- Drag and drop into, out of and between folders: the drop indicator and the target rules.
- Layer Property > Mask expression > Show gradients: its other values (L-008).
