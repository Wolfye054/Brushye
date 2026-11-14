#!/usr/bin/env python3
"""B-008: brush ink blending modes vs layer blending (pure Python + Pillow).

Inputs: the base image (make_b008_input.py: R = x, G = y, B = 255 - x; alpha 255 for y < 128,
alpha 2*(y-128) below), one size-400 AA-None dab of colour S = (64,128,192) covering the canvas,
exported per mode: DATA/B008_op<O>_<NN>_<mode>.png.

For separable modes, the opaque rows (y < 128) are compared channel-wise with CSP's own LAYER
blend result for the same (backdrop, source) pair, read from the L-001 export
specs/behavior/blending/data/observed/opaque_NN_<mode>.png (pixel (x=b, y=s) = f(b, s)).
At opacity < 100 the layer result is lerped with alpha a8 using L-002's 15-bit rule.
The semi-transparent rows (y >= 128) are summarised (alpha in -> alpha out) per mode.

usage: b008_analyze.py BRUSH_DATA_DIR BLEND_OBSERVED_DIR [OPACITY ...]
"""
import glob
import os
import sys

from PIL import Image

S = (64, 128, 192)
LAYER_INDEX = {"normal": 0, "darken": 1, "multiply": 2, "color_burn": 3, "linear_burn": 4,
               "subtract": 5, "lighten": 6, "screen": 7, "color_dodge": 8, "glow_dodge": 9,
               "add": 10, "add_glow": 11, "overlay": 12, "soft_light": 13, "hard_light": 14,
               "difference": 15, "vivid_light": 16, "linear_light": 17, "pin_light": 18,
               "hard_mix": 19, "exclusion": 20, "darker_color": 21, "lighter_color": 22,
               "divide": 23, "hue": 24, "saturation": 25, "color": 26, "brightness": 27}
SEPARABLE = set(LAYER_INDEX) - {"darker_color", "lighter_color", "hue", "saturation", "color",
                                "brightness"}


def base(x, y):
    return (x, y, 255 - x, 255 if y < 128 else 2 * (y - 128))


def a15(a):
    return (a * 32768 + 127) // 255


def lerp15(b, r, a8):
    w = a15(a8)
    return (b * (32768 - w) + r * w + 16384) >> 15


def main():
    d, bd = sys.argv[1], sys.argv[2]
    ops = sys.argv[3:] or ["100", "50"]
    tables = {}
    for op in ops:
        a8 = int(op) * 255 // 100
        print(f"== opacity {op} (a8 = {a8})")
        for f in sorted(glob.glob(f"{d}/B008_op{op}_*.png")):
            mode = os.path.basename(f)[len(f"B008_op{op}_") + 3:-4]
            im = Image.open(f).convert("RGBA")
            px = im.load()
            line = f"  {mode:16}"
            if mode in SEPARABLE:
                if mode not in tables:
                    t = Image.open(f"{bd}/opaque_{LAYER_INDEX[mode]:02d}_{mode}.png").convert("RGB").load()
                    tables[mode] = t
                t = tables[mode]
                tot = bad = worst = 0
                for y in range(128):
                    for x in range(256):
                        b = base(x, y)
                        o = px[x, y]
                        for c in range(3):
                            r = t[b[c], S[c]][0]
                            want = r if a8 == 255 else lerp15(b[c], r, a8)
                            tot += 1
                            if o[c] != want:
                                bad += 1
                                worst = max(worst, abs(o[c] - want))
                        if o[3] != 255:
                            bad += 1
                line += f" opaque rows vs layer mode: {tot - bad}/{tot} exact, max {worst}"
            else:
                vals = {px[x, y] for y in range(0, 128, 9) for x in range(0, 256, 37)}
                line += f" opaque rows: {len(vals)} distinct samples, e.g. " + ", ".join(
                    f"{base(x, y)[:3]}->{px[x, y]}" for x, y in ((0, 0), (128, 64), (255, 127)))
            # semi-transparent rows: alpha mapping at x = 64
            amap = [(base(64, y)[3], px[64, y][3]) for y in (128, 129, 140, 160, 192, 224, 255)]
            line += "\n" + " " * 18 + "alpha in->out: " + " ".join(f"{i}->{o}" for i, o in amap)
            line += "\n" + " " * 18 + "rgb at a_in 64/190: " + f"{px[64, 160]} {px[200, 223]}"
            print(line)


if __name__ == "__main__":
    main()
