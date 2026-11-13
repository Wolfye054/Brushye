//! The CPU reference renderer (`render::flatten`) on whole documents rebuilt from the observed
//! CSP data: layer order, tile iteration, opacity and transparent backdrops end to end.

mod common;

use brushye_core::render::flatten;
use brushye_core::{BlendMode, Document, Layer, LayerId, LayerKind, Rgba8};
use common::Img;

fn layer_from(img: &Img, id: u32, rows: std::ops::Range<usize>) -> Layer {
    let mut l = Layer::new(LayerId(id), format!("L{id}"), LayerKind::Raster);
    let block: Vec<u8> = rows
        .clone()
        .flat_map(|y| (0..img.w).flat_map(move |x| img.at(x, y)))
        .collect();
    l.put_pixels(
        (img.w as u32, rows.len() as u32),
        0,
        0,
        img.w as u32,
        rows.len() as u32,
        &block,
    );
    l
}

/// Base + top as in the observation setup: a PNG opened in CSP has no paper layer.
fn doc(base: &Layer, top: Layer, w: u32, h: u32) -> Document {
    let mut d = Document::new(w, h, 72, None).unwrap_or_else(|e| panic!("{e}"));
    d.add_layer(base.clone(), None)
        .unwrap_or_else(|e| panic!("{e}"));
    d.add_layer(top, None).unwrap_or_else(|e| panic!("{e}"));
    d
}

fn count_exact(
    img: &brushye_core::render::Image,
    obs: &Img,
    map_row: impl Fn(usize) -> usize,
    channels: usize,
) -> (u64, u64, u8) {
    let (mut n, mut exact, mut max) = (0, 0, 0u8);
    for y in 0..obs.h {
        for x in 0..obs.w {
            let got = img.pixel(x as u32, map_row(y) as u32);
            let want = obs.at(x, y);
            for c in 0..channels {
                n += 1;
                let d = got[c].abs_diff(want[c]);
                exact += u64::from(d == 0);
                max = max.max(d);
            }
        }
    }
    (n, exact, max)
}

#[test]
fn source_alpha_documents_render_exactly() {
    let base = layer_from(&Img::load("inputs/base.png"), 1, 0..512);
    let top_img = Img::load("inputs/top_alpha.png");
    for (m, nn) in [
        (BlendMode::Normal, 0),
        (BlendMode::ColorBurn, 3),
        (BlendMode::Screen, 7),
        (BlendMode::AddGlow, 11),
        (BlendMode::Overlay, 12),
        (BlendMode::Divide, 23),
    ] {
        let mut top = layer_from(&top_img, 2, 0..512);
        top.blend = m;
        let img = flatten(&doc(&base, top, 256, 512));
        let obs = Img::load(&format!("observed/alpha_{nn:02}_{}.png", m.name()));
        let (n, exact, max) = count_exact(&img, &obs, |y| y, 3);
        assert_eq!((exact, max), (n, 0), "{}", m.name());
        assert!(img.px.chunks_exact(4).all(|p| p[3] == 255));
    }
}

#[test]
fn layer_opacity_documents_render_exactly() {
    let base_img = Img::load("inputs2/combo_base.png");
    let top_img = Img::load("inputs2/combo_top.png");
    let base = layer_from(&base_img, 1, 0..1280);
    for (m, op) in [
        (BlendMode::Normal, 37u8),
        (BlendMode::AddGlow, 1),
        (BlendMode::Screen, 99),
    ] {
        let mut top = layer_from(&top_img, 2, 0..1280);
        top.blend = m;
        top.opacity = op;
        let img = flatten(&doc(&base, top, 256, 1280));
        let obs = Img::load(&format!("observed2/combo_{}_op{op}.png", m.name()));
        let (n, exact, max) =
            count_exact(&img, &obs, |y| if y < 192 { y } else { y - 192 + 512 }, 3);
        assert_eq!((exact, max), (n, 0), "{} {op}%", m.name());
    }
}

#[test]
fn transparent_backdrop_alpha_is_exact_and_normal_colour_within_one() {
    let base = layer_from(&Img::load("inputs2/bd_base.png"), 1, 0..512);
    let top_img = Img::load("inputs2/bd_top.png");
    for (m, nn) in [
        (BlendMode::Normal, 0),
        (BlendMode::Darken, 1),
        (BlendMode::Screen, 7),
        (BlendMode::Hue, 24),
    ] {
        let mut top = layer_from(&top_img, 2, 0..512);
        top.blend = m;
        let img = flatten(&doc(&base, top, 256, 512));
        let obs = Img::load(&format!("observed2/bd_{nn:02}_{}.png", m.name()));
        // Alpha: exact for every mode (L-005).
        let mut alpha_exact = true;
        for y in 0..obs.h {
            for x in 0..obs.w {
                alpha_exact &= img.pixel(x as u32, y as u32)[3] == obs.at(x, y)[3];
            }
        }
        assert!(alpha_exact, "{} alpha", m.name());
        if m == BlendMode::Normal {
            let (n, exact, max) = count_exact(&img, &obs, |y| y, 3);
            assert!(
                max <= 1 && exact as f64 / n as f64 >= 0.996,
                "normal colour {exact}/{n} max {max}"
            );
        }
    }
}

#[test]
fn paper_hidden_layers_and_canvas_clipping() {
    let mut d = Document::new(300, 20, 72, Some(Rgba8::WHITE)).unwrap_or_else(|e| panic!("{e}"));
    let mut l = Layer::new(LayerId(1), "a", LayerKind::Raster);
    // A block that runs past the right edge and into a second tile.
    l.put_pixels((300, 20), 250, 5, 100, 2, &[10, 20, 30, 255].repeat(200));
    d.add_layer(l, None).unwrap_or_else(|e| panic!("{e}"));
    let img = flatten(&d);
    assert_eq!(img.pixel(0, 0), [255, 255, 255, 255]);
    assert_eq!(img.pixel(260, 5), [10, 20, 30, 255]);
    assert_eq!(img.pixel(299, 6), [10, 20, 30, 255]);
    assert_eq!(img.pixel(260, 7), [255, 255, 255, 255]);
    assert_eq!(d.layers()[0].tiles.len(), 2);
    d.layer_mut(LayerId(1))
        .unwrap_or_else(|e| panic!("{e}"))
        .visible = false;
    assert!(
        flatten(&d)
            .px
            .chunks_exact(4)
            .all(|p| p == [255, 255, 255, 255])
    );
    // No paper and no layers: fully transparent.
    let empty = Document::new(4, 4, 72, None).unwrap_or_else(|e| panic!("{e}"));
    assert!(flatten(&empty).px.iter().all(|&v| v == 0));
}
