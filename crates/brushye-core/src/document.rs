//! The document: canvas, paper and a layer stack.
//!
//! Layers are `Arc`-shared between document versions; editing a layer clones only that layer
//! (and, when pixels change, only the touched tiles) — DECISIONS D-013.
//!
//! **Folders (D-023):** the stack is one bottom-to-top list. A layer's `parent` names its
//! folder; a folder's subtree is the contiguous run of layers directly below the folder's own
//! entry (its children, theirs, …), as in CSP's Layer palette. Every structural edit keeps
//! that invariant; [`Document::check_structure`] verifies it (e.g. for loaded files).

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
    /// A layer folder: no pixels of its own; renders its children (L-004).
    Folder,
}

impl LayerKind {
    pub fn name(self) -> &'static str {
        match self {
            LayerKind::Raster => "raster",
            LayerKind::Folder => "folder",
        }
    }
    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "raster" => Some(LayerKind::Raster),
            "folder" => Some(LayerKind::Folder),
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
    /// The folder this layer is in (`None` = top level).
    pub parent: Option<LayerId>,
    /// Clip to Layer Below (L-006): drawn only where the clip base (the nearest unclipped
    /// sibling below) has pixels.
    pub clip: bool,
    /// Folders: open in the Layer palette (not a History step, H-002).
    pub expanded: bool,
    pub tiles: TileGrid,
}

impl Layer {
    /// Write a `w`×`h` block of straight RGBA8 pixels (row-major) with its top-left at
    /// (`x`, `y`) in canvas pixels. Pixels outside `bounds` (canvas width/height) are skipped.
    /// Tiles that end up fully transparent are dropped.
    pub fn put_pixels(&mut self, bounds: (u32, u32), x: i64, y: i64, w: u32, h: u32, rgba: &[u8]) {
        use crate::tile::TILE_SIZE;
        let ts = TILE_SIZE as i64;
        for row in 0..i64::from(h) {
            let cy = y + row;
            if cy < 0 || cy >= i64::from(bounds.1) {
                continue;
            }
            for col in 0..i64::from(w) {
                let cx = x + col;
                if cx < 0 || cx >= i64::from(bounds.0) {
                    continue;
                }
                let src = ((row * i64::from(w) + col) * 4) as usize;
                let Some(p) = rgba.get(src..src + 4) else {
                    continue;
                };
                let (tx, ty) = (cx.div_euclid(ts), cy.div_euclid(ts));
                let (ix, iy) = (cx.rem_euclid(ts), cy.rem_euclid(ts));
                if p[3] == 0 && self.tiles.get((tx as i32, ty as i32)).is_none() {
                    continue;
                }
                let tile = self.tiles.get_mut((tx as i32, ty as i32));
                let dst = ((iy * ts + ix) * 4) as usize;
                tile.px[dst..dst + 4].copy_from_slice(p);
            }
        }
        self.tiles.prune();
    }

    /// Fill the whole canvas area of the layer with one colour.
    pub fn fill(&mut self, bounds: (u32, u32), c: Rgba8) {
        let row: Vec<u8> =
            c.0.iter()
                .copied()
                .cycle()
                .take(bounds.0 as usize * 4)
                .collect();
        let mut block = Vec::with_capacity(row.len() * bounds.1 as usize);
        for _ in 0..bounds.1 {
            block.extend_from_slice(&row);
        }
        self.put_pixels(bounds, 0, 0, bounds.0, bounds.1, &block);
    }

    /// Pixel at canvas (`x`, `y`); transparent where no tile is stored.
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        use crate::tile::TILE_SIZE;
        let (tx, ty) = (
            (x as usize / TILE_SIZE) as i32,
            (y as usize / TILE_SIZE) as i32,
        );
        let i = ((y as usize % TILE_SIZE) * TILE_SIZE + x as usize % TILE_SIZE) * 4;
        self.tiles
            .get((tx, ty))
            .map_or([0; 4], |t| [t.px[i], t.px[i + 1], t.px[i + 2], t.px[i + 3]])
    }

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
            parent: None,
            clip: false,
            expanded: true,
            tiles: TileGrid::default(),
        }
    }

    pub fn is_folder(&self) -> bool {
        self.kind == LayerKind::Folder
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DocError {
    DuplicateLayerId(LayerId),
    NoSuchLayer(LayerId),
    InvalidSize {
        width: u32,
        height: u32,
    },
    OpacityOutOfRange(u32),
    LayerLocked(LayerId),
    NotAFolder(LayerId),
    /// A folder can't go into itself or one of its own descendants.
    IntoOwnSubtree(LayerId),
    /// Through is a folder-only mode.
    ThroughOnRaster(LayerId),
    /// The layer list breaks the folder invariant (loaded files).
    BadStructure(String),
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
            DocError::NotAFolder(id) => write!(f, "{id} is not a folder"),
            DocError::IntoOwnSubtree(id) => {
                write!(f, "can't move {id} into itself or a folder inside it")
            }
            DocError::ThroughOnRaster(id) => {
                write!(f, "{id}: Through is only for folders")
            }
            DocError::BadStructure(m) => write!(f, "layer structure: {m}"),
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

    /// Index range of a layer's subtree: the layer itself and, for a folder, everything in it
    /// (the contiguous run directly below its entry).
    pub fn subtree(&self, id: LayerId) -> Result<std::ops::Range<usize>, DocError> {
        let i = self.index(id)?;
        let mut start = i;
        while start > 0 && self.is_inside(self.layers[start - 1].id, id) {
            start -= 1;
        }
        Ok(start..i + 1)
    }

    /// Whether `id` is (transitively) inside folder `folder`.
    pub fn is_inside(&self, id: LayerId, folder: LayerId) -> bool {
        let mut p = self.layer(id).and_then(|l| l.parent);
        // The structure is a tree, so this terminates; the bound guards a corrupt list.
        for _ in 0..=self.layers.len() {
            match p {
                Some(x) if x == folder => return true,
                Some(x) => p = self.layer(x).and_then(|l| l.parent),
                None => return false,
            }
        }
        false
    }

    /// The direct children of `parent` (`None` = top level), bottom to top.
    pub fn children(&self, parent: Option<LayerId>) -> Vec<&Arc<Layer>> {
        self.layers.iter().filter(|l| l.parent == parent).collect()
    }

    fn folder(&self, id: LayerId) -> Result<&Arc<Layer>, DocError> {
        let f = self.layer(id).ok_or(DocError::NoSuchLayer(id))?;
        if f.is_folder() {
            Ok(f)
        } else {
            Err(DocError::NotAFolder(id))
        }
    }

    /// Insert a layer directly above `above` (in `above`'s folder), or on top of the stack
    /// when `None`. The layer's own `parent` is overwritten.
    pub fn add_layer(&mut self, mut layer: Layer, above: Option<LayerId>) -> Result<(), DocError> {
        if self.layer(layer.id).is_some() {
            return Err(DocError::DuplicateLayerId(layer.id));
        }
        let (at, parent) = match above {
            Some(a) => (self.index(a)? + 1, self.layers[self.index(a)?].parent),
            None => (self.layers.len(), None),
        };
        layer.parent = parent;
        self.layers.insert(at, Arc::new(layer));
        Ok(())
    }

    /// Insert a layer at the top of folder `folder`'s contents.
    pub fn add_layer_into(&mut self, mut layer: Layer, folder: LayerId) -> Result<(), DocError> {
        if self.layer(layer.id).is_some() {
            return Err(DocError::DuplicateLayerId(layer.id));
        }
        self.folder(folder)?;
        let at = self.index(folder)?;
        layer.parent = Some(folder);
        self.layers.insert(at, Arc::new(layer));
        Ok(())
    }

    /// Remove a layer; a folder is removed with everything in it. Returns the removed layer.
    pub fn remove_layer(&mut self, id: LayerId) -> Result<Layer, DocError> {
        let r = self.subtree(id)?;
        let removed: Vec<_> = self.layers.drain(r).collect();
        removed
            .into_iter()
            .last()
            .map(Arc::unwrap_or_clone)
            .ok_or(DocError::NoSuchLayer(id))
    }

    /// Take a layer's subtree out of the list, for re-inserting (a move).
    fn take_subtree(&mut self, id: LayerId) -> Result<Vec<Arc<Layer>>, DocError> {
        let r = self.subtree(id)?;
        Ok(self.layers.drain(r).collect())
    }

    fn set_parent(run: &mut [Arc<Layer>], parent: Option<LayerId>) {
        if let Some(root) = run.last_mut() {
            Arc::make_mut(root).parent = parent;
        }
    }

    /// Move a layer (a folder with its contents) directly above `above`, into `above`'s
    /// folder; `None` = to the bottom of the stack, at the top level.
    pub fn move_layer(&mut self, id: LayerId, above: Option<LayerId>) -> Result<(), DocError> {
        if above == Some(id) {
            return Ok(());
        }
        self.index(id)?;
        if let Some(a) = above {
            self.index(a)?;
            if self.is_inside(a, id) {
                return Err(DocError::IntoOwnSubtree(id));
            }
        }
        let mut run = self.take_subtree(id)?;
        let (at, parent) = match above {
            Some(a) => (self.index(a)? + 1, self.layers[self.index(a)?].parent),
            None => (0, None),
        };
        Self::set_parent(&mut run, parent);
        self.layers.splice(at..at, run);
        Ok(())
    }

    /// Move a layer (with its contents) to the top of folder `folder`.
    pub fn move_into(&mut self, id: LayerId, folder: LayerId) -> Result<(), DocError> {
        self.folder(folder)?;
        if id == folder || self.is_inside(folder, id) {
            return Err(DocError::IntoOwnSubtree(id));
        }
        let mut run = self.take_subtree(id)?;
        let at = self.index(folder)?;
        Self::set_parent(&mut run, Some(folder));
        self.layers.splice(at..at, run);
        Ok(())
    }

    /// Wrap a layer (with its contents) in a new folder, which takes its place
    /// (Create folder and insert layer, Ctrl+G).
    pub fn group(&mut self, mut folder: Layer, id: LayerId) -> Result<(), DocError> {
        if self.layer(folder.id).is_some() {
            return Err(DocError::DuplicateLayerId(folder.id));
        }
        if !folder.is_folder() {
            return Err(DocError::NotAFolder(folder.id));
        }
        let i = self.index(id)?;
        folder.parent = self.layers[i].parent;
        Arc::make_mut(&mut self.layers[i]).parent = Some(folder.id);
        self.layers.insert(i + 1, Arc::new(folder));
        Ok(())
    }

    /// Remove a folder and leave its contents in its place (Ungroup Layer Folder).
    pub fn ungroup(&mut self, id: LayerId) -> Result<(), DocError> {
        let parent = self.folder(id)?.parent;
        let i = self.index(id)?;
        for l in &mut self.layers {
            if l.parent == Some(id) {
                Arc::make_mut(l).parent = parent;
            }
        }
        self.layers.remove(i);
        Ok(())
    }

    /// Check the folder invariant: unique ids, parents are folders, subtrees contiguous,
    /// Through only on folders.
    pub fn check_structure(&self) -> Result<(), DocError> {
        let bad = |m: String| Err(DocError::BadStructure(m));
        for (i, l) in self.layers.iter().enumerate() {
            if self.layers[..i].iter().any(|o| o.id == l.id) {
                return Err(DocError::DuplicateLayerId(l.id));
            }
            if l.blend == BlendMode::Through && !l.is_folder() {
                return Err(DocError::ThroughOnRaster(l.id));
            }
            if let Some(p) = l.parent {
                let Some(pi) = self.layers.iter().position(|o| o.id == p) else {
                    return bad(format!("{} is in missing folder {p}", l.id));
                };
                if !self.layers[pi].is_folder() {
                    return bad(format!("{} is in {p}, which is not a folder", l.id));
                }
                if pi <= i {
                    return bad(format!("{} is above its folder {p}", l.id));
                }
                // Everything between the layer and its folder must be inside that folder.
                if let Some(o) = self.layers[i + 1..pi]
                    .iter()
                    .find(|o| !self.is_inside(o.id, p))
                {
                    return bad(format!("{} sits between {} and its folder {p}", o.id, l.id));
                }
            }
        }
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
            // D-023: new fields join the hash only when not at their default, so documents
            // without folders keep the hashes they always had.
            if l.parent.is_some() || l.clip || !l.expanded {
                h.bytes(b"struct");
                h.u32(l.parent.map_or(0, |p| p.0 + 1));
                h.bytes(&[u8::from(l.clip), u8::from(l.expanded)]);
            }
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

    fn layer(id: u32, kind: LayerKind) -> Layer {
        Layer::new(LayerId(id), format!("L{id}"), kind)
    }

    fn ok<T>(r: Result<T, DocError>) -> T {
        r.unwrap_or_else(|e| unreachable!("{e}"))
    }

    /// (id, parent) bottom to top.
    fn tree(d: &Document) -> Vec<(u32, u32)> {
        d.layers()
            .iter()
            .map(|l| (l.id.0, l.parent.map_or(0, |p| p.0)))
            .collect()
    }

    #[test]
    fn folders_keep_subtrees_contiguous() {
        let mut d = doc();
        ok(d.add_layer(layer(1, LayerKind::Raster), None));
        ok(d.add_layer(layer(2, LayerKind::Raster), None));
        // Ctrl+G on L1: F10 takes its place, L1 inside.
        ok(d.group(layer(10, LayerKind::Folder), LayerId(1)));
        assert_eq!(tree(&d), vec![(1, 10), (10, 0), (2, 0)]);
        // New layer above L1 stays in the folder; into= puts it at the folder's top.
        ok(d.add_layer(layer(3, LayerKind::Raster), Some(LayerId(1))));
        ok(d.add_layer_into(layer(4, LayerKind::Raster), LayerId(10)));
        assert_eq!(tree(&d), vec![(1, 10), (3, 10), (4, 10), (10, 0), (2, 0)]);
        // Nested folder, then move the outer folder (with everything) above L2.
        ok(d.group(layer(11, LayerKind::Folder), LayerId(3)));
        assert_eq!(
            tree(&d),
            vec![(1, 10), (3, 11), (11, 10), (4, 10), (10, 0), (2, 0)]
        );
        ok(d.move_layer(LayerId(10), Some(LayerId(2))));
        assert_eq!(
            tree(&d),
            vec![(2, 0), (1, 10), (3, 11), (11, 10), (4, 10), (10, 0)]
        );
        ok(d.check_structure());
        assert_eq!(ok(d.subtree(LayerId(10))), 1..6);
        // A folder can't go into itself or its descendants.
        assert!(d.move_into(LayerId(10), LayerId(11)).is_err());
        assert!(d.move_layer(LayerId(10), Some(LayerId(3))).is_err());
        assert!(
            d.add_layer_into(layer(5, LayerKind::Raster), LayerId(2))
                .is_err()
        );
        // Move a layer out to the top level, and into a folder.
        ok(d.move_layer(LayerId(4), None));
        ok(d.move_into(LayerId(2), LayerId(11)));
        assert_eq!(
            tree(&d),
            vec![(4, 0), (1, 10), (3, 11), (2, 11), (11, 10), (10, 0)]
        );
        ok(d.check_structure());
        // Ungroup F11: its children go to F10 in place.
        ok(d.ungroup(LayerId(11)));
        assert_eq!(tree(&d), vec![(4, 0), (1, 10), (3, 10), (2, 10), (10, 0)]);
        // Deleting a folder deletes its contents.
        ok(d.remove_layer(LayerId(10)));
        assert_eq!(tree(&d), vec![(4, 0)]);
        ok(d.check_structure());
    }

    #[test]
    fn structure_check_rejects_broken_lists() {
        let mut d = doc();
        ok(d.add_layer(layer(1, LayerKind::Raster), None));
        ok(d.add_layer(layer(2, LayerKind::Folder), None));
        ok(d.add_layer(layer(3, LayerKind::Raster), None));
        // L1 inside F2 but with L3 in between? No: L3 is above F2. Make L1's parent F2 (ok),
        // then L3's parent F2 (above its folder: bad).
        d.layer_mut(LayerId(1))
            .map(|l| l.parent = Some(LayerId(2)))
            .ok();
        ok(d.check_structure());
        d.layer_mut(LayerId(3))
            .map(|l| l.parent = Some(LayerId(2)))
            .ok();
        assert!(d.check_structure().is_err());
        d.layer_mut(LayerId(3))
            .map(|l| l.parent = Some(LayerId(1)))
            .ok();
        assert!(d.check_structure().is_err());
        d.layer_mut(LayerId(3))
            .map(|l| {
                l.parent = None;
                l.blend = BlendMode::Through;
            })
            .ok();
        assert_eq!(
            d.check_structure(),
            Err(DocError::ThroughOnRaster(LayerId(3)))
        );
    }

    #[test]
    fn hash_of_flat_documents_is_unchanged_by_new_fields() {
        let mut a = doc();
        ok(a.add_layer(layer(1, LayerKind::Raster), None));
        let h = a.content_hash();
        ok(a.group(layer(2, LayerKind::Folder), LayerId(1)));
        assert_ne!(a.content_hash(), h);
        ok(a.ungroup(LayerId(2)));
        assert_eq!(a.content_hash(), h);
        a.layer_mut(LayerId(1)).map(|l| l.clip = true).ok();
        assert_ne!(a.content_hash(), h);
    }
}
