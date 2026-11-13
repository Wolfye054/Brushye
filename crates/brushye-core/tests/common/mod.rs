//! Shared test helpers: loading spec data PNGs.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

pub fn data(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../specs/behavior/blending/data")
        .join(rel)
}

/// RGBA8 image (RGB inputs get alpha 255).
pub struct Img {
    pub w: usize,
    pub h: usize,
    pub px: Vec<[u8; 4]>,
}

impl Img {
    pub fn load(rel: &str) -> Img {
        Img::load_path(&data(rel), rel)
    }
    /// Load from `specs/behavior/<area>/data/<rel>`.
    pub fn load_spec(area: &str, rel: &str) -> Img {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../specs/behavior")
            .join(area)
            .join("data")
            .join(rel);
        Img::load_path(&path, rel)
    }
    fn load_path(path: &Path, rel: &str) -> Img {
        let file = std::fs::File::open(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let mut reader = png::Decoder::new(std::io::BufReader::new(file))
            .read_info()
            .unwrap_or_else(|e| panic!("{rel}: {e}"));
        let mut buf = vec![0u8; reader.output_buffer_size().unwrap_or(0)];
        let info = reader
            .next_frame(&mut buf)
            .unwrap_or_else(|e| panic!("{rel}: {e}"));
        assert_eq!(info.bit_depth, png::BitDepth::Eight, "{rel}");
        let ch = match info.color_type {
            png::ColorType::Rgb => 3,
            png::ColorType::Rgba => 4,
            other => panic!("{rel}: unexpected colour type {other:?}"),
        };
        let px = buf[..info.buffer_size()]
            .chunks_exact(ch)
            .map(|p| [p[0], p[1], p[2], if ch == 4 { p[3] } else { 255 }])
            .collect();
        Img {
            w: info.width as usize,
            h: info.height as usize,
            px,
        }
    }
    pub fn at(&self, x: usize, y: usize) -> [u8; 4] {
        self.px[y * self.w + x]
    }
}
