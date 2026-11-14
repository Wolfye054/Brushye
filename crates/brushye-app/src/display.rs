//! CPU side of the canvas display (D-012 4): the flattened document, kept per 256² tile and
//! recomposited only where something changed.
//!
//! A display tile's *signature* lists, for every layer, the properties that affect rendering
//! and the identity of its stored tile there. Tiles are shared copy-on-write between document
//! versions (D-013) and the signature holds a reference to each, so an unchanged tile is the
//! very same allocation and a changed one never is. No dirty flags to keep in step.

use std::collections::HashMap;
use std::sync::Arc;

use brushye_command::core::render::flatten_rect;
use brushye_command::core::{BlendMode, Document, LayerId, Rgba8, TILE_SIZE, Tile};

pub type TileKey = (i32, i32);

#[derive(Clone, PartialEq)]
struct LayerSig {
    id: LayerId,
    visible: bool,
    opacity: u8,
    blend: BlendMode,
    tile: Option<Arc<Tile>>,
}

#[derive(Clone)]
struct Sig {
    paper: Option<Rgba8>,
    layers: Vec<LayerSig>,
}

impl PartialEq for Sig {
    fn eq(&self, o: &Self) -> bool {
        self.paper == o.paper
            && self.layers.len() == o.layers.len()
            && self.layers.iter().zip(&o.layers).all(|(a, b)| {
                a.id == b.id
                    && a.visible == b.visible
                    && a.opacity == b.opacity
                    && a.blend == b.blend
                    && match (&a.tile, &b.tile) {
                        (None, None) => true,
                        (Some(x), Some(y)) => Arc::ptr_eq(x, y),
                        _ => false,
                    }
            })
    }
}

/// A composited tile to upload: straight RGBA8, `w`×`h` (edge tiles are clipped).
#[derive(Clone)]
pub struct Upload {
    pub key: TileKey,
    pub w: u32,
    pub h: u32,
    pub px: Arc<Vec<u8>>,
}

#[derive(Default)]
pub struct Display {
    size: (u32, u32),
    sigs: HashMap<TileKey, Sig>,
    /// The composited pixels of every tile (for the navigator and the eyedropper).
    tiles: HashMap<TileKey, Upload>,
    /// Increases whenever any tile changes.
    pub generation: u64,
}

impl Display {
    /// Recomposite changed tiles. Returns the uploads and whether the GPU must drop all
    /// textures first (canvas size changed).
    pub fn update(&mut self, doc: &Document) -> (Vec<Upload>, bool) {
        let reset = self.size != (doc.width, doc.height);
        if reset {
            self.sigs.clear();
            self.tiles.clear();
            self.size = (doc.width, doc.height);
        }
        let ts = TILE_SIZE as u32;
        let (nx, ny) = (doc.width.div_ceil(ts), doc.height.div_ceil(ts));
        let mut out = Vec::new();
        for ty in 0..ny {
            for tx in 0..nx {
                let key = (tx as i32, ty as i32);
                let sig = Sig {
                    paper: doc.paper,
                    layers: doc
                        .layers()
                        .iter()
                        .map(|l| LayerSig {
                            id: l.id,
                            visible: l.visible,
                            opacity: l.opacity,
                            blend: l.blend,
                            tile: l.tiles.get(key).cloned(),
                        })
                        .collect(),
                };
                if self.sigs.get(&key) == Some(&sig) {
                    continue;
                }
                let (x0, y0) = (tx * ts, ty * ts);
                let (w, h) = (ts.min(doc.width - x0), ts.min(doc.height - y0));
                let img = flatten_rect(doc, x0, y0, w, h);
                let u = Upload {
                    key,
                    w,
                    h,
                    px: Arc::new(img.px),
                };
                self.tiles.insert(key, u.clone());
                out.push(u);
                self.sigs.insert(key, sig);
            }
        }
        if !out.is_empty() {
            self.generation += 1;
        }
        (out, reset)
    }

    /// The displayed (flattened) pixel at canvas (x, y).
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        let ts = TILE_SIZE as u32;
        let u = self.tiles.get(&((x / ts) as i32, (y / ts) as i32))?;
        let (ix, iy) = (x % ts, y % ts);
        if ix >= u.w || iy >= u.h {
            return None;
        }
        let i = ((iy * u.w + ix) * 4) as usize;
        Some([u.px[i], u.px[i + 1], u.px[i + 2], u.px[i + 3]])
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use brushye_command::Session;

    #[test]
    fn only_changed_tiles_are_recomposited_and_tiles_match_the_renderer() {
        let mut s = Session::new();
        let run = |s: &mut Session, t: &str| {
            s.run_script(t).unwrap_or_else(|e| unreachable!("{e}"));
        };
        run(
            &mut s,
            "doc.new 600 300\nlayer.new L1\nlayer.fill L1 color=#336699\nlayer.new L2",
        );
        let mut d = Display::default();
        let doc = s.document().unwrap_or_else(|| unreachable!("doc"));
        let (up, reset) = d.update(&doc);
        assert!(reset);
        assert_eq!(up.len(), 6);
        assert!(d.update(&doc).0.is_empty());
        // A stroke inside tile (1, 0) changes that tile only.
        run(
            &mut s,
            "brush.stroke L2 color=#ffffff size=10 samples=[300,100,1,0]",
        );
        let doc = s.document().unwrap_or_else(|| unreachable!("doc"));
        let (up, reset) = d.update(&doc);
        assert!(!reset);
        assert_eq!(up.iter().map(|u| u.key).collect::<Vec<_>>(), [(1, 0)]);
        // Layer opacity changes every tile; the edge tile is clipped to the canvas.
        run(&mut s, "layer.set L1 opacity=50");
        let doc = s.document().unwrap_or_else(|| unreachable!("doc"));
        let (up, _) = d.update(&doc);
        assert_eq!(up.len(), 6);
        let full = brushye_command::core::render::flatten(&doc);
        for u in &up {
            assert_eq!(
                (u.w, u.h),
                (
                    if u.key.0 == 2 { 88 } else { 256 },
                    if u.key.1 == 1 { 44 } else { 256 }
                )
            );
            for y in 0..u.h {
                for x in 0..u.w {
                    let i = ((y * u.w + x) * 4) as usize;
                    let p = full.pixel(u.key.0 as u32 * 256 + x, u.key.1 as u32 * 256 + y);
                    assert_eq!(&u.px[i..i + 4], &p);
                }
            }
        }
    }
}
