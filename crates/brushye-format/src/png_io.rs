//! PNG export and import of straight RGBA8 images.
//!
//! Export writes 8-bit RGBA with no colour-space chunks (no iCCP/gAMA/sRGB), so the stored
//! values are exactly the rendered values. The output bytes are deterministic for a given
//! image (fixed compression settings).

use std::io::Write;

use brushye_core::render::Image;

#[derive(Debug)]
pub struct PngError(pub String);

impl std::fmt::Display for PngError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PNG: {}", self.0)
    }
}

impl std::error::Error for PngError {}

fn err(e: impl std::fmt::Display) -> PngError {
    PngError(e.to_string())
}

/// Encode an image as 8-bit RGBA PNG.
pub fn write_png(img: &Image, out: impl Write) -> Result<(), PngError> {
    let mut enc = png::Encoder::new(out, img.width, img.height);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.set_compression(png::Compression::Balanced);
    enc.set_filter(png::Filter::Adaptive);
    let mut w = enc.write_header().map_err(err)?;
    w.write_image_data(&img.px).map_err(err)?;
    w.finish().map_err(err)
}

/// Encode to a byte vector.
pub fn png_bytes(img: &Image) -> Result<Vec<u8>, PngError> {
    let mut v = Vec::new();
    write_png(img, &mut v)?;
    Ok(v)
}

/// Decode an 8-bit RGB/RGBA/Gray/GrayAlpha PNG to straight RGBA8. Colour-space chunks are
/// ignored (values are taken as stored, like CSP's PNG open; spec D-002).
pub fn read_png(data: &[u8]) -> Result<Image, PngError> {
    let mut dec = png::Decoder::new(std::io::Cursor::new(data));
    dec.set_transformations(png::Transformations::EXPAND);
    let mut r = dec.read_info().map_err(err)?;
    let mut buf = vec![
        0u8;
        r.output_buffer_size()
            .ok_or_else(|| PngError("image too large".into()))?
    ];
    let info = r.next_frame(&mut buf).map_err(err)?;
    if info.bit_depth != png::BitDepth::Eight {
        return Err(PngError(format!(
            "unsupported bit depth {:?} (8-bit only for now)",
            info.bit_depth
        )));
    }
    let src = &buf[..info.buffer_size()];
    let px: Vec<u8> = match info.color_type {
        png::ColorType::Rgba => src.to_vec(),
        png::ColorType::Rgb => src
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        png::ColorType::Grayscale => src.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        png::ColorType::GrayscaleAlpha => src
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        png::ColorType::Indexed => return Err(PngError("indexed PNG was not expanded".into())),
    };
    Ok(Image {
        width: info.width,
        height: info.height,
        px,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_is_exact_and_deterministic() {
        let px: Vec<u8> = (0..64u32 * 32 * 4).map(|i| (i * 37 % 256) as u8).collect();
        let img = Image {
            width: 64,
            height: 32,
            px,
        };
        let a = png_bytes(&img).unwrap_or_else(|e| unreachable!("{e}"));
        let b = png_bytes(&img).unwrap_or_else(|e| unreachable!("{e}"));
        assert_eq!(a, b);
        assert_eq!(read_png(&a).unwrap_or_else(|e| unreachable!("{e}")), img);
        // No colour-space chunks.
        for chunk in [&b"iCCP"[..], b"gAMA", b"sRGB", b"cHRM"] {
            assert!(!a.windows(4).any(|w| w == chunk));
        }
    }
}
