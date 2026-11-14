//! `brushye` — headless Brushye (DECISIONS D-010/D-011/D-014).
//!
//! ```text
//! brushye run SCRIPT [--open FILE.brsh] [--journal OUT] [--png OUT] [--brsh OUT [--embed-journal]]
//! brushye open FILE.brsh [--query Q]... [--png OUT]
//! brushye replay JOURNAL [--base FILE.brsh] [--query Q]...
//! brushye render JOURNAL|FILE.brsh OUT.png
//! brushye dump FILE.brsh [--layers DIR]
//! brushye journal show JOURNAL|FILE.brsh [--json]
//! brushye brush new OUT.brshbrush NAME [key=value ...]
//! brushye brush show FILE.brshbrush
//! brushye brush list DIR
//! brushye brush import-sut FILE.sut [OUT_DIR]
//! brushye commands
//! brushye --version
//! ```
//! Exit status: 0 on success, 1 on a script/journal/file error (reported as `FILE:LINE: message`
//! where there is a line), 2 on usage errors.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::io::Read as _;
use std::path::Path;
use std::process::ExitCode;

use brushye_command::Session;
use brushye_command::brush_preset::BrushPreset;
use brushye_command::core::{Document, LayerId};
use brushye_format::brsh::{self, SaveOptions, TileCache};
use brushye_format::{brush_file, png_io};
use serde_json::json;

const USAGE: &str = "usage:
  brushye run SCRIPT [--open FILE.brsh] [--journal OUT] [--png OUT] [--brsh OUT [--embed-journal]]
                                          run a script (- = stdin); query results print as JSON lines
  brushye open FILE.brsh [--query Q]... [--png OUT]
                                          open a document, answer queries, export it
  brushye replay JOURNAL [--base FILE.brsh] [--query Q]...
                                          replay a journal (on its base document if it has one)
  brushye render JOURNAL|FILE.brsh OUT.png
                                          export the flattened image as PNG
  brushye dump FILE.brsh [--layers DIR]   show a document's manifest, layers and tiles;
                                          --layers writes each layer as DIR/<id>.png
  brushye journal show JOURNAL|FILE.brsh [--json]
                                          list a journal's commands (or a document's embedded
                                          journal): strokes summarised, undone steps marked
  brushye brush new OUT.brshbrush NAME [key=value ...]
                                          save a user brush (brush.stroke settings)
  brushye brush show FILE.brshbrush       print a brush and its brush.stroke settings
  brushye brush list DIR                  list a brush library folder
  brushye brush import-sut FILE.sut [OUT_DIR]
                                          import a Clip Studio Paint sub tool: print its
                                          settings and what couldn't be imported; with OUT_DIR,
                                          save each sub tool as OUT_DIR/<name>.brshbrush
  brushye commands [--markdown]           print the command/query registry (JSON, or the
                                          Markdown reference in docs/commands.md)
  brushye --version";

fn read_input(path: &str) -> Result<String, String> {
    if path == "-" {
        let mut s = String::new();
        std::io::stdin()
            .read_to_string(&mut s)
            .map_err(|e| format!("stdin: {e}"))?;
        Ok(s)
    } else {
        std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
    }
}

/// Flatten a document and write it as PNG.
fn export_png(doc: &Document, out: &str) -> Result<(), Fail> {
    let img = brushye_command::core::render::flatten(doc);
    let bytes = png_io::png_bytes(&img).map_err(|e| Fail::Run(e.to_string()))?;
    std::fs::write(out, bytes).map_err(|e| Fail::Run(format!("{out}: {e}")))
}

fn current(s: &Session) -> Result<std::sync::Arc<Document>, Fail> {
    s.document()
        .ok_or_else(|| Fail::Run("no document (run doc.new first)".into()))
}

fn open_brsh(path: &str) -> Result<brsh::Loaded, Fail> {
    let l = brsh::load_file(Path::new(path)).map_err(|e| Fail::Run(e.to_string()))?;
    for w in &l.warnings {
        eprintln!("warning: {path}: {w}");
    }
    Ok(l)
}

fn print_json(j: &serde_json::Value) {
    println!("{j}");
}

fn run_queries(s: &mut Session, queries: &[String]) -> Result<(), Fail> {
    for q in queries {
        if !q.trim_start().starts_with('?') {
            return Err(Fail::Usage(format!(
                "--query must be a query (starting with '?'): {q}\n{USAGE}"
            )));
        }
        match s.run_line(q) {
            Ok(Some(j)) => print_json(&j),
            Ok(None) => {}
            Err(e) => return Err(Fail::Run(format!("{q}: {e}"))),
        }
    }
    Ok(())
}

enum Fail {
    Usage(String),
    Run(String),
}

/// Parsed flags: positional arguments and `--flag value` / `--switch` options.
struct Args {
    pos: Vec<String>,
    opts: Vec<(String, Option<String>)>,
}

impl Args {
    fn parse(args: &[String], with_value: &[&str], switches: &[&str]) -> Result<Self, Fail> {
        let usage = |m: String| Fail::Usage(format!("{m}\n{USAGE}"));
        let (mut pos, mut opts) = (Vec::new(), Vec::new());
        let mut it = args.iter();
        while let Some(a) = it.next() {
            if with_value.contains(&a.as_str()) {
                let v = it
                    .next()
                    .ok_or_else(|| usage(format!("{a} needs a value")))?;
                opts.push((a.clone(), Some(v.clone())));
            } else if switches.contains(&a.as_str()) {
                opts.push((a.clone(), None));
            } else if a.starts_with("--") {
                return Err(usage(format!("unknown option '{a}'")));
            } else {
                pos.push(a.clone());
            }
        }
        Ok(Self { pos, opts })
    }
    fn one(&self, k: &str) -> Option<&str> {
        self.opts
            .iter()
            .rev()
            .find(|(n, _)| n == k)
            .and_then(|(_, v)| v.as_deref())
    }
    fn all(&self, k: &str) -> Vec<String> {
        self.opts
            .iter()
            .filter(|(n, _)| n == k)
            .filter_map(|(_, v)| v.clone())
            .collect()
    }
    fn has(&self, k: &str) -> bool {
        self.opts.iter().any(|(n, _)| n == k)
    }
    fn exactly(&self, n: usize, what: &str) -> Result<(), Fail> {
        if self.pos.len() == n {
            Ok(())
        } else {
            Err(Fail::Usage(format!("{what}\n{USAGE}")))
        }
    }
}

fn save_brsh(s: &Session, out: &str, embed_journal: bool) -> Result<(), Fail> {
    let doc = current(s)?;
    let journal = s.journal_text();
    let opts = SaveOptions {
        active: s.active_layer(),
        journal: embed_journal.then_some(journal.as_str()),
    };
    brsh::save_file(Path::new(out), &doc, &opts, &mut TileCache::default())
        .map_err(|e| Fail::Run(e.to_string()))
}

fn cmd_run(args: &[String]) -> Result<(), Fail> {
    let a = Args::parse(
        args,
        &["--open", "--journal", "--png", "--brsh"],
        &["--embed-journal"],
    )?;
    a.exactly(1, "run needs exactly one SCRIPT")?;
    let path = &a.pos[0];
    let text = read_input(path).map_err(Fail::Run)?;
    let mut s = match a.one("--open") {
        Some(f) => {
            let l = open_brsh(f)?;
            Session::from_document(l.doc, l.active)
        }
        None => Session::new(),
    };
    for (i, line) in text.lines().enumerate() {
        match s.run_line(line) {
            Ok(Some(j)) => print_json(&j),
            Ok(None) => {}
            Err(e) => return Err(Fail::Run(format!("{path}:{}: {e}", i + 1))),
        }
    }
    if let Some(out) = a.one("--journal") {
        std::fs::write(out, s.journal_text()).map_err(|e| Fail::Run(format!("{out}: {e}")))?;
    }
    if let Some(out) = a.one("--png") {
        export_png(&*current(&s)?, out)?;
    }
    if let Some(out) = a.one("--brsh") {
        save_brsh(&s, out, a.has("--embed-journal"))?;
    } else if a.has("--embed-journal") {
        return Err(Fail::Usage(format!(
            "--embed-journal needs --brsh\n{USAGE}"
        )));
    }
    Ok(())
}

fn cmd_open(args: &[String]) -> Result<(), Fail> {
    let a = Args::parse(args, &["--query", "--png"], &[])?;
    a.exactly(1, "open needs exactly one FILE.brsh")?;
    let l = open_brsh(&a.pos[0])?;
    let mut s = Session::from_document(l.doc, l.active);
    run_queries(&mut s, &a.all("--query"))?;
    if let Some(out) = a.one("--png") {
        export_png(&*current(&s)?, out)?;
    }
    Ok(())
}

fn cmd_replay(args: &[String]) -> Result<(), Fail> {
    let a = Args::parse(args, &["--query", "--base"], &[])?;
    a.exactly(1, "replay needs exactly one JOURNAL")?;
    let path = &a.pos[0];
    let text = read_input(path).map_err(Fail::Run)?;
    let replayed = match a.one("--base") {
        Some(b) => {
            let l = open_brsh(b)?;
            Session::replay_on(l.doc, l.active, &text)
        }
        None => Session::replay(&text),
    };
    let mut s = replayed.map_err(|e| Fail::Run(format!("{path}:{}: {}", e.line, e.msg)))?;
    run_queries(&mut s, &a.all("--query"))
}

fn cmd_render(args: &[String]) -> Result<(), Fail> {
    let a = Args::parse(args, &[], &[])?;
    a.exactly(
        2,
        "render needs an input (JOURNAL or FILE.brsh) and OUT.png",
    )?;
    let (input, out) = (&a.pos[0], &a.pos[1]);
    let doc = if input.ends_with(".brsh") {
        open_brsh(input)?.doc
    } else {
        let text = read_input(input).map_err(Fail::Run)?;
        let s = Session::replay(&text)
            .map_err(|e| Fail::Run(format!("{input}:{}: {}", e.line, e.msg)))?;
        Document::clone(&*current(&s)?)
    };
    export_png(&doc, out)
}

fn cmd_dump(args: &[String]) -> Result<(), Fail> {
    let a = Args::parse(args, &["--layers"], &[])?;
    a.exactly(1, "dump needs exactly one FILE.brsh")?;
    let path = &a.pos[0];
    let bytes = std::fs::read(path).map_err(|e| Fail::Run(format!("{path}: {e}")))?;
    let l = brsh::load(&bytes, false).map_err(|e| Fail::Run(e.to_string()))?;
    let entries = brsh::list_entries(&bytes).map_err(|e| Fail::Run(e.to_string()))?;
    let layers: Vec<_> = l
        .doc
        .layers()
        .iter()
        .map(|layer| {
            let prefix = format!("layers/{}/tiles/", layer.id);
            let (tiles, stored, raw) = entries
                .iter()
                .filter(|e| e.0.starts_with(&prefix))
                .fold((0u64, 0u64, 0u64), |(n, s, r), e| (n + 1, s + e.1, r + e.2));
            json!({
                "id": layer.id.to_string(), "name": layer.name, "kind": layer.kind.name(),
                "visible": layer.visible, "opacity": layer.opacity, "blend": layer.blend.name(),
                "locked": layer.locked, "lock_alpha": layer.lock_alpha,
                "parent": layer.parent.map(|p| p.to_string()), "clip": layer.clip,
                "tiles": tiles, "tile_bytes": stored, "pixel_bytes": tiles * 256 * 256 * 4,
                "zip_raw_bytes": raw,
            })
        })
        .collect();
    let out = json!({
        "file": path,
        "format_version": l.file_version,
        "app": l.app,
        "width": l.doc.width, "height": l.doc.height, "dpi": l.doc.dpi,
        "paper": l.doc.paper.map(|p| format!("#{:02x}{:02x}{:02x}{:02x}", p.0[0], p.0[1], p.0[2], p.0[3])),
        "active": l.active.map(|x| x.to_string()),
        "hash": format!("{:016x}", l.doc.content_hash()),
        "journal_lines": l.journal.as_ref().map(|j| j.lines().count()),
        "layers": layers,
        "entries": entries.iter().map(|e| json!({"name": e.0, "stored": e.1, "size": e.2, "method": e.3})).collect::<Vec<_>>(),
        "warnings": l.warnings,
    });
    println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
    if let Some(dir) = a.one("--layers") {
        std::fs::create_dir_all(dir).map_err(|e| Fail::Run(format!("{dir}: {e}")))?;
        for layer in l.doc.layers() {
            let only = layer_image(&l.doc, layer.id)?;
            let out = Path::new(dir).join(format!("{}.png", layer.id));
            std::fs::write(
                &out,
                png_io::png_bytes(&only).map_err(|e| Fail::Run(e.to_string()))?,
            )
            .map_err(|e| Fail::Run(format!("{}: {e}", out.display())))?;
        }
    }
    Ok(())
}

/// One layer's own pixels as an image (straight RGBA, no paper, no blending).
fn layer_image(doc: &Document, id: LayerId) -> Result<brushye_command::core::render::Image, Fail> {
    let layer = doc
        .layer(id)
        .ok_or_else(|| Fail::Run(format!("no layer {id}")))?;
    let (w, h) = (doc.width, doc.height);
    let mut px = Vec::with_capacity(w as usize * h as usize * 4);
    for y in 0..h {
        for x in 0..w {
            px.extend_from_slice(&layer.pixel(x, y));
        }
    }
    Ok(brushye_command::core::render::Image {
        width: w,
        height: h,
        px,
    })
}

fn cmd_brush(args: &[String]) -> Result<(), Fail> {
    let usage = |m: &str| Fail::Usage(format!("{m}\n{USAGE}"));
    match args.first().map(String::as_str) {
        Some("new") => {
            let (Some(out), Some(name)) = (args.get(1), args.get(2)) else {
                return Err(usage("brush new needs OUT.brshbrush and NAME"));
            };
            let preset = BrushPreset::from_text(&args[3..].join(" "))
                .map_err(|e| Fail::Run(format!("brush settings: {e}")))?;
            let b = brush_file::BrushFile {
                name: name.clone(),
                settings: preset.to_json(),
                images: Default::default(),
            };
            let bytes = brush_file::save(&b).map_err(|e| Fail::Run(e.to_string()))?;
            brsh::write_atomic(Path::new(out), &bytes).map_err(|e| Fail::Run(format!("{out}: {e}")))
        }
        Some("show") => {
            let Some(path) = args.get(1).filter(|_| args.len() == 2) else {
                return Err(usage("brush show needs one FILE.brshbrush"));
            };
            let bytes = std::fs::read(path).map_err(|e| Fail::Run(format!("{path}: {e}")))?;
            let (b, warnings) =
                brush_file::load(&bytes, false).map_err(|e| Fail::Run(e.to_string()))?;
            let preset = BrushPreset::from_json(&b.settings)
                .map_err(|e| Fail::Run(format!("{path}: settings: {e}")))?;
            let out = json!({
                "name": b.name,
                "settings": b.settings,
                "stroke_args": preset.to_text(),
                "images": b.images.keys().collect::<Vec<_>>(),
                "warnings": warnings,
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
            Ok(())
        }
        Some("list") => {
            let Some(dir) = args.get(1).filter(|_| args.len() == 2) else {
                return Err(usage("brush list needs one DIR"));
            };
            let lib = brush_file::scan_library(Path::new(dir))
                .map_err(|e| Fail::Run(format!("{dir}: {e}")))?;
            for e in lib {
                let line = match &e.brush {
                    Ok(b) => match BrushPreset::from_json(&b.settings) {
                        Ok(p) => {
                            json!({"file": e.path, "name": b.name, "stroke_args": p.to_text()})
                        }
                        Err(x) => json!({"file": e.path, "name": b.name, "error": x}),
                    },
                    Err(x) => json!({"file": e.path, "error": x.to_string()}),
                };
                print_json(&line);
            }
            Ok(())
        }
        Some("import-sut") => {
            let (Some(path), out_dir) = (args.get(1), args.get(2)) else {
                return Err(usage("brush import-sut needs FILE.sut"));
            };
            if args.len() > 3 {
                return Err(usage(
                    "brush import-sut takes FILE.sut and an optional OUT_DIR",
                ));
            }
            let bytes = std::fs::read(path).map_err(|e| Fail::Run(format!("{path}: {e}")))?;
            let subs =
                brushye_sut::import(&bytes).map_err(|e| Fail::Run(format!("{path}: {e}")))?;
            for sub in subs {
                let mut file = None;
                if let Some(dir) = out_dir {
                    let safe: String = sub
                        .name
                        .chars()
                        .map(|c| {
                            if c.is_alphanumeric() || " -_.".contains(c) {
                                c
                            } else {
                                '_'
                            }
                        })
                        .collect();
                    let out = Path::new(dir).join(format!("{}.brshbrush", safe.trim()));
                    let b = brush_file::BrushFile {
                        name: sub.name.clone(),
                        settings: sub.preset.to_json(),
                        images: Default::default(),
                    };
                    let bytes = brush_file::save(&b).map_err(|e| Fail::Run(e.to_string()))?;
                    brsh::write_atomic(&out, &bytes)
                        .map_err(|e| Fail::Run(format!("{}: {e}", out.display())))?;
                    file = Some(out.display().to_string());
                }
                print_json(&json!({
                    "name": sub.name,
                    "stroke_args": sub.preset.to_text(),
                    "notes": sub.notes.iter().map(ToString::to_string).collect::<Vec<_>>(),
                    "file": file,
                }));
            }
            Ok(())
        }
        _ => Err(usage("brush needs new, show, list or import-sut")),
    }
}

fn cmd_journal(args: &[String]) -> Result<(), Fail> {
    if args.first().map(String::as_str) != Some("show") {
        return Err(Fail::Usage(format!("journal needs 'show'\n{USAGE}")));
    }
    let a = Args::parse(&args[1..], &[], &["--json"])?;
    a.exactly(1, "journal show needs exactly one JOURNAL or FILE.brsh")?;
    let path = &a.pos[0];
    let text = if path.ends_with(".brsh") {
        open_brsh(path)?
            .journal
            .ok_or_else(|| Fail::Run(format!("{path}: no embedded journal")))?
    } else {
        read_input(path).map_err(Fail::Run)?
    };
    let v = brushye_command::journal::view(&text);
    if a.has("--json") {
        println!(
            "{}",
            serde_json::to_string_pretty(&v.to_json()).unwrap_or_default()
        );
    } else {
        if let Some(h) = &v.header {
            println!("{h}");
        }
        for e in &v.entries {
            let mark = if e.undone { "  (undone)" } else { "" };
            println!("{:>5}  {}{mark}", e.line, e.text);
        }
        let undone = v.entries.iter().filter(|e| e.undone).count();
        let strokes = v.entries.iter().filter(|e| e.stroke.is_some()).count();
        println!(
            "-- {} commands, {strokes} strokes, {undone} undone at the end",
            v.entries.len()
        );
    }
    for (line, msg) in &v.errors {
        eprintln!("{path}:{line}: {msg}");
    }
    if v.errors.is_empty() {
        Ok(())
    } else {
        Err(Fail::Run(format!(
            "{path}: {} unreadable lines",
            v.errors.len()
        )))
    }
}

fn real_main(args: &[String]) -> Result<(), Fail> {
    let rest = args.get(1..).unwrap_or(&[]);
    match args.first().map(String::as_str) {
        Some("--version" | "-V") => {
            println!("brushye {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("--help" | "-h") | None => {
            println!("{USAGE}");
            Ok(())
        }
        Some("commands") if args.get(1).map(String::as_str) == Some("--markdown") => {
            print!("{}", brushye_command::commands::reference_markdown());
            Ok(())
        }
        Some("commands") => {
            let mut s = Session::new();
            match s.run_line("?commands") {
                Ok(Some(j)) => {
                    println!("{}", serde_json::to_string_pretty(&j).unwrap_or_default());
                    Ok(())
                }
                other => Err(Fail::Run(format!("{other:?}"))),
            }
        }
        Some("run") => cmd_run(rest),
        Some("open") => cmd_open(rest),
        Some("replay") => cmd_replay(rest),
        Some("render") => cmd_render(rest),
        Some("dump") => cmd_dump(rest),
        Some("brush") => cmd_brush(rest),
        Some("journal") => cmd_journal(rest),
        Some(other) => Err(Fail::Usage(format!(
            "unknown subcommand '{other}'\n{USAGE}"
        ))),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match real_main(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(Fail::Usage(m)) => {
            eprintln!("{m}");
            ExitCode::from(2)
        }
        Err(Fail::Run(m)) => {
            eprintln!("error: {m}");
            ExitCode::FAILURE
        }
    }
}
