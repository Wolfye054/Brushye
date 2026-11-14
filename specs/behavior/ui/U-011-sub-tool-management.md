# U-011 Sub tool management
- Status: **observed**
- See README. Date 2026-11-13, automated-Wine, observer Claude. Every dialog was opened and then
  cancelled. A right-click changed the selected sub tool once; G-pen was selected again.

## Sub Tool palette ≡ menu (in order; — = separator)
Sub tool group settings… — Duplicate sub tool… · Sub tool settings… · Delete sub tool · Create
custom sub tool(N)… — Add from default… — Lock · Reset to default · Save as default(I) —
Modifier Key Settings… — Import sub tool… · Export sub tool… · Register sub tool as material… ·
Add sub tool… — View ▸ · ✓Show group name of sub tool · ✓Show command bar · Change Order(T) ▸ —
Hide Sub Tool: <tool> palette.
- View ▸: ✓Stroke · Tile · Tile small · Text (row display modes).
- Change Order ▸: ✓Drag · Ctrl + Drag (sub tools are reordered by dragging rows; the option
  chooses whether Ctrl is required).
- "Show command bar" refers to the palette's bottom button bar.

## Right-click menus
- On a **group tab**: one item, Sub tool group settings….
- On a **sub tool row**: the row becomes the selected sub tool, then a menu: Duplicate sub tool…
  · Sub tool settings… · Delete sub tool · Create custom sub tool(N)… — Add from default… —
  Lock · Reset to default · Save as default(I) — Modifier Key Settings… — Import sub tool… ·
  Export sub tool… · Register sub tool as material… · Add sub tool….

## Dialogs
- **Sub tool group settings**: Name field (current group name, selected); OK / Cancel. The group
  is renamed here.
- **Duplicate sub tool**:
  - Name (default "<name> 2");
  - Output process and Input process, shown read-only (e.g. Direct draw / Pen);
  - Tool icon: radio choice between a preset icon dropdown (e.g. Pen) and "User Settings…";
  - "Background color of icon" checkbox with a colour swatch;
  - OK / Cancel.
- **Create custom sub tool**: the same fields, but Output process and Input process are
  dropdowns. The default name is the group's tool name.
- **Add sub tool** (bottom bar, first button): a material browser for sub tools. It has a
  search box, type filters (not yet added to a palette / created / downloaded), a link to
  search CSP's online asset store, view-mode buttons and "Add palette" / Cancel.

## Bottom bar buttons (tooltips)
Add sub tool · Create copy of currently selected sub tool · Delete selected sub tool.

## What Brushye should match
- The same ≡ menu and row context menu structure for Brushye's features:
  - duplicate, settings (name and icon), delete, create custom;
  - lock, reset, save as default;
  - import and export (`.brshbrush`);
  - view mode, show group names, command bar, change order.

  CSP-ecosystem items (register as material, asset store) are omitted or shown disabled.
- Group rename through a "Sub tool group settings" dialog with a Name field.
- Drag rows to reorder sub tools.
- Bottom bar: add, duplicate, delete. "Add sub tool" in Brushye imports from a brush library or
  file, since there is no asset store.
