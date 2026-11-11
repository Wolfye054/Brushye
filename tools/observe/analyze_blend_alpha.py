#!/usr/bin/env python3
"""Test how CSP mixes a blend result by per-pixel source alpha (opaque base layer).

usage: analyze_blend_alpha.py INPUT_DIR OBS_DIR
INPUT_DIR has base.png, top.png and top_alpha.png (make_blend_inputs.py). OBS_DIR has
opaque_NN_mode.png and alpha_NN_mode.png exported from CSP.
B(Cb,Cs) is taken from the *observed* opaque export, so this isolates the alpha step.
"""
import sys, glob, os
import numpy as np
from PIL import Image

inp, obs = sys.argv[1], sys.argv[2]
def load(p): return np.asarray(Image.open(p).convert("RGBA")).astype(np.int64)
Cb = load(f"{inp}/base.png")[..., :3]
TA = load(f"{inp}/top_alpha.png"); Cs = TA[..., :3]; a = TA[..., 3:4]

def cands(name, O_opaque):
    c = {
        "lerp round  (Cb*(255-a)+B*a+127)//255": (Cb * (255 - a) + O_opaque * a + 127) // 255,
        "lerp floor": (Cb * (255 - a) + O_opaque * a) // 255,
        "lerp round-half-down": (Cb * (255 - a) + O_opaque * a + 254) // 255 - 1 + ((Cb * (255 - a) + O_opaque * a) % 255 == 0),
    }
    pm = (Cs * a + 127) // 255  # premultiplied source
    if name == "add_glow":
        c["glow: min(255, Cb + round(Cs*a/255))"] = np.minimum(255, Cb + pm)
        c["glow: min(255, Cb + floor(Cs*a/255))"] = np.minimum(255, Cb + (Cs * a) // 255)
    if name == "glow_dodge":
        c["glow: dodge(Cb, round(Cs*a/255)) floor-int"] = np.where(Cb == 0, 0, np.where(pm == 255, 255, np.minimum(255, Cb * 255 // np.maximum(255 - pm, 1))))
    return c

for f in sorted(glob.glob(f"{obs}/alpha_*.png")):
    name = os.path.basename(f)[9:-4]
    O = load(f)
    Oop = load(f.replace("/alpha_", "/opaque_"))[..., :3]
    print(f"== {name}   (output alpha min/max {O[...,3].min()}/{O[...,3].max()})")
    for k, E in cands(name, Oop).items():
        d = O[..., :3] - E
        print(f"   {k:48s} exact {np.mean(d == 0)*100:7.3f}%  max|d| {abs(d).max()}")
