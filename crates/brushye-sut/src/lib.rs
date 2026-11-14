//! Import of Clip Studio Paint 1.13.2 sub tool files (`.sut`), which the user supplies at
//! runtime. Nothing CSP-made is bundled. The format knowledge comes only from the black-box
//! study in `specs/formats/sut.md`, and every mapping here cites it.
//!
//! A `.sut` file is an SQLite database. It is loaded from bytes into an in-memory, read-only
//! connection with SQLite's defensive settings, and only SELECT is run. The brush settings
//! Brushye has become a [`BrushPreset`]. Everything else is listed as a [`Note`]: either
//! approximated, or not imported because Brushye lacks the feature.

mod effector;

use std::collections::HashMap;

use brushye_command::brush_preset::BrushPreset;
use brushye_command::core::brush::{Aa, Curve, Gap, InkMode, TipAxis};
use rusqlite::types::Value;
use rusqlite::{Connection, OpenFlags};

pub use effector::Effector;

/// Largest file accepted (a sub tool with embedded images is a few hundred KiB).
pub const MAX_FILE: usize = 64 << 20;

/// `Manager.Version` of every CSP 1.13.2 sample. Other versions are read, with a note.
pub const KNOWN_VERSION: i64 = 138;

/// Column names of `Variant` in the samples (sut.md). Columns outside this list are reported.
const KNOWN_COLUMNS: &str = include_str!("variant_columns.txt");

#[derive(Debug)]
pub struct SutError(pub String);

impl std::fmt::Display for SutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SutError {}

impl From<rusqlite::Error> for SutError {
    fn from(e: rusqlite::Error) -> Self {
        SutError(format!("not a readable .sut file ({e})"))
    }
}

/// Something about the import the user should know.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Note {
    /// Imported, but drawn differently from CSP or with a value Brushye can't hold exactly.
    Approximated { setting: String, detail: String },
    /// A CSP feature that is set in the file but that Brushye doesn't have; it is ignored.
    NotImported { feature: String, detail: String },
}

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Note::Approximated { setting, detail } => {
                write!(f, "approximated: {setting}: {detail}")
            }
            Note::NotImported { feature, detail } => write!(f, "not imported: {feature}: {detail}"),
        }
    }
}

/// One imported sub tool.
#[derive(Clone, Debug, PartialEq)]
pub struct SubTool {
    pub name: String,
    pub preset: BrushPreset,
    pub notes: Vec<Note>,
}

/// Import every sub tool in a `.sut` file (one in the samples).
pub fn import(bytes: &[u8]) -> Result<Vec<SubTool>, SutError> {
    if bytes.len() > MAX_FILE {
        return Err(SutError(format!("file is over {} MiB", MAX_FILE >> 20)));
    }
    if !bytes.starts_with(b"SQLite format 3\0") {
        return Err(SutError("not a .sut file (no SQLite header)".into()));
    }
    let con = open(bytes)?;
    let version: i64 = con.query_row("SELECT Version FROM Manager", [], |r| r.get(0))?;
    let mut file_notes = Vec::new();
    if version != KNOWN_VERSION {
        file_notes.push(Note::Approximated {
            setting: "file version".into(),
            detail: format!(
                "version {version}; Brushye knows version {KNOWN_VERSION} (CSP 1.13.2)"
            ),
        });
    }
    let mut stmt = con.prepare("SELECT NodeName, NodeVariantID FROM Node")?;
    let nodes: Vec<(String, Option<i64>)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?;
    let mut out = Vec::new();
    for (name, variant) in nodes {
        // Nodes without settings are groups (not in the samples); skip them.
        let Some(variant) = variant else { continue };
        let row = variant_row(&con, variant)?;
        let mut sub = map(&name, &row)?;
        sub.notes.splice(0..0, file_notes.iter().cloned());
        out.push(sub);
    }
    if out.is_empty() {
        return Err(SutError("the file has no sub tool".into()));
    }
    Ok(out)
}

fn open(bytes: &[u8]) -> Result<Connection, SutError> {
    let mut con = Connection::open_with_flags(
        ":memory:",
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    con.deserialize_read_exact(rusqlite::MAIN_DB, bytes, bytes.len(), true)?;
    // Defensive settings for files from elsewhere (sqlite.org/security.html).
    con.set_db_config(rusqlite::config::DbConfig::SQLITE_DBCONFIG_DEFENSIVE, true)?;
    con.set_db_config(
        rusqlite::config::DbConfig::SQLITE_DBCONFIG_TRUSTED_SCHEMA,
        false,
    )?;
    con.pragma_update(None, "query_only", true)?;
    con.pragma_update(None, "cell_size_check", true)?;
    Ok(con)
}

type Row = HashMap<String, Value>;

fn variant_row(con: &Connection, id: i64) -> Result<Row, SutError> {
    // Column names are read as raw bytes from the table info: rusqlite's column_name and
    // column_names panic on a name that isn't UTF-8 (possible in a damaged file).
    let mut info = con.prepare("SELECT CAST(name AS BLOB) FROM pragma_table_info('Variant')")?;
    let names: Vec<String> = info
        .query_map([], |r| r.get::<_, Vec<u8>>(0))?
        .map(|n| n.map(|b| String::from_utf8_lossy(&b).into_owned()))
        .collect::<Result<_, _>>()?;
    let mut stmt = con.prepare("SELECT * FROM Variant WHERE VariantID = ?1")?;
    if stmt.column_count() != names.len() {
        return Err(SutError("the settings table is damaged".into()));
    }
    let mut rows = stmt.query([id])?;
    let Some(r) = rows.next()? else {
        return Err(SutError(format!("settings row {id} is missing")));
    };
    let mut out = HashMap::new();
    for (i, n) in names.into_iter().enumerate() {
        // Text that isn't UTF-8 is treated as missing (no setting here is text).
        let v = Value::try_from(r.get_ref(i)?).unwrap_or(Value::Null);
        out.insert(n, v);
    }
    Ok(out)
}

fn int(row: &Row, col: &str) -> Option<i64> {
    match row.get(col) {
        Some(Value::Integer(v)) => Some(*v),
        Some(Value::Real(v)) => Some(*v as i64),
        _ => None,
    }
}

fn real(row: &Row, col: &str) -> Option<f64> {
    match row.get(col) {
        Some(Value::Real(v)) => Some(*v),
        Some(Value::Integer(v)) => Some(*v as f64),
        _ => None,
    }
}

fn blob<'a>(row: &'a Row, col: &str) -> Option<&'a [u8]> {
    match row.get(col) {
        Some(Value::Blob(b)) => Some(b),
        _ => None,
    }
}

fn on(row: &Row, col: &str) -> bool {
    int(row, col).is_some_and(|v| v != 0)
}

fn pct(v: Option<i64>) -> Option<u8> {
    v.map(|v| v.clamp(0, 100) as u8).filter(|v| *v != 100)
}

/// `CompositeMode` value → Brushye ink mode name, one sample each (sut.md: S-01, S-44..S-47,
/// S-70..S-99).
const INK_MODES: &[(i64, &str)] = &[
    (0, "normal"),
    (1, "darken"),
    (2, "multiply"),
    (3, "color_burn"),
    (4, "linear_burn"),
    (5, "subtract"),
    (6, "darker_color"),
    (7, "lighten"),
    (8, "screen"),
    (9, "color_dodge"),
    (10, "glow_dodge"),
    (11, "add"),
    (12, "add_glow"),
    (13, "lighter_color"),
    (14, "overlay"),
    (15, "soft_light"),
    (16, "hard_light"),
    (17, "vivid_light"),
    (18, "linear_light"),
    (19, "pin_light"),
    (20, "hard_mix"),
    (21, "difference"),
    (22, "exclusion"),
    (23, "hue"),
    (24, "saturation"),
    (25, "color"),
    (26, "brightness"),
    (27, "erase"),
    (29, "background"),
    (31, "replace_alpha"),
    (32, "black_burn"),
    (33, "white_burn"),
    (34, "compare_density"),
    (35, "erase_compare"),
    (36, "divide"),
];

fn ink_mode(v: i64) -> Option<InkMode> {
    INK_MODES
        .iter()
        .find(|(k, _)| *k == v)
        .and_then(|(_, name)| InkMode::from_name(name))
}

/// `BrushHardness` of the 5 segments of CSP's hardness bar, levels 1..=5 (S-63..S-67). These
/// are the levels Brushye's tips were measured at (B-009).
const HARDNESS_LEVELS: [i64; 5] = [1, 30, 60, 95, 100];

/// `BrushRotationEffector` values seen: 3 = none (S-01), 67 = Direction of line (S-110).
const ANGLE_NONE: i64 = 3;
const ANGLE_DIRECTION_OF_LINE: i64 = 67;

/// Map one settings row to a preset (sut.md, "Brush settings").
fn map(name: &str, row: &Row) -> Result<SubTool, SutError> {
    let mut notes = Vec::new();
    let mut approx = |setting: &str, detail: String| {
        notes.push(Note::Approximated {
            setting: setting.into(),
            detail,
        })
    };
    let mut size = real(row, "BrushSize").ok_or_else(|| SutError("no brush size".into()))?;
    if int(row, "BrushSizeUnit").unwrap_or(0) != 0 {
        approx(
            "Brush size",
            format!(
                "size unit {} isn't known; {size} read as pixels",
                int(row, "BrushSizeUnit").unwrap_or(0)
            ),
        );
    }
    if !(0.5..=2000.0).contains(&size) {
        approx("Brush size", format!("{size} px clamped to 0.5..2000"));
        size = size.clamp(0.5, 2000.0);
    }
    let aa = match int(row, "AntiAlias") {
        Some(0) => Some(Aa::None),
        Some(1) => Some(Aa::Weak),
        Some(3) => Some(Aa::Strong),
        _ => None,
    };
    let hardness = match int(row, "BrushHardness") {
        None | Some(100) => None,
        Some(v) => {
            // The nearest of the 5 measured levels; exact for the bar's own values.
            let (i, level) = HARDNESS_LEVELS
                .iter()
                .enumerate()
                .min_by_key(|(_, l)| (*l - v).abs())
                .unwrap_or((4, &100));
            if *level != v {
                approx(
                    "Hardness",
                    format!(
                        "{v} of 100 drawn as level {} ({level}); values between the bar's steps aren't measured",
                        i + 1
                    ),
                );
            }
            Some(i as u8 + 1).filter(|l| *l != 5)
        }
    };
    let thickness = int(row, "BrushThickness")
        .map(|v| v.clamp(1, 100) as u8)
        .filter(|v| *v != 100);
    let direction = Some(if on(row, "BrushVerticalThicknes") {
        TipAxis::Vertical
    } else {
        TipAxis::Horizontal
    });
    let angle = real(row, "BrushRotation").filter(|a| *a != 0.0);
    let gap = match int(row, "BrushAutoIntervalType") {
        Some(3) => Some(Gap::Narrow),
        Some(1) => Some(Gap::Wide),
        // Fixed: the stored value is the typed one, in percent of size (S-35, B-002).
        Some(0) => real(row, "BrushInterval").map(Gap::Fixed),
        _ => None,
    };
    let ink = match int(row, "CompositeMode") {
        None | Some(0) => None,
        Some(v) => match ink_mode(v) {
            Some(m) => Some(m),
            None => {
                notes.push(Note::NotImported {
                    feature: "Ink blending mode".into(),
                    detail: format!("mode {v} isn't mapped yet; Normal is used"),
                });
                None
            }
        },
    };
    let stabilization = int(row, "FlickerReduction")
        .map(|v| v.clamp(0, 100) as u8)
        .filter(|v| *v != 0);
    if stabilization.is_some() && on(row, "FlickerReductionBySpeed") {
        notes.push(Note::Approximated {
            setting: "Stabilization".into(),
            detail: "\"adjust by speed\" isn't modelled yet (B-007); plain stabilization is used"
                .into(),
        });
    }
    let angle_line = match int(row, "BrushRotationEffector") {
        None | Some(ANGLE_NONE) => None,
        Some(ANGLE_DIRECTION_OF_LINE) => Some(true),
        Some(v) => {
            notes.push(Note::NotImported {
                feature: "Angle dynamics".into(),
                detail: format!("input {v} (direction of pen, tilt or random) isn't supported"),
            });
            None
        }
    };
    let density_by_gap = on(row, "BrushAdjustFlowByInterval").then_some(true);
    let mut dynamics = |label: &str, col: &str| -> (Option<u8>, Option<Curve>) {
        let Some(b) = blob(row, col) else {
            return (None, None);
        };
        match Effector::parse(b) {
            Ok(e) => {
                let (min, curve, n) = e.to_brushye(label);
                notes.extend(n);
                (min, curve)
            }
            Err(err) => {
                notes.push(Note::NotImported {
                    feature: format!("{label} dynamics"),
                    detail: err,
                });
                (None, None)
            }
        }
    };
    let (size_pressure, size_curve) = dynamics("Brush size", "BrushSizeEffector");
    let (opacity_pressure, opacity_curve) = dynamics("Opacity", "BrushOpacityEffector");
    let (density_pressure, density_curve) = dynamics("Brush density", "BrushFlowEffector");
    let preset = BrushPreset {
        size,
        aa,
        hardness,
        opacity: pct(int(row, "Opacity")),
        density: pct(int(row, "BrushFlow")),
        ink,
        gap,
        density_by_gap,
        thickness,
        direction,
        angle,
        angle_line,
        stabilization,
        size_pressure,
        size_curve,
        opacity_pressure,
        opacity_curve,
        density_pressure,
        density_curve,
    };
    notes.extend(unmapped(row));
    preset
        .validate()
        .map_err(|e| SutError(format!("“{name}”: imported settings are invalid: {e}")))?;
    Ok(SubTool {
        name: name.to_owned(),
        preset,
        notes,
    })
}

/// Features that are on in the file but that Brushye doesn't have (sut.md, "not mapped").
fn unmapped(row: &Row) -> Vec<Note> {
    let mut out = Vec::new();
    let mut add = |feature: &str, detail: &str| {
        out.push(Note::NotImported {
            feature: feature.into(),
            detail: detail.into(),
        })
    };
    let flags = [
        (
            "BrushUseWaterColor",
            "Color mixing",
            "blend, running colour or smear",
        ),
        ("BrushUseSpray", "Spray effect", "particles"),
        ("BrushUseWaterEdge", "Watercolor edge", "edge darkening"),
        (
            "BrushChangePatternColor",
            "Color jitter",
            "per-dab colour change",
        ),
        (
            "BrushUsePatternImage",
            "Material tip",
            "image tips; the round tip is used",
        ),
        ("UseDualBrush", "2-Brush shape", "dual brush"),
        ("BrushRibbon", "Ribbon", "ribbon tip"),
        (
            "BrushSizeSyncViewScale",
            "Specify by size on screen",
            "size follows the zoom",
        ),
        ("BrushAtLeast1Pixel", "At least 1 pixel", "minimum dab size"),
    ];
    // BrushUseVectorEraser (on for eraser sub tools, S-115) only affects vector layers, which
    // Brushye doesn't have, so it isn't reported.
    for (col, feature, detail) in flags {
        if on(row, col) {
            add(feature, detail);
        }
    }
    if matches!(row.get("TextureImage"), Some(Value::Blob(_))) {
        add("Texture", "paper texture");
    }
    for (col, setting) in [
        ("BrushThicknessEffector", "Thickness"),
        ("BrushSpraySizeEffector", "Particle size"),
    ] {
        if let Some(b) = blob(row, col)
            && Effector::parse(b).is_ok_and(|e| e.mask != 0)
        {
            add(
                &format!("{setting} dynamics"),
                "Brushye has no dynamics for this setting",
            );
        }
    }
    let known: Vec<&str> = KNOWN_COLUMNS.lines().collect();
    let mut extra: Vec<&String> = row
        .keys()
        .filter(|k| !known.contains(&k.as_str()))
        .collect();
    extra.sort();
    if !extra.is_empty() {
        add(
            "Unknown settings",
            &format!(
                "columns not in the study samples: {}",
                extra
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal synthetic .sut (our own, no CSP data): the three tables with a few columns.
    fn synthetic() -> Vec<u8> {
        let mut con = Connection::open_in_memory().unwrap_or_else(|e| unreachable!("{e}"));
        con.execute_batch(
            "CREATE TABLE Manager (_PW_ID INTEGER PRIMARY KEY, Version INTEGER);
             INSERT INTO Manager VALUES (1, 138);
             CREATE TABLE Node (_PW_ID INTEGER PRIMARY KEY, NodeName TEXT, NodeVariantID INTEGER);
             INSERT INTO Node VALUES (1, 'Test', 7);
             CREATE TABLE Variant (_PW_ID INTEGER PRIMARY KEY, VariantID INTEGER, BrushSize REAL,
               Opacity INTEGER, AntiAlias INTEGER, CompositeMode INTEGER, BrushUseSpray INTEGER);
             INSERT INTO Variant VALUES (1, 7, 12.5, 40, 0, 2, 1);",
        )
        .unwrap_or_else(|e| unreachable!("{e}"));
        con.serialize(rusqlite::MAIN_DB)
            .unwrap_or_else(|e| unreachable!("{e}"))
            .to_vec()
    }

    #[test]
    fn imports_a_synthetic_file() {
        let subs = import(&synthetic()).unwrap_or_else(|e| unreachable!("{e}"));
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].name, "Test");
        assert_eq!(
            subs[0].preset.to_text(),
            "size=12.5 aa=none opacity=40 ink=multiply direction=horizontal"
        );
        assert!(
            subs[0]
                .notes
                .iter()
                .any(|n| n.to_string().contains("Spray"))
        );
    }

    #[test]
    fn damaged_files_fail_cleanly() {
        let good = synthetic();
        assert!(import(b"not a database").is_err());
        assert!(import(&good[..100]).is_err());
        // Flip bytes all over the file: every result is Ok or Err, never a panic.
        let mut seed = 0x2545_f491_u32;
        for _ in 0..3000 {
            let mut b = good.clone();
            for _ in 0..8 {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                let i = seed as usize % b.len();
                b[i] ^= (seed >> 8) as u8 | 1;
            }
            let _ = import(&b);
        }
    }
}
