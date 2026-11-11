#!/usr/bin/env python3
"""Check the current Brushye blend model against observed CSP 1.13.2 exports.

usage: check_blend_model.py INPUT_DIR OBS_DIR
  INPUT_DIR: base.png, top.png, top_alpha.png (make_blend_inputs.py)
  OBS_DIR:   opaque_NN_<mode>.png, alpha_NN_<mode>.png (blend_export_loop.sh)

Prints, per mode, % of channel values that match exactly and the max abs error, for the
gray half (rows 0-255, all 256x256 (base, top) pairs) and the random-colour half (rows 256-511).
Model status is documented in specs/behavior/blending/L-001 and L-002.
"""
import sys, os, glob
import numpy as np
from PIL import Image

inp, obs = sys.argv[1], sys.argv[2]
def load(p): return np.asarray(Image.open(p).convert("RGBA")).astype(np.int64)
b = load(f"{inp}/base.png")[..., :3]
s = load(f"{inp}/top.png")[..., :3]
TA = load(f"{inp}/top_alpha.png"); sa = TA[..., :3]; a = TA[..., 3:4]
Q = 32768

def div_floor(n, d): return n // np.maximum(d, 1)

# ---- separable modes: exact 8-bit integer formulas (opaque) ----
def burn(b, s):  return np.where(b == 255, 255, np.where(s == 0, 0, np.clip(255 - div_floor((255 - b) * 255, s), 0, 255)))
def dodge(b, s): return np.where(b == 0, 0, np.where(s == 255, 255, np.clip(div_floor(b * 255, 255 - s), 0, 255)))
def mul(b, s):   return (b * s + 127) // 255
def softlight(b, s):
    fb, fs = b / 255, s / 255
    d = np.where(fb <= .25, ((16 * fb - 12) * fb + 4) * fb, np.sqrt(fb))
    return np.rint(255 * np.where(fs <= .5, fb - (1 - 2 * fs) * fb * (1 - fb), fb + (2 * fs - 1) * (d - fb))).astype(np.int64)
def hardlight_w3c(b, s):  # closest known; NOT exact in the screen half (open)
    return np.where(s < 128, (2 * b * s + 127) // 255, 255 - (2 * (255 - b) * (255 - s) + 127) // 255)
def vivid_w3c(b, s):      # closest known; NOT exact (open)
    fb, fs = b / 255, s / 255
    v = np.where(fs <= .5, np.where(s == 0, 0, np.clip(1 - (1 - fb) / np.maximum(2 * fs, 1e-9), 0, 1)),
                 np.where(s == 255, 1, np.clip(fb / np.maximum(2 * (1 - fs), 1e-9), 0, 1)))
    return np.rint(v * 255).astype(np.int64)

# ---- non-separable: W3C with .30/.59/.11 (closest; NOT exact, open) ----
def _lum(c): return c[..., 0:1] * .3 + c[..., 1:2] * .59 + c[..., 2:3] * .11
def _clip(c):
    l = _lum(c); n = c.min(-1, keepdims=True); x = c.max(-1, keepdims=True)
    c = np.where(n < 0, l + (c - l) * l / np.maximum(l - n, 1e-12), c)
    return np.where(x > 1, l + (c - l) * (1 - l) / np.maximum(x - l, 1e-12), c)
def _setlum(c, l): return _clip(c + (l - _lum(c)))
def _sat(c): return c.max(-1, keepdims=True) - c.min(-1, keepdims=True)
def _setsat(c, v):
    mx = c.max(-1, keepdims=True); mn = c.min(-1, keepdims=True); r = mx - mn
    return np.where(r > 0, (c - mn) * v / np.maximum(r, 1e-12), 0)
def nonsep(f):
    return lambda b, s: np.rint(np.clip(f(b / 255, s / 255), 0, 1) * 255).astype(np.int64)

MODEL = {
    "normal": lambda b, s: s,
    "darken": np.minimum, "multiply": mul, "color_burn": burn,
    "linear_burn": lambda b, s: np.clip(b + s - 255, 0, 255),
    "subtract": lambda b, s: np.clip(b - s, 0, 255),
    "lighten": np.maximum, "screen": lambda b, s: b + s - mul(b, s),
    "color_dodge": dodge, "glow_dodge": dodge,
    "add": lambda b, s: np.minimum(255, b + s), "add_glow": lambda b, s: np.minimum(255, b + s),
    "overlay": lambda b, s: hardlight_w3c(s, b), "soft_light": softlight, "hard_light": hardlight_w3c,
    "difference": lambda b, s: abs(b - s), "vivid_light": vivid_w3c,
    "linear_light": lambda b, s: np.clip(b + 2 * s - 255, 0, 255),
    "pin_light": lambda b, s: np.where(s < 128, np.minimum(b, 2 * s), np.maximum(b, 2 * s - 255)),
    "hard_mix": lambda b, s: np.where(s < 128, (b + s >= 255) * 255, (b + s > 255) * 255),
    "exclusion": lambda b, s: b + s - (2 * b * s + 127) // 255,
    "darker_color": nonsep(lambda b, s: np.where(_lum(s) < _lum(b), s, b)),
    "lighter_color": nonsep(lambda b, s: np.where(_lum(s) > _lum(b), s, b)),
    "divide": lambda b, s: np.where(s == 0, np.where(b > 0, 255, 0), np.minimum(255, div_floor(b * 255 + s // 2, s))),
    "hue": nonsep(lambda b, s: _setlum(_setsat(s, _sat(b)), _lum(b))),
    "saturation": nonsep(lambda b, s: _setlum(_setsat(b, _sat(s)), _lum(b))),
    "color": nonsep(lambda b, s: _setlum(s, _lum(b))),
    "brightness": nonsep(lambda b, s: _setlum(b, _lum(s))),
}

def with_alpha(mode, b, s, a, B_opaque):
    """Source alpha step (opaque backdrop). B_opaque = blend result at full alpha."""
    a15 = (a * Q + 127) // 255
    if mode in ("add_glow", "glow_dodge"):            # glow variants premultiply the source
        pm = (s * a15 + Q // 2) >> 15
        return np.minimum(255, b + pm) if mode == "add_glow" else dodge(b, pm)
    return (b * (Q - a15) + B_opaque * a15 + Q // 2) >> 15   # 15-bit alpha lerp

def report(tag, O, E):
    g, c = (O - E)[:256], (O - E)[256:]
    return f"{tag}: gray {np.mean(g == 0)*100:7.3f}%/{abs(g).max():3d}  color {np.mean(c == 0)*100:7.3f}%/{abs(c).max():3d}"

for f in sorted(glob.glob(f"{obs}/opaque_*.png")):
    mode = os.path.basename(f)[10:-4]
    E = MODEL[mode](b, s)
    line = f"{mode:14s} " + report("opaque", load(f)[..., :3], E)
    fa = f.replace("/opaque_", "/alpha_")
    if os.path.exists(fa):
        Bop = MODEL[mode](b, sa)
        line += "  | " + report("alpha", load(fa)[..., :3], with_alpha(mode, b, sa, a, Bop))
    print(line)
