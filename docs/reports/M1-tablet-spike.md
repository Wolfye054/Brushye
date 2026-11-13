# M1 report: tablet input spike (2026-11-13)

Status: **exit criterion met on Linux** (native Wayland + X11). Windows and macOS **deferred**:
the maintainer has no Windows or macOS machine for testing.

## What works
- `crates/brushye-tablet`: one ordered pen-event stream (`In/Pose/Down/Up/Button/Out`) and two
  delivery modes (Full, PressureOnly), as in DECISIONS D-015. The shared `PenTracker`
  guarantees `In` before any pose, complete axes in every pose, and `Up` before `Out`; it also
  filters a driver's never-changing twist value. 14 unit tests.
- **Native Wayland** (tablet protocol on the window's own connection), tested by the maintainer
  on GNOME with an XP-Pen/UGEE Artist 10 (2nd Gen) via OpenTabletDriver:
  - pressure (up to ~0.75 in normal use), tilt (about ±60°, 1° steps), both barrel buttons
    (BTN_STYLUS 331 / BTN_STYLUS2 332), proximity in/out;
  - samples every ~4 ms (250 Hz), continuous strokes, no time gaps or jumps;
  - the pen operates egui buttons (Full delivery: the pen is injected into the UI).
- **X11 / Xwayland** (XInput2 on a second connection), automated with the virtual stylus
  (D-003): pressure ramp, tilt in degrees, sub-pixel positions, tip/barrel buttons, and the
  eraser end (`In Eraser … Out, In Pen`).
- `cargo pen-log`: the spike app that logs and draws the stream.

## Known limits / findings
- X11 has no proximity events: `Out` only appears when another pointer moves. Timestamps are
  milliseconds (X server time). At proximity-in X repeats the previous position (hover poses
  before `Down`, harmless for strokes).
- Wayland: GNOME sends tilt/pressure/rotation per the tool's advertised capabilities; with
  OpenTabletDriver it advertises rotation and sends a constant 90°, now filtered (twist is
  `None` until it varies).
- The eraser end is untested with real hardware (the maintainer's pen has none).
- Not yet measured: whether *every* coalesced high-rate sample arrives under heavy UI load;
  the spike app's frame rate was never the limit in testing.

## Deferred
- Windows backend (Windows Ink default, WinTab option) and macOS backend (AppKit): need
  hardware to test. They compile today and report `Unsupported`.
- Switch to winit 0.31's tablet events once eframe adopts winit 0.31 (D-015).

## Specs waiting on observation
None from M1. (Pressure specs B-004/B-005/B-006 are observed; B-010 tilt dynamics in CSP is
next on the brush side.)

## Next step
M2: document model + command layer core (headless).
