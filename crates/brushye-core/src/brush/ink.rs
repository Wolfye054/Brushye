//! Ink blending modes: how a finished stroke lands on the layer (specs B-003, B-008).
//!
//! The stroke alpha `a` (opacity × stroke buffer) arrives **unquantised**; only results are
//! rounded (B-008 2, B-003 run 2). `round(v)` below is ⌊v + 0.5⌋.
//!
//! - **Paint** modes add alpha: A' = A + round((255 − A)·a), and an empty pixel gets
//!   ⌊65535·a⌋ >> 8 (B-003 run 2: 25 of 25 stacked strokes). Colour is
//!   round((r·255a + b·(A' − 255a)) / A'), with r the blend against the backdrop composited
//!   over white (B-008 3).
//! - **Keep-alpha** modes change colour only: round(b + (B(b, S) − b)·a); transparent pixels
//!   stay transparent.
//! - Brush-only modes (Erase, Compare density, …) have their own rules (B-008 4).
//!
//! Open (B-008): paint colour over semi-transparent pixels at opacity < 100 % (±1 on ~1 %),
//! Multiply at partial alpha (±2), the colour modes (L-007 tolerances), Background (±1),
//! and Black/White burn at opacities other than 50 and 100 %.

use crate::blend::BlendMode;
use crate::blend::math::{blend_channel, blend_rgb};

macro_rules! ink_modes {
    ($($v:ident => $name:literal),* $(,)?) => {
        /// Brush ink blending mode, in CSP 1.13.2's dropdown order (B-008).
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
        pub enum InkMode {
            #[default]
            $($v),*
        }
        impl InkMode {
            pub const ALL: &'static [InkMode] = &[$(InkMode::$v),*];
            pub fn name(self) -> &'static str {
                match self { $(InkMode::$v => $name),* }
            }
            pub fn from_name(s: &str) -> Option<Self> {
                match s { $($name => Some(InkMode::$v),)* _ => None }
            }
        }
    };
}

ink_modes! {
    Normal => "normal", Darken => "darken", Multiply => "multiply", ColorBurn => "color_burn",
    LinearBurn => "linear_burn", BlackBurn => "black_burn", Subtract => "subtract",
    Lighten => "lighten", Screen => "screen", ColorDodge => "color_dodge",
    GlowDodge => "glow_dodge", WhiteBurn => "white_burn", Add => "add", AddGlow => "add_glow",
    Overlay => "overlay", SoftLight => "soft_light", HardLight => "hard_light",
    Difference => "difference", Erase => "erase", Background => "background",
    ReplaceAlpha => "replace_alpha", CompareDensity => "compare_density",
    EraseCompare => "erase_compare", VividLight => "vivid_light", LinearLight => "linear_light",
    PinLight => "pin_light", HardMix => "hard_mix", Exclusion => "exclusion",
    DarkerColor => "darker_color", LighterColor => "lighter_color", Divide => "divide",
    Hue => "hue", Saturation => "saturation", Color => "color", Brightness => "brightness",
}

/// How a mode treats alpha (B-008 3/4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Paint,
    KeepAlpha,
    BrushOnly,
}

impl InkMode {
    /// The layer blend mode with the same name, if any.
    pub fn blend(self) -> Option<BlendMode> {
        BlendMode::from_name(self.name())
    }

    pub fn family(self) -> Family {
        use InkMode as M;
        match self {
            M::Normal
            | M::Darken
            | M::Multiply
            | M::LinearBurn
            | M::Subtract
            | M::HardLight
            | M::Difference
            | M::Exclusion
            | M::Brightness => Family::Paint,
            M::Erase | M::EraseCompare | M::ReplaceAlpha | M::CompareDensity | M::Background => {
                Family::BrushOnly
            }
            _ => Family::KeepAlpha,
        }
    }
}

fn round8(v: f64) -> u8 {
    (v + 0.5).floor().clamp(0.0, 255.0) as u8
}

/// Stroke alpha truncated to 8 bits (B-001/B-003).
pub fn trunc8(a: f64) -> u8 {
    ((65535.0 * a) as u32 >> 8).min(255) as u8
}

/// Alpha after a paint-family stroke with alpha `a` lands on alpha `ab` (B-003 run 2).
pub fn paint_alpha(ab: u8, a: f64) -> u8 {
    if ab == 0 {
        trunc8(a)
    } else {
        let ab = f64::from(ab);
        round8(ab + (255.0 - ab) * a)
    }
}

/// Composite one stroke pixel: layer pixel `dst`, ink colour `s`, unquantised stroke alpha `a`.
pub fn apply(mode: InkMode, dst: [u8; 4], s: [u8; 3], a: f64, lock_alpha: bool) -> [u8; 4] {
    if a <= 0.0 {
        return dst;
    }
    let ab = dst[3];
    let b = [dst[0], dst[1], dst[2]];
    let keep = |c: [u8; 3]| [c[0], c[1], c[2], ab];
    match mode.family() {
        Family::KeepAlpha => {
            if ab == 0 {
                return dst;
            }
            keep(keep_colour(mode, b, s, a))
        }
        Family::Paint => {
            let r = paint_target(mode, b, ab, s);
            if lock_alpha {
                if ab == 0 {
                    return dst;
                }
                return keep([0, 1, 2].map(|i| {
                    let bf = f64::from(b[i]);
                    round8(bf + (r[i] - bf) * a)
                }));
            }
            let ao = paint_alpha(ab, a);
            if ao == 0 {
                return dst;
            }
            if ab == 0 {
                let c = r.map(round8);
                return [c[0], c[1], c[2], ao];
            }
            let (as_, aof) = (255.0 * a, f64::from(ao));
            let c = [0, 1, 2].map(|i| round8((r[i] * as_ + f64::from(b[i]) * (aof - as_)) / aof));
            [c[0], c[1], c[2], ao]
        }
        Family::BrushOnly => brush_only(mode, dst, s, a, lock_alpha),
    }
}

/// Keep-alpha colour: round(b + (B(b, S) − b)·a), with the brush-only premultiplied rules.
fn keep_colour(mode: InkMode, b: [u8; 3], s: [u8; 3], a: f64) -> [u8; 3] {
    use InkMode as M;
    let lerp = |bv: u8, r: f64| {
        let bf = f64::from(bv);
        round8(bf + (r - bf) * a)
    };
    match mode {
        M::AddGlow => [0, 1, 2].map(|i| round8(f64::from(b[i]) + f64::from(s[i]) * a)),
        M::GlowDodge => [0, 1, 2].map(|i| {
            if b[i] == 0 {
                return 0;
            }
            let den = 255.0 - f64::from(s[i]) * a;
            if den <= 0.0 {
                255
            } else {
                (f64::from(b[i]) * 255.0 / den).floor().min(255.0) as u8
            }
        }),
        // k = 64·O², independent of the brush colour (B-008; only 50 % and 100 % measured).
        M::BlackBurn => b.map(|v| round8(f64::from(v) - 64.0 * a * a)),
        M::WhiteBurn => b.map(|v| round8(f64::from(v) + 64.0 * a * a)),
        // Screen uses its unrounded value.
        M::Screen => [0, 1, 2].map(|i| {
            let (bf, sf) = (f64::from(b[i]), f64::from(s[i]));
            lerp(b[i], bf + sf - bf * sf / 255.0)
        }),
        _ => {
            let blend = mode.blend().unwrap_or(BlendMode::Normal);
            let r = blend_rgb(blend, b, s);
            [0, 1, 2].map(|i| lerp(b[i], f64::from(r[i])))
        }
    }
}

/// Paint-family blend target r (B-008 3): against the backdrop over white.
fn paint_target(mode: InkMode, b: [u8; 3], ab: u8, s: [u8; 3]) -> [f64; 3] {
    let abu = u32::from(ab);
    match mode {
        InkMode::Normal => s.map(f64::from),
        InkMode::Multiply => [0, 1, 2].map(|i| {
            f64::from(s[i]) * (65025.0 - f64::from(ab) * (255.0 - f64::from(b[i]))) / 65025.0
        }),
        InkMode::Brightness => blend_rgb(BlendMode::Brightness, b, s).map(f64::from),
        _ => {
            let bw = b.map(|v| ((u32::from(v) * abu + 255 * (255 - abu) + 127) / 255) as u8);
            let blend = mode.blend().unwrap_or(BlendMode::Normal);
            match blend {
                BlendMode::Brightness => blend_rgb(blend, bw, s).map(f64::from),
                _ => [0, 1, 2].map(|i| f64::from(blend_channel(blend, bw[i], s[i]))),
            }
        }
    }
}

fn brush_only(mode: InkMode, dst: [u8; 4], s: [u8; 3], a: f64, lock_alpha: bool) -> [u8; 4] {
    let ab = dst[3];
    let abf = f64::from(ab);
    let a8 = trunc8(a);
    match mode {
        InkMode::Erase if !lock_alpha => [dst[0], dst[1], dst[2], round8(abf * (1.0 - a))],
        InkMode::EraseCompare if !lock_alpha => {
            [dst[0], dst[1], dst[2], ab.min(round8(255.0 * (1.0 - a)))]
        }
        InkMode::ReplaceAlpha => {
            if lock_alpha {
                if ab == 0 { dst } else { [s[0], s[1], s[2], ab] }
            } else {
                [s[0], s[1], s[2], a8]
            }
        }
        InkMode::CompareDensity => {
            if ab > a8 || a8 == 0 {
                dst
            } else if lock_alpha {
                if ab == 0 { dst } else { [s[0], s[1], s[2], ab] }
            } else {
                [s[0], s[1], s[2], a8]
            }
        }
        InkMode::Background if !lock_alpha => {
            let ao = paint_alpha(ab, a);
            let f = abf / 255.0;
            let den = 1.0 - (1.0 - f) * (1.0 - a);
            if ao == 0 || den <= 0.0 {
                return dst;
            }
            let c = [0, 1, 2]
                .map(|i| round8((f64::from(dst[i]) * f + f64::from(s[i]) * a * (1.0 - f)) / den));
            [c[0], c[1], c[2], ao]
        }
        _ => dst,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip_and_layer_modes_map() {
        assert_eq!(InkMode::ALL.len(), 35);
        for m in InkMode::ALL {
            assert_eq!(InkMode::from_name(m.name()), Some(*m));
        }
        let shared = InkMode::ALL.iter().filter(|m| m.blend().is_some()).count();
        assert_eq!(shared, 28);
    }

    #[test]
    fn b003_commit_levels() {
        // 25 stacked strokes at opacity 10 % (B-003 run 2).
        let want = [
            25, 48, 69, 88, 105, 120, 134, 146, 157, 167, 176, 184, 191, 197, 203, 208, 213, 217,
            221, 224, 227, 230, 233, 235, 237,
        ];
        let mut px = [0u8; 4];
        for w in want {
            px = apply(InkMode::Normal, px, [0, 0, 0], 0.1, false);
            assert_eq!(px[3], w);
        }
        // Run-1 stroke-over-stroke levels.
        assert_eq!(paint_alpha(63, 0.25), 111);
        assert_eq!(paint_alpha(95, 0.25), 135);
        assert_eq!(paint_alpha(127, 0.5), 191);
        // Normal on an opaque pixel: round(b + (S − b)·O); 32.5 → 33.
        assert_eq!(
            apply(InkMode::Normal, [0, 0, 0, 255], [65, 0, 0], 0.5, false)[0],
            33
        );
    }
}
