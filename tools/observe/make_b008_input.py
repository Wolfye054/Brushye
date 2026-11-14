#!/usr/bin/env python3
"""B-008 base image (256x256 RGBA): R = x, G = y, B = 255 - x; alpha 255 for y < 128,
alpha = 2*(y-128) for y >= 128 (0..254). Opened in CSP as the layer the brush paints on.
usage: make_b008_input.py OUT.png"""
import sys
from PIL import Image
im = Image.new("RGBA", (256, 256))
im.putdata([(x, y, 255 - x, 255 if y < 128 else 2 * (y - 128)) for y in range(256) for x in range(256)])
im.save(sys.argv[1])
