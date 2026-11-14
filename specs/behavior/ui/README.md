# UI behaviour specs (U-xxx)

How CSP 1.13.2 organises its window. Brushye matches the organisation and the look (palette
placement, order, sizes, colours, metrics); icons are Brushye's own (Phosphor) and wording is
Brushye's own. Screenshots used for these specs stay in `obs-out/ui/` on the observation machine
and are never committed (CSP UI artwork).

Common header for U-001 … U-009:
- CSP version: 1.13.2 (PRO, 64-bit), observation prefix `~/.wine-csp-observe`.
- Date / method / observer: 2026-11-13 / automated-Wine (Xorg :78, 1920×1080 virtual desktop,
  XTEST mouse/keys, `import` screenshots) / Claude.
- Workspace: restored with **Window > Workspace > Reset to Default(I)**, confirmed with **Yes**
  in the "layout of palette will be reset" dialog. Doing it twice gives pixel-identical
  screenshots, so the layout below is CSP's default palette layout (not a customised one).
- Open documents at the time: three restored tabs from earlier observation runs, the active one
  800×800 px with one raster layer. Document content doesn't affect the layout.
- Coordinates are screen pixels at 1920×1080; the CSP window is maximised inside the Wine
  desktop. Analysis helper: `tools/observe/ui_scan.py` (run-length colour scans).
- Wine caveats: the window caption (title bar with min/max/close, y 4–27) is drawn under Wine
  without a window manager, and fonts are Wine substitutes. On Windows the caption is the OS's
  and the UI font is the system font (verify-native).
