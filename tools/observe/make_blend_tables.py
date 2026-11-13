#!/usr/bin/env python3
"""Generate the observed blend tables shipped in brushye-core (DECISIONS D-017).

usage: make_blend_tables.py SPEC_OBSERVED_DIR OUT_DIR
Reads specs/behavior/blending/data/observed/opaque_NN_<mode>.png (L-001). In the gray half
(rows 0..255) pixel (x, y) is the result for base value b = x and top value s = y, identical in
R, G and B. Writes OUT_DIR/<mode>.bin: 65536 bytes, index b*256 + s.
Only modes without a known exact formula are generated: overlay, hard_light, vivid_light.
"""
import sys
from PIL import Image

MODES = {"overlay": 12, "hard_light": 14, "vivid_light": 16}
src, out = sys.argv[1], sys.argv[2]
for mode, nn in MODES.items():
    im = Image.open(f"{src}/opaque_{nn:02d}_{mode}.png").convert("RGB")
    px = im.load()
    table = bytearray(65536)
    for s in range(256):
        for b in range(256):
            r, g, bl = px[b, s]
            if not (r == g == bl):
                raise SystemExit(f"{mode}: channels differ at b={b} s={s}")
            table[b * 256 + s] = r
    with open(f"{out}/{mode}.bin", "wb") as f:
        f.write(table)
    print(mode, "ok")
