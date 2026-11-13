"""Candidate models for CSP's non-separable blend modes (spec L-007). Inputs/outputs uint8 arrays."""
import numpy as np

def _lum(c, w):
    return c[..., 0:1] * w[0] + c[..., 1:2] * w[1] + c[..., 2:3] * w[2]

def _clip(c, w):
    l = _lum(c, w); n = c.min(-1, keepdims=True); x = c.max(-1, keepdims=True)
    c = np.where(n < 0, l + (c - l) * l / np.maximum(l - n, 1e-12), c)
    return np.where(x > 1, l + (c - l) * (1 - l) / np.maximum(x - l, 1e-12), c)

def setlum(c, l, w): return _clip(c + (l - _lum(c, w)), w)
def sat(c): return c.max(-1, keepdims=True) - c.min(-1, keepdims=True)
def setsat(c, s):
    mx = c.max(-1, keepdims=True); mn = c.min(-1, keepdims=True); r = mx - mn
    return np.where(r > 0, (c - mn) * s / np.maximum(r, 1e-12), 0)

def w3c(mode, b8, s8, w=(.3, .59, .11), rnd=np.rint):
    b = b8 / 255.0; s = s8 / 255.0
    if mode == "hue": r = setlum(setsat(s, sat(b)), _lum(b, w), w)
    elif mode == "saturation": r = setlum(setsat(b, sat(s)), _lum(b, w), w)
    elif mode == "color": r = setlum(s, _lum(b, w), w)
    elif mode == "brightness": r = setlum(b, _lum(s, w), w)
    else: raise ValueError(mode)
    return rnd(np.clip(r, 0, 1) * 255).astype(np.int64)

def pick(mode, b8, s8, lum_fn, tie="base"):
    """Darker/Lighter color: whole-pixel choice by luma; tie -> 'base' or 'top'."""
    lb, ls = lum_fn(b8), lum_fn(s8)
    if mode == "darker_color":
        take_s = (ls < lb) if tie == "base" else (ls <= lb)
    else:
        take_s = (ls > lb) if tie == "base" else (ls >= lb)
    return np.where(take_s[..., None], s8, b8)

# ---- integer pipeline found for CSP 1.13.2 (L-007): 8-bit integers, luma = round((30R+59G+11B)/100)
def _ilum(a):
    return (30 * a[..., 0:1] + 59 * a[..., 1:2] + 11 * a[..., 2:3] + 50) // 100

def _isetsat(c, s):
    mn = c.min(-1, keepdims=True); r = c.max(-1, keepdims=True) - mn; den = np.maximum(r, 1)
    return np.where(r > 0, ((c - mn) * s + den // 2) // den, 0)

def _iclip(c, l, variant="w3c_round"):
    """Clip an integer colour c (may leave 0..255) towards luma l, W3C ClipColor in float."""
    c = c.astype(float); l = l.astype(float)
    n = c.min(-1, keepdims=True); x = c.max(-1, keepdims=True)
    if variant.startswith("w3c_lumc"):
        l = (30 * c[..., 0:1] + 59 * c[..., 1:2] + 11 * c[..., 2:3]) / 100
    c = np.where(n < 0, l + (c - l) * l / np.maximum(l - n, 1e-12), c)
    c = np.where(x > 255, l + (c - l) * (255 - l) / np.maximum(x - l, 1e-12), c)
    if variant.endswith("floor"):
        return np.clip(np.floor(c), 0, 255).astype(np.int64)
    if variant.endswith("trunc_half"):
        return np.clip(np.floor(c + 0.5), 0, 255).astype(np.int64)
    return np.clip(np.rint(c), 0, 255).astype(np.int64)

def csp_int(mode, B, T, clip="w3c_round"):
    if mode == "hue":
        c = _isetsat(T, (B.max(-1, keepdims=True) - B.min(-1, keepdims=True))); lt = _ilum(B)
    elif mode == "saturation":
        c = _isetsat(B, (T.max(-1, keepdims=True) - T.min(-1, keepdims=True))); lt = _ilum(B)
    elif mode == "color":
        c = T.copy(); lt = _ilum(B)
    elif mode == "brightness":
        c = B.copy(); lt = _ilum(T)
    else:
        raise ValueError(mode)
    c2 = c + (lt - _ilum(c))
    return _iclip(c2, lt, clip)
