#!/usr/bin/env python3
"""Generate Brushye's app icon: Phosphor's paint-brush glyph (MIT) on a rounded square.

Writes crates/brushye-app/assets/icon/brushye.png (256 px, for the AppImage / .desktop) and
brushye-64.rgba (raw 64x64 RGBA, the window icon). Needs Pillow.
"""
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "crates/brushye-app/assets/icon"
FONT = ROOT / "crates/brushye-app/assets/fonts/Phosphor-Fill.ttf"
GLYPH = ""  # PAINT_BRUSH (src/icons.rs)
BG = (104, 121, 160, 255)  # the interface's default-button blue (ui/theme.rs)


def icon(size):
    s = 4  # supersample, then downscale for smooth edges
    n = size * s
    im = Image.new("RGBA", (n, n), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    d.rounded_rectangle([0, 0, n - 1, n - 1], radius=n * 0.22, fill=BG)
    font = ImageFont.truetype(str(FONT), int(n * 0.66))
    box = d.textbbox((0, 0), GLYPH, font=font)
    w, h = box[2] - box[0], box[3] - box[1]
    d.text(((n - w) / 2 - box[0], (n - h) / 2 - box[1]), GLYPH, font=font, fill=(255, 255, 255, 255))
    return im.resize((size, size), Image.LANCZOS)


OUT.mkdir(parents=True, exist_ok=True)
icon(256).save(OUT / "brushye.png", optimize=True)
(OUT / "brushye-64.rgba").write_bytes(icon(64).tobytes())
print("wrote", OUT)
