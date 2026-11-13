//! Sparse copy-on-write tile storage: 256×256 tiles of 8-bit straight RGBA (D-012).
//! Missing tiles are fully transparent. Tiles are `Arc`-shared between document versions;
//! writing to a tile clones it only if another version still uses it (D-013).

use std::collections::BTreeMap;
use std::sync::Arc;

pub const TILE_SIZE: usize = 256;
const TILE_BYTES: usize = TILE_SIZE * TILE_SIZE * 4;

/// One tile of straight (non-premultiplied) RGBA8, row-major.
#[derive(Clone, PartialEq, Eq)]
pub struct Tile {
    pub px: Box<[u8]>,
}

impl Tile {
    pub fn transparent() -> Self {
        Self {
            px: vec![0u8; TILE_BYTES].into_boxed_slice(),
        }
    }
    pub fn is_transparent(&self) -> bool {
        self.px.chunks_exact(4).all(|p| p[3] == 0)
    }
}

impl std::fmt::Debug for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tile({} bytes)", self.px.len())
    }
}

/// Tile coordinates (tile column, tile row).
pub type TileCoord = (i32, i32);

/// Sparse grid; iteration order is the sorted coordinate order (deterministic, D-012).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TileGrid {
    tiles: BTreeMap<TileCoord, Arc<Tile>>,
}

impl TileGrid {
    pub fn get(&self, c: TileCoord) -> Option<&Arc<Tile>> {
        self.tiles.get(&c)
    }
    /// Mutable access, cloning the tile first if another version shares it.
    pub fn get_mut(&mut self, c: TileCoord) -> &mut Tile {
        Arc::make_mut(
            self.tiles
                .entry(c)
                .or_insert_with(|| Arc::new(Tile::transparent())),
        )
    }
    pub fn iter(&self) -> impl Iterator<Item = (&TileCoord, &Arc<Tile>)> {
        self.tiles.iter()
    }
    pub fn len(&self) -> usize {
        self.tiles.len()
    }
    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }
    /// Drop tiles that became fully transparent.
    pub fn prune(&mut self) {
        self.tiles.retain(|_, t| !t.is_transparent());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_on_write_shares_until_written() {
        let mut a = TileGrid::default();
        a.get_mut((0, 0)).px[3] = 255;
        let mut b = a.clone();
        assert!(Arc::ptr_eq(
            a.get((0, 0)).unwrap_or_else(|| unreachable!()),
            b.get((0, 0)).unwrap_or_else(|| unreachable!())
        ));
        b.get_mut((0, 0)).px[3] = 7;
        assert_eq!(a.get((0, 0)).map(|t| t.px[3]), Some(255));
        assert_eq!(b.get((0, 0)).map(|t| t.px[3]), Some(7));
    }

    #[test]
    fn prune_removes_transparent_tiles() {
        let mut g = TileGrid::default();
        g.get_mut((1, 2));
        g.get_mut((0, 0)).px[3] = 1;
        g.prune();
        assert_eq!(g.len(), 1);
    }
}
