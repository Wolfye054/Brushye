#!/usr/bin/env python3
"""Check the layer-opacity model (spec L-003) against observed CSP exports.

usage: check_opacity_model.py INPUT_DIR OBS_DIR
  INPUT_DIR: combo_base.png, combo_top.png (make_blend_inputs2.py)
  OBS_DIR:   combo_<mode>_op<NN>.png (blend_export_runs.sh)

Model (CSP 1.13.2, observed 2026-11-13):
  o8  = floor(opacity% * 255 / 100)                 layer opacity to 8 bit
  ae8 = (a * (o8 + 1)) >> 8                         combined with per-pixel alpha a (8 bit)
  a15 = (ae8 * 32768 + 127) // 255                  to 15 bit (as in L-002)
  out = (b * (32768 - a15) + B(b, s) * a15 + 16384) >> 15
  Glow modes (Add (Glow), Glow dodge) premultiply the source with a15 instead (L-002).
"""
import sys, glob, os, re
import numpy as np
from PIL import Image

inp, obs = sys.argv[1], sys.argv[2]
B = np.asarray(Image.open(f"{inp}/combo_base.png").convert("RGB")).astype(np.int64)
TA = np.asarray(Image.open(f"{inp}/combo_top.png")).astype(np.int64)
S, A = TA[..., :3], TA[..., 3:4]
Q = 32768

BLEND = {
    "normal": lambda b, s: s,
    "multiply": lambda b, s: (b * s + 127) // 255,
    "screen": lambda b, s: b + s - (b * s + 127) // 255,
    "add": lambda b, s: np.minimum(255, b + s),
}

def model(mode, op):
    o8 = op * 255 // 100
    ae8 = (A * (o8 + 1)) >> 8
    a15 = (ae8 * Q + 127) // 255
    if mode == "add_glow":
        return np.minimum(255, B + ((S * a15 + Q // 2) >> 15))
    return (B * (Q - a15) + BLEND[mode](B, S) * a15 + Q // 2) >> 15

for f in sorted(glob.glob(f"{obs}/combo_*_op*.png")):
    m = re.match(r"combo_(.+)_op(\d+)\.png", os.path.basename(f))
    mode, op = m.group(1), int(m.group(2))
    if mode not in BLEND and mode != "add_glow":
        continue
    O = np.asarray(Image.open(f).convert("RGB")).astype(np.int64)
    E = model(mode, op)
    if O.shape[0] == 960:  # stored crop: rows 0-191 + 512-1279 (specs data/observed2)
        E = np.concatenate([E[:192], E[512:]], axis=0)
        rows = {"opaque sweep rows 0-191": slice(0, 192), "alpha rows 512-1279": slice(192, None)}
    else:
        rows = {"opaque rows 0-511": slice(0, 512), "alpha rows 512-1279": slice(512, None)}
    d = O - E
    print(f"{mode:9s} op{op:3d}  " + "  ".join(
        f"{k}: {np.mean(d[v] == 0) * 100:7.3f}%/max {abs(d[v]).max()}" for k, v in rows.items()))
