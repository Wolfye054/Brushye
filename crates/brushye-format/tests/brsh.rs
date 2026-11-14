//! `.brsh` save/load (DECISIONS D-014): round trips, determinism, the tile cache, versioning,
//! strictness, crash-safe writes and journals that continue an opened document.

use std::io::{Cursor, Write};

use brushye_command::Session;
use brushye_format::brsh::{self, Entries, SaveOptions, TileCache};
use serde_json::Value as Json;

fn run(s: &mut Session, script: &str) {
    if let Err(e) = s.run_script(script) {
        panic!("script failed: {e}");
    }
}

fn hash(s: &mut Session) -> String {
    match s.run_line("?doc.hash") {
        Ok(Some(Json::String(h))) => h,
        other => panic!("{other:?}"),
    }
}

const SCRIPT: &str = "\
doc.new 600 300 dpi=350 paper=#fdf6e3
layer.new L1 name=\"Base\"
layer.fill L1 color=#80a0c0
layer.new L2 name=\"Ink ✒ \\\"quoted\\\"\"
brush.stroke L2 color=#202020 size=14 size_pressure=10 samples=[20,20,0.2,0;200,150,1,8000;580,40,0.5,16000]
brush.stroke L2 color=#c02020 size=30 hardness=2 opacity=60 samples=[300,250,1,0]
layer.set L2 blend=multiply opacity=85
layer.new L3 name=\"Hidden\" above=L1
layer.pixels L3 x=510 y=280 w=2 h=1 data=b64:/wAA//8AAIA=
layer.show L3 visible=false
layer.set L1 lock_alpha=true locked=true
layer.select L2
";

fn session() -> Session {
    let mut s = Session::new();
    run(&mut s, SCRIPT);
    s
}

fn save(s: &Session, journal: bool) -> Vec<u8> {
    let doc = s.document().unwrap_or_else(|| panic!("no document"));
    let j = s.journal_text();
    let opts = SaveOptions {
        active: s.active_layer(),
        journal: journal.then_some(j.as_str()),
    };
    brsh::save(&doc, &opts, &mut TileCache::default()).unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn save_load_replay_gives_the_same_document() {
    let mut s = session();
    let h = hash(&mut s);
    let bytes = save(&s, true);
    let l = brsh::load(&bytes, true).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(format!("{:016x}", l.doc.content_hash()), h);
    assert_eq!(l.active, s.active_layer());
    assert_eq!(l.file_version, brsh::FORMAT_VERSION);
    assert!(l.warnings.is_empty());
    // The embedded journal replays to the same document (M5 exit criterion).
    let j = l.journal.unwrap_or_else(|| panic!("journal missing"));
    assert_eq!(j, s.journal_text());
    let mut r = Session::replay(&j).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(hash(&mut r), h);
    // Without the opt-in there is no journal.
    let l = brsh::load(&save(&s, false), true).unwrap_or_else(|e| panic!("{e}"));
    assert!(l.journal.is_none());
}

#[test]
fn output_is_deterministic_and_well_formed() {
    let s = session();
    let a = save(&s, false);
    assert_eq!(a, save(&s, false));
    let names: Vec<String> = brsh::list_entries(&a)
        .unwrap_or_else(|e| panic!("{e}"))
        .into_iter()
        .map(|e| e.0)
        .collect();
    assert_eq!(names[0], "manifest.json");
    assert_eq!(names[1], "document.json");
    assert!(names.contains(&"layers/L2/layer.json".to_owned()));
    assert!(names.contains(&"layers/L2/tiles/0_0.zst".to_owned()));
    // L3 holds 2 px in tile (1, 1) only; empty tiles are omitted.
    assert!(names.contains(&"layers/L3/tiles/1_1.zst".to_owned()));
    assert!(!names.contains(&"layers/L3/tiles/0_0.zst".to_owned()));
    assert!(names.contains(&"preview.png".to_owned()));
    let entries = brsh::list_entries(&a).unwrap_or_else(|e| panic!("{e}"));
    for (name, _, _, method) in entries {
        if name.ends_with(".zst") || name.ends_with(".png") {
            assert_eq!(method, "Stored", "{name}: no double compression");
        }
    }
}

#[test]
fn preview_and_thumbnail_match_the_renderer() {
    let s = session();
    let doc = s.document().unwrap_or_else(|| panic!("no document"));
    let bytes = save(&s, false);
    let e = entries(&bytes);
    let preview =
        brushye_format::png_io::read_png(&e["preview.png"]).unwrap_or_else(|x| panic!("{x}"));
    assert_eq!(preview, brushye_core::render::flatten(&doc));
    let thumb =
        brushye_format::png_io::read_png(&e["thumbnail.png"]).unwrap_or_else(|x| panic!("{x}"));
    assert_eq!((thumb.width, thumb.height), (256, 128));
}

#[test]
fn tile_cache_recompresses_only_changed_tiles() {
    let mut s = Session::new();
    run(
        &mut s,
        "doc.new 768 512\nlayer.new L1\nlayer.fill L1 color=#336699\nlayer.new L2\nlayer.fill L2 color=#ffffff80",
    );
    let mut cache = TileCache::default();
    let opts = SaveOptions::default();
    let doc = s.document().unwrap_or_else(|| panic!("no document"));
    let first = brsh::save(&doc, &opts, &mut cache).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(cache.last_compressed, 12);
    // A small stroke inside tile (1, 0) of L2 changes one tile.
    run(
        &mut s,
        "brush.stroke L2 color=#000000 size=10 samples=[300,100,1,0]",
    );
    let doc = s.document().unwrap_or_else(|| panic!("no document"));
    let second = brsh::save(&doc, &opts, &mut cache).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(cache.last_compressed, 1);
    // A cache never changes the bytes.
    let fresh =
        brsh::save(&doc, &opts, &mut TileCache::default()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(second, fresh);
    assert_ne!(first, second);
}

fn entries(bytes: &[u8]) -> Entries {
    let mut za = zip::ZipArchive::new(Cursor::new(bytes)).unwrap_or_else(|e| panic!("{e}"));
    let mut out = Entries::new();
    for i in 0..za.len() {
        let mut f = za.by_index(i).unwrap_or_else(|e| panic!("{e}"));
        let mut v = Vec::new();
        std::io::Read::read_to_end(&mut f, &mut v).unwrap_or_else(|e| panic!("{e}"));
        out.insert(f.name().to_owned(), v);
    }
    out
}

fn repack(e: &Entries) -> Vec<u8> {
    let mut zw = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in e {
        zw.start_file(name.as_str(), zip::write::SimpleFileOptions::default())
            .unwrap_or_else(|x| panic!("{x}"));
        zw.write_all(data).unwrap_or_else(|x| panic!("{x}"));
    }
    zw.finish().unwrap_or_else(|x| panic!("{x}")).into_inner()
}

fn edit_json(e: &mut Entries, name: &str, f: impl FnOnce(&mut serde_json::Map<String, Json>)) {
    let mut j: Json = serde_json::from_slice(&e[name]).unwrap_or_else(|x| panic!("{x}"));
    if let Json::Object(m) = &mut j {
        f(m);
    }
    e.insert(
        name.into(),
        serde_json::to_vec(&j).unwrap_or_else(|x| panic!("{x}")),
    );
}

fn load_err(e: &Entries, strict: bool) -> String {
    match brsh::load(&repack(e), strict) {
        Ok(_) => panic!("expected an error"),
        Err(x) => x.to_string(),
    }
}

#[test]
fn newer_versions_are_refused_and_bad_files_rejected() {
    let base = entries(&save(&session(), false));
    // Repacking with deflate everywhere still loads (other tools may recompress the ZIP).
    brsh::load(&repack(&base), true).unwrap_or_else(|e| panic!("{e}"));

    let mut e = base.clone();
    edit_json(&mut e, "manifest.json", |m| {
        m.insert("version".into(), Json::from(brsh::FORMAT_VERSION + 1));
    });
    let msg = load_err(&e, false);
    assert!(
        msg.contains("format version 2") && msg.contains("Update Brushye"),
        "{msg}"
    );

    let mut e = base.clone();
    edit_json(&mut e, "manifest.json", |m| {
        m.insert("format".into(), Json::from("something-else"));
    });
    assert!(load_err(&e, false).contains("not a Brushye document"));

    // Unknown entries/keys: errors when strict, warnings otherwise.
    let mut e = base.clone();
    e.insert("extra/notes.txt".into(), b"hi".to_vec());
    edit_json(&mut e, "document.json", |m| {
        m.insert("future_key".into(), Json::from(1));
    });
    assert!(load_err(&e, true).contains("unknown entry extra/notes.txt"));
    let l = brsh::load(&repack(&e), false).unwrap_or_else(|x| panic!("{x}"));
    assert_eq!(l.warnings.len(), 2, "{:?}", l.warnings);

    // A tile of the wrong size, or outside the canvas.
    let mut e = base.clone();
    let short = zstd::bulk::compress(&[0u8; 100], 3).unwrap_or_else(|x| panic!("{x}"));
    e.insert("layers/L2/tiles/0_0.zst".into(), short.clone());
    assert!(load_err(&e, false).contains("expected 262144"));
    let mut e = base.clone();
    let tile = e["layers/L2/tiles/0_0.zst"].clone();
    e.insert("layers/L2/tiles/9_0.zst".into(), tile);
    assert!(load_err(&e, false).contains("outside the canvas"));

    // Missing pieces and bad values.
    let mut e = base.clone();
    e.remove("layers/L1/layer.json");
    assert!(load_err(&e, false).contains("missing layers/L1/layer.json"));
    let mut e = base.clone();
    edit_json(&mut e, "document.json", |m| {
        m.insert("active".into(), Json::from("L99"));
    });
    assert!(load_err(&e, false).contains("active layer L99"));
    assert!(brsh::load(b"not a zip", false).is_err());
}

#[test]
fn migration_engine_runs_the_chain() {
    fn v1_to_v2(e: &mut Entries) -> Result<(), String> {
        e.insert("v2.txt".into(), b"2".to_vec());
        Ok(())
    }
    fn v2_to_v3(e: &mut Entries) -> Result<(), String> {
        let v = e.remove("v2.txt").ok_or("v2 marker missing")?;
        e.insert("v3.txt".into(), v);
        Ok(())
    }
    let table: &[brsh::Migration] = &[(1, v1_to_v2), (2, v2_to_v3)];
    let mut e = Entries::new();
    brsh::migrate_entries(table, &mut e, 1, 3).unwrap_or_else(|x| panic!("{x}"));
    assert_eq!(e.keys().collect::<Vec<_>>(), ["v3.txt"]);
    let mut e = Entries::new();
    assert!(brsh::migrate_entries(table, &mut e, 0, 3).is_err());
}

#[test]
fn atomic_write_replaces_the_file_and_leaves_no_temp_files() {
    let dir = std::env::temp_dir().join(format!("brushye-brsh-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
    let path = dir.join("doc.brsh");
    brsh::write_atomic(&path, b"old").unwrap_or_else(|e| panic!("{e}"));
    let s = session();
    let doc = s.document().unwrap_or_else(|| panic!("no document"));
    brsh::save_file(
        &path,
        &doc,
        &SaveOptions::default(),
        &mut TileCache::default(),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let l = brsh::load_file(&path).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(l.doc.content_hash(), doc.content_hash());
    let left: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{e}"))
        .filter_map(Result::ok)
        .map(|e| e.file_name())
        .collect();
    assert_eq!(left, vec![std::ffi::OsString::from("doc.brsh")]);
    std::fs::remove_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn journals_continue_an_opened_document() {
    let s = session();
    let l = brsh::load(&save(&s, false), true).unwrap_or_else(|e| panic!("{e}"));
    let base = l.doc.clone();
    let mut o = Session::from_document(l.doc, l.active);
    run(
        &mut o,
        "brush.stroke L2 color=#00ff00 size=8 samples=[10,290,1,0;590,10,1,9000]\nlayer.new L4\nhistory.undo",
    );
    let h = hash(&mut o);
    let j = o.journal_text();
    assert!(
        j.lines().next().is_some_and(|l| l.contains(" base=")),
        "{j}"
    );
    // Undo can't go past the opened document.
    run(&mut o, "history.undo\nhistory.undo");
    assert_eq!(
        o.document().map(|d| d.content_hash()),
        Some(base.content_hash())
    );
    // Replay needs the same base document.
    let mut r = Session::replay_on(base.clone(), l.active, &j).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(hash(&mut r), h);
    assert!(Session::replay(&j).is_err());
    let mut other = base.clone();
    other.dpi = 72;
    assert!(Session::replay_on(other, None, &j).is_err());
    // A doc.new journal can't be replayed on a base.
    assert!(Session::replay_on(base, None, &s.journal_text()).is_err());
}

mod roundtrip {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(24))]
        #[test]
        fn random_documents_round_trip(
            w in 1u32..700, h in 1u32..400,
            blocks in proptest::collection::vec((0u32..3, 0i64..700, 0i64..400, 1u32..40, 1u32..40, any::<u8>()), 0..8),
            paper in any::<bool>(),
        ) {
            let mut s = Session::new();
            let paper = if paper { "" } else { " paper=none" };
            run(&mut s, &format!("doc.new {w} {h}{paper}\nlayer.new L1\nlayer.new L2\nlayer.new L3 above=L1"));
            for (layer, x, y, bw, bh, seed) in blocks {
                let data: Vec<u8> = (0..bw * bh * 4).map(|i| (i as u8).wrapping_mul(seed | 1)).collect();
                let line = format!(
                    "layer.pixels L{} x={x} y={y} w={bw} h={bh} data=b64:{}",
                    layer + 1,
                    base64_encode(&data)
                );
                run(&mut s, &line);
            }
            let bytes = save(&s, false);
            let l = brsh::load(&bytes, true).map_err(|e| TestCaseError::fail(e.to_string()))?;
            let doc = s.document().unwrap_or_else(|| panic!("no document"));
            prop_assert_eq!(l.doc.content_hash(), doc.content_hash());
            prop_assert_eq!(&l.doc, &*doc);
        }
    }

    fn base64_encode(data: &[u8]) -> String {
        const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for c in data.chunks(3) {
            let n = (u32::from(c[0]) << 16)
                | (u32::from(*c.get(1).unwrap_or(&0)) << 8)
                | u32::from(*c.get(2).unwrap_or(&0));
            for i in 0..4 {
                if i <= c.len() {
                    out.push(T[(n >> (18 - 6 * i) & 63) as usize] as char);
                } else {
                    out.push('=');
                }
            }
        }
        out
    }
}

/// Golden files (D-014 6): files written by format version 1 must keep opening, unchanged.
mod golden {
    use super::*;
    use brushye_command::brush_preset::BrushPreset;
    use brushye_format::brush_file;

    fn data(f: &str) -> Vec<u8> {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data")
            .join(f);
        std::fs::read(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
    }

    #[test]
    fn golden_v1_document_opens_and_replays() {
        let l = brsh::load(&data("golden_v1.brsh"), true).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(l.file_version, 1);
        assert_eq!(format!("{:016x}", l.doc.content_hash()), "fe80e51a0cdadafa");
        assert_eq!(l.active.map(|a| a.to_string()), Some("L2".into()));
        let ids: Vec<String> = l.doc.layers().iter().map(|x| x.id.to_string()).collect();
        assert_eq!(ids, ["L1", "L3", "L2"]);
        assert!(!l.doc.layers()[1].visible);
        // Its embedded journal still replays to the same document. If this alone fails, the
        // rendering of a command changed: that needs a new command version (D-011).
        let mut r =
            Session::replay(&l.journal.unwrap_or_default()).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(hash(&mut r), "fe80e51a0cdadafa");
    }

    #[test]
    fn golden_v1_brush_opens() {
        let (b, w) =
            brush_file::load(&data("golden_v1.brshbrush"), true).unwrap_or_else(|e| panic!("{e}"));
        assert!(w.is_empty());
        assert_eq!(b.name, "Golden pen");
        let p = BrushPreset::from_json(&b.settings).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(
            p.to_text(),
            "size=14.0 hardness=4 gap=narrow thickness=60 angle=15.0 size_pressure=10 size_curve=[0.5,0.3]"
        );
    }
}
