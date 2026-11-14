//! Brush presets: the settings of `brush.stroke` without layer, colour and samples. They are
//! what a `.brshbrush` file stores (DECISIONS D-014 5), in the same JSON form as the command.

use brushye_core::brush::{
    Aa, BrushSettings, Curve, Dynamics, Gap, HARDNESS_MAX, InkMode, TipAxis,
};
use serde_json::{Map, Value as Json};

use crate::commands::BrushStroke;
use crate::params;
use crate::params::{JsonReader, Reader};
use crate::syntax::{Line, RawLine, parse_line, print_line};

params! {
    /// All settings of a round brush (see `brush.stroke` for each key).
    pub struct BrushPreset {
        size: f64 = key "Brush size (diameter) in px",
        aa: Option<Aa> = opt "Anti-aliasing (default middle)",
        hardness: Option<u8> = opt "Tip hardness 1 (soft) ..= 5 (hard, default)",
        opacity: Option<u8> = opt "Ink opacity in percent (default 100)",
        density: Option<u8> = opt "Brush density in percent (default 100)",
        ink: Option<InkMode> = opt "Ink blending mode (default normal)",
        gap: Option<Gap> = opt "Dab gap: narrow, normal (default), wide, or percent of size",
        density_by_gap: Option<bool> = opt "Adjust brush density by gap",
        thickness: Option<u8> = opt "Tip thickness in percent, 1..=100 (default 100 = round)",
        direction: Option<TipAxis> = opt "Axis that thickness shortens (default vertical)",
        angle: Option<f64> = opt "Tip angle in degrees, clockwise (default 0)",
        angle_line: Option<bool> = opt "Tip angle follows the direction of the line",
        stabilization: Option<u8> = opt "Stabilization 0..=100 (default 0)",
        size_pressure: Option<u8> = opt "Pen pressure controls size; minimum in percent",
        size_curve: Option<Curve> = opt "Size pressure curve points [x,y;…]",
        opacity_pressure: Option<u8> = opt "Pen pressure controls opacity; minimum in percent",
        opacity_curve: Option<Curve> = opt "Opacity pressure curve points [x,y;…]",
        density_pressure: Option<u8> = opt "Pen pressure controls density; minimum in percent",
        density_curve: Option<Curve> = opt "Density pressure curve points [x,y;…]",
    }
}

impl BrushPreset {
    /// The preset part of a stroke command.
    pub fn from_stroke(p: &BrushStroke) -> Self {
        Self {
            size: p.size,
            aa: p.aa,
            hardness: p.hardness,
            opacity: p.opacity,
            density: p.density,
            ink: p.ink,
            gap: p.gap,
            density_by_gap: p.density_by_gap,
            thickness: p.thickness,
            direction: p.direction,
            angle: p.angle,
            angle_line: p.angle_line,
            stabilization: p.stabilization,
            size_pressure: p.size_pressure,
            size_curve: p.size_curve.clone(),
            opacity_pressure: p.opacity_pressure,
            opacity_curve: p.opacity_curve.clone(),
            density_pressure: p.density_pressure,
            density_curve: p.density_curve.clone(),
        }
    }

    /// Engine settings with an ink colour; validated.
    pub fn settings(&self, color: [u8; 3]) -> Result<BrushSettings, String> {
        let pct = |name: &str, v: Option<u8>, default: f64| -> Result<f64, String> {
            match v {
                None => Ok(default),
                Some(v) if v <= 100 => Ok(f64::from(v) / 100.0),
                Some(v) => Err(format!("{name} {v} is over 100")),
            }
        };
        let dynamics = |name: &str,
                        min: Option<u8>,
                        curve: &Option<Curve>|
         -> Result<Option<Dynamics>, String> {
            match (min, curve) {
                (None, None) => Ok(None),
                (None, Some(_)) => Err(format!("{name}_curve needs {name}_pressure=…")),
                (Some(m), c) => Ok(Some(Dynamics {
                    min: pct(&format!("{name}_pressure"), Some(m), 0.0)?,
                    curve: c.clone().unwrap_or_default(),
                })),
            }
        };
        let s = BrushSettings {
            color,
            size: self.size,
            aa: self.aa.unwrap_or_default(),
            hardness: self.hardness.unwrap_or(HARDNESS_MAX),
            density: pct("density", self.density, 1.0)?,
            opacity: pct("opacity", self.opacity, 1.0)?,
            ink: self.ink.unwrap_or_default(),
            gap: self.gap.unwrap_or_default(),
            density_by_gap: self.density_by_gap.unwrap_or(false),
            thickness: match self.thickness {
                None => 1.0,
                Some(t @ 1..=100) => f64::from(t) / 100.0,
                Some(t) => return Err(format!("thickness {t} is not in 1..=100")),
            },
            tip_axis: self.direction.unwrap_or_default(),
            angle: self.angle.unwrap_or(0.0),
            angle_follows_line: self.angle_line.unwrap_or(false),
            stabilization: self.stabilization.unwrap_or(0),
            size_pressure: dynamics("size", self.size_pressure, &self.size_curve)?,
            opacity_pressure: dynamics("opacity", self.opacity_pressure, &self.opacity_curve)?,
            density_pressure: dynamics("density", self.density_pressure, &self.density_curve)?,
        };
        s.validate().map_err(|e| e.0)?;
        Ok(s)
    }

    /// Validate without a colour.
    pub fn validate(&self) -> Result<(), String> {
        self.settings([0, 0, 0]).map(|_| ())
    }

    /// JSON object form (the `settings` of a `.brshbrush`).
    pub fn to_json(&self) -> Map<String, Json> {
        self.write_json()
    }

    pub fn from_json(m: &Map<String, Json>) -> Result<Self, String> {
        let mut r = JsonReader::new(m);
        let p = Self::read_json(&mut r)?;
        r.finish()?;
        p.validate()?;
        Ok(p)
    }

    /// Text form: `key=value …`, the same as on a `brush.stroke` line.
    pub fn to_text(&self) -> String {
        let mut raw = RawLine {
            name: "x".into(),
            version: 1,
            ..Default::default()
        };
        self.write(&mut raw.positional, &mut raw.named);
        let line = print_line(&raw);
        line.strip_prefix("x ").unwrap_or("").to_owned()
    }

    pub fn from_text(s: &str) -> Result<Self, String> {
        let raw = match parse_line(&format!("x {s}")).map_err(|e| e.to_string())? {
            Line::Item(r) if !r.query && r.seq.is_none() => r,
            _ => return Err("expected key=value settings".into()),
        };
        let mut r = Reader::new(&raw.positional, &raw.named);
        let p = Self::read(&mut r)?;
        r.finish()?;
        p.validate()?;
        Ok(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_and_json_round_trip() {
        let p = BrushPreset::from_text(
            "size=12.5 hardness=3 gap=wide thickness=40 angle_line=true size_pressure=20 size_curve=[0.5,0.25] ink=multiply",
        )
        .unwrap_or_else(|e| unreachable!("{e}"));
        assert_eq!(BrushPreset::from_text(&p.to_text()), Ok(p.clone()));
        assert_eq!(BrushPreset::from_json(&p.to_json()), Ok(p.clone()));
        assert_eq!(
            p.to_text(),
            "size=12.5 hardness=3 ink=multiply gap=wide thickness=40 angle_line=true size_pressure=20 size_curve=[0.5,0.25]"
        );
        assert!(BrushPreset::from_text("size=5 hardness=9").is_err());
        assert!(BrushPreset::from_text("size=5 colour=#ff0000").is_err());
    }
}
