//! Per-user application settings (like CSP's preferences): stored as JSON in the user's
//! configuration folder and applied to every session as commands, so they're journaled.

use std::path::PathBuf;

use serde_json::{Value as Json, json};

/// User configuration folder: `<config>/brushye`.
pub fn config_dir() -> PathBuf {
    let base = if cfg!(windows) {
        std::env::var("APPDATA").ok().map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join("Library/Application Support"))
    } else {
        std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".config"))
            })
    };
    base.unwrap_or_else(|| PathBuf::from(".")).join("brushye")
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Settings {
    /// Global pen pressure curve: interior control points.
    pub pressure_curve: Vec<[f64; 2]>,
}

impl Settings {
    fn path() -> PathBuf {
        config_dir().join("settings.json")
    }

    /// Load, falling back to defaults on any problem.
    pub fn load() -> Self {
        let Ok(text) = std::fs::read_to_string(Self::path()) else {
            return Self::default();
        };
        let Ok(j) = serde_json::from_str::<Json>(&text) else {
            return Self::default();
        };
        let pressure_curve = j["pressure_curve"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|p| Some([p.get(0)?.as_f64()?, p.get(1)?.as_f64()?]))
                    .collect()
            })
            .unwrap_or_default();
        Self { pressure_curve }
    }

    pub fn save(&self) -> Result<(), String> {
        let dir = config_dir();
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let j = json!({ "pressure_curve": self.pressure_curve });
        let text = serde_json::to_string_pretty(&j).map_err(|e| e.to_string())?;
        brushye_format::brsh::write_atomic(&Self::path(), text.as_bytes())
            .map_err(|e| e.to_string())
    }
}
