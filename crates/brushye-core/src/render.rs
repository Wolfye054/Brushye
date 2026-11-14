//! CPU reference renderer (DECISIONS D-012): flattens a document into one straight-RGBA8
//! image. This defines correct output; GPU display and export are tested against it.

use crate::blend::BlendMode;
use crate::blend::math::{a15, composite, effective_alpha};
use crate::document::{Document, Layer};
use crate::tile::TILE_SIZE;

/// A flattened image: straight (non-premultiplied) RGBA8, row-major.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub px: Vec<u8>,
}

impl Image {
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let i = (y as usize * self.width as usize + x as usize) * 4;
        [self.px[i], self.px[i + 1], self.px[i + 2], self.px[i + 3]]
    }
}

/// Flatten the document: paper (if any), then each visible layer bottom to top, composited
/// with its blend mode and opacity (L-002/L-003/L-005). Tiles are visited in sorted order;
/// layers without a stored tile at a pixel leave it unchanged (a transparent source is the
/// identity in every mode).
///
/// Without paper, the **first visible layer is copied** onto the empty canvas (its alpha
/// scaled by opacity) instead of being composited: CSP's observed backdrop alpha equals the
/// bottom layer's stored alpha, while compositing onto transparent pixels follows L-005's
/// lossy 15-bit rule. Observed at 100 % opacity in Normal mode; other cases are open (L-005).
pub fn flatten(doc: &Document) -> Image {
    flatten_rect(doc, 0, 0, doc.width, doc.height)
}

/// Flatten only the canvas rectangle at (`x0`, `y0`) of size `w`×`h` (clipped to the canvas),
/// with exactly the same per-pixel arithmetic as [`flatten`]. The result is `w`×`h`; pixels
/// outside the canvas stay transparent. Used for incremental display (D-012 4).
pub fn flatten_rect(doc: &Document, x0: u32, y0: u32, w: u32, h: u32) -> Image {
    let (cw, ch) = (i64::from(doc.width), i64::from(doc.height));
    let (rx0, ry0) = (i64::from(x0), i64::from(y0));
    let (rx1, ry1) = ((rx0 + i64::from(w)).min(cw), (ry0 + i64::from(h)).min(ch));
    let r = Rect {
        x0: rx0,
        y0: ry0,
        stride: w as usize,
        rows: h as usize,
        x1: rx1,
        y1: ry1,
    };
    let mut px = vec![0u8; r.stride * r.rows * 4];
    if let Some(p) = doc.paper {
        for y in ry0..ry1 {
            for x in rx0..rx1 {
                let d = r.at(x, y);
                px[d..d + 4].copy_from_slice(&p.0);
            }
        }
    }
    let top: Vec<&Layer> = doc.children(None).into_iter().map(|l| &**l).collect();
    render_list(doc, &top, &r, &mut px, doc.paper.is_none());
    Image {
        width: w,
        height: h,
        px,
    }
}

/// The output rectangle: origin (`x0`, `y0`), `stride`×`rows` pixels; only canvas pixels
/// (x < `x1`, y < `y1`) are drawn.
struct Rect {
    x0: i64,
    y0: i64,
    stride: usize,
    rows: usize,
    x1: i64,
    y1: i64,
}

impl Rect {
    fn at(&self, x: i64, y: i64) -> usize {
        ((y - self.y0) as usize * self.stride + (x - self.x0) as usize) * 4
    }
    fn len(&self) -> usize {
        self.stride * self.rows * 4
    }
}

/// Visit a raster layer's stored pixels inside the rectangle: `f(buffer index, rgba)`.
/// Pixels without a stored tile are transparent and skipped.
fn each_pixel(layer: &Layer, r: &Rect, mut f: impl FnMut(usize, [u8; 4])) {
    let ts = TILE_SIZE as i64;
    for (&(tx, ty), tile) in layer.tiles.iter() {
        let (tx0, ty0) = (i64::from(tx) * ts, i64::from(ty) * ts);
        let (xa, xb) = (tx0.max(r.x0), (tx0 + ts).min(r.x1));
        let (ya, yb) = (ty0.max(r.y0), (ty0 + ts).min(r.y1));
        if xa >= xb || ya >= yb {
            continue;
        }
        for y in ya..yb {
            let iy = (y - ty0) as usize;
            for x in xa..xb {
                let ix = (x - tx0) as usize;
                let s = &tile.px[(iy * TILE_SIZE + ix) * 4..][..4];
                if s[3] != 0 {
                    f(r.at(x, y), [s[0], s[1], s[2], s[3]]);
                }
            }
        }
    }
}

/// Composite one source pixel (straight RGBA, alpha already including opacity) into `px[d]`.
/// `copy`: the canvas is still empty (no paper, nothing drawn yet): the pixel is copied, as
/// CSP's observed backdrop alpha is the bottom layer's own (L-005).
fn put(px: &mut [u8], d: usize, mode: BlendMode, s: [u8; 3], at: u8, copy: bool) {
    if at == 0 {
        return;
    }
    if copy {
        px[d..d + 4].copy_from_slice(&[s[0], s[1], s[2], at]);
        return;
    }
    let b = [px[d], px[d + 1], px[d + 2], px[d + 3]];
    px[d..d + 4].copy_from_slice(&composite(mode, b, s, at));
}

/// A layer's own content in the rectangle, as a straight-RGBA buffer: a raster layer's
/// pixels, or a folder's children composited in isolation onto transparent (L-004).
fn content(doc: &Document, layer: &Layer, r: &Rect) -> Vec<u8> {
    let mut buf = vec![0u8; r.len()];
    if layer.is_folder() {
        let kids = children(doc, layer);
        render_list(doc, &kids, r, &mut buf, true);
    } else {
        each_pixel(layer, r, |d, s| buf[d..d + 4].copy_from_slice(&s));
    }
    buf
}

fn children<'a>(doc: &'a Document, folder: &Layer) -> Vec<&'a Layer> {
    doc.children(Some(folder.id))
        .into_iter()
        .map(|l| &**l)
        .collect()
}

/// `px = lerp15(before, px, a15(o8))` on every canvas pixel of the rectangle: a Through
/// folder's opacity (L-004).
fn opacity_lerp(before: &[u8], px: &mut [u8], r: &Rect, opacity: u8) {
    let w = a15(effective_alpha(255, opacity));
    for y in r.y0..r.y1 {
        for x in r.x0..r.x1 {
            let d = r.at(x, y);
            for i in d..d + 4 {
                px[i] = ((u32::from(before[i]) * (32768 - w) + u32::from(px[i]) * w + 16384) >> 15)
                    as u8;
            }
        }
    }
}

/// Draw one layer (no clipping) onto `px`. Returns whether anything may have been drawn.
fn draw(doc: &Document, layer: &Layer, r: &Rect, px: &mut [u8], first: bool) -> bool {
    if !layer.visible {
        return false;
    }
    if layer.is_folder() && layer.blend == BlendMode::Through {
        // Through: the children blend straight into the backdrop (L-004).
        let before = (layer.opacity < 100).then(|| px.to_vec());
        let kids = children(doc, layer);
        let still_first = render_list(doc, &kids, r, px, first);
        if let Some(b) = before {
            opacity_lerp(&b, px, r, layer.opacity);
        }
        return !still_first;
    }
    if layer.is_folder() {
        // A Normal folder's composited result blends like one layer: its opacity scales the
        // result's alpha exactly as layer opacity does (L-004 run 3, partly transparent
        // result; only Through folders lerp).
        let src = content(doc, layer, r);
        composite_buffer(&src, r, px, layer.blend, layer.opacity, first);
        return true;
    }
    each_pixel(layer, r, |d, s| {
        let at = effective_alpha(s[3], layer.opacity);
        put(px, d, layer.blend, [s[0], s[1], s[2]], at, first);
    });
    true
}

/// Composite a whole source buffer onto `px` with a blend mode and layer opacity (L-003).
fn composite_buffer(src: &[u8], r: &Rect, px: &mut [u8], mode: BlendMode, opacity: u8, copy: bool) {
    for y in r.y0..r.y1 {
        for x in r.x0..r.x1 {
            let d = r.at(x, y);
            let at = effective_alpha(src[d + 3], opacity);
            put(px, d, mode, [src[d], src[d + 1], src[d + 2]], at, copy);
        }
    }
}

/// Draw a list of sibling layers (bottom to top) onto `px`, handling clipping groups.
/// `first`: nothing has been drawn on an empty, paperless canvas yet. Returns the updated
/// `first`.
fn render_list(doc: &Document, list: &[&Layer], r: &Rect, px: &mut [u8], mut first: bool) -> bool {
    let mut i = 0;
    while i < list.len() {
        let base = list[i];
        let mut j = i + 1;
        while j < list.len() && list[j].clip {
            j += 1;
        }
        let clipped = &list[i + 1..j];
        let through = base.is_folder() && base.blend == BlendMode::Through;
        if clipped.is_empty() || through {
            // No clipping group; with a Through folder as base, clipping is off and the
            // clipped layers draw as ordinary layers (L-006 run 2).
            for l in &list[i..j] {
                if draw(doc, l, r, px, first) && l.visible {
                    first = false;
                }
            }
        } else if base.visible {
            // Clipping group (L-006): the clipped layers composite onto the base's colour,
            // treated as opaque; the group keeps the base's alpha, then blends like the base.
            let mut group = content(doc, base, r);
            let alpha: Vec<u8> = group.iter().skip(3).step_by(4).copied().collect();
            for k in (3..group.len()).step_by(4) {
                group[k] = 255;
            }
            for c in clipped.iter().filter(|c| c.visible) {
                let src = content(doc, c, r);
                composite_buffer(&src, r, &mut group, c.blend, c.opacity, false);
            }
            for (k, a) in alpha.into_iter().enumerate() {
                group[k * 4 + 3] = a;
            }
            // The group blends like its base, including a folder base's opacity (L-004 run 3).
            composite_buffer(&group, r, px, base.blend, base.opacity, first);
            first = false;
        }
        i = j;
    }
    first
}
