//! The brush engine (milestone M4), matching CSP 1.13.2 as observed (specs B-001 … B-010).
//!
//! A stroke is rendered in four steps:
//! 1. **Path** (`path`, B-002/B-007): input events, optionally stabilized, are the control
//!    points of a quadratic B-spline.
//! 2. **Dab placement** (B-002): dab k sits at arc length t_k along the path from the press
//!    point, t_k = k·s with s = max(1 px, gap% · size); with pressure-driven size the gap after
//!    a dab follows that dab's size. The stroke ends at the last dab at or before the release
//!    point.
//! 3. **Stroke buffer** (B-003): per pixel, dabs of one stroke build up as
//!    B = 1 − ∏(1 − cₖ·Dₖ), with tip coverage c (`tip`, B-001/B-009/B-010) and density D;
//!    the stroke's alpha is a = O·B (opacity scales the buffer, it is not a cap).
//! 4. **Commit** when the stroke ends (`ink`, B-003/B-008): the ink mode combines the
//!    unquantised a with the 8-bit layer.
//!
//! Determinism (D-012): only `+ - * / sqrt` on f64 in a fixed order, plus `libm` sin/cos for
//! the tip angle (pure Rust, identical on every platform).
//!
//! Open (see the specs): the exact soft-tip falloff (table model), AA None ties, stabilization
//! above 20, colour mixing (B-011), tilt input (B-010), per-dab varying opacity (each pixel
//! keeps the highest opacity of the dabs covering it).

pub mod curve;
mod hardness_tables;
pub mod ink;
pub mod path;
pub mod tip;

use std::collections::BTreeMap;

use crate::document::Layer;
use crate::input::Sample;

pub use curve::{Curve, CurveError};
pub use ink::InkMode;
pub use tip::{Aa, HARDNESS_MAX, Shape, TipAxis};

/// Dab gap preset (B-002): spacing = max(1 px, k · size).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Gap {
    /// k = 4 %.
    Narrow,
    /// k = 8 %.
    #[default]
    Normal,
    /// k = 12 %.
    Wide,
    /// k = the value in percent of size (not pixels, B-002).
    Fixed(f64),
}

impl Gap {
    /// Gap in percent of the dab size.
    pub fn percent(self) -> f64 {
        match self {
            Gap::Narrow => 4.0,
            Gap::Normal => 8.0,
            Gap::Wide => 12.0,
            Gap::Fixed(v) => v,
        }
    }
}

/// Fixed gap range accepted, in percent of size (CSP's range is not known; 10–300 observed).
pub const FIXED_GAP_RANGE: std::ops::RangeInclusive<f64> = 0.1..=1000.0;
/// Largest brush size in px.
pub const MAX_SIZE: f64 = 5000.0;
/// Largest stabilization setting (CSP's range).
pub const MAX_STABILIZATION: u8 = 100;

/// Pen pressure dynamics for one setting: value × (min + (1 − min)·curve(P)) (B-004/B-005).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Dynamics {
    /// Minimum value, 0..=1.
    pub min: f64,
    pub curve: Curve,
}

impl Dynamics {
    fn factor(&self, p: f64) -> f64 {
        self.min + (1.0 - self.min) * self.curve.eval(p)
    }
}

/// Everything that shapes one stroke.
#[derive(Clone, Debug, PartialEq)]
pub struct BrushSettings {
    pub color: [u8; 3],
    /// Diameter in px.
    pub size: f64,
    pub aa: Aa,
    /// 1 (softest) ..= 5 (hard).
    pub hardness: u8,
    /// Brush density, 0..=1.
    pub density: f64,
    /// Ink opacity, 0..=1.
    pub opacity: f64,
    pub ink: InkMode,
    pub gap: Gap,
    /// "Adjust brush density by gap" (B-002 run 2).
    pub density_by_gap: bool,
    /// Tip thickness, (0, 1]; 1 = round.
    pub thickness: f64,
    pub tip_axis: TipAxis,
    /// Tip angle in degrees, clockwise on screen (B-010).
    pub angle: f64,
    /// Angle follows the direction of the line (B-010).
    pub angle_follows_line: bool,
    /// Stabilization 0..=100 (B-007).
    pub stabilization: u8,
    pub size_pressure: Option<Dynamics>,
    pub opacity_pressure: Option<Dynamics>,
    pub density_pressure: Option<Dynamics>,
}

impl Default for BrushSettings {
    fn default() -> Self {
        Self {
            color: [0, 0, 0],
            size: 10.0,
            aa: Aa::Middle,
            hardness: HARDNESS_MAX,
            density: 1.0,
            opacity: 1.0,
            ink: InkMode::Normal,
            gap: Gap::Normal,
            density_by_gap: false,
            thickness: 1.0,
            tip_axis: TipAxis::Vertical,
            angle: 0.0,
            angle_follows_line: false,
            stabilization: 0,
            size_pressure: None,
            opacity_pressure: None,
            density_pressure: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrushError(pub String);

impl std::fmt::Display for BrushError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "brush: {}", self.0)
    }
}

impl std::error::Error for BrushError {}

impl BrushSettings {
    pub fn validate(&self) -> Result<(), BrushError> {
        let bad = |m: String| Err(BrushError(m));
        if !(self.size > 0.0 && self.size <= MAX_SIZE) {
            return bad(format!("size must be in (0, {MAX_SIZE}]"));
        }
        if !(1..=HARDNESS_MAX).contains(&self.hardness) {
            return bad(format!("hardness must be 1..={HARDNESS_MAX}"));
        }
        if !(0.0..=1.0).contains(&self.density) || !(0.0..=1.0).contains(&self.opacity) {
            return bad("density and opacity must be 0..=100 %".into());
        }
        if !(self.thickness > 0.0 && self.thickness <= 1.0) {
            return bad("thickness must be 1..=100 %".into());
        }
        if !self.angle.is_finite() {
            return bad("angle must be finite".into());
        }
        if self.stabilization > MAX_STABILIZATION {
            return bad(format!("stabilization must be 0..={MAX_STABILIZATION}"));
        }
        if let Gap::Fixed(v) = self.gap
            && !FIXED_GAP_RANGE.contains(&v)
        {
            return bad(format!(
                "fixed gap must be {}..={} % of size",
                FIXED_GAP_RANGE.start(),
                FIXED_GAP_RANGE.end()
            ));
        }
        for d in [
            &self.size_pressure,
            &self.opacity_pressure,
            &self.density_pressure,
        ]
        .into_iter()
        .flatten()
        {
            if !(0.0..=1.0).contains(&d.min) {
                return bad("pressure minimum must be 0..=100 %".into());
            }
        }
        Ok(())
    }

    /// Unit vector of the tip's long axis for a dab travelling along `dir` (B-010).
    fn axis(&self, dir: [f64; 2]) -> [f64; 2] {
        let rad = self.angle.rem_euclid(360.0) * (std::f64::consts::PI / 180.0);
        let (s, c) = (libm::sin(rad), libm::cos(rad));
        // Angle 0 = long axis horizontal; "Horizontal" shortens the other axis (+90°).
        let base = if self.angle_follows_line {
            dir
        } else {
            [1.0, 0.0]
        };
        let base = match self.tip_axis {
            TipAxis::Vertical => base,
            TipAxis::Horizontal => [-base[1], base[0]],
        };
        // Rotate clockwise on screen (y down) by the angle.
        [base[0] * c - base[1] * s, base[0] * s + base[1] * c]
    }
}

/// One placed dab.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dab {
    /// Input point (before the tip's centre offset).
    pub x: f64,
    pub y: f64,
    /// Diameter in px.
    pub size: f64,
    pub opacity: f64,
    pub density: f64,
    /// Unit vector of the tip's long axis.
    pub axis: [f64; 2],
}

fn path_points(samples: &[Sample]) -> Vec<path::Pt> {
    // Mouse input counts as full pressure (B-004).
    let raw: Vec<path::Pt> = samples
        .iter()
        .map(|q| path::Pt {
            x: q.x,
            y: q.y,
            p: q.pressure.map_or(1.0, f64::from),
        })
        .collect();
    path::dedup(raw)
}

/// The path a stroke's dabs follow: stabilized, smoothed input (B-002, B-007).
pub fn stroke_path(s: &BrushSettings, samples: &[Sample]) -> path::Polyline {
    let pts = path::stabilize(&path_points(samples), s.stabilization);
    path::Polyline::new(path::spline(&pts))
}

/// Dabs of a stroke, in order (B-002 placement, B-004/B-005 dynamics).
pub fn place_dabs(s: &BrushSettings, samples: &[Sample]) -> Vec<Dab> {
    lattice(s, &stroke_path(s, samples), None).0
}

/// Dabs of a stroke whose pen is still down, and how many of them are final: those cannot
/// change as more samples arrive (they lie on the final prefix of the path, see
/// [`path::final_length`]). The final ones equal the first dabs of the finished stroke when
/// stabilization is off; with stabilization the finished stroke adds the catch-up at release.
pub fn place_dabs_live(s: &BrushSettings, samples: &[Sample]) -> (Vec<Dab>, usize) {
    let pts = path::stabilize_live(&path_points(samples), s.stabilization);
    let stable = path::final_length(&pts);
    lattice(s, &path::Polyline::new(path::spline(&pts)), Some(stable))
}

/// Dabs along a path. With `stable = Some(len)`, also counts the dabs at arc length ≤ len
/// (`Some(None)`: none are final yet).
fn lattice(
    s: &BrushSettings,
    line: &path::Polyline,
    stable: Option<Option<f64>>,
) -> (Vec<Dab>, usize) {
    if line.is_empty() {
        return (Vec::new(), 0);
    }
    let total = line.len();
    let gap = s.gap.percent();
    // Constant size: t_k = k·gap·size/100 computed directly (no accumulated rounding; B-002
    // shows ties at exact pixel boundaries resolve as exact arithmetic).
    let constant = s.size_pressure.is_none() && gap * s.size / 100.0 >= 1.0;
    let f = |d: &Option<Dynamics>, p: f64| d.as_ref().map_or(1.0, |d| d.factor(p));

    let mut dabs = Vec::new();
    let mut n_stable = 0usize;
    let mut seg = 0usize;
    let mut t = 0.0f64;
    let mut k: u64 = 0;
    const EPS: f64 = 1e-9;
    while t <= total + EPS {
        let Some(at) = line.at(t, &mut seg) else {
            break;
        };
        let p = at.pt.p;
        let size = s.size * f(&s.size_pressure, p);
        let spacing = (gap * size / 100.0).max(1.0);
        let mut density = s.density * f(&s.density_pressure, p);
        if s.density_by_gap && size > 0.0 {
            // d' = D·s / (size/3): density normalised to a gap of a third of the size
            // (B-002 run 2; gaps above 33 % unobserved, capped at full density).
            density = (density * 3.0 * spacing / size).min(1.0);
        }
        // Zero pressure places no ink (B-004), but the lattice continues.
        if p > 0.0 && size > 0.0 {
            dabs.push(Dab {
                x: at.pt.x,
                y: at.pt.y,
                size,
                opacity: s.opacity * f(&s.opacity_pressure, p),
                density,
                axis: s.axis(at.dir),
            });
            if let Some(Some(len)) = stable
                && t <= len
            {
                n_stable = dabs.len();
            }
        }
        if total == 0.0 {
            break;
        }
        k += 1;
        t = if constant {
            (k as f64 * gap * s.size) / 100.0
        } else {
            t + spacing
        };
    }
    if stable.is_none() {
        n_stable = dabs.len();
    }
    (dabs, n_stable)
}

const CHUNK: usize = 64;
type ChunkKey = (i32, i32);

/// Per-pixel stroke state: transmittance ∏(1 − c·D) and the highest dab opacity.
#[derive(Clone)]
struct Chunk {
    t: [f64; CHUNK * CHUNK],
    o: [f64; CHUNK * CHUNK],
}

impl Chunk {
    fn empty() -> Box<Self> {
        Box::new(Chunk {
            t: [1.0; CHUNK * CHUNK],
            o: [0.0; CHUNK * CHUNK],
        })
    }
}

fn shape_of(s: &BrushSettings, d: &Dab) -> Shape {
    Shape {
        r: d.size / 2.0,
        thickness: s.thickness,
        axis: d.axis,
        aa: s.aa,
        hardness: s.hardness,
    }
}

/// Pixel bounds [x0, x1) × [y0, y1) a dab can touch, clipped to the canvas.
fn dab_bounds(bounds: (u32, u32), s: &BrushSettings, d: &Dab) -> (i64, i64, i64, i64) {
    let shape = shape_of(s, d);
    let (ox, oy) = shape.centre_offset();
    let (cx, cy) = (d.x + ox, d.y + oy);
    let reach = shape.reach();
    (
        ((cx - reach - 1.0).floor().max(0.0)) as i64,
        ((cy - reach - 1.0).floor().max(0.0)) as i64,
        ((cx + reach + 1.0).ceil()).min(f64::from(bounds.0)) as i64,
        ((cy + reach + 1.0).ceil()).min(f64::from(bounds.1)) as i64,
    )
}

/// Chunks a dab's bounds overlap.
fn dab_chunks(bounds: (u32, u32), s: &BrushSettings, d: &Dab) -> Vec<ChunkKey> {
    let (x0, y0, x1, y1) = dab_bounds(bounds, s, d);
    let c = CHUNK as i64;
    let mut out = Vec::new();
    if x0 >= x1 || y0 >= y1 {
        return out;
    }
    for cy in y0 / c..=(y1 - 1) / c {
        for cx in x0 / c..=(x1 - 1) / c {
            out.push((cx as i32, cy as i32));
        }
    }
    out
}

/// Add one dab to a chunk map. Each pixel is updated at most once per dab, so the visiting
/// order inside a dab doesn't matter; dabs are applied in stroke order.
fn dab_into(
    chunks: &mut BTreeMap<ChunkKey, Box<Chunk>>,
    bounds: (u32, u32),
    s: &BrushSettings,
    d: &Dab,
) {
    let shape = shape_of(s, d);
    let (ox, oy) = shape.centre_offset();
    let (cx, cy) = (d.x + ox, d.y + oy);
    let (x0, y0, x1, y1) = dab_bounds(bounds, s, d);
    let c = CHUNK as i64;
    let mut hits: Vec<(usize, f64)> = Vec::new();
    for key in dab_chunks(bounds, s, d) {
        let (kx, ky) = (i64::from(key.0) * c, i64::from(key.1) * c);
        let (xa, xb) = (x0.max(kx), x1.min(kx + c));
        let (ya, yb) = (y0.max(ky), y1.min(ky + c));
        hits.clear();
        for py in ya..yb {
            let dy = py as f64 + 0.5 - cy;
            for px in xa..xb {
                let cov = shape.coverage(px as f64 + 0.5 - cx, dy);
                if cov > 0.0 {
                    hits.push(((py - ky) as usize * CHUNK + (px - kx) as usize, cov));
                }
            }
        }
        if hits.is_empty() {
            continue;
        }
        let chunk = chunks.entry(key).or_insert_with(Chunk::empty);
        for &(i, cov) in &hits {
            chunk.t[i] *= 1.0 - cov * d.density;
            if d.opacity > chunk.o[i] {
                chunk.o[i] = d.opacity;
            }
        }
    }
}

/// The stroke buffer (B-003): built up by the dabs of one stroke, committed at the end.
pub struct StrokeBuffer {
    bounds: (u32, u32),
    chunks: BTreeMap<ChunkKey, Box<Chunk>>,
}

impl StrokeBuffer {
    pub fn new(bounds: (u32, u32)) -> Self {
        Self {
            bounds,
            chunks: BTreeMap::new(),
        }
    }

    /// Add one dab.
    pub fn dab(&mut self, s: &BrushSettings, d: &Dab) {
        dab_into(&mut self.chunks, self.bounds, s, d);
    }

    /// The stroke's unquantised alpha a = O·B at each touched pixel, in a fixed order.
    pub fn alphas(&self) -> impl Iterator<Item = (u32, u32, f64)> + '_ {
        self.chunks.iter().flat_map(|(&(cx, cy), ch)| {
            (0..CHUNK * CHUNK).filter_map(move |i| {
                let a = ch.o[i] * (1.0 - ch.t[i]);
                (a > 0.0).then(|| {
                    (
                        cx as u32 * CHUNK as u32 + (i % CHUNK) as u32,
                        cy as u32 * CHUNK as u32 + (i / CHUNK) as u32,
                        a,
                    )
                })
            })
        })
    }

    /// Composite the stroke onto a layer (straight RGBA8) with an ink mode, honouring lock
    /// transparency.
    pub fn commit(&self, layer: &mut Layer, color: [u8; 3], mode: InkMode) {
        use crate::tile::TILE_SIZE;
        for (x, y, a) in self.alphas() {
            let (tx, ty) = (
                (x as usize / TILE_SIZE) as i32,
                (y as usize / TILE_SIZE) as i32,
            );
            let i = ((y as usize % TILE_SIZE) * TILE_SIZE + x as usize % TILE_SIZE) * 4;
            let dst = match layer.tiles.get((tx, ty)) {
                Some(t) => [t.px[i], t.px[i + 1], t.px[i + 2], t.px[i + 3]],
                None => [0; 4],
            };
            let out = ink::apply(mode, dst, color, a, layer.lock_alpha);
            if out != dst {
                layer.tiles.get_mut((tx, ty)).px[i..i + 4].copy_from_slice(&out);
            }
        }
        layer.tiles.prune();
    }
}

/// A stroke being drawn: an always-current preview of the layer, updated incrementally.
///
/// Final dabs ([`place_dabs_live`]) go into a persistent stroke buffer once; the few tail dabs
/// that may still move are drawn each update on a scratch copy of the chunks they touch. Only
/// changed chunks are committed again. The preview equals [`stroke`] on the same samples
/// (exactly, without stabilization; with stabilization the finished stroke also catches up to
/// the release point). The finished stroke is still made by the `brush.stroke` command.
pub struct LiveStroke {
    s: BrushSettings,
    bounds: (u32, u32),
    samples: Vec<Sample>,
    base: Layer,
    preview: Layer,
    buf: StrokeBuffer,
    applied: usize,
    tail_chunks: Vec<ChunkKey>,
}

impl LiveStroke {
    pub fn new(base: Layer, bounds: (u32, u32), s: BrushSettings) -> Result<Self, BrushError> {
        s.validate()?;
        Ok(Self {
            preview: base.clone(),
            base,
            bounds,
            s,
            samples: Vec::new(),
            buf: StrokeBuffer::new(bounds),
            applied: 0,
            tail_chunks: Vec::new(),
        })
    }

    pub fn settings(&self) -> &BrushSettings {
        &self.s
    }

    pub fn samples(&self) -> &[Sample] {
        &self.samples
    }

    /// The layer as it looks with the stroke so far.
    pub fn preview(&self) -> &Layer {
        &self.preview
    }

    /// Add samples; returns the 256² tiles whose preview pixels may have changed.
    pub fn push(&mut self, samples: &[Sample]) -> Vec<(i32, i32)> {
        self.samples.extend_from_slice(samples);
        let (dabs, stable) = place_dabs_live(&self.s, &self.samples);
        let mut dirty: Vec<ChunkKey> = std::mem::take(&mut self.tail_chunks);
        let stable = stable.max(self.applied).min(dabs.len());
        for d in &dabs[self.applied..stable] {
            dirty.extend(dab_chunks(self.bounds, &self.s, d));
            self.buf.dab(&self.s, d);
        }
        self.applied = stable;
        // Tail: on copies of the chunks it touches.
        let mut overlay: BTreeMap<ChunkKey, Box<Chunk>> = BTreeMap::new();
        for d in &dabs[stable..] {
            for key in dab_chunks(self.bounds, &self.s, d) {
                if let std::collections::btree_map::Entry::Vacant(v) = overlay.entry(key)
                    && let Some(c) = self.buf.chunks.get(&key)
                {
                    v.insert(c.clone());
                }
            }
            dab_into(&mut overlay, self.bounds, &self.s, d);
        }
        self.tail_chunks = overlay.keys().copied().collect();
        dirty.extend(self.tail_chunks.iter().copied());
        dirty.sort_unstable();
        dirty.dedup();
        for &key in &dirty {
            let chunk = overlay.get(&key).or_else(|| self.buf.chunks.get(&key));
            recommit(
                &mut self.preview,
                &self.base,
                &self.s,
                self.bounds,
                key,
                chunk.map(|c| &**c),
            );
        }
        self.preview.tiles.prune();
        let ratio = (crate::tile::TILE_SIZE / CHUNK) as i32;
        let mut tiles: Vec<(i32, i32)> = dirty.iter().map(|k| (k.0 / ratio, k.1 / ratio)).collect();
        tiles.dedup();
        tiles.sort_unstable();
        tiles.dedup();
        tiles
    }
}

/// Recompute one chunk of a live stroke's preview from the base layer and the stroke state.
fn recommit(
    preview: &mut Layer,
    base_layer: &Layer,
    s: &BrushSettings,
    bounds: (u32, u32),
    key: ChunkKey,
    chunk: Option<&Chunk>,
) {
    {
        use crate::tile::TILE_SIZE;
        let c = CHUNK as i64;
        let (kx, ky) = (i64::from(key.0) * c, i64::from(key.1) * c);
        let (w, h) = (i64::from(bounds.0), i64::from(bounds.1));
        for iy in 0..CHUNK {
            let y = ky + iy as i64;
            if y >= h {
                break;
            }
            for ix in 0..CHUNK {
                let x = kx + ix as i64;
                if x >= w {
                    break;
                }
                let base = base_layer.pixel(x as u32, y as u32);
                let a = chunk.map_or(0.0, |ch| {
                    let i = iy * CHUNK + ix;
                    ch.o[i] * (1.0 - ch.t[i])
                });
                let out = if a > 0.0 {
                    ink::apply(s.ink, base, s.color, a, base_layer.lock_alpha)
                } else {
                    base
                };
                if preview.pixel(x as u32, y as u32) != out {
                    let ts = TILE_SIZE as i64;
                    let tile = preview.tiles.get_mut(((x / ts) as i32, (y / ts) as i32));
                    let i = (((y % ts) * ts + x % ts) * 4) as usize;
                    tile.px[i..i + 4].copy_from_slice(&out);
                }
            }
        }
    }
}

/// Render one stroke onto a layer: path, dabs, stroke buffer, commit.
pub fn stroke(
    layer: &mut Layer,
    bounds: (u32, u32),
    s: &BrushSettings,
    samples: &[Sample],
) -> Result<(), BrushError> {
    s.validate()?;
    if samples.is_empty() {
        return Err(BrushError("a stroke needs at least one sample".into()));
    }
    let mut buf = StrokeBuffer::new(bounds);
    for d in place_dabs(s, samples) {
        buf.dab(s, &d);
    }
    buf.commit(layer, s.color, s.ink);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn smp(x: f64, y: f64) -> Sample {
        Sample {
            x,
            y,
            t_us: 0,
            pressure: Some(1.0),
            tilt: None,
            twist: None,
            eraser: false,
            barrel: [false; 2],
        }
    }

    #[test]
    fn spacing_follows_gap_presets_and_ends_before_release() {
        let mut s = BrushSettings {
            size: 40.0,
            gap: Gap::Wide,
            ..Default::default()
        };
        let d = place_dabs(&s, &[smp(40.5, 10.5), smp(100.5, 10.5), smp(472.5, 10.5)]);
        // s = 4.8: dabs at 40.5 + 4.8k up to 472.5 (k = 90 lands exactly on the release point).
        assert_eq!(d.len(), 91);
        assert_eq!(d[0].x, 40.5);
        assert_eq!(d[90].x, 472.5);
        assert_eq!(d[5].x, 64.5);
        s.gap = Gap::Normal;
        let d = place_dabs(&s, &[smp(0.0, 0.0), smp(10.0, 0.0)]);
        // s = 3.2: 0, 3.2, 6.4, 9.6 (no dab at the release point).
        assert_eq!(
            d.iter().map(|d| d.x).collect::<Vec<_>>(),
            [0.0, 3.2, 6.4, 9.6]
        );
        // Small sizes: at least 1 px.
        s.size = 3.0;
        assert_eq!(place_dabs(&s, &[smp(0.0, 0.0), smp(3.0, 0.0)]).len(), 4);
    }

    #[test]
    fn a_click_places_one_dab_and_zero_pressure_none() {
        let s = BrushSettings::default();
        assert_eq!(place_dabs(&s, &[smp(5.0, 5.0)]).len(), 1);
        let mut z = smp(5.0, 5.0);
        z.pressure = Some(0.0);
        assert!(place_dabs(&s, &[z]).is_empty());
    }

    #[test]
    fn one_dab_at_density_30_is_76() {
        let mut buf = StrokeBuffer::new((8, 8));
        let s = BrushSettings {
            aa: Aa::None,
            density: 0.3,
            size: 4.0,
            ..Default::default()
        };
        for d in place_dabs(&s, &[smp(4.0, 4.0)]) {
            buf.dab(&s, &d);
        }
        assert!(buf.alphas().all(|(_, _, a)| ink::trunc8(a) == 76));
    }

    #[test]
    fn axis_follows_angle_and_line() {
        let close =
            |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).abs() < 1e-12 && (a[1] - b[1]).abs() < 1e-12;
        let mut s = BrushSettings::default();
        assert!(close(s.axis([0.0, 1.0]), [1.0, 0.0]));
        s.angle = 90.0;
        // Clockwise on screen: the long axis points down.
        assert!(close(s.axis([1.0, 0.0]), [0.0, 1.0]));
        s.angle = 0.0;
        s.tip_axis = TipAxis::Horizontal;
        assert!(close(s.axis([1.0, 0.0]), [0.0, 1.0]));
        s.tip_axis = TipAxis::Vertical;
        s.angle_follows_line = true;
        assert!(close(s.axis([0.6, -0.8]), [0.6, -0.8]));
    }
}
