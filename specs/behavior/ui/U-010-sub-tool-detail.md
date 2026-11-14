# U-010 Sub Tool Detail palette
- Status: **observed**
- See README (CSP 1.13.2, automated-Wine, :78, 1920×1080, default workspace). Date 2026-11-13,
  observer Claude. Screenshots in `obs-out/m7/` (not committed).
- Opened with the wrench button at the right of Tool property's bottom bar (tooltip: show the
  Sub Tool Detail palette to edit hidden parameter values and their displayed status), for
  Pen > first sub tool, an Eraser sub tool and a Brush sub tool.

## Window
- A **floating palette**, not docked: x 733–1559, y 325–1075 (≈ 826 × 750) at 1920×1080, title
  "Sub Tool Detail" centred in a 17 px title bar, close button at the right. Escape closes it.
- Top: a **preview strip** ≈ 820 × 100. The sub tool's name sits in a small label at the top-left.
  The left half (≈ 410 px) is a light grey band with the stroke preview in the drawing colour;
  the preview continues over the dark right half.
- Left: **category list**, ≈ 135 px wide, text only (no icons), 19 px rows, selected row in the
  selection accent. The palette remembers the last category selected.
- Right: the **page** for the selected category. It starts with a one-line description of the
  category on a darker band (≈ 50 px), then the setting rows.
- Under the page: a **help box** with an "i" icon and the title "About parameter help". It
  explains the row under the cursor ("About [<setting>]" plus one or two sentences); with no
  row hovered it says that descriptions appear when you hover an item.
- Bottom-left: a **Show category** checkbox. Its tooltip: show category names (labels) in the
  Tool property palette. Bottom-right: two buttons, **Reset all settings to default** (with a
  reset icon) and **Save all settings as default**.

## Category list (same 19 entries for Pen, Eraser and Brush sub tools)
Brush Size · Ink · Color Jitter · Anti-aliasing · Brush shape · (indented) Brush tip · Spraying
effect · Stroke · Texture · 2 - Brush shape · (indented) 2 - Brush tip · 2 - Spraying effect ·
2 - Stroke · 2 - Texture · Watercolor edge · Erase · Correction · Starting and ending ·
Anti-overflow.
The "2 -" entries are the second brush of a dual brush. Settings that don't apply are shown
disabled, never hidden.

## Row anatomy (every page)
- Two square buttons at the far left of each row, each ≈ 14 × 22:
  1. **Eye**: when on (accent background with an eye icon), the row is also shown in Tool
     property. Turning it on for a nested row ("At least 1 pixel" under Brush Size) adds a "+"
     expander to the parent row in Tool property. Tested and reverted.
  2. **▸ menu**: a pop-up with **Copy value ▸** and **Copy displayed status(V) ▸**, which copy
     this setting to other sub tools.
- Nested settings are connected to their parent by a thin bracket line at the left, and are
  indented.
- Control types:
  - **slider row**: label at the top-left, a thin value bar under it, the value right-aligned
    with an up/down spinner, and at the far right the **dynamics button**. The dynamics
    button's icon reflects which dynamics are on (a different glyph when pressure is linked);
    it is greyed where dynamics don't apply;
  - **checkbox row**;
  - **segmented control** (icon cells, selected cell in accent), with a ▾ dropdown at the end
    for some;
  - **full-width dropdown**;
  - **two-option toggle** (a wide button pair, selected half in accent);
  - **5-step bar** (Hardness, Particle density, Post correction, Taper), with a ▸ at the end.
- Rows are ≈ 25–30 px tall, with thin separator lines between groups.

## Pages, rows in order (Pen > G-pen; the Eraser and Brush pages have the same rows)
- **Brush Size**: Brush Size (slider + dynamics) › Specify by size on screen (checkbox) › At
  least 1 pixel (checkbox). Below the rows, a large empty area shows a dot preview of the brush
  size.
- **Ink**:
  - Opacity (slider + dynamics);
  - Blending mode (dropdown; "Erase" for the eraser);
  - Color mixing (checkbox, with a 3-cell mode segmented control ▾), then Amount of paint
    (slider + dynamics), Density of paint (slider + dynamics), Color stretch (slider);
  - Intensity of blur (toggle Automatic / Fixed value, slider + dynamics).
- **Color Jitter**:
  - Change brush tip color (checkbox) › Hue, Saturation, Luminosity (sliders + dynamics), Blend
    with sub color (slider + dynamics);
  - Randomize per stroke (checkbox) › Hue, Saturation, Luminosity, Blend with sub color
    (sliders);
  - Change (dropdown: Main color).
- **Anti-aliasing**: one row, a 4-cell segmented control (none / weak / middle / strong) with
  ▾. Below it, the same dot preview area.
- **Brush shape**:
  - "Current brush shape" label and an **Add to presets** button;
  - a preview strip of the current shape;
  - a grid of shape presets (3 columns, each a stroke preview with a right-aligned name);
  - **Apply brush shape** button at the bottom, with settings/delete icons.
  Brushye uses its own presets; CSP's names are not recorded as content.
- **Brush tip**:
  - Tip shape toggle (Circle / Material) and a material list area (with add/delete icons);
  - Hardness (5-step bar);
  - Thickness (slider + dynamics) › Direction toggle (Horizontal / Vertical);
  - Angle (slider + dynamics);
  - Flip horizontal, Flip vertical (dropdowns, "Off");
  - Brush density (slider + dynamics) › Adjust brush density by gap (checkbox).
- **Spraying effect**: Spraying effect (checkbox) › Particle size (slider + dynamics) › Brush
  Size (checkbox); Particle density (5-step bar + dynamics); Spray deviation (5-step bar);
  Direction of particle (slider + dynamics).
- **Stroke**:
  - Gap (4-cell segmented control: fixed / wide / normal / narrow, with ▾) › gap value (slider
    + dynamics, enabled for Fixed);
  - Continuous spraying (checkbox);
  - Correct velocity input (checkbox);
  - Ribbon (checkbox) › Repeat method (segmented control ▾);
  - Blend brush tips with Darken (checkbox).
- **Texture**: Texture (dropdown "None" with a delete icon) › Texture density (slider +
  dynamics) › Invert texture, Emphasize density (checkboxes); Scale ratio, Rotation angle,
  Brightness, Contrast (sliders); Texture mode (dropdown) › Apply by each plot (checkbox).
- **2 - Brush shape**: Dual brush (checkbox) › Mode (dropdown) › Apply RGB value (checkbox); 2-Brush Size (slider + dynamics) › Link to main brush size (checkbox); 2-Anti-aliasing (segmented); shape preset grid with Apply brush shape.
- **2 - Brush tip / 2 - Spraying effect / 2 - Stroke / 2 - Texture**: the same rows as the main
  pages with a "2-" prefix (2-Stroke has no Correct velocity input row).
- **Watercolor edge**: Watercolor edge (checkbox, slider) › Opacity, Darkness (5-step bars) ›
  Process after brush stroke (checkbox) › Blurring width (slider).
- **Erase**: Vector eraser (checkbox + 3-cell segmented ▾) › Refer all layers (checkbox); Erase
  on all layers (checkbox).
- **Correction**:
  - Sharp angles (checkbox);
  - Stabilization (slider) › Adjust by speed (checkbox) › Stabilization mode (toggle: increase
    stabilization when drawing slowly / reduce it when drawing quickly);
  - Post correction (checkbox + 5-step bar) › Adjust by speed, Adjust by scale, Bezier curve
    (checkboxes);
  - Taper (5-step bar);
  - Enable snapping (checkbox);
  - Vector magnet (checkbox + 5-step bar).
- **Starting and ending**: Starting and ending (dropdown "None") › How to specify (dropdown) ›
  Starting (checkbox + slider), Ending (checkbox + slider) › Starting and ending by speed
  (checkbox).
- **Anti-overflow**: Do not cross lines of reference layer (checkbox); Fill up to vector path
  (checkbox); Tolerance (slider); Area scaling (checkbox + slider) › Scaling mode (segmented ▾).

## Dynamics pop-up (from any dynamics button; here Brush Size)
- A pop-up ≈ 430 × 357 px, opened next to the button. Title "<Setting> Dynamics".
- Section "Input affecting <setting>". One row per input, each with an icon, a checkbox and its
  values:
  - **Pen pressure**: minimum value (slider + value);
  - **Tilt**: minimum and maximum value;
  - **Velocity**: minimum value;
  - **Random**: minimum value.

  An input's value rows are disabled until its checkbox is on.
- Under the inputs, two graphs side by side, each ≈ 200 × 176:
  - **Pen pressure settings**: axes Output (0%–100%) against Pen pressure (0%–100%);
  - **Tilt settings**: axes Output against Horizontal – Tilt – Vertical.
  - Grid of 4 × 4 cells. Control points are small blue squares, and the curve is a white line.
    When the curve isn't linear, the control polygon is drawn too (thin blue lines between the
    points).
- Editing the graph:
  - clicking on the graph adds a point (the selected point turns red), and the tooltip reads
    "Specify output amount";
  - dragging moves it, and the curve follows as a smooth curve through the polygon's interior;
  - dragging a point outside the graph removes it.

  Adding, moving and removing a point returned the curve to linear (tested, reverted).

## What Brushye should match
- A floating Sub Tool Detail window with: the preview strip on top; the text category list at
  the left in this order (unbuilt categories listed and disabled); a page with the description
  band and rows; the help box explaining the hovered row; Show category; and the two bottom
  buttons.
- Per row: the eye button (show in Tool property) and the ▸ menu (copy value / displayed status
  to other sub tools); nested rows bracketed and indented; the same control types and order.
- The dynamics pop-up: inputs with checkboxes and min/max values, the pressure (and later
  tilt) graph with click-to-add, drag-to-move, drag-out-to-delete points, and the control
  polygon drawn when the curve isn't linear.
- Brushye's sub tools, shapes and presets are its own; only this structure is matched.
