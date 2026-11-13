//! The brush engine (milestone M4): round tips, spacing, in-stroke build-up and pressure
//! dynamics, matching CSP 1.13.2 as observed (specs B-001 … B-006, B-009).
//!
//! A stroke is rendered in three steps:
//! 1. **Dab placement** (B-002). Dab k sits at arc length t_k along the input polyline from the
//!    press point, t_k = k·s with s = max(1 px, gap% · size); with pressure-driven size the
//!    gap follows each dab's size. The stroke ends at the last dab at or before the release
//!    point.
//! 2. **Stroke buffer** (B-003). Per pixel, dabs of one stroke build up as
//!    B = 1 − ∏(1 − cₖ·Dₖ), with coverage c (tip, B-001/B-009) and density D; the stroke's
//!    alpha is a = O·B (opacity scales the buffer, it is not a cap).
//! 3. **Commit** when the stroke ends: a8 = ⌊65535·a⌋ >> 8 (CSP truncates, B-001/B-003), then
//!    over the layer in 8-bit: A' = A + a8 − ⌊A·a8 / 255⌋, which reproduces the observed
//!    stroke-over-stroke levels (111, 135, 191 in B-003).
//!
//! Determinism (D-012): only `+ - * / sqrt` on f64, in a fixed order, so results are
//! bit-identical on every platform.
//!
//! Open (see the specs): the AA None edge rule, the exact soft-tip falloff and its centre,
//! curves with several points, spacing with pressure-driven size, colour when a stroke lands
//! on differently coloured pixels, and per-dab varying opacity (the buffer keeps the highest
//! opacity seen at each pixel).

pub mod curve;
mod hardness_tables;
pub mod tip;

use std::collections::BTreeMap;

use crate::document::Layer;
use crate::input::Sample;

pub use curve::{Curve, CurveError};
pub use tip::{Aa, HARDNESS_MAX};

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

/// Fixed gap range accepted, in percent of size (CSP's range is not yet known; 10–90 observed).
pub const FIXED_GAP_RANGE: std::ops::RangeInclusive<f64> = 0.1..=1000.0;
/// Largest brush size in px.
pub const MAX_SIZE: f64 = 5000.0;

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
    pub gap: Gap,
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
            gap: Gap::Normal,
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
        if let Gap::Fixed(v) = self.gap {
            if !FIXED_GAP_RANGE.contains(&v) {
                return bad(format!(
                    "fixed gap must be {}..={} % of size",
                    FIXED_GAP_RANGE.start(),
                    FIXED_GAP_RANGE.end()
                ));
            }
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
}

/// One placed dab.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dab {
    pub x: f64,
    pub y: f64,
    /// Diameter in px.
    pub size: f64,
    pub opacity: f64,
    pub density: f64,
}

/// Point of the input polyline.
#[derive(Clone, Copy, Debug)]
struct Pt {
    x: f64,
    y: f64,
    p: f64,
}

/// Dabs of a stroke, in order (B-002 placement, B-004/B-005 dynamics).
pub fn place_dabs(s: &BrushSettings, samples: &[Sample]) -> Vec<Dab> {
    // Mouse input counts as full pressure (B-004).
    let mut pts: Vec<Pt> = Vec::with_capacity(samples.len());
    for smp in samples {
        let p = smp.pressure.map_or(1.0, f64::from);
        match pts.last_mut() {
            // Repeated positions add no length; the latest pressure wins.
            Some(l) if l.x == smp.x && l.y == smp.y => l.p = p,
            _ => pts.push(Pt {
                x: smp.x,
                y: smp.y,
                p,
            }),
        }
    }
    let Some(&first) = pts.first() else {
        return Vec::new();
    };
    // Cumulative arc length at each point.
    let mut cum = Vec::with_capacity(pts.len());
    let mut total = 0.0;
    cum.push(0.0);
    for w in pts.windows(2) {
        let (dx, dy) = (w[1].x - w[0].x, w[1].y - w[0].y);
        total += (dx * dx + dy * dy).sqrt();
        cum.push(total);
    }

    let dab_at = |pt: Pt| -> Dab {
        let f = |d: &Option<Dynamics>| d.as_ref().map_or(1.0, |d| d.factor(pt.p));
        Dab {
            x: pt.x,
            y: pt.y,
            size: s.size * f(&s.size_pressure),
            opacity: s.opacity * f(&s.opacity_pressure),
            density: s.density * f(&s.density_pressure),
        }
    };
    let gap = s.gap.percent();
    // Constant size: t_k = k·gap·size/100 computed directly (no accumulated rounding; B-002
    // shows ties at exact pixel boundaries resolve as exact arithmetic).
    let constant = s.size_pressure.is_none() && gap * s.size / 100.0 >= 1.0;

    let mut dabs = Vec::new();
    let mut seg = 0usize;
    let mut t = 0.0f64;
    let mut k: u64 = 0;
    const EPS: f64 = 1e-9;
    loop {
        if t > total + EPS {
            break;
        }
        while seg + 1 < cum.len() - 1 && cum[seg + 1] < t {
            seg += 1;
        }
        let pt = if pts.len() == 1 {
            first
        } else {
            let (a, b) = (pts[seg], pts[seg + 1]);
            let len = cum[seg + 1] - cum[seg];
            let u = if len > 0.0 {
                ((t - cum[seg]) / len).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let along = (t - cum[seg]).clamp(0.0, len);
            let (ux, uy) = ((b.x - a.x) / len, (b.y - a.y) / len);
            Pt {
                x: a.x + along * ux,
                y: a.y + along * uy,
                p: a.p + (b.p - a.p) * u,
            }
        };
        let dab = dab_at(pt);
        // Zero pressure places no ink (B-004), but the lattice continues.
        if pt.p > 0.0 && dab.size > 0.0 {
            dabs.push(dab);
        }
        k += 1;
        t = if constant {
            (k as f64 * gap * s.size) / 100.0
        } else {
            t + (gap * dab.size / 100.0).max(1.0)
        };
        if total == 0.0 {
            break;
        }
    }
    dabs
}

const CHUNK: usize = 64;

/// Per-pixel stroke state: transmittance ∏(1 − c·D) and the highest dab opacity.
struct Chunk {
    t: [f64; CHUNK * CHUNK],
    o: [f64; CHUNK * CHUNK],
}

/// The stroke buffer (B-003): built up by the dabs of one stroke, committed at the end.
pub struct StrokeBuffer {
    bounds: (u32, u32),
    chunks: BTreeMap<(i32, i32), Box<Chunk>>,
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
        let r = d.size / 2.0;
        let soft = s.hardness < HARDNESS_MAX;
        let off = if soft { tip::SOFT_CENTRE_OFFSET } else { 0.0 };
        let (cx, cy) = (d.x + off, d.y + off);
        let reach = tip::reach(r, s.hardness);
        let x0 = ((cx - reach - 1.0).floor().max(0.0)) as i64;
        let y0 = ((cy - reach - 1.0).floor().max(0.0)) as i64;
        let x1 = ((cx + reach + 1.0).ceil()).min(f64::from(self.bounds.0)) as i64;
        let y1 = ((cy + reach + 1.0).ceil()).min(f64::from(self.bounds.1)) as i64;
        for py in y0..y1 {
            for px in x0..x1 {
                let c = tip::coverage(
                    px as f64 + 0.5 - cx,
                    py as f64 + 0.5 - cy,
                    r,
                    s.aa,
                    s.hardness,
                );
                if c <= 0.0 {
                    continue;
                }
                let key = ((px / CHUNK as i64) as i32, (py / CHUNK as i64) as i32);
                let ch = self.chunks.entry(key).or_insert_with(|| {
                    Box::new(Chunk {
                        t: [1.0; CHUNK * CHUNK],
                        o: [0.0; CHUNK * CHUNK],
                    })
                });
                let i = (py as usize % CHUNK) * CHUNK + px as usize % CHUNK;
                ch.t[i] *= 1.0 - c * d.density;
                if d.opacity > ch.o[i] {
                    ch.o[i] = d.opacity;
                }
            }
        }
    }

    /// The stroke's 8-bit alpha at each touched pixel, in a fixed order: (x, y, a8), a8 > 0.
    pub fn alphas(&self) -> impl Iterator<Item = (u32, u32, u8)> + '_ {
        self.chunks.iter().flat_map(|(&(cx, cy), ch)| {
            (0..CHUNK * CHUNK).filter_map(move |i| {
                let a = ch.o[i] * (1.0 - ch.t[i]);
                // CSP truncates to 8 bits (B-001/B-003).
                let a8 = ((65535.0 * a) as u32 >> 8).min(255) as u8;
                (a8 > 0).then(|| {
                    (
                        cx as u32 * CHUNK as u32 + (i % CHUNK) as u32,
                        cy as u32 * CHUNK as u32 + (i / CHUNK) as u32,
                        a8,
                    )
                })
            })
        })
    }

    /// Composite the stroke onto a layer (straight RGBA8), honouring lock transparency.
    pub fn commit(&self, layer: &mut Layer, color: [u8; 3]) {
        use crate::tile::TILE_SIZE;
        for (x, y, a8) in self.alphas() {
            let (tx, ty) = (
                (x as usize / TILE_SIZE) as i32,
                (y as usize / TILE_SIZE) as i32,
            );
            let i = ((y as usize % TILE_SIZE) * TILE_SIZE + x as usize % TILE_SIZE) * 4;
            if layer.lock_alpha && layer.tiles.get((tx, ty)).is_none() {
                continue;
            }
            let tile = layer.tiles.get_mut((tx, ty));
            let dst = [tile.px[i], tile.px[i + 1], tile.px[i + 2], tile.px[i + 3]];
            let out = over(dst, color, a8, layer.lock_alpha);
            tile.px[i..i + 4].copy_from_slice(&out);
        }
        layer.tiles.prune();
    }
}

/// A stroke pixel (colour `s`, alpha `a8`) over a layer pixel `b` (B-003 alpha; colour open).
pub fn over(b: [u8; 4], s: [u8; 3], a8: u8, lock_alpha: bool) -> [u8; 4] {
    let (ab, at) = (u32::from(b[3]), u32::from(a8));
    if lock_alpha {
        if ab == 0 {
            return b;
        }
        let c =
            |i: usize| ((u32::from(b[i]) * (255 - at) + u32::from(s[i]) * at + 127) / 255) as u8;
        return [c(0), c(1), c(2), b[3]];
    }
    let ao = ab + at - ab * at / 255;
    if ao == 0 {
        return b;
    }
    let wb = ao - at;
    let c = |i: usize| ((u32::from(s[i]) * at + u32::from(b[i]) * wb + ao / 2) / ao) as u8;
    [c(0), c(1), c(2), ao as u8]
}

/// Render one stroke onto a layer: place dabs, build the stroke buffer, commit.
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
    buf.commit(layer, s.color);
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
    fn build_up_matches_b003_levels() {
        // One dab at density 30 % → 76; two strokes of 63 → 111; 95 under 63 → 135.
        let mut buf = StrokeBuffer::new((8, 8));
        let s = BrushSettings {
            aa: Aa::None,
            density: 0.3,
            size: 4.0,
            ..Default::default()
        };
        buf.dab(
            &s,
            &Dab {
                x: 4.0,
                y: 4.0,
                size: 4.0,
                opacity: 1.0,
                density: 0.3,
            },
        );
        assert!(buf.alphas().all(|(_, _, a)| a == 76));
        assert_eq!(over([0, 0, 0, 63], [0, 0, 0], 63, false)[3], 111);
        assert_eq!(over([0, 0, 0, 95], [0, 0, 0], 63, false)[3], 135);
        assert_eq!(over([0, 0, 0, 127], [0, 0, 0], 127, false)[3], 191);
        assert_eq!(over([9, 9, 9, 255], [0, 0, 0], 200, false)[3], 255);
        assert_eq!(over([0; 4], [7, 8, 9], 1, false), [7, 8, 9, 1]);
    }
}
