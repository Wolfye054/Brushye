#!/usr/bin/env python3
"""Check the semi-transparent-backdrop compositing model (spec L-005) against CSP exports.

usage: check_backdrop_model.py INPUT_DIR OBS_DIR L001_OBSERVED_DIR
  INPUT_DIR:          bd_base.png, bd_top.png (make_blend_inputs2.py)
  OBS_DIR:            bd_NN_<mode>.png
  L001_OBSERVED_DIR:  specs/behavior/blending/data/observed (opaque_NN_<mode>.png), used as the
                      exact blend function B(b, s) for separable modes (LUT[s][b] per channel).

Observed (L-005, 2026-11-13): output alpha follows one of two paths --
  Normal:      ao = (ab*(32768 - at15) + 255*at15) >> 15            (15-bit lerp, floor)
  Screen, Color dodge, Glow dodge, Soft light, Hard light: ao = min(255, at + ab)   (additive)
  all other modes: ao = at + round(ab*(255 - at)/255)              (8-bit, rounded)
The colour model below is exact-ish for Normal (99.6 %); for other modes over a semi-transparent
backdrop the colour rule is still open (W3C is far off); the script reports both.

Model under test (straight alpha, 8-bit in/out, 15-bit alpha as in L-002):
  at15 = (at*32768 + 127) // 255
  ao   = (ab*(32768 - at15) + 255*at15) >> 15                    # output alpha (floor)
  s'   = lerp15(s, B(b, s), ab15)                                 # W3C: mix source by backdrop alpha
  wb   = ao - at                                                  # base weight, 8-bit integer
  out  = (s'*at + b*wb + ao//2) // ao                             # straight colour, rounded
"""
import sys, glob, os, re
import numpy as np
from PIL import Image

inp, obs, l001 = sys.argv[1], sys.argv[2], sys.argv[3]
Bb = np.asarray(Image.open(f"{inp}/bd_base.png")).astype(np.int64)
Tt = np.asarray(Image.open(f"{inp}/bd_top.png")).astype(np.int64)
b, ab = Bb[..., :3], Bb[..., 3:4]
s, at = Tt[..., :3], Tt[..., 3:4]
Q = 32768
ab15, at15 = (ab * Q + 127) // 255, (at * Q + 127) // 255
ao = (ab * (Q - at15) + 255 * at15) >> 15
wb = ao - at

def lut(mode_file):
    g = np.asarray(Image.open(mode_file).convert("RGB")).astype(np.int64)[:256, :, 0]  # [s, b]
    return lambda bb, ss: g[ss, bb]

def composite(Bf):
    smix = (s * (Q - ab15) + Bf * ab15 + Q // 2) >> 15
    return np.where(ao > 0, (smix * at + b * wb + ao // 2) // np.maximum(ao, 1), 0)

m = None
for f in sorted(glob.glob(f"{obs}/bd_*.png")):
    mm = re.match(r"bd_(\d\d)_(.+)\.png", os.path.basename(f))
    idx, mode = mm.group(1), mm.group(2)
    O = np.asarray(Image.open(f).convert("RGBA")).astype(np.int64)
    if O.shape[0] < b.shape[0]:  # stored crop: gray half only (specs data/observed2)
        O = np.concatenate([O, np.zeros((b.shape[0] - O.shape[0],) + O.shape[1:], np.int64)], axis=0)
    vis = O[..., 3] > 0
    ao_round = at + (ab * (255 - at) + 127) // 255
    rows = slice(0, int(vis.any(-1).nonzero()[0].max()) + 1)
    ao_add = np.minimum(255, at + ab)
    a_ok = np.mean(O[rows, :, 3:4] == ao[rows]) * 100
    a_ok_r = np.mean(O[rows, :, 3:4] == ao_round[rows]) * 100
    a_ok_add = np.mean(O[rows, :, 3:4] == ao_add[rows]) * 100
    src = glob.glob(f"{l001}/opaque_{idx}_{mode}.png")
    if mode in ("hue", "saturation", "color", "brightness", "darker_color", "lighter_color") or not src:
        print(f"{mode:13s} alpha normal {a_ok:7.3f}% / general {a_ok_r:7.3f}% / additive {a_ok_add:7.3f}%  (colour: non-separable, see L-007)")
        continue
    Bf = lut(src[0])(b, s)
    E = composite(Bf)
    d = (O[..., :3] - E)[vis]
    g = (O[:256, :, :3] - E[:256])[vis[:256]]
    aof = (at + ab * (255 - at) / 255).astype(float)
    w3c = (s * at * (255 - ab) / 255 + Bf * at * ab / 255 + b * ab * (255 - at) / 255) / np.maximum(aof, 1e-9)
    dw = (O[..., :3] - np.rint(w3c))[vis]
    print(f"{mode:13s} alpha normal {a_ok:7.3f}% / general {a_ok_r:7.3f}% / additive {a_ok_add:7.3f}%  colour model "
          f"{np.mean(d == 0) * 100:7.3f}%/max {abs(d).max():3d}  W3C float {np.mean(dw == 0) * 100:7.3f}%/max {abs(dw).max():.0f}")
