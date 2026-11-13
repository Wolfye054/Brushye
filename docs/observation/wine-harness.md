# Automated observation harness: CSP 1.13.2 under Wine

Set up on 2026-11-11 (Phase 1). It follows the *Automated observation* rules in CLAUDE.md:
only the UI, screenshots and exported files are used. No tracing, debuggers, hooking, or reading
CSP's installed files, settings or databases.

## What's verified
| Item | Result |
|---|---|
| CSP version under automation | Help > Version information: **CLIP STUDIO PAINT PRO (64 bit) Version 1.13.2 202301210208** |
| Isolated display | Xvfb `:77`, 1920×1080×24, `-nolisten tcp` |
| Wine | cspenguin's bundled Wine 11.4 (`~/.local/share/cspenguin/wine-11.4`) |
| Prefix | `~/.wine-csp-observe`, a btrfs reflink clone of the maintainer's `~/.wine-csp` (DECISIONS D-001) |
| Driving the UI | XTEST via `tools/observe/xdrive.py` (python-xlib): clicks, drags, keys, typing paths into file dialogs, menus, dropdowns |
| Opening files | File > Open of a generated PNG works (Wine file dialog, typed `Z:\…` path) |
| Exporting | File > Export (Single Layer) > .png works; settings persist between exports |
| Exported pixels are exact | Open PNG → export → **bit-identical RGB** on 131,072 px; alpha 255 (D-002) |
| Batch observation | 56 blend-mode exports scripted end to end without failures |
| Virtual tablet pressure (Xvfb) | **Not available.** Xvfb has no pressure-capable XInput device; see D-003 |
| Virtual tablet pressure (Xorg :78, 2026-11-13) | Pressure reaches CSP. With the evdev driver, pen-up draws a stray segment to (0,0); the fix is pending a udev rule change (D-003 update) |

## Launch
```bash
W=~/.local/share/cspenguin/wine-11.4
export LD_LIBRARY_PATH="$W/lib/freetype2-2.13.2/lib64:$W/lib/freetype2-2.13.2/lib32"
export PATH="$W/bin:$PATH" WINESERVER="$W/bin/wineserver"
export WINEPREFIX=~/.wine-csp-observe WINEDEBUG=-all DISPLAY=:77
export WINEDLLPATH=~/.local/share/cspenguin
Xvfb :77 -screen 0 1920x1080x24 -nolisten tcp &
wine explorer /desktop=csp,1920x1080 \
  "$WINEPREFIX/drive_c/Program Files/CELSYS/CLIP STUDIO 1.5/CLIP STUDIO PAINT/CLIPStudioPaint.exe" &
```
`WINEDEBUG=-all` turns Wine's own logging *off*. No debug channels are ever enabled (CLAUDE.md
forbids Wine debug/relay tracing).

The virtual desktop matters: with no window manager, CSP's main window comes up at 800×600.
Inside a fixed-size Wine desktop it fills 1920×1080 with the default layout, so palette
coordinates are stable.

Stop: `WINEPREFIX=~/.wine-csp-observe wineserver -k; pkill -f 'Xvfb :77'`.

## Driver
`tools/observe/xdrive.py` commands, chained with `';;'`:
`move X Y`, `click X Y [btn]`, `dclick`, `down/up X Y`, `drag X1 Y1 X2 Y2 [steps] [ms]`,
`key ctrl+o`, `type TEXT`, `sleep S`, `shot FILE.png` (screenshot via ImageMagick `import`),
`windows`. It needs `python-xlib` (installed in a scratch venv, not system-wide).

## Coordinates (default 1.13.2 workspace, 1920×1080 virtual desktop)
| UI element | Point |
|---|---|
| File menu | (28, 41) |
| File > Export (Single Layer) | hover (104, 249) → submenu `.png` (398, 292) |
| PNG export settings OK | (1170, 282) |
| Layer palette blend dropdown | (1765, 605); item *i* at (1760, 71 + 19·i) |
| Layer palette delete layer | (1912, 649) |
| Help > Version information | (437, 41) → (513, 239) |

These depend on the workspace layout. Recheck with a screenshot if the layout changes.

## Gotchas found
- After Select All, CSP marks a document as changed. Closing it prompts Save / Don't Save,
  and the button positions differ between the two prompts. Prefer opening documents in new tabs
  over closing them mid-script.
- Keyboard shortcuts go to whatever has focus. A stray keypress once switched the active tool to
  Gradient. Always click the canvas/tab before sending shortcuts, and screenshot-check before
  long batches.
- The export dialog remembers the last folder and settings ("RGB color", preview off).
- Exported PNGs carry an sRGB2014 iCCP chunk. Pixel values are unaffected; it's stripped
  before data goes into the repo.

## Pen-capable display (D-003)
`tools/observe/start_pen_display.sh WORKDIR [78]` starts `vpen.py` (uinput stylus "Brushye Virtual
Pen"), a rootless Xorg using `xorg-pen.conf.in` (dummy video, auto-add off, only that device),
and CSP. Pen commands go to `WORKDIR/fifo` (`in`, `move X Y P TX TY`, `down`, `up`, `out`,
`wait MS`). XTEST (xdrive.py) still works on the same display for menus and keys.
Pitfall: never `pkill -f 'Xorg :78'`, because it matches the calling shell's command line too.
Anchor it: `pkill -f '^/usr/libexec/Xorg :78'`.

## Getting pressure later (D-003)
See DECISIONS D-003 for the two options (Xorg dummy + libinput + uinput stylus with a udev rule,
or a headless Wayland compositor with a virtual tablet). Both need maintainer action (root).
Until one is proven, pressure/tilt specs are marked *maintainer + real pen*.
