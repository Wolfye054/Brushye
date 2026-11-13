//! The 28 raster-layer blending modes of CSP 1.13.2, in its dropdown order (spec L-001),
//! and their math (`math`, DECISIONS D-017).

pub mod math;

macro_rules! blend_modes {
    ($($v:ident => $name:literal),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
        pub enum BlendMode {
            #[default]
            $($v),*
        }
        impl BlendMode {
            pub const ALL: &'static [BlendMode] = &[$(BlendMode::$v),*];
            /// Command-syntax name (an enum word, e.g. `color_dodge`).
            pub fn name(self) -> &'static str {
                match self { $(BlendMode::$v => $name),* }
            }
            pub fn from_name(s: &str) -> Option<Self> {
                match s { $($name => Some(BlendMode::$v),)* _ => None }
            }
        }
    };
}

blend_modes! {
    Normal => "normal", Darken => "darken", Multiply => "multiply", ColorBurn => "color_burn",
    LinearBurn => "linear_burn", Subtract => "subtract", Lighten => "lighten", Screen => "screen",
    ColorDodge => "color_dodge", GlowDodge => "glow_dodge", Add => "add", AddGlow => "add_glow",
    Overlay => "overlay", SoftLight => "soft_light", HardLight => "hard_light",
    Difference => "difference", VividLight => "vivid_light", LinearLight => "linear_light",
    PinLight => "pin_light", HardMix => "hard_mix", Exclusion => "exclusion",
    DarkerColor => "darker_color", LighterColor => "lighter_color", Divide => "divide",
    Hue => "hue", Saturation => "saturation", Color => "color", Brightness => "brightness",
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twenty_eight_modes_with_unique_round_tripping_names() {
        assert_eq!(BlendMode::ALL.len(), 28);
        for m in BlendMode::ALL {
            assert_eq!(BlendMode::from_name(m.name()), Some(*m));
        }
        let mut names: Vec<_> = BlendMode::ALL.iter().map(|m| m.name()).collect();
        names.dedup();
        assert_eq!(names.len(), 28);
    }
}
