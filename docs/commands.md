# Brushye command reference

Generated from the command registry by `brushye commands --markdown`; a test keeps this
file current. Syntax: `docs/command-format.md`. Classes: **session** starts a new document
and history, **undoable** is one undo step, **appstate** is journaled but not undoable,
**history** is undo/redo, **tool** changes tools, sub tools and colours (not undoable, not
journaled).

## Commands

### `doc.new` (v1, session)
New document; starts a fresh history.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `width` | u32 | positional | Canvas width in pixels |
| `height` | u32 | positional | Canvas height in pixels |
| `dpi` | u32 | optional `key=` | Resolution (default 72) |
| `paper` | colour or none | optional `key=` | Paper colour, or none (default #ffffff) |

### `layer.new` (v1, undoable)
New layer with a caller-chosen id.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `layer` | layer id | positional | Id for the new layer (must be unused) |
| `kind` | layer kind | optional `key=` | Layer kind (default raster) |
| `name` | string | optional `key=` | Name (default "Layer <n>") |
| `above` | layer id | optional `key=` | Insert directly above this layer (default: on top) |

### `layer.delete` (v1, undoable)
Delete a layer.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `layer` | layer id | positional | Layer to delete |

### `layer.move` (v1, undoable)
Move a layer above another layer, or to the top or bottom.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `layer` | layer id | positional | Layer to move |
| `above` | layer id | optional `key=` | Place directly above this layer |
| `to` | place (top\|bottom) | optional `key=` | top or bottom |

### `layer.set` (v1, undoable)
Set layer properties (name, opacity, blend, locks).

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `layer` | layer id | positional | Target layer |
| `name` | string | optional `key=` | Name |
| `opacity` | u8 | optional `key=` | Opacity in percent, 0..=100 |
| `blend` | blend mode | optional `key=` | Blending mode |
| `locked` | bool | optional `key=` | Lock layer |
| `lock_alpha` | bool | optional `key=` | Lock transparent pixels |

### `layer.fill` (v1, undoable)
Fill a layer with one colour.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `layer` | layer id | positional | Target layer |
| `color` | colour | required `key=` | Fill colour #rrggbb or #rrggbbaa (straight alpha) |

### `layer.pixels` (v1, undoable)
Write raw RGBA pixels into a layer.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `layer` | layer id | positional | Target layer |
| `x` | i64 | required `key=` | Left edge in canvas pixels |
| `y` | i64 | required `key=` | Top edge in canvas pixels |
| `w` | u32 | required `key=` | Block width |
| `h` | u32 | required `key=` | Block height |
| `data` | blob | required `key=` | w*h*4 bytes, row-major RGBA |

### `brush.stroke` (v1, undoable)
Paint one brush stroke from input samples.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `layer` | layer id | positional | Target layer |
| `color` | colour | required `key=` | Ink colour #rrggbb |
| `size` | f64 | required `key=` | Brush size (diameter) in px |
| `aa` | anti-aliasing (none\|weak\|middle\|strong) | optional `key=` | Anti-aliasing (default middle) |
| `hardness` | u8 | optional `key=` | Tip hardness 1 (soft) ..= 5 (hard, default) |
| `opacity` | u8 | optional `key=` | Ink opacity in percent (default 100) |
| `density` | u8 | optional `key=` | Brush density in percent (default 100) |
| `ink` | ink mode | optional `key=` | Ink blending mode (default normal): the layer modes plus erase, background, replace_alpha, compare_density, erase_compare, black_burn, white_burn |
| `gap` | gap (narrow\|normal\|wide or percent of size) | optional `key=` | Dab gap: narrow, normal (default), wide, or percent of size |
| `density_by_gap` | bool | optional `key=` | Adjust brush density by gap |
| `thickness` | u8 | optional `key=` | Tip thickness in percent, 1..=100 (default 100 = round) |
| `direction` | tip direction (vertical\|horizontal) | optional `key=` | Axis that thickness shortens (default vertical) |
| `angle` | f64 | optional `key=` | Tip angle in degrees, clockwise (default 0) |
| `angle_line` | bool | optional `key=` | Tip angle follows the direction of the line |
| `stabilization` | u8 | optional `key=` | Stabilization 0..=100 (default 0) |
| `size_pressure` | u8 | optional `key=` | Pen pressure controls size; minimum in percent |
| `size_curve` | curve points [x,y;…] | optional `key=` | Size pressure curve points [x,y;…] |
| `opacity_pressure` | u8 | optional `key=` | Pen pressure controls opacity; minimum in percent |
| `opacity_curve` | curve points [x,y;…] | optional `key=` | Opacity pressure curve points [x,y;…] |
| `density_pressure` | u8 | optional `key=` | Pen pressure controls density; minimum in percent |
| `density_curve` | curve points [x,y;…] | optional `key=` | Density pressure curve points [x,y;…] |
| `samples` | stroke samples | required `key=` | Input samples (b64:… or [x,y,p,t;…]) |

### `layer.show` (v1, appstate)
Show or hide a layer (not undoable, as in CSP).

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `layer` | layer id | positional | Target layer |
| `visible` | bool | required `key=` | Visible |

### `layer.select` (v1, appstate)
Make a layer the active layer.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `layer` | layer id | positional | Layer to select |

### `input.pressure_curve` (v1, appstate)
Set the global pen pressure adjustment curve.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `points` | curve points [x,y;…] | required `key=` | Interior control points [x,y;…]; [] = linear |

### `tool.select` (v1, tool)
Select a tool.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `tool` | tool | positional | Tool |

### `subtool.select` (v1, tool)
Select a sub tool.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `tool` | tool | optional `key=` | Tool (default: the current tool) |
| `group` | u32 | optional `key=` | Group tab, from 0 (default: the current group) |
| `item` | u32 | optional `key=` | Sub tool in the group, from 0 (default: the selected one) |

### `subtool.set` (v1, tool)
Change a sub tool's name or settings.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `tool` | tool | optional `key=` | Tool (default: the current tool) |
| `group` | u32 | optional `key=` | Group tab, from 0 (default: the current group) |
| `item` | u32 | optional `key=` | Sub tool in the group, from 0 (default: the selected one) |
| `name` | string | optional `key=` | New name |
| `size` | f64 | optional `key=` | Brush size (diameter) in px |
| `aa` | anti-aliasing (none\|weak\|middle\|strong) | optional `key=` | Anti-aliasing |
| `hardness` | u8 | optional `key=` | Tip hardness 1 (soft) ..= 5 (hard) |
| `opacity` | u8 | optional `key=` | Ink opacity in percent |
| `density` | u8 | optional `key=` | Brush density in percent |
| `ink` | ink mode | optional `key=` | Ink blending mode |
| `gap` | gap (narrow\|normal\|wide or percent of size) | optional `key=` | Dab gap: narrow, normal, wide, or percent of size |
| `density_by_gap` | bool | optional `key=` | Adjust brush density by gap |
| `thickness` | u8 | optional `key=` | Tip thickness in percent, 1..=100 |
| `direction` | tip direction (vertical\|horizontal) | optional `key=` | Axis that thickness shortens |
| `angle` | f64 | optional `key=` | Tip angle in degrees, clockwise |
| `angle_line` | bool | optional `key=` | Tip angle follows the direction of the line |
| `stabilization` | u8 | optional `key=` | Stabilization 0..=100 |
| `size_pressure` | u8 | optional `key=` | Pen pressure controls size; minimum in percent |
| `size_curve` | curve points [x,y;…] | optional `key=` | Size pressure curve points [x,y;…] |
| `opacity_pressure` | u8 | optional `key=` | Pen pressure controls opacity; minimum in percent |
| `opacity_curve` | curve points [x,y;…] | optional `key=` | Opacity pressure curve points [x,y;…] |
| `density_pressure` | u8 | optional `key=` | Pen pressure controls density; minimum in percent |
| `density_curve` | curve points [x,y;…] | optional `key=` | Density pressure curve points [x,y;…] |
| `clear` | string | optional `key=` | Comma-separated settings to turn off (not size) |

### `subtool.add` (v1, tool)
Add a sub tool to a group.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `tool` | tool | optional `key=` | Tool (default: the current tool) |
| `group` | u32 | optional `key=` | Group tab, from 0 (default: the current group) |
| `name` | string | required `key=` | Name |
| `settings` | string | optional `key=` | Settings as `key=value …` (default: the tool's first default sub tool) |

### `subtool.copy` (v1, tool)
Copy a sub tool.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `tool` | tool | optional `key=` | Tool (default: the current tool) |
| `group` | u32 | optional `key=` | Group tab, from 0 (default: the current group) |
| `item` | u32 | optional `key=` | Sub tool in the group, from 0 (default: the selected one) |
| `name` | string | optional `key=` | Name of the copy (default: "<name> 2") |

### `subtool.delete` (v1, tool)
Delete a sub tool.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `tool` | tool | optional `key=` | Tool (default: the current tool) |
| `group` | u32 | optional `key=` | Group tab, from 0 (default: the current group) |
| `item` | u32 | optional `key=` | Sub tool in the group, from 0 (default: the selected one) |

### `subtool.move` (v1, tool)
Move a sub tool within its group.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `tool` | tool | optional `key=` | Tool (default: the current tool) |
| `group` | u32 | optional `key=` | Group tab, from 0 (default: the current group) |
| `item` | u32 | optional `key=` | Sub tool in the group, from 0 (default: the selected one) |
| `to` | u32 | required `key=` | New position, from 0 |

### `subtool.lock` (v1, tool)
Lock or unlock a sub tool.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `tool` | tool | optional `key=` | Tool (default: the current tool) |
| `group` | u32 | optional `key=` | Group tab, from 0 (default: the current group) |
| `item` | u32 | optional `key=` | Sub tool in the group, from 0 (default: the selected one) |
| `locked` | bool | required `key=` | Locked |

### `subtool.reset` (v1, tool)
Reset a sub tool to its default.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `tool` | tool | optional `key=` | Tool (default: the current tool) |
| `group` | u32 | optional `key=` | Group tab, from 0 (default: the current group) |
| `item` | u32 | optional `key=` | Sub tool in the group, from 0 (default: the selected one) |

### `subtool.save_default` (v1, tool)
Register a sub tool's settings as its default.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `tool` | tool | optional `key=` | Tool (default: the current tool) |
| `group` | u32 | optional `key=` | Group tab, from 0 (default: the current group) |
| `item` | u32 | optional `key=` | Sub tool in the group, from 0 (default: the selected one) |

### `subtool.show_row` (v1, tool)
Show or hide a Tool property row.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `tool` | tool | optional `key=` | Tool (default: the current tool) |
| `group` | u32 | optional `key=` | Group tab, from 0 (default: the current group) |
| `item` | u32 | optional `key=` | Sub tool in the group, from 0 (default: the selected one) |
| `row` | string | required `key=` | Row key (as in Sub Tool Detail, e.g. size, hardness, aa) |
| `shown` | bool | required `key=` | Shown in Tool property |

### `subtool.add_defaults` (v1, tool)
Add back missing built-in sub tools.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `tool` | tool | optional `key=` | Tool (default: the current tool) |

### `subtool.group_set` (v1, tool)
Rename a sub tool group.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `group` | u32 | positional | Group tab, from 0 |
| `tool` | tool | optional `key=` | Tool (default: the current tool) |
| `name` | string | required `key=` | Name |

### `color.set` (v1, tool)
Set the main or sub colour.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `chip` | colour chip (main\|sub\|transparent) | positional | main or sub |
| `color` | colour | required `key=` | Colour #rrggbb |

### `color.active` (v1, tool)
Choose the active colour chip.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `chip` | colour chip (main\|sub\|transparent) | positional | main, sub or transparent |

### `history.undo` (v1, history)
Undo one step.

No parameters.

### `history.redo` (v1, history)
Redo one step.

No parameters.

## Queries
Queries start with `?` and return JSON; they never change state.

### `?doc.info`
Document size, dpi, paper and active layer.

No parameters.

### `?layer.list`
All layers, bottom to top.

No parameters.

### `?layer.get`
One layer's properties.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `layer` | layer id | positional | Layer |

### `?doc.hash`
Deterministic content hash of the document (hex).

No parameters.

### `?pixel`
Rendered pixel at x, y (RGBA).

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `x` | u32 | required `key=` | x |
| `y` | u32 | required `key=` | y |

### `?layer.pixel`
A layer's own pixel at x, y (RGBA).

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `layer` | layer id | positional | Layer |
| `x` | u32 | required `key=` | x |
| `y` | u32 | required `key=` | y |

### `?history`
Undo history: step labels and current position.

No parameters.

### `?input.pressure_curve`
The global pen pressure curve's control points.

No parameters.

### `?tools`
The current tool and every tool's sub tools.

No parameters.

### `?subtool`
One sub tool's name, settings, rows and lock.

| Parameter | Kind | Form | Description |
|---|---|---|---|
| `tool` | tool | optional `key=` | Tool (default: the current tool) |
| `group` | u32 | optional `key=` | Group tab, from 0 (default: the current group) |
| `item` | u32 | optional `key=` | Sub tool in the group, from 0 (default: the selected one) |

### `?color`
The main and sub colours and the active chip.

No parameters.

### `?commands`
Command and query registry with parameter schemas.

No parameters.
