#!/usr/bin/env python3
"""Tiny XTEST driver for automated CSP observation (UI-level input only).

Usage: x.py <cmd> [args...]; multiple commands separated by ';;'
  move X Y | click X Y [button] | dclick X Y | down X Y | up X Y
  drag X1 Y1 X2 Y2 [steps] [delay_ms]   (left-button drag, linear path)
  key KEYSYM[+KEYSYM...]  e.g. key ctrl+z   | type TEXT | sleep SEC
  shot PATH   (screenshot via ImageMagick import)
  windows     (list top-level window names)
"""
import os, sys, time, subprocess
from Xlib import X, XK, display
from Xlib.ext import xtest

d = display.Display(os.environ.get("DISPLAY", ":77"))
root = d.screen().root

def sync(): d.sync()
def move(x, y): xtest.fake_input(d, X.MotionNotify, x=int(x), y=int(y)); sync()
def btn(b, down): xtest.fake_input(d, X.ButtonPress if down else X.ButtonRelease, int(b)); sync()

KEYALIAS = {"ctrl": "Control_L", "shift": "Shift_L", "alt": "Alt_L", "enter": "Return",
            "esc": "Escape", "tab": "Tab", "del": "Delete", "space": "space", "bs": "BackSpace"}

def keycode(name):
    name = KEYALIAS.get(name.lower(), name)
    ks = XK.string_to_keysym(name)
    if ks == 0 and len(name) == 1:
        ks = ord(name)
    kc = d.keysym_to_keycode(ks)
    if not kc:
        raise SystemExit(f"no keycode for {name}")
    return kc

def key(combo):
    parts = combo.split("+")
    kcs = [keycode(p) for p in parts]
    for kc in kcs: xtest.fake_input(d, X.KeyPress, kc); sync(); time.sleep(0.02)
    for kc in reversed(kcs): xtest.fake_input(d, X.KeyRelease, kc); sync(); time.sleep(0.02)

SHIFTED = {'_': 'underscore', ':': 'colon', '"': 'quotedbl', '?': 'question', '>': 'greater', '<': 'less',
           '!': 'exclam', '(': 'parenleft', ')': 'parenright', '#': 'numbersign', '%': 'percent'}
PLAIN = {' ': 'space', '.': 'period', '/': 'slash', '\\': 'backslash', '-': 'minus', ',': 'comma', '=': 'equal'}

def type_text(t):
    for ch in t:
        if ch in SHIFTED: key("shift+" + SHIFTED[ch])
        elif ch in PLAIN: key(PLAIN[ch])
        elif ch.isupper(): key("shift+" + ch.lower())
        else: key(ch)
        time.sleep(0.01)

def windows():
    def walk(w, depth=0):
        try:
            name = w.get_wm_name()
            g = w.get_geometry()
            attrs = w.get_attributes()
            if name and attrs.map_state == X.IsViewable:
                tr = w.translate_coords(root, 0, 0)
                print(f"{w.id:#x} {-tr.x},{-tr.y} {g.width}x{g.height} {name!r}")
        except Exception:
            pass
        if depth < 1:
            for c in w.query_tree().children: walk(c, depth + 1)
    walk(root)

def run(args):
    c, a = args[0], args[1:]
    if c == "move": move(*a)
    elif c == "click":
        move(a[0], a[1]); time.sleep(0.05); b = a[2] if len(a) > 2 else 1
        btn(b, True); time.sleep(0.05); btn(b, False)
    elif c == "dclick":
        move(a[0], a[1])
        for _ in range(2): btn(1, True); btn(1, False); time.sleep(0.08)
    elif c == "down": move(a[0], a[1]); btn(1, True)
    elif c == "up": move(a[0], a[1]); btn(1, False)
    elif c == "drag":
        x1, y1, x2, y2 = map(float, a[:4]); steps = int(a[4]) if len(a) > 4 else 40
        delay = (float(a[5]) if len(a) > 5 else 10) / 1000
        move(x1, y1); time.sleep(0.1); btn(1, True); time.sleep(0.05)
        for i in range(1, steps + 1):
            t = i / steps; move(x1 + (x2 - x1) * t, y1 + (y2 - y1) * t); time.sleep(delay)
        time.sleep(0.05); btn(1, False)
    elif c == "key": key(a[0])
    elif c == "type": type_text(" ".join(a))
    elif c == "sleep": time.sleep(float(a[0]))
    elif c == "shot":
        subprocess.run(["import", "-display", os.environ.get("DISPLAY", ":77"), "-window", "root", a[0]], check=True)
    elif c == "windows": windows()
    else: raise SystemExit(f"unknown cmd {c}")

argv = sys.argv[1:]
cmd = []
for tok in argv + [";;"]:
    if tok == ";;":
        if cmd: run(cmd); time.sleep(0.15)
        cmd = []
    else:
        cmd.append(tok)
