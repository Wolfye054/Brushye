#!/usr/bin/env python3
"""Make Brushye's app icon files from the master image (the maintainer's own artwork).

    make_app_icon.py [SOURCE.png]

SOURCE defaults to crates/brushye-app/assets/icon/brushye-master.png; a given SOURCE is first
stored as the 1024 px master. Writes brushye.png (256 px: AppImage / .desktop) and
brushye-64.rgba (raw 64x64 RGBA: the window icon). Needs Pillow.
"""
import sys
from pathlib import Path

from PIL import Image

OUT = Path(__file__).resolve().parent.parent / "crates/brushye-app/assets/icon"
MASTER = OUT / "brushye-master.png"

if len(sys.argv) > 1:
    Image.open(sys.argv[1]).convert("RGBA").resize((1024, 1024), Image.LANCZOS).save(
        MASTER, optimize=True
    )
src = Image.open(MASTER).convert("RGBA")
src.resize((256, 256), Image.LANCZOS).save(OUT / "brushye.png", optimize=True)
(OUT / "brushye-64.rgba").write_bytes(src.resize((64, 64), Image.LANCZOS).tobytes())
print("wrote", OUT)
