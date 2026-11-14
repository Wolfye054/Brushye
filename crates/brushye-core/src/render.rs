//! CPU reference renderer (DECISIONS D-012): flattens a document into one straight-RGBA8
//! image. This defines correct output; GPU display and export are tested against it.

use crate::blend::math::{composite, effective_alpha};
use crate::document::Document;
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
    let stride = w as usize;
    let mut px = vec![0u8; stride * h as usize * 4];
    if let Some(p) = doc.paper {
        for y in ry0..ry1 {
            for x in rx0..rx1 {
                let d = ((y - ry0) as usize * stride + (x - rx0) as usize) * 4;
                px[d..d + 4].copy_from_slice(&p.0);
            }
        }
    }
    let ts = TILE_SIZE as i64;
    let mut first = doc.paper.is_none();
    for layer in doc.layers() {
        if !layer.visible {
            continue;
        }
        let copy = std::mem::replace(&mut first, false);
        for (&(tx, ty), tile) in layer.tiles.iter() {
            let (tx0, ty0) = (i64::from(tx) * ts, i64::from(ty) * ts);
            // Intersection of the tile, the canvas and the rectangle.
            let (xa, xb) = (tx0.max(rx0), (tx0 + ts).min(rx1));
            let (ya, yb) = (ty0.max(ry0), (ty0 + ts).min(ry1));
            if xa >= xb || ya >= yb {
                continue;
            }
            for y in ya..yb {
                let iy = (y - ty0) as usize;
                for x in xa..xb {
                    let ix = (x - tx0) as usize;
                    let s = &tile.px[(iy * TILE_SIZE + ix) * 4..][..4];
                    if s[3] == 0 {
                        continue;
                    }
                    let at = effective_alpha(s[3], layer.opacity);
                    let d = ((y - ry0) as usize * stride + (x - rx0) as usize) * 4;
                    if copy {
                        px[d..d + 4].copy_from_slice(&[s[0], s[1], s[2], at]);
                        continue;
                    }
                    let b = [px[d], px[d + 1], px[d + 2], px[d + 3]];
                    let out = composite(layer.blend, b, [s[0], s[1], s[2]], at);
                    px[d..d + 4].copy_from_slice(&out);
                }
            }
        }
    }
    Image {
        width: w,
        height: h,
        px,
    }
}
