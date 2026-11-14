//! User brushes: `.brshbrush` files and the brush library (DECISIONS D-014 5).
//!
//! A ZIP archive with `brush.json` (format id and version, the brush's name and its settings)
//! plus optional tip/texture images under `images/`. The settings object uses the same keys
//! and value forms as the `brush.stroke` command's JSON form; `brushye-command` validates it.
//! Brushes are not stored in documents.

use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

use serde_json::{Map, Value as Json, json};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

/// Current `.brshbrush` format version.
pub const BRUSH_FORMAT_VERSION: u32 = 1;
pub const BRUSH_FORMAT_ID: &str = "brushye-brush";
/// File extension of user brushes.
pub const EXTENSION: &str = "brshbrush";
const MAX_ENTRY: u64 = 64 << 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrushFileError(pub String);

impl std::fmt::Display for BrushFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ".brshbrush: {}", self.0)
    }
}

impl std::error::Error for BrushFileError {}

fn err(m: impl std::fmt::Display) -> BrushFileError {
    BrushFileError(m.to_string())
}

/// A user brush.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct BrushFile {
    pub name: String,
    /// Settings, in `brush.stroke` JSON form (without layer, colour and samples).
    pub settings: Map<String, Json>,
    /// Tip/texture images by file name (PNG bytes), stored under `images/`.
    pub images: BTreeMap<String, Vec<u8>>,
}

/// Encode a brush (deterministic bytes).
pub fn save(b: &BrushFile) -> Result<Vec<u8>, BrushFileError> {
    if b.name.trim().is_empty() {
        return Err(err("a brush needs a name"));
    }
    let mut zw = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = |m| {
        SimpleFileOptions::default()
            .compression_method(m)
            .last_modified_time(DateTime::default())
            .unix_permissions(0o644)
    };
    let j = json!({
        "format": BRUSH_FORMAT_ID,
        "version": BRUSH_FORMAT_VERSION,
        "app": env!("CARGO_PKG_VERSION"),
        "name": b.name,
        "settings": b.settings,
    });
    zw.start_file("brush.json", opts(CompressionMethod::Deflated))
        .map_err(err)?;
    zw.write_all(&serde_json::to_vec_pretty(&j).map_err(err)?)
        .map_err(err)?;
    for (name, data) in &b.images {
        if name.contains(['/', '\\']) || name.starts_with('.') || !name.ends_with(".png") {
            return Err(err(format!(
                "bad image name {name:?} (plain *.png names only)"
            )));
        }
        zw.start_file(format!("images/{name}"), opts(CompressionMethod::Stored))
            .map_err(err)?;
        zw.write_all(data).map_err(err)?;
    }
    Ok(zw.finish().map_err(err)?.into_inner())
}

/// Decode a brush. With `strict`, unknown entries or keys are errors; otherwise warnings.
pub fn load(bytes: &[u8], strict: bool) -> Result<(BrushFile, Vec<String>), BrushFileError> {
    let mut za = ZipArchive::new(Cursor::new(bytes)).map_err(err)?;
    let mut json_bytes = None;
    let mut images = BTreeMap::new();
    let mut warnings = Vec::new();
    for i in 0..za.len() {
        let f = za.by_index(i).map_err(err)?;
        if f.is_dir() {
            continue;
        }
        let name = f.name().to_owned();
        let mut data = Vec::new();
        f.take(MAX_ENTRY + 1).read_to_end(&mut data).map_err(err)?;
        if data.len() as u64 > MAX_ENTRY {
            return Err(err(format!("{name}: entry too large")));
        }
        if name == "brush.json" {
            json_bytes = Some(data);
        } else if let Some(img) = name.strip_prefix("images/").filter(|n| !n.contains('/')) {
            images.insert(img.to_owned(), data);
        } else {
            warnings.push(format!("unknown entry {name}"));
        }
    }
    let data = json_bytes.ok_or_else(|| err("missing brush.json"))?;
    let Json::Object(m) = serde_json::from_slice(&data).map_err(err)? else {
        return Err(err("brush.json: expected a JSON object"));
    };
    if m.get("format").and_then(Json::as_str) != Some(BRUSH_FORMAT_ID) {
        return Err(err("not a Brushye brush (format id)"));
    }
    let version = m
        .get("version")
        .and_then(Json::as_u64)
        .ok_or_else(|| err("brush.json: missing version"))?;
    if version > u64::from(BRUSH_FORMAT_VERSION) {
        let app = m.get("app").and_then(Json::as_str).unwrap_or("?");
        return Err(err(format!(
            "this brush is format version {version}, written by Brushye {app}; this Brushye reads up to version {BRUSH_FORMAT_VERSION}. Update Brushye to use it."
        )));
    }
    let name = m
        .get("name")
        .and_then(Json::as_str)
        .filter(|n| !n.trim().is_empty())
        .ok_or_else(|| err("brush.json: missing name"))?
        .to_owned();
    let settings = match m.get("settings") {
        Some(Json::Object(s)) => s.clone(),
        _ => return Err(err("brush.json: \"settings\" must be an object")),
    };
    for k in m.keys() {
        if !["format", "version", "app", "name", "settings"].contains(&k.as_str()) {
            warnings.push(format!("brush.json: unknown key \"{k}\""));
        }
    }
    if strict && !warnings.is_empty() {
        return Err(err(warnings.join("; ")));
    }
    Ok((
        BrushFile {
            name,
            settings,
            images,
        },
        warnings,
    ))
}

/// One entry of a brush library folder.
#[derive(Clone, Debug, PartialEq)]
pub struct LibraryEntry {
    pub path: PathBuf,
    /// The brush, or why it couldn't be read.
    pub brush: Result<BrushFile, BrushFileError>,
}

/// List the `.brshbrush` files of a library folder, sorted by file name. Unreadable brushes
/// are listed with their error rather than hidden.
pub fn scan_library(dir: &Path) -> std::io::Result<Vec<LibraryEntry>> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == EXTENSION) && p.is_file())
        .collect();
    paths.sort();
    Ok(paths
        .into_iter()
        .map(|path| {
            let brush = std::fs::read(&path)
                .map_err(err)
                .and_then(|b| load(&b, false).map(|x| x.0));
            LibraryEntry { path, brush }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> BrushFile {
        let mut settings = Map::new();
        settings.insert("size".into(), Json::from(12.5));
        settings.insert("hardness".into(), Json::from(3));
        let mut images = BTreeMap::new();
        images.insert("tip.png".into(), vec![1, 2, 3]);
        BrushFile {
            name: "Soft pen ✒".into(),
            settings,
            images,
        }
    }

    #[test]
    fn round_trip_is_exact_and_deterministic() {
        let b = sample();
        let bytes = save(&b).unwrap_or_else(|e| unreachable!("{e}"));
        assert_eq!(bytes, save(&b).unwrap_or_else(|e| unreachable!("{e}")));
        let (back, warnings) = load(&bytes, true).unwrap_or_else(|e| unreachable!("{e}"));
        assert_eq!(back, b);
        assert!(warnings.is_empty());
    }

    #[test]
    fn rejects_bad_brushes() {
        let mut b = sample();
        b.images.insert("../evil.png".into(), vec![]);
        assert!(save(&b).is_err());
        let mut b = sample();
        b.name = " ".into();
        assert!(save(&b).is_err());
        assert!(load(b"nope", false).is_err());
    }
}
