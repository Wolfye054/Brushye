import sys, random
from PIL import Image
out = sys.argv[1]
W, H = 256, 512
base = Image.new("RGB", (W, H)); top = Image.new("RGB", (W, H))
bp, tp = base.load(), top.load()
for y in range(256):
    for x in range(256):
        bp[x, y] = (x, x, x); tp[x, y] = (y, y, y)
rng = random.Random(1132)
for y in range(256, 512):
    for x in range(256):
        bp[x, y] = tuple(rng.randrange(256) for _ in range(3))
        tp[x, y] = tuple(rng.randrange(256) for _ in range(3))
base.save(f"{out}/base.png"); top.save(f"{out}/top.png")
# top with per-pixel alpha: alpha = x (columns), rgb = y gray; base stays as is
ta = Image.new("RGBA", (W, H)); tap = ta.load()
for y in range(H):
    for x in range(W):
        r, g, b = tp[x, y]; tap[x, y] = (r, g, b, (x * 7 + y * 13) % 256)
ta.save(f"{out}/top_alpha.png")
