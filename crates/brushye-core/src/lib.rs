//! Brushye core: document model, colour/blend math (`blend::math`), CPU reference renderer
//! (`render`) and the brush engine (`brush`).
//!
//! No UI, GPU or OS dependencies (DECISIONS D-010). Documents are immutable values shared via
//! `Arc`: every edit produces a new version that shares unchanged layers and tiles with the
//! previous one (D-013). Only the command executor (`brushye-command`) holds the session's
//! current version, so clients can read any version but can change state only by executing
//! commands.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod blend;
pub mod brush;
pub mod document;
pub mod hash;
pub mod ids;
pub mod input;
pub mod render;
pub mod rng;
pub mod tile;

pub use blend::BlendMode;
pub use document::{Document, Layer, LayerKind, Rgba8};
pub use ids::LayerId;
pub use tile::{TILE_SIZE, Tile, TileGrid};
