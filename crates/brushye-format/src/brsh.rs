//! The native document format `.brsh` (DECISIONS D-014).
//!
//! A ZIP archive:
//! - `manifest.json`: format id and version, writing app version, feature list;
//! - `document.json`: canvas, paper, layer stack (bottom to top) and properties, active layer;
//! - `layers/<id>/layer.json`: pixel format and tile size;
//! - `layers/<id>/tiles/<tx>_<ty>.zst`: each non-empty 256² tile, raw straight RGBA8,
//!   zstd-compressed and stored without ZIP compression;
//! - `preview.png` (flattened, full size) and `thumbnail.png` (at most 256 px);
//! - `journal.bcmd`: only when the document opts in.
//!
//! Output is deterministic for a given document and library versions: fixed entry order,
//! fixed timestamps, fixed compression levels. Saves are crash-safe ([`write_atomic`]).
//! Files from a newer format version are refused; older ones are migrated on open.

use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::sync::{Arc, Weak};

use brushye_core::render::{Image, flatten};
use brushye_core::tile::Tile;
use brushye_core::{BlendMode, Document, Layer, LayerId, LayerKind, Rgba8, TILE_SIZE};
use serde_json::{Map, Value as Json, json};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

use crate::png_io;

/// Current `.brsh` format version.
pub const FORMAT_VERSION: u32 = 1;
/// Format id in `manifest.json`.
pub const FORMAT_ID: &str = "brushye-document";
/// zstd level for tiles: fast saves (D-014).
pub const ZSTD_LEVEL: i32 = 3;
/// Largest thumbnail side.
pub const THUMBNAIL_MAX: u32 = 256;
/// Largest JSON/journal entry read (protects against hostile files).
const MAX_TEXT_ENTRY: u64 = 256 << 20;
const TILE_BYTES: usize = TILE_SIZE * TILE_SIZE * 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrshError(pub String);

impl std::fmt::Display for BrshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ".brsh: {}", self.0)
    }
}

impl std::error::Error for BrshError {}

fn err(m: impl std::fmt::Display) -> BrshError {
    BrshError(m.to_string())
}

/// What to write besides the document itself.
#[derive(Clone, Debug, Default)]
pub struct SaveOptions<'a> {
    /// Active layer (UI state, restored on open).
    pub active: Option<LayerId>,
    /// Session journal to embed (opt-in per document, D-014 4).
    pub journal: Option<&'a str>,
}

/// Layer id and tile coordinates.
type TileKey = (u32, i32, i32);

/// Compressed tile bytes kept between saves, so a save recompresses only changed tiles
/// (D-014 2). Tiles are shared copy-on-write (D-013): an unchanged tile is the same `Arc`, so
/// the cache keys on tile identity, holding a `Weak` so the address can't be reused.
#[derive(Default)]
pub struct TileCache {
    map: BTreeMap<TileKey, (Weak<Tile>, Arc<[u8]>)>,
    /// Tiles compressed by the last save (for tests and statistics).
    pub last_compressed: usize,
}

impl TileCache {
    fn encode(
        &mut self,
        layer: LayerId,
        c: (i32, i32),
        tile: &Arc<Tile>,
    ) -> Result<Arc<[u8]>, BrshError> {
        let key = (layer.0, c.0, c.1);
        if let Some((w, bytes)) = self.map.get(&key) {
            if w.upgrade().is_some_and(|t| Arc::ptr_eq(&t, tile)) {
                return Ok(bytes.clone());
            }
        }
        let bytes: Arc<[u8]> = zstd::bulk::compress(&tile.px, ZSTD_LEVEL)
            .map_err(err)?
            .into();
        self.last_compressed += 1;
        self.map.insert(key, (Arc::downgrade(tile), bytes.clone()));
        Ok(bytes)
    }
}

/// A document read from a `.brsh` file.
#[derive(Debug)]
pub struct Loaded {
    pub doc: Document,
    pub active: Option<LayerId>,
    pub journal: Option<String>,
    /// Format version the file was written with (before migration).
    pub file_version: u32,
    /// App that wrote the file.
    pub app: String,
    /// Non-fatal problems (unknown entries or keys); errors when loading strictly.
    pub warnings: Vec<String>,
}

fn color_text(c: Rgba8) -> String {
    let [r, g, b, a] = c.0;
    if a == 255 {
        format!("#{r:02x}{g:02x}{b:02x}")
    } else {
        format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
    }
}

fn parse_color(s: &str) -> Option<Rgba8> {
    let h = s.strip_prefix('#')?;
    if !h.is_ascii() {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(h.get(i..i + 2)?, 16).ok();
    match h.len() {
        6 => Some(Rgba8([byte(0)?, byte(2)?, byte(4)?, 255])),
        8 => Some(Rgba8([byte(0)?, byte(2)?, byte(4)?, byte(6)?])),
        _ => None,
    }
}

fn layer_dir(id: LayerId) -> String {
    format!("layers/{id}")
}

/// `document.json` for a document.
pub fn document_json(doc: &Document, active: Option<LayerId>) -> Json {
    json!({
        "width": doc.width,
        "height": doc.height,
        "dpi": doc.dpi,
        "paper": doc.paper.map(color_text),
        "active": active.map(|a| a.to_string()),
        "layers": doc.layers().iter().map(|l| json!({
            "id": l.id.to_string(),
            "name": l.name,
            "kind": l.kind.name(),
            "visible": l.visible,
            "opacity": l.opacity,
            "blend": l.blend.name(),
            "locked": l.locked,
            "lock_alpha": l.lock_alpha,
        })).collect::<Vec<_>>(),
    })
}

/// Downscale to fit `max` px (area average, alpha-weighted; integer arithmetic).
pub fn thumbnail(img: &Image, max: u32) -> Image {
    let (w, h) = (img.width.max(1), img.height.max(1));
    let scale = |v: u32| ((u64::from(v) * u64::from(max)) / u64::from(w.max(h))).max(1) as u32;
    let (tw, th) = if w.max(h) <= max {
        (w, h)
    } else {
        (scale(w), scale(h))
    };
    let mut px = Vec::with_capacity(tw as usize * th as usize * 4);
    for ty in 0..th {
        let (y0, y1) = (ty * h / th, ((ty + 1) * h / th).max(ty * h / th + 1));
        for tx in 0..tw {
            let (x0, x1) = (tx * w / tw, ((tx + 1) * w / tw).max(tx * w / tw + 1));
            let (mut c, mut a, mut n) = ([0u64; 3], 0u64, 0u64);
            for y in y0..y1 {
                for x in x0..x1 {
                    let p = img.pixel(x, y);
                    let pa = u64::from(p[3]);
                    for i in 0..3 {
                        c[i] += u64::from(p[i]) * pa;
                    }
                    a += pa;
                    n += 1;
                }
            }
            let out_a = (a + n / 2) / n;
            let col = |v: u64| (v + a / 2).checked_div(a).unwrap_or(0) as u8;
            px.extend_from_slice(&[col(c[0]), col(c[1]), col(c[2]), out_a as u8]);
        }
    }
    Image {
        width: tw,
        height: th,
        px,
    }
}

/// Encode a document as `.brsh` bytes.
pub fn save(
    doc: &Document,
    opts: &SaveOptions<'_>,
    cache: &mut TileCache,
) -> Result<Vec<u8>, BrshError> {
    cache.last_compressed = 0;
    let mut zw = ZipWriter::new(Cursor::new(Vec::new()));
    let ts = DateTime::default();
    let deflated = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(6))
        .last_modified_time(ts)
        .unix_permissions(0o644);
    let stored = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .last_modified_time(ts)
        .unix_permissions(0o644);
    let put =
        |zw: &mut ZipWriter<Cursor<Vec<u8>>>, name: &str, o: SimpleFileOptions, data: &[u8]| {
            zw.start_file(name, o).map_err(err)?;
            zw.write_all(data).map_err(err)
        };
    let pretty = |j: &Json| serde_json::to_vec_pretty(j).map_err(err);
    let manifest = json!({
        "format": FORMAT_ID,
        "version": FORMAT_VERSION,
        "app": env!("CARGO_PKG_VERSION"),
        "features": if opts.journal.is_some() { vec!["journal"] } else { vec![] },
    });
    put(&mut zw, "manifest.json", deflated, &pretty(&manifest)?)?;
    put(
        &mut zw,
        "document.json",
        deflated,
        &pretty(&document_json(doc, opts.active))?,
    )?;
    for l in doc.layers() {
        let dir = layer_dir(l.id);
        let meta = json!({ "format": "rgba8", "tile_size": TILE_SIZE });
        put(
            &mut zw,
            &format!("{dir}/layer.json"),
            deflated,
            &pretty(&meta)?,
        )?;
        for (&(tx, ty), tile) in l.tiles.iter() {
            if tile.is_transparent() {
                continue; // empty tiles are omitted (D-014)
            }
            let bytes = cache.encode(l.id, (tx, ty), tile)?;
            put(
                &mut zw,
                &format!("{dir}/tiles/{tx}_{ty}.zst"),
                stored,
                &bytes,
            )?;
        }
    }
    let img = flatten(doc);
    let preview = png_io::png_bytes(&img).map_err(err)?;
    put(&mut zw, "preview.png", stored, &preview)?;
    let thumb = png_io::png_bytes(&thumbnail(&img, THUMBNAIL_MAX)).map_err(err)?;
    put(&mut zw, "thumbnail.png", stored, &thumb)?;
    if let Some(j) = opts.journal {
        put(&mut zw, "journal.bcmd", deflated, j.as_bytes())?;
    }
    Ok(zw.finish().map_err(err)?.into_inner())
}

/// Write `bytes` to `path` crash-safely: a temporary file in the same folder, flushed to
/// disk, then renamed over the old file (D-014 3).
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "document".into());
    let tmp = dir.join(format!(".{name}.{}.tmp", std::process::id()));
    let result = (|| {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, path)?;
        // Persist the rename itself (Unix; directories can't be opened this way on Windows).
        #[cfg(unix)]
        std::fs::File::open(dir)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// Save a document to a file (crash-safe).
pub fn save_file(
    path: &Path,
    doc: &Document,
    opts: &SaveOptions<'_>,
    cache: &mut TileCache,
) -> Result<(), BrshError> {
    let bytes = save(doc, opts, cache)?;
    write_atomic(path, &bytes).map_err(|e| err(format!("{}: {e}", path.display())))
}

/// All entries of an archive, by name.
pub type Entries = BTreeMap<String, Vec<u8>>;

/// A migration from format version `n` to `n + 1`, applied to the raw entries.
pub type Migration = (u32, fn(&mut Entries) -> Result<(), String>);

/// All format migrations (none yet: every file is version 1).
pub const MIGRATIONS: &[Migration] = &[];

fn read_entries(bytes: &[u8]) -> Result<Entries, BrshError> {
    let mut za = ZipArchive::new(Cursor::new(bytes)).map_err(err)?;
    let mut out = Entries::new();
    for i in 0..za.len() {
        let f = za.by_index(i).map_err(err)?;
        if f.is_dir() {
            continue;
        }
        let name = f.name().to_owned();
        let limit = if name.ends_with(".zst") || name.ends_with(".png") {
            // Stored entries; size is bounded by the archive itself.
            bytes.len() as u64
        } else {
            MAX_TEXT_ENTRY
        };
        let mut data = Vec::new();
        f.take(limit + 1).read_to_end(&mut data).map_err(err)?;
        if data.len() as u64 > limit {
            return Err(err(format!("{name}: entry too large")));
        }
        if out.insert(name.clone(), data).is_some() {
            return Err(err(format!("duplicate entry {name}")));
        }
    }
    Ok(out)
}

fn json_entry(e: &Entries, name: &str) -> Result<Map<String, Json>, BrshError> {
    let data = e.get(name).ok_or_else(|| err(format!("missing {name}")))?;
    match serde_json::from_slice(data).map_err(|x| err(format!("{name}: {x}")))? {
        Json::Object(m) => Ok(m),
        _ => Err(err(format!("{name}: expected a JSON object"))),
    }
}

/// Reads typed fields from a JSON object and reports unknown keys.
struct Fields<'a> {
    what: String,
    m: &'a Map<String, Json>,
    used: Vec<&'static str>,
}

impl<'a> Fields<'a> {
    fn new(what: impl Into<String>, m: &'a Map<String, Json>) -> Self {
        Self {
            what: what.into(),
            m,
            used: Vec::new(),
        }
    }
    fn get(&mut self, k: &'static str) -> Option<&'a Json> {
        self.used.push(k);
        self.m.get(k).filter(|v| !v.is_null())
    }
    fn req(&mut self, k: &'static str) -> Result<&'a Json, BrshError> {
        let what = self.what.clone();
        self.get(k)
            .ok_or_else(|| err(format!("{what}: missing \"{k}\"")))
    }
    fn u32(&mut self, k: &'static str) -> Result<u32, BrshError> {
        let what = self.what.clone();
        self.req(k)?
            .as_u64()
            .and_then(|v| u32::try_from(v).ok())
            .ok_or_else(|| err(format!("{what}: \"{k}\" must be a u32")))
    }
    fn bool(&mut self, k: &'static str) -> Result<bool, BrshError> {
        let what = self.what.clone();
        self.req(k)?
            .as_bool()
            .ok_or_else(|| err(format!("{what}: \"{k}\" must be true or false")))
    }
    fn str(&mut self, k: &'static str) -> Result<&'a str, BrshError> {
        let what = self.what.clone();
        self.req(k)?
            .as_str()
            .ok_or_else(|| err(format!("{what}: \"{k}\" must be a string")))
    }
    fn unknown(&self, warnings: &mut Vec<String>) {
        for k in self.m.keys() {
            if !self.used.contains(&k.as_str()) {
                warnings.push(format!("{}: unknown key \"{k}\"", self.what));
            }
        }
    }
}

fn parse_layer_id(s: &str) -> Option<LayerId> {
    let n = s.strip_prefix(LayerId::PREFIX)?;
    if n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit()) || (n.len() > 1 && n.starts_with('0'))
    {
        return None;
    }
    n.parse().ok().map(LayerId)
}

/// Bring raw entries from version `from` to the current version (D-014 6).
pub fn migrate_entries(
    table: &[Migration],
    e: &mut Entries,
    from: u32,
    to: u32,
) -> Result<(), BrshError> {
    let mut v = from;
    while v < to {
        let step = table
            .iter()
            .find(|m| m.0 == v)
            .ok_or_else(|| err(format!("no migration from format version {v}")))?;
        (step.1)(e).map_err(|m| err(format!("migrating from version {v}: {m}")))?;
        v += 1;
    }
    Ok(())
}

/// Decode a `.brsh` file. With `strict`, unknown entries or keys are errors (tests); otherwise
/// they are returned as warnings (the app).
pub fn load(bytes: &[u8], strict: bool) -> Result<Loaded, BrshError> {
    let mut e = read_entries(bytes)?;
    let mut warnings = Vec::new();

    let man = json_entry(&e, "manifest.json")?;
    let mut mf = Fields::new("manifest.json", &man);
    if mf.str("format")? != FORMAT_ID {
        return Err(err("not a Brushye document (manifest format id)"));
    }
    let version = mf.u32("version")?;
    let app = mf.str("app")?.to_owned();
    let features: Vec<String> = mf
        .get("features")
        .and_then(Json::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    mf.unknown(&mut warnings);
    if version > FORMAT_VERSION {
        return Err(err(format!(
            "this file is format version {version}, written by Brushye {app}; this Brushye reads up to version {FORMAT_VERSION}. Update Brushye to open it."
        )));
    }
    if version == 0 {
        return Err(err("invalid format version 0"));
    }
    migrate_entries(MIGRATIONS, &mut e, version, FORMAT_VERSION)?;
    for f in &features {
        if f != "journal" {
            warnings.push(format!("manifest.json: unknown feature \"{f}\""));
        }
    }

    let dj = json_entry(&e, "document.json")?;
    let mut df = Fields::new("document.json", &dj);
    let (width, height, dpi) = (df.u32("width")?, df.u32("height")?, df.u32("dpi")?);
    let paper = match df.get("paper") {
        None => None,
        Some(Json::String(s)) => Some(
            parse_color(s).ok_or_else(|| err(format!("document.json: bad paper colour {s:?}")))?,
        ),
        Some(_) => {
            return Err(err(
                "document.json: \"paper\" must be a colour string or null",
            ));
        }
    };
    let active = match df.get("active") {
        None => None,
        Some(Json::String(s)) => Some(
            parse_layer_id(s)
                .ok_or_else(|| err(format!("document.json: bad active layer {s:?}")))?,
        ),
        Some(_) => return Err(err("document.json: \"active\" must be a layer id or null")),
    };
    let layers = df
        .req("layers")?
        .as_array()
        .ok_or_else(|| err("document.json: \"layers\" must be an array"))?;
    df.unknown(&mut warnings);

    let mut doc = Document::new(width, height, dpi, paper).map_err(err)?;
    let mut known: Vec<String> = vec![
        "manifest.json".into(),
        "document.json".into(),
        "preview.png".into(),
        "thumbnail.png".into(),
    ];
    let (tiles_x, tiles_y) = (
        width.div_ceil(TILE_SIZE as u32) as i32,
        height.div_ceil(TILE_SIZE as u32) as i32,
    );
    for (i, lj) in layers.iter().enumerate() {
        let lm = lj
            .as_object()
            .ok_or_else(|| err(format!("document.json: layer {i} must be an object")))?;
        let mut lf = Fields::new(format!("document.json layer {i}"), lm);
        let id_s = lf.str("id")?;
        let id = parse_layer_id(id_s).ok_or_else(|| err(format!("layer {i}: bad id {id_s:?}")))?;
        let kind_s = lf.str("kind")?;
        let kind = LayerKind::from_name(kind_s)
            .ok_or_else(|| err(format!("{id}: unknown layer kind {kind_s:?}")))?;
        let blend_s = lf.str("blend")?;
        let blend = BlendMode::from_name(blend_s)
            .ok_or_else(|| err(format!("{id}: unknown blend mode {blend_s:?}")))?;
        let opacity = lf.u32("opacity")?;
        if opacity > 100 {
            return Err(err(format!("{id}: opacity {opacity} out of range")));
        }
        let mut layer = Layer::new(id, lf.str("name")?, kind);
        layer.visible = lf.bool("visible")?;
        layer.opacity = opacity as u8;
        layer.blend = blend;
        layer.locked = lf.bool("locked")?;
        layer.lock_alpha = lf.bool("lock_alpha")?;
        lf.unknown(&mut warnings);

        let dir = layer_dir(id);
        let meta_name = format!("{dir}/layer.json");
        let meta = json_entry(&e, &meta_name)?;
        let mut mfl = Fields::new(meta_name.clone(), &meta);
        if mfl.str("format")? != "rgba8" || mfl.u32("tile_size")? != TILE_SIZE as u32 {
            return Err(err(format!(
                "{meta_name}: only rgba8 with {TILE_SIZE} px tiles is supported"
            )));
        }
        mfl.unknown(&mut warnings);
        known.push(meta_name);

        let prefix = format!("{dir}/tiles/");
        for (name, data) in e.range(prefix.clone()..) {
            let Some(rest) = name.strip_prefix(&prefix) else {
                break;
            };
            let coords = rest
                .strip_suffix(".zst")
                .and_then(|c| c.split_once('_'))
                .and_then(|(a, b)| Some((a.parse::<i32>().ok()?, b.parse::<i32>().ok()?)));
            let Some((tx, ty)) = coords else {
                continue; // reported as unknown below
            };
            if !(0..tiles_x).contains(&tx) || !(0..tiles_y).contains(&ty) {
                return Err(err(format!("{name}: tile outside the canvas")));
            }
            let px = zstd::bulk::decompress(data, TILE_BYTES)
                .map_err(|x| err(format!("{name}: {x}")))?;
            if px.len() != TILE_BYTES {
                return Err(err(format!(
                    "{name}: tile has {} bytes, expected {TILE_BYTES}",
                    px.len()
                )));
            }
            layer.tiles.get_mut((tx, ty)).px.copy_from_slice(&px);
            known.push(name.clone());
        }
        layer.tiles.prune();
        doc.add_layer(layer, None).map_err(err)?;
    }
    if let Some(a) = active {
        if doc.layer(a).is_none() {
            return Err(err(format!(
                "document.json: active layer {a} does not exist"
            )));
        }
    }
    let journal = match e.get("journal.bcmd") {
        Some(j) => {
            known.push("journal.bcmd".into());
            Some(String::from_utf8(j.clone()).map_err(|_| err("journal.bcmd is not UTF-8"))?)
        }
        None => None,
    };
    for name in e.keys() {
        if !known.contains(name) {
            warnings.push(format!("unknown entry {name}"));
        }
    }
    if strict && !warnings.is_empty() {
        return Err(err(warnings.join("; ")));
    }
    Ok(Loaded {
        doc,
        active,
        journal,
        file_version: version,
        app,
        warnings,
    })
}

/// Load a `.brsh` file from disk (warnings returned, not fatal).
pub fn load_file(path: &Path) -> Result<Loaded, BrshError> {
    let bytes = std::fs::read(path).map_err(|e| err(format!("{}: {e}", path.display())))?;
    load(&bytes, false)
}

/// Per-entry listing for `brushye dump`: (name, stored size, compression method).
pub fn list_entries(bytes: &[u8]) -> Result<Vec<(String, u64, u64, String)>, BrshError> {
    let mut za = ZipArchive::new(Cursor::new(bytes)).map_err(err)?;
    let mut out = Vec::new();
    for i in 0..za.len() {
        let f = za.by_index_raw(i).map_err(err)?;
        out.push((
            f.name().to_owned(),
            f.compressed_size(),
            f.size(),
            format!("{:?}", f.compression()),
        ));
    }
    Ok(out)
}
