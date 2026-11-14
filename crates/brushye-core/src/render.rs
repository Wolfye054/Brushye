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
    let (w, h) = (doc.width as usize, doc.height as usize);
    let mut px = vec![0u8; w * h * 4];
    if let Some(p) = doc.paper {
        for c in px.as_chunks_mut::<4>().0 {
            c.copy_from_slice(&p.0);
        }
    }
    let mut first = doc.paper.is_none();
    for layer in doc.layers() {
        if !layer.visible {
            continue;
        }
        let copy = std::mem::replace(&mut first, false);
        for (&(tx, ty), tile) in layer.tiles.iter() {
            let (x0, y0) = (
                i64::from(tx) * TILE_SIZE as i64,
                i64::from(ty) * TILE_SIZE as i64,
            );
            for iy in 0..TILE_SIZE {
                let y = y0 + iy as i64;
                if y < 0 || y >= h as i64 {
                    continue;
                }
                for ix in 0..TILE_SIZE {
                    let x = x0 + ix as i64;
                    if x < 0 || x >= w as i64 {
                        continue;
                    }
                    let s = &tile.px[(iy * TILE_SIZE + ix) * 4..][..4];
                    if s[3] == 0 {
                        continue;
                    }
                    let at = effective_alpha(s[3], layer.opacity);
                    let d = (y as usize * w + x as usize) * 4;
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
        width: doc.width,
        height: doc.height,
        px,
    }
}
