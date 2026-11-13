#!/usr/bin/env python3
"""Second-round blend test inputs (D-017 pre-M3 observations; specs L-003, L-005, L-007).

usage: make_blend_inputs2.py OUT_DIR

Writes (no ICC/gAMA chunks):
  combo_base.png  256x1280 RGB    combo_top.png  256x1280 RGBA
    rows    0..191  L-007 one-channel sweeps (top alpha 255)
      block k = row//64 (0..2) is the swept channel; row%64 picks a fixed colour pair from
      FIXED; column x = the swept value. Rows 0..31 of each block sweep the TOP channel
      (base fixed), rows 32..63 sweep the BASE channel (top fixed).
    rows  192..511  L-007 equal-luma pairs (top alpha 255):
      192..351 pairs with equal (30R+59G+11B), 352..511 pairs with equal (77R+151G+28B).
    rows  512..1279 Multiply / opacity alpha sweep: 3 blocks of 256 rows; base colour fixed per
      block (BALPHA); top = gray x; top alpha = row%256.
  bd_base.png  256x512 RGBA  /  bd_top.png  256x512 RGBA   (L-005 transparent backdrop)
    rows 0..255: base gray x, top gray y, base alpha (7x+13y)%256, top alpha (11x+5y+91)%256
    rows 256..511: random colours (seed 2027) with the same alpha pattern.
"""
import sys, random
from PIL import Image

out = sys.argv[1]
W = 256
FIXED = [  # (base_or_top_fixed_colour, other_two_channel_values) pairs; deterministic list
    (0, 0, 0), (255, 255, 255), (128, 128, 128), (255, 0, 0), (0, 255, 0), (0, 0, 255),
    (200, 60, 30), (30, 200, 60), (60, 30, 200), (90, 90, 200), (240, 200, 10), (10, 240, 200),
    (128, 64, 32), (32, 128, 64), (64, 32, 128), (250, 120, 180),
]
BALPHA = [(10, 64, 128), (191, 230, 255), (37, 100, 170)]

def eq_luma_pairs(rng, w, n):
    """n pairs (b, s) of distinct colours with equal integer luma sum w·rgb."""
    pairs = []
    while len(pairs) < n:
        b = tuple(rng.randrange(256) for _ in range(3))
        target = sum(c * k for c, k in zip(b, w))
        # pick s0, s1 randomly, solve s2
        for _ in range(50):
            s0, s1 = rng.randrange(256), rng.randrange(256)
            rem = target - s0 * w[0] - s1 * w[1]
            if rem % w[2] == 0 and 0 <= rem // w[2] <= 255:
                s = (s0, s1, rem // w[2])
                if s != b:
                    pairs.append((b, s))
                break
    return pairs

def combo():
    H = 1280
    base = Image.new("RGB", (W, H)); top = Image.new("RGBA", (W, H))
    bp, tp = base.load(), top.load()
    # sweeps
    for y in range(192):
        ch, r = y // 64, y % 64
        fixed = FIXED[r % 16]
        other = FIXED[(r * 7 + 3) % 16]
        for x in range(W):
            if r < 32:  # sweep top channel
                b = fixed; s = list(other); s[ch] = x; s = tuple(s)
            else:       # sweep base channel
                s = fixed; b = list(other); b[ch] = x; b = tuple(b)
            bp[x, y] = b; tp[x, y] = s + (255,)
    rng = random.Random(1133)
    for y0, w in ((192, (30, 59, 11)), (352, (77, 151, 28))):
        pairs = eq_luma_pairs(rng, w, 160 * W)
        for i, (b, s) in enumerate(pairs):
            x, y = i % W, y0 + i // W
            bp[x, y] = b; tp[x, y] = s + (255,)
    for k, bc in enumerate(BALPHA):
        for r in range(256):
            y = 512 + k * 256 + r
            for x in range(W):
                bp[x, y] = bc; tp[x, y] = (x, x, x, r)
    base.save(f"{out}/combo_base.png"); top.save(f"{out}/combo_top.png")

def backdrop():
    H = 512
    base = Image.new("RGBA", (W, H)); top = Image.new("RGBA", (W, H))
    bp, tp = base.load(), top.load()
    rng = random.Random(2027)
    for y in range(H):
        for x in range(W):
            ab, at = (7 * x + 13 * y) % 256, (11 * x + 5 * y + 91) % 256
            if y < 256:
                b, s = (x, x, x), (y, y, y)
            else:
                b = tuple(rng.randrange(256) for _ in range(3)); s = tuple(rng.randrange(256) for _ in range(3))
            bp[x, y] = b + (ab,); tp[x, y] = s + (at,)
    base.save(f"{out}/bd_base.png"); top.save(f"{out}/bd_top.png")

combo(); backdrop()
