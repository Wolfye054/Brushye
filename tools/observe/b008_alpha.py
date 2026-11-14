#!/usr/bin/env python3
"""B-008: the brush ink blending modes, whole image, against the models in the spec.
Reproduces the B-008 result tables from specs/behavior/brush/data. Needs numpy (scratch venv)
for the L-007 non-separable pipeline (nonsep_models.py).

Base (make_b008_input.py): b = (x, y, 255 - x), alpha ab = 255 (y < 128) or 2(y - 128).
Stroke: one size-400 AA-None dab of colour S covering the canvas, opacity O (exactly 0.5 or 1).
r = B(., S) is the layer blend (L-001 tables for separable modes, L-007 integer model for the
others). Models (O as an exact fraction, A = 255 O):
  keep    alpha ab;  colour round(b + (B(b,S) - b) O)    [Screen: unrounded screen value]
  paint   alpha floor(65535 (1 - (1 - ab/255)(1 - O))) >> 8 = A';
          colour round((r A + b (A' - A)) / A') with r = B(bw, S),
          bw = (b ab + 255 (255 - ab) + 127) // 255  (backdrop over white);
          Multiply: r = S (65025 - ab (255 - b)) / 65025 (unrounded);  Brightness: r = B(b, S)
  glow    Add (Glow): round(b + S O); Glow dodge: floor(b 255 / (255 - S O))  (premultiplied, alpha kept)
  burn    Black/White burn: b -/+ 64 O^2, alpha kept
  erase   alpha round(ab (1 - O));  erase_compare: alpha min(ab, round(255 (1 - O)))
  replace_alpha: (S, a8);  compare_density: (S, a8) where ab <= a8, else unchanged
  background: alpha A'; colour round((b ab/255 + S O (1 - ab/255)) / (1 - (1 - ab/255)(1 - O)))
usage: b008_alpha.py DATA_DIR BLEND_OBSERVED_DIR [OPACITY ...] [--prefix B008_]
"""
import glob
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(__file__))
from nonsep_models import csp_int  # noqa: E402

S = np.array([64, 128, 192], dtype=np.int64)
LAYER_INDEX = {"normal": 0, "darken": 1, "multiply": 2, "color_burn": 3, "linear_burn": 4,
               "subtract": 5, "lighten": 6, "screen": 7, "color_dodge": 8, "glow_dodge": 9,
               "add": 10, "add_glow": 11, "overlay": 12, "soft_light": 13, "hard_light": 14,
               "difference": 15, "vivid_light": 16, "linear_light": 17, "pin_light": 18,
               "hard_mix": 19, "exclusion": 20, "divide": 23}
KEEP = {"color_burn", "lighten", "screen", "color_dodge", "add", "overlay", "soft_light",
        "vivid_light", "linear_light", "pin_light", "hard_mix", "divide", "darker_color",
        "lighter_color", "hue", "saturation", "color"}
PAINT = {"normal", "darken", "multiply", "linear_burn", "subtract", "hard_light", "difference",
         "exclusion", "brightness"}


def base():
    y, x = np.mgrid[0:256, 0:256]
    b = np.stack([x, y, 255 - x], -1).astype(np.int64)
    ab = np.where(y < 128, 255, 2 * (y - 128)).astype(np.int64)
    return b, ab


def ilum(a):
    return (30 * a[..., 0] + 59 * a[..., 1] + 11 * a[..., 2] + 50) // 100


def blend(mode, b, bd, s=S):
    T = np.broadcast_to(s, b.shape)
    if mode == "normal":
        return T.copy()
    if mode in LAYER_INDEX:
        t = np.asarray(Image.open(f"{bd}/opaque_{LAYER_INDEX[mode]:02d}_{mode}.png").convert("RGB"),
                       dtype=np.int64)[..., 0]  # pixel (x = b, y = s) holds f(b, s)
        return np.stack([t[s[c], b[..., c]] for c in range(3)], -1)
    if mode in ("hue", "saturation", "color", "brightness"):
        return csp_int(mode, b, T)
    if mode in ("darker_color", "lighter_color"):
        lb, ls = ilum(b), ilum(T)
        take = (ls <= lb) if mode == "darker_color" else (ls >= lb)
        return np.where(take[..., None], T, b)
    raise ValueError(mode)


def rnd(v):
    return np.floor(v + 0.5).astype(np.int64)


def model(mode, b, ab, O, bd, s=S):
    """(colour, alpha) predicted for one export."""
    ab3 = ab[..., None]
    A = 255.0 * O
    a8 = int(65535 * O) >> 8
    aof = 1 - (1 - ab / 255) * (1 - O)
    ap = np.floor(65535 * aof).astype(np.int64) >> 8  # A'
    T = np.broadcast_to(s, b.shape)
    if mode == "screen":  # unrounded screen
        bf = b.astype(float)
        return rnd(bf + ((bf + s - bf * s / 255) - bf) * O), ab
    if mode in KEEP:
        r = blend(mode, b, bd, s)
        return rnd(b + (r - b) * O), ab
    if mode in PAINT:
        if mode == "multiply":
            r = s * (65025 - ab3 * (255 - b)) / 65025
        elif mode == "brightness":
            r = blend(mode, b, bd, s)
        else:
            bw = (b * ab3 + 255 * (255 - ab3) + 127) // 255
            r = blend(mode, bw, bd, s)
        den = np.maximum(ap, 1)[..., None]
        return rnd((r * A + b * (den - A)) / den), ap
    if mode == "add_glow":
        return rnd(np.minimum(255, b + s * O)), ab
    if mode == "glow_dodge":
        sa = s * O
        # L-001 dodge (floor division) against the premultiplied colour S·O
        return np.where(b == 0, 0, np.minimum(255, np.floor(b * 255 / np.maximum(255 - sa, 1e-9)))).astype(np.int64), ab
    if mode in ("black_burn", "white_burn"):
        k = 64 * O * O
        return rnd(np.clip(b - k if mode == "black_burn" else b + k, 0, 255)), ab
    if mode == "erase":
        return b, rnd(ab * (1 - O))
    if mode == "erase_compare":
        return b, np.minimum(ab, int(np.floor(255 * (1 - O) + 0.5)))
    if mode == "replace_alpha":
        return T.copy(), np.full_like(ab, a8)
    if mode == "compare_density":
        win = ab <= a8
        return np.where(win[..., None], T, b), np.where(win, a8, ab)
    if mode == "background":
        abf = (ab / 255)[..., None]
        cf = (b * abf + s * O * (1 - abf)) / np.maximum(aof, 1e-12)[..., None]
        return rnd(cf), ap
    raise ValueError(mode)


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    prefix = next((a.split("=", 1)[1] for a in sys.argv if a.startswith("--prefix=")), "B008_")
    colour = next((a.split("=", 1)[1] for a in sys.argv if a.startswith("--colour=")), None)
    s = np.array([int(v) for v in colour.split(",")], dtype=np.int64) if colour else S
    d, bd = args[0], args[1]
    ops = args[2:] or ["100", "50"]
    b, ab = base()
    for op in ops:
        O = int(op) / 100
        print(f"opacity {op}:   mode: px exact (all 65,536 px; colour ignored where alpha is 0) "
              f"| opaque rows | max |dA| | max |dC|")
        for f in sorted(glob.glob(f"{d}/{prefix}op{op}_*.png")):
            mode = os.path.basename(f)[len(f"{prefix}op{op}_") + 3:-4]
            o = np.asarray(Image.open(f).convert("RGBA"), dtype=np.int64)
            c, ao = model(mode, b, ab, O, bd, s)
            vis = ao > 0
            good = (o[..., 3] == ao) & (np.all(o[..., :3] == c, -1) | ~vis)
            fam = "keep" if mode in KEEP else "paint" if mode in PAINT else "brush-only"
            print(f"  {mode:16} {fam:10} {int(good.sum()):6} | {int(good[:128].sum()):6} | "
                  f"{int(np.abs(o[..., 3] - ao).max()):3} | {int((np.abs(o[..., :3] - c).max(-1) * vis).max()):3}")


if __name__ == "__main__":
    main()
