#!/usr/bin/env python3
"""Virtual stylus for automated observation (DECISIONS D-003), pure Python, no deps.

Creates a uinput device named "Brushye Virtual Pen". The maintainer's udev rule
(70-brushye-virtual-pen.rules) gives the user ownership of its event node and sets
LIBINPUT_IGNORE_DEVICE so the desktop session ignores it. Only the observation Xorg server
(evdev driver, see docs/observation/wine-harness.md) opens it.

usage: vpen.py FIFO_PATH [--width 1920 --height 1080]
Prints the event node path, then executes command lines written to FIFO_PATH:
  move X Y [P] [TX] [TY]   pen at screen px (floats ok), pressure 0..1, tilt -1..1
  in | out                 pen enters/leaves proximity (BTN_TOOL_PEN)
  rubber_in | rubber_out   eraser end proximity
  down | up                tip contact (BTN_TOUCH)
  wait MS                  sleep (timing is done here, so strokes are evenly paced)
  quit
Pressure range 0..8191, tilt -64..63, position resolution 10 units per screen pixel.
"""
import fcntl, os, struct, sys, time

UI_SET_EVBIT, UI_SET_KEYBIT, UI_SET_ABSBIT, UI_SET_PROPBIT = 0x40045564, 0x40045565, 0x40045567, 0x4004556E
UI_DEV_CREATE, UI_DEV_DESTROY = 0x5501, 0x5502
EV_SYN, EV_KEY, EV_ABS = 0, 1, 3
ABS_X, ABS_Y, ABS_PRESSURE, ABS_TILT_X, ABS_TILT_Y = 0x00, 0x01, 0x18, 0x1A, 0x1B
BTN_TOOL_PEN, BTN_TOOL_RUBBER, BTN_TOUCH, BTN_STYLUS = 0x140, 0x141, 0x14A, 0x14B
INPUT_PROP_DIRECT = 1
NAME = b"Brushye Virtual Pen"
SCALE, PMAX, TMAX = 10, 8191, 63

args = sys.argv[1:]
fifo = args[0]
W = int(args[args.index("--width") + 1]) if "--width" in args else 1920
H = int(args[args.index("--height") + 1]) if "--height" in args else 1080

fd = os.open("/dev/uinput", os.O_WRONLY | os.O_NONBLOCK)
fcntl.ioctl(fd, UI_SET_EVBIT, EV_KEY); fcntl.ioctl(fd, UI_SET_EVBIT, EV_ABS)
for k in (BTN_TOOL_PEN, BTN_TOOL_RUBBER, BTN_TOUCH, BTN_STYLUS):
    fcntl.ioctl(fd, UI_SET_KEYBIT, k)
# Modern uinput setup (UI_DEV_SETUP + UI_ABS_SETUP) so the axes carry a resolution:
# libinput rejects tablets whose X/Y have no resolution.
UI_DEV_SETUP, UI_ABS_SETUP = 0x405C5503, 0x401C5504
# (code, min, max, resolution): X/Y 10 units per screen px, at 40 units/mm (~0.25 mm px);
# tilt in degrees, resolution 57 units/rad (1 unit = 1 degree).
for code, lo, hi, res in ((ABS_X, 0, W * SCALE - 1, 40), (ABS_Y, 0, H * SCALE - 1, 40),
                          (ABS_PRESSURE, 0, PMAX, 0), (ABS_TILT_X, -64, TMAX, 57), (ABS_TILT_Y, -64, TMAX, 57)):
    fcntl.ioctl(fd, UI_SET_ABSBIT, code)
    fcntl.ioctl(fd, UI_ABS_SETUP, struct.pack("HH6i", code, 0, 0, lo, hi, 0, 0, res))
fcntl.ioctl(fd, UI_SET_PROPBIT, INPUT_PROP_DIRECT)
# struct uinput_setup: input_id (bustype, vendor, product, version), name[80], ff_effects_max
fcntl.ioctl(fd, UI_DEV_SETUP, struct.pack("HHHH80sI", 0x03, 0x1D50, 0x6B5E, 1, NAME, 0))
fcntl.ioctl(fd, UI_DEV_CREATE)
time.sleep(0.5)

node = None
for ev in sorted(os.listdir("/sys/class/input")):
    if ev.startswith("event"):
        try:
            with open(f"/sys/class/input/{ev}/device/name") as f:
                if f.read().strip() == NAME.decode():
                    node = f"/dev/input/{ev}"
        except OSError:
            pass
print(node, flush=True)

def emit(t, c, v):
    s = time.time()
    os.write(fd, struct.pack("qqHHi", int(s), int((s % 1) * 1e6), t, c, int(v)))
def syn(): emit(EV_SYN, 0, 0)

# Last absolute position in device units. The X evdev driver attaches only the axes that changed in
# the same SYN frame to a button event, and Wine's wintab then reads X/Y as 0 for a button
# event without them (seen 2026-11-13: pen-up drew a line to screen (0,0)). So every
# touch down/up frame also carries X/Y, nudged by 1 unit (0.1 px) because the kernel drops
# repeated identical values.
pos = [0, 0]; nudge = [1]
def emit_pos_nudged():
    nudge[0] = -nudge[0]
    emit(EV_ABS, ABS_X, max(0, pos[0] + nudge[0])); emit(EV_ABS, ABS_Y, max(0, pos[1] + nudge[0]))

if not os.path.exists(fifo):
    os.mkfifo(fifo)
try:
    running = True
    while running:
        with open(fifo) as f:          # blocks until a writer connects; re-open after EOF
            for line in f:
                p = line.split()
                if not p: continue
                c = p[0]
                if c == "move":
                    x, y = float(p[1]), float(p[2])
                    pos[:] = [round(x * SCALE), round(y * SCALE)]
                    emit(EV_ABS, ABS_X, pos[0]); emit(EV_ABS, ABS_Y, pos[1])
                    if len(p) > 3: emit(EV_ABS, ABS_PRESSURE, round(float(p[3]) * PMAX))
                    if len(p) > 4: emit(EV_ABS, ABS_TILT_X, round(float(p[4]) * TMAX))
                    if len(p) > 5: emit(EV_ABS, ABS_TILT_Y, round(float(p[5]) * TMAX))
                    syn()
                elif c in ("in", "out"): emit(EV_KEY, BTN_TOOL_PEN, c == "in"); syn()
                elif c in ("rubber_in", "rubber_out"): emit(EV_KEY, BTN_TOOL_RUBBER, c == "rubber_in"); syn()
                elif c in ("down", "up"):
                    emit_pos_nudged()
                    emit(EV_KEY, BTN_TOUCH, c == "down")
                    if c == "up": emit(EV_ABS, ABS_PRESSURE, 0)
                    syn()
                elif c == "wait": time.sleep(float(p[1]) / 1000)
                elif c == "quit": running = False; break
finally:
    fcntl.ioctl(fd, UI_DEV_DESTROY); os.close(fd)
