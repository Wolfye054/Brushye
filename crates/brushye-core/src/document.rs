//! The document: canvas, paper and a layer stack.
//!
//! Layers are `Arc`-shared between document versions; editing a layer clones only that layer
//! (and, when pixels change, only the touched tiles) — DECISIONS D-013.

use std::fmt;
use std::sync::Arc;

use crate::blend::BlendMode;
use crate::hash::Fnv64;
use crate::ids::LayerId;
use crate::tile::TileGrid;

/// Straight (non-premultiplied) 8-bit RGBA colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Rgba8(pub [u8; 4]);

impl Rgba8 {
    pub const WHITE: Rgba8 = Rgba8([255, 255, 255, 255]);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum LayerKind {
    #[default]
    Raster,
}

impl LayerKind {
    pub fn name(self) -> &'static str {
        match self {
            LayerKind::Raster => "raster",
        }
    }
    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "raster" => Some(LayerKind::Raster),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub kind: LayerKind,
    pub visible: bool,
    /// Layer opacity in percent, 0..=100 (CSP's integer opacity; spec L-003).
    pub opacity: u8,
    pub blend: BlendMode,
    /// Lock layer (no edits to pixels or properties other than unlocking).
    pub locked: bool,
    /// Lock transparent pixels.
    pub lock_alpha: bool,
    pub tiles: TileGrid,
}

impl Layer {
    pub fn new(id: LayerId, name: impl Into<String>, kind: LayerKind) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            visible: true,
            opacity: 100,
            blend: BlendMode::Normal,
            locked: false,
            lock_alpha: false,
            tiles: TileGrid::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DocError {
    DuplicateLayerId(LayerId),
    NoSuchLayer(LayerId),
    InvalidSize { width: u32, height: u32 },
    OpacityOutOfRange(u32),
    LayerLocked(LayerId),
}

impl fmt::Display for DocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocError::DuplicateLayerId(id) => write!(f, "layer id {id} already exists"),
            DocError::NoSuchLayer(id) => write!(f, "no layer {id}"),
            DocError::InvalidSize { width, height } => {
                write!(
                    f,
                    "invalid canvas size {width}×{height} (1..={MAX_SIDE} per side)"
                )
            }
            DocError::OpacityOutOfRange(v) => write!(f, "opacity {v} out of range 0..=100"),
            DocError::LayerLocked(id) => write!(f, "layer {id} is locked"),
        }
    }
}

impl std::error::Error for DocError {}

/// Largest canvas side in pixels.
pub const MAX_SIDE: u32 = 32_768;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub width: u32,
    pub height: u32,
    pub dpi: u32,
    /// Paper colour; `None` = no paper (transparent background).
    pub paper: Option<Rgba8>,
    /// Layers from bottom to top.
    layers: Vec<Arc<Layer>>,
}

impl Document {
    pub fn new(width: u32, height: u32, dpi: u32, paper: Option<Rgba8>) -> Result<Self, DocError> {
        if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE {
            return Err(DocError::InvalidSize { width, height });
        }
        Ok(Self {
            width,
            height,
            dpi,
            paper,
            layers: Vec::new(),
        })
    }

    /// Layers from bottom to top.
    pub fn layers(&self) -> &[Arc<Layer>] {
        &self.layers
    }

    pub fn layer(&self, id: LayerId) -> Option<&Arc<Layer>> {
        self.layers.iter().find(|l| l.id == id)
    }

    fn index(&self, id: LayerId) -> Result<usize, DocError> {
        self.layers
            .iter()
            .position(|l| l.id == id)
            .ok_or(DocError::NoSuchLayer(id))
    }

    /// Insert a layer directly above `above` (or at the bottom when `None`... see `top`).
    /// `position`: `Some(id)` = directly above that layer; `None` = on top.
    pub fn add_layer(&mut self, layer: Layer, above: Option<LayerId>) -> Result<(), DocError> {
        if self.layer(layer.id).is_some() {
            return Err(DocError::DuplicateLayerId(layer.id));
        }
        let at = match above {
            Some(a) => self.index(a)? + 1,
            None => self.layers.len(),
        };
        self.layers.insert(at, Arc::new(layer));
        Ok(())
    }

    pub fn remove_layer(&mut self, id: LayerId) -> Result<Layer, DocError> {
        let i = self.index(id)?;
        let l = self.layers.remove(i);
        Ok(Arc::unwrap_or_clone(l))
    }

    /// Move a layer directly above `above` (`None` = to the bottom).
    pub fn move_layer(&mut self, id: LayerId, above: Option<LayerId>) -> Result<(), DocError> {
        if above == Some(id) {
            return Ok(());
        }
        let i = self.index(id)?;
        if let Some(a) = above {
            self.index(a)?;
        }
        let l = self.layers.remove(i);
        let at = match above {
            Some(a) => self.index(a)? + 1,
            None => 0,
        };
        self.layers.insert(at, l);
        Ok(())
    }

    /// Mutable access to a layer; clones it only if another document version shares it.
    pub fn layer_mut(&mut self, id: LayerId) -> Result<&mut Layer, DocError> {
        let i = self.index(id)?;
        Ok(Arc::make_mut(&mut self.layers[i]))
    }

    /// Deterministic content hash (D-012): identical on every platform for identical documents.
    pub fn content_hash(&self) -> u64 {
        let mut h = Fnv64::default();
        h.bytes(b"brushye-doc-v1")
            .u32(self.width)
            .u32(self.height)
            .u32(self.dpi);
        match self.paper {
            Some(p) => h.bytes(&[1]).bytes(&p.0),
            None => h.bytes(&[0]),
        };
        h.u32(u32::try_from(self.layers.len()).unwrap_or(u32::MAX));
        for l in &self.layers {
            h.u32(l.id.0)
                .str(&l.name)
                .str(l.kind.name())
                .str(l.blend.name());
            h.bytes(&[
                u8::from(l.visible),
                l.opacity,
                u8::from(l.locked),
                u8::from(l.lock_alpha),
            ]);
            h.u32(u32::try_from(l.tiles.len()).unwrap_or(u32::MAX));
            for ((tx, ty), t) in l.tiles.iter() {
                h.i32(*tx).i32(*ty).bytes(&t.px);
            }
        }
        h.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc() -> Document {
        Document::new(100, 80, 72, Some(Rgba8::WHITE)).unwrap_or_else(|e| unreachable!("{e}"))
    }

    fn ids(d: &Document) -> Vec<u32> {
        d.layers().iter().map(|l| l.id.0).collect()
    }

    #[test]
    fn rejects_bad_sizes() {
        assert!(Document::new(0, 10, 72, None).is_err());
        assert!(Document::new(10, MAX_SIDE + 1, 72, None).is_err());
    }

    #[test]
    fn add_move_remove_keep_order() {
        let mut d = doc();
        for i in 1..=3 {
            d.add_layer(
                Layer::new(LayerId(i), format!("L{i}"), LayerKind::Raster),
                None,
            )
            .unwrap_or_else(|e| unreachable!("{e}"));
        }
        assert_eq!(ids(&d), vec![1, 2, 3]);
        d.move_layer(LayerId(3), None)
            .unwrap_or_else(|e| unreachable!("{e}"));
        assert_eq!(ids(&d), vec![3, 1, 2]);
        d.move_layer(LayerId(3), Some(LayerId(1)))
            .unwrap_or_else(|e| unreachable!("{e}"));
        assert_eq!(ids(&d), vec![1, 3, 2]);
        d.add_layer(
            Layer::new(LayerId(4), "x", LayerKind::Raster),
            Some(LayerId(1)),
        )
        .unwrap_or_else(|e| unreachable!("{e}"));
        assert_eq!(ids(&d), vec![1, 4, 3, 2]);
        d.remove_layer(LayerId(3))
            .unwrap_or_else(|e| unreachable!("{e}"));
        assert_eq!(ids(&d), vec![1, 4, 2]);
        assert_eq!(
            d.add_layer(Layer::new(LayerId(1), "dup", LayerKind::Raster), None),
            Err(DocError::DuplicateLayerId(LayerId(1)))
        );
        assert_eq!(
            d.remove_layer(LayerId(9)),
            Err(DocError::NoSuchLayer(LayerId(9)))
        );
    }

    #[test]
    fn edits_share_unchanged_layers_between_versions() {
        let mut v1 = doc();
        v1.add_layer(Layer::new(LayerId(1), "a", LayerKind::Raster), None)
            .unwrap_or_else(|e| unreachable!("{e}"));
        v1.add_layer(Layer::new(LayerId(2), "b", LayerKind::Raster), None)
            .unwrap_or_else(|e| unreachable!("{e}"));
        let mut v2 = v1.clone();
        v2.layer_mut(LayerId(2))
            .unwrap_or_else(|e| unreachable!("{e}"))
            .opacity = 50;
        assert!(Arc::ptr_eq(&v1.layers()[0], &v2.layers()[0]));
        assert!(!Arc::ptr_eq(&v1.layers()[1], &v2.layers()[1]));
        assert_eq!(v1.layers()[1].opacity, 100);
    }

    #[test]
    fn hash_changes_with_content_and_is_stable() {
        let mut a = doc();
        a.add_layer(Layer::new(LayerId(1), "a", LayerKind::Raster), None)
            .unwrap_or_else(|e| unreachable!("{e}"));
        let b = a.clone();
        assert_eq!(a.content_hash(), b.content_hash());
        a.layer_mut(LayerId(1))
            .unwrap_or_else(|e| unreachable!("{e}"))
            .blend = BlendMode::Multiply;
        assert_ne!(a.content_hash(), b.content_hash());
    }
}
