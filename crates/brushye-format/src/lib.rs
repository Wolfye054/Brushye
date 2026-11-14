//! Brushye file formats: PNG export/import, the native `.brsh` document format and
//! `.brshbrush` user brushes (DECISIONS D-014).
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod brsh;
pub mod brush_file;
pub mod png_io;
