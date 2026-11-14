//! Folders, Through and clipping against observed CSP 1.13.2 exports (L-004, L-006; D-023).
//! Each case rebuilds the observed CSP document with the core API, flattens it with the CPU
//! reference renderer and compares every pixel. Known cases are bit-exact; the one open case
//! (Multiply with partial alpha, L-002) carries the same stated tolerance as L-002.

mod common;

use brushye_core::render::flatten;
use brushye_core::{BlendMode, Document, Layer, LayerId, LayerKind};
use common::Img;

fn ok<T, E: std::fmt::Display>(r: Result<T, E>) -> T {
    r.unwrap_or_else(|e| panic!("{e}"))
}

/// A raster layer holding an input image.
fn raster(id: u32, img: &Img) -> Layer {
    let mut l = Layer::new(LayerId(id), format!("L{id}"), LayerKind::Raster);
    let rgba: Vec<u8> = img.px.iter().flatten().copied().collect();
    l.put_pixels(
        (img.w as u32, img.h as u32),
        0,
        0,
        img.w as u32,
        img.h as u32,
        &rgba,
    );
    l
}

fn folder(id: u32) -> Layer {
    Layer::new(LayerId(id), format!("F{id}"), LayerKind::Folder)
}

struct Inputs {
    base: Img,
    top: Img,
    top_alpha: Img,
}

fn inputs() -> Inputs {
    Inputs {
        base: Img::load("inputs/base.png"),
        top: Img::load("inputs/top.png"),
        top_alpha: Img::load("inputs/top_alpha.png"),
    }
}

/// A paperless 256×512 document with `base` at the bottom (as CSP opens base.png).
fn doc_with_base(i: &Inputs) -> Document {
    let mut d = ok(Document::new(256, 512, 72, None));
    ok(d.add_layer(raster(1, &i.base), None));
    d
}

fn set(d: &mut Document, id: u32, f: impl FnOnce(&mut Layer)) {
    f(ok(d.layer_mut(LayerId(id))));
}

/// (pixels differing, max channel error).
fn diff(d: &Document, want: &str) -> (usize, u8) {
    let got = flatten(d);
    let want = Img::load(want);
    assert_eq!((got.width as usize, got.height as usize), (want.w, want.h));
    let mut n = 0;
    let mut max = 0;
    for (g, w) in got.px.as_chunks::<4>().0.iter().zip(&want.px) {
        let m = g
            .iter()
            .zip(w)
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap_or(0);
        if m > 0 {
            n += 1;
            max = max.max(m);
        }
    }
    (n, max)
}

fn exact(d: &Document, want: &str) {
    assert_eq!(diff(d, want), (0, 0), "{want}");
}

/// L-004 run 1: base, Folder [top (Multiply)], folder Normal/Through at 100 % and 50 %.
#[test]
fn l004_folder_normal_and_through_with_opacity() {
    let i = inputs();
    for (blend, name) in [
        (BlendMode::Normal, "normal"),
        (BlendMode::Through, "through"),
    ] {
        for op in [100, 50] {
            let mut d = doc_with_base(&i);
            ok(d.add_layer(raster(2, &i.top), None));
            set(&mut d, 2, |l| l.blend = BlendMode::Multiply);
            ok(d.group(folder(3), LayerId(2)));
            set(&mut d, 3, |l| {
                l.blend = blend;
                l.opacity = op;
            });
            exact(&d, &format!("observed2/folder_{name}_op{op}.png"));
        }
    }
}

/// L-004 run 2: Folder [top (Multiply), top_alpha (Screen)].
fn two_child_folder(i: &Inputs, blend: BlendMode, child_op: u8) -> Document {
    let mut d = doc_with_base(i);
    ok(d.add_layer(raster(2, &i.top), None));
    set(&mut d, 2, |l| l.blend = BlendMode::Multiply);
    ok(d.group(folder(4), LayerId(2)));
    ok(d.add_layer(raster(3, &i.top_alpha), Some(LayerId(2))));
    set(&mut d, 3, |l| {
        l.blend = BlendMode::Screen;
        l.opacity = child_op;
    });
    set(&mut d, 4, |l| l.blend = blend);
    ok(d.check_structure());
    d
}

#[test]
fn l004_several_children_and_child_opacity() {
    let i = inputs();
    exact(
        &two_child_folder(&i, BlendMode::Normal, 100),
        "observed3/folder_normal.png",
    );
    exact(
        &two_child_folder(&i, BlendMode::Through, 100),
        "observed3/folder_through.png",
    );
    exact(
        &two_child_folder(&i, BlendMode::Through, 50),
        "observed3/folder_through_child_op50.png",
    );
}

/// L-006: base, mid = top_alpha (clip base), top clipped.
fn clip_doc(i: &Inputs, f: impl FnOnce(&mut Document)) -> Document {
    let mut d = doc_with_base(i);
    ok(d.add_layer(raster(2, &i.top_alpha), None));
    ok(d.add_layer(raster(3, &i.top), None));
    set(&mut d, 3, |l| l.clip = true);
    f(&mut d);
    d
}

#[test]
fn l006_clipping() {
    let i = inputs();
    exact(&clip_doc(&i, |_| {}), "observed2/clip_00_normal.png");
    exact(
        &clip_doc(&i, |d| set(d, 3, |l| l.blend = BlendMode::Multiply)),
        "observed2/clip_02_multiply.png",
    );
    exact(&clip_doc(&i, |_| {}), "observed3/clip_normal.png");
    exact(
        &clip_doc(&i, |d| set(d, 3, |l| l.opacity = 50)),
        "observed3/clip_clipped_op50.png",
    );
    // A Normal folder as clip base is the folder's composited result.
    exact(
        &clip_doc(&i, |d| ok(d.group(folder(4), LayerId(2)))),
        "observed3/clip_onto_folder_normal.png",
    );
    // A Through folder as clip base: clipping is off (grey clip bar in CSP).
    exact(
        &clip_doc(&i, |d| {
            ok(d.group(folder(4), LayerId(2)));
            set(d, 4, |l| l.blend = BlendMode::Through);
        }),
        "observed3/clip_onto_folder_through.png",
    );
}

/// Clip base in Multiply: CSP's output equals its own plain Multiply layer with the base's
/// alpha (L-006 run 2), so the remaining error is L-002's open Multiply-with-partial-alpha
/// rule: the same tolerance as there (max 2), and only on partially transparent pixels.
#[test]
fn l006_clip_base_multiply_open_l002() {
    let i = inputs();
    let d = clip_doc(&i, |d| set(d, 2, |l| l.blend = BlendMode::Multiply));
    let (n, max) = diff(&d, "observed3/clip_base_multiply.png");
    assert!(max <= 2, "max {max}");
    let partial = i
        .top_alpha
        .px
        .iter()
        .filter(|p| p[3] > 0 && p[3] < 255)
        .count();
    assert!(
        n <= partial,
        "{n} differing > {partial} partial-alpha pixels"
    );
}
