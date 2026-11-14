//! Brushye MCP server core (M9). The tools here are thin clients of the command layer
//! (D-010, D-011): every state change goes through [`Session::run_line`] or
//! [`Session::execute`], exactly as the CLI and the app do. File tools (open, save, export,
//! replay) work like the matching `brushye` CLI subcommands.
//!
//! This module is synchronous and has no MCP types, so it is tested directly; `main.rs` wires
//! it to `rmcp` over stdio.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use brushye_command::core::brush::InkMode;
use brushye_command::core::render::{self, Image};
use brushye_command::core::{Document, LayerId};
use brushye_command::params::{Arg, Samples};
use brushye_command::tools::Tool;
use brushye_command::{Command, Session, commands};
use brushye_format::brsh::{self, SaveOptions, TileCache};
use brushye_format::png_io;
use serde_json::{Map, Value as Json, json};

/// The command text syntax, served by the `help` tool.
pub const COMMAND_FORMAT: &str = include_str!("../../../docs/command-format.md");

/// Instructions sent to the client when it connects.
pub const INSTRUCTIONS: &str = "Brushye is a painting app. This server holds one headless \
Brushye session: a document, its undo history and journal, and the tool state. Change it only \
with commands: `run` takes command and query lines in Brushye's text syntax (one per line; \
queries start with `?` and return JSON), and `command` takes one command in JSON form. Call \
`help` first for the syntax and the full command reference. `render` returns the image so you \
can look at it. Start with `doc.new W H` and `layer.new L2`; brush strokes are `brush.stroke` \
with samples `[x,y,pressure,t_us;…]`.";

/// One tool: name, description, JSON Schema of its arguments, and whether it only reads.
pub struct ToolDef {
    pub name: &'static str,
    pub description: String,
    pub schema: Json,
    pub read_only: bool,
}

/// What a tool call returns.
#[derive(Debug, PartialEq)]
pub enum Out {
    Text(String),
    /// PNG bytes.
    Png(Vec<u8>),
}

/// The server state: one session.
pub struct Server {
    session: Session,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

fn obj(props: Json, required: &[&str]) -> Json {
    json!({ "type": "object", "properties": props, "required": required, "additionalProperties": false })
}

/// The tool list (stable order).
pub fn tools() -> Vec<ToolDef> {
    let cmd_names: Vec<&str> = commands::COMMANDS.iter().map(|s| s.name).collect();
    let query_list = commands::QUERIES
        .iter()
        .map(|s| format!("?{}", s.name))
        .collect::<Vec<_>>()
        .join(", ");
    vec![
        ToolDef {
            name: "help",
            description: "The command text syntax and the full command and query reference \
                (parameters, kinds, defaults). Read this before using `run` or `command`."
                .into(),
            schema: obj(json!({}), &[]),
            read_only: true,
        },
        ToolDef {
            name: "run",
            description: format!(
                "Run command and query lines in Brushye's text syntax, one per line, in order; \
                 `#` starts a comment. Stops at the first failing line (earlier lines stay \
                 applied). Returns each query's JSON result. Queries: {query_list}."
            ),
            schema: obj(
                json!({ "script": { "type": "string", "description": "Lines, e.g. \"doc.new 800 600\\nlayer.new L2 name=\\\"Ink\\\"\\n?layer.list\"" } }),
                &["script"],
            ),
            read_only: false,
        },
        ToolDef {
            name: "command",
            description: "Execute one command in its JSON form {cmd, v, args}; args use the \
                parameter names from `help` (ids and colours as strings like \"L2\", \
                \"#ff8800\"; stroke samples as [[x,y,pressure,t_us],…])."
                .into(),
            schema: obj(
                json!({
                    "cmd": { "type": "string", "enum": cmd_names },
                    "v": { "type": "integer", "minimum": 1, "description": "Command version (default 1)" },
                    "args": { "type": "object", "description": "Parameters by name" },
                }),
                &["cmd"],
            ),
            read_only: false,
        },
        ToolDef {
            name: "query",
            description: format!(
                "Answer one query and return its JSON (never changes state). Queries: {query_list}."
            ),
            schema: obj(
                json!({ "query": { "type": "string", "description": "e.g. \"?layer.list\" or \"?pixel x=10 y=20\"" } }),
                &["query"],
            ),
            read_only: true,
        },
        ToolDef {
            name: "render",
            description: "Return the document as a PNG image: flattened (paper and visible \
                layers, as exported), or one layer's own pixels. Optionally a region, and \
                downscaled to fit max_size (default 1024)."
                .into(),
            schema: obj(
                json!({
                    "layer": { "type": "string", "description": "Layer id (e.g. \"L2\"); default: the flattened image" },
                    "x": { "type": "integer", "minimum": 0 }, "y": { "type": "integer", "minimum": 0 },
                    "w": { "type": "integer", "minimum": 1 }, "h": { "type": "integer", "minimum": 1 },
                    "max_size": { "type": "integer", "minimum": 16, "maximum": 4096 },
                }),
                &[],
            ),
            read_only: true,
        },
        ToolDef {
            name: "stroke",
            description: "Paint one stroke with the current tool's selected sub tool and the \
                active colour, as drawing in the app does (the eraser tool and the transparent \
                colour erase). Records a full `brush.stroke` command. Pick the brush with \
                tool.select / subtool.select / subtool.set and the colour with color.set."
                .into(),
            schema: obj(
                json!({
                    "samples": {
                        "type": "array",
                        "description": "Pen samples [x, y, pressure 0..1, t in µs from the stroke start], in canvas pixels; t must increase. Optional tilt: [x,y,p,t,tilt_x,tilt_y,twist].",
                        "items": { "type": "array", "items": { "type": "number" }, "minItems": 4, "maxItems": 7 },
                    },
                    "layer": { "type": "string", "description": "Layer id (default: the active layer)" },
                }),
                &["samples"],
            ),
            read_only: false,
        },
        ToolDef {
            name: "journal",
            description: "The session journal: every journaled command so far, as replayable \
                text (`#brushye-journal` header, one `@n` line per command)."
                .into(),
            schema: obj(json!({}), &[]),
            read_only: true,
        },
        ToolDef {
            name: "doc_open",
            description: "Open a .brsh document, replacing the session (its history and \
                journal start fresh, recorded on top of the opened document)."
                .into(),
            schema: obj(json!({ "path": { "type": "string" } }), &["path"]),
            read_only: false,
        },
        ToolDef {
            name: "doc_save",
            description: "Save the document as .brsh (optionally embedding the journal).".into(),
            schema: obj(
                json!({ "path": { "type": "string" }, "embed_journal": { "type": "boolean" } }),
                &["path"],
            ),
            read_only: false,
        },
        ToolDef {
            name: "export_png",
            description: "Export the flattened document (or one layer's pixels) as a PNG file."
                .into(),
            schema: obj(
                json!({ "path": { "type": "string" }, "layer": { "type": "string" } }),
                &["path"],
            ),
            read_only: false,
        },
        ToolDef {
            name: "replay",
            description: "Replace the session by replaying a journal (text or a file). A \
                journal recorded on an opened document needs that document as `base`."
                .into(),
            schema: obj(
                json!({
                    "text": { "type": "string", "description": "Journal text" },
                    "path": { "type": "string", "description": "Journal file (instead of text)" },
                    "base": { "type": "string", "description": ".brsh the journal was recorded on" },
                }),
                &[],
            ),
            read_only: false,
        },
    ]
}

fn str_arg<'a>(a: &'a Map<String, Json>, k: &str) -> Result<Option<&'a str>, String> {
    match a.get(k) {
        None | Some(Json::Null) => Ok(None),
        Some(Json::String(s)) => Ok(Some(s)),
        Some(_) => Err(format!("'{k}' must be a string")),
    }
}
fn req_str<'a>(a: &'a Map<String, Json>, k: &str) -> Result<&'a str, String> {
    str_arg(a, k)?.ok_or_else(|| format!("missing '{k}'"))
}
fn u32_arg(a: &Map<String, Json>, k: &str) -> Result<Option<u32>, String> {
    match a.get(k) {
        None | Some(Json::Null) => Ok(None),
        Some(v) => v
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .map(Some)
            .ok_or_else(|| format!("'{k}' must be a non-negative integer")),
    }
}
fn check_keys(a: &Map<String, Json>, allowed: &[&str]) -> Result<(), String> {
    match a.keys().find(|k| !allowed.contains(&k.as_str())) {
        Some(k) => Err(format!("unknown argument '{k}'")),
        None => Ok(()),
    }
}

fn pretty(j: &Json) -> String {
    serde_json::to_string_pretty(j).unwrap_or_else(|_| j.to_string())
}

/// One layer's own pixels (straight RGBA, no paper, no blending).
fn layer_image(doc: &Document, id: LayerId) -> Result<Image, String> {
    let layer = doc.layer(id).ok_or_else(|| format!("no layer {id}"))?;
    let (w, h) = (doc.width, doc.height);
    let mut px = Vec::with_capacity(w as usize * h as usize * 4);
    for y in 0..h {
        for x in 0..w {
            px.extend_from_slice(&layer.pixel(x, y));
        }
    }
    Ok(Image {
        width: w,
        height: h,
        px,
    })
}

fn crop(img: &Image, x: u32, y: u32, w: u32, h: u32) -> Result<Image, String> {
    if u64::from(x) + u64::from(w) > u64::from(img.width)
        || u64::from(y) + u64::from(h) > u64::from(img.height)
    {
        return Err(format!(
            "region {x},{y} {w}x{h} is outside the {}x{} canvas",
            img.width, img.height
        ));
    }
    let mut px = Vec::with_capacity(w as usize * h as usize * 4);
    for row in y..y + h {
        let start = (row as usize * img.width as usize + x as usize) * 4;
        px.extend_from_slice(&img.px[start..start + w as usize * 4]);
    }
    Ok(Image {
        width: w,
        height: h,
        px,
    })
}

fn cmd_samples(c: &Command) -> usize {
    match c {
        Command::BrushStroke(b) => b.samples.0.len(),
        _ => 0,
    }
}

fn parse_layer(s: &str) -> Result<LayerId, String> {
    s.strip_prefix('L')
        .and_then(|n| n.parse().ok())
        .map(LayerId)
        .ok_or_else(|| format!("'{s}' is not a layer id like \"L2\""))
}

fn open_file(path: &str) -> Result<brsh::Loaded, String> {
    brsh::load_file(Path::new(path)).map_err(|e| format!("{path}: {e}"))
}

impl Server {
    pub fn new() -> Self {
        Self {
            session: Session::new(),
        }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    fn doc(&self) -> Result<std::sync::Arc<Document>, String> {
        self.session
            .document()
            .ok_or_else(|| "no document (run doc.new first)".into())
    }

    /// The `brush.stroke` the app would record for this input: the current brush tool's
    /// selected sub tool, in the active colour; the eraser tool and the transparent colour
    /// erase (as `begin_stroke` in `brushye-app`).
    fn current_brush_stroke(&self, layer: LayerId, samples: Samples) -> Result<Command, String> {
        let tools = self.session.tools();
        if !tools.tool.is_brush() {
            return Err(format!(
                "the current tool ({}) doesn't paint; tool.select pen, pencil, brush, airbrush or eraser",
                tools.tool.name()
            ));
        }
        let mut preset = tools
            .current(tools.tool)
            .ok_or("the current tool has no sub tool selected")?
            .preset
            .clone();
        let color = match (tools.tool, tools.color()) {
            (Tool::Eraser, _) => [0, 0, 0],
            (_, Some(c)) => c,
            (_, None) => {
                preset.ink = Some(InkMode::Erase);
                [0, 0, 0]
            }
        };
        Ok(Command::BrushStroke(
            preset.to_stroke(layer, color, samples),
        ))
    }

    fn image(&self, layer: Option<&str>) -> Result<Image, String> {
        let doc = self.doc()?;
        match layer {
            Some(l) => layer_image(&doc, parse_layer(l)?),
            None => Ok(render::flatten(&doc)),
        }
    }

    /// Call a tool. `Err` is a tool error, reported to the client as `isError`.
    pub fn call(&mut self, name: &str, args: &Map<String, Json>) -> Result<Vec<Out>, String> {
        match name {
            "help" => {
                check_keys(args, &[])?;
                Ok(vec![Out::Text(format!(
                    "{COMMAND_FORMAT}\n\n{}",
                    commands::reference_markdown()
                ))])
            }
            "run" => {
                check_keys(args, &["script"])?;
                let script = req_str(args, "script")?;
                let mut results = Vec::new();
                for (i, line) in script.lines().enumerate() {
                    match self.session.run_line(line) {
                        Ok(Some(j)) => results.push(j),
                        Ok(None) => {}
                        Err(e) => {
                            let mut msg = format!("line {}: {}: {e}", i + 1, line.trim());
                            if !results.is_empty() {
                                msg.push_str(&format!(
                                    "\nquery results before the error: {}",
                                    Json::Array(results)
                                ));
                            }
                            return Err(msg);
                        }
                    }
                }
                Ok(vec![Out::Text(if results.is_empty() {
                    "ok".into()
                } else {
                    Json::Array(results).to_string()
                })])
            }
            "command" => {
                check_keys(args, &["cmd", "v", "args"])?;
                let mut j = Json::Object(args.clone());
                if let Some(o) = j.as_object_mut() {
                    o.entry("v").or_insert(json!(1));
                }
                let cmd = Command::from_json(&j)?;
                let line = brushye_command::syntax::print_line(&cmd.to_raw());
                self.session.execute(cmd).map_err(|e| e.0)?;
                Ok(vec![Out::Text(format!("ok: {line}"))])
            }
            "query" => {
                check_keys(args, &["query"])?;
                let q = req_str(args, "query")?.trim();
                let q = if q.starts_with('?') {
                    q.to_owned()
                } else {
                    format!("?{q}")
                };
                if q.contains('\n') {
                    return Err("one query per call (use `run` for several)".into());
                }
                match self.session.run_line(&q).map_err(|e| e.0)? {
                    Some(j) => Ok(vec![Out::Text(pretty(&j))]),
                    None => Err("not a query".into()),
                }
            }
            "render" => {
                check_keys(args, &["layer", "x", "y", "w", "h", "max_size"])?;
                let mut img = self.image(str_arg(args, "layer")?)?;
                let region = [
                    u32_arg(args, "x")?,
                    u32_arg(args, "y")?,
                    u32_arg(args, "w")?,
                    u32_arg(args, "h")?,
                ];
                if region.iter().any(Option::is_some) {
                    let x = region[0].unwrap_or(0);
                    let y = region[1].unwrap_or(0);
                    let w = region[2].unwrap_or(img.width.saturating_sub(x));
                    let h = region[3].unwrap_or(img.height.saturating_sub(y));
                    img = crop(&img, x, y, w, h)?;
                }
                let (w0, h0) = (img.width, img.height);
                let max = u32_arg(args, "max_size")?.unwrap_or(1024).clamp(16, 4096);
                let img = brsh::thumbnail(&img, max);
                let png = png_io::png_bytes(&img).map_err(|e| e.to_string())?;
                let note = if (img.width, img.height) == (w0, h0) {
                    format!("{}x{} px", w0, h0)
                } else {
                    format!(
                        "{w0}x{h0} px, shown downscaled to {}x{}",
                        img.width, img.height
                    )
                };
                Ok(vec![Out::Png(png), Out::Text(note)])
            }
            "stroke" => {
                check_keys(args, &["samples", "layer"])?;
                let samples = args.get("samples").ok_or("missing 'samples'")?;
                let samples = <Samples as Arg>::from_json(samples)?;
                let layer = match str_arg(args, "layer")? {
                    Some(l) => parse_layer(l)?,
                    None => self
                        .session
                        .active_layer()
                        .ok_or("no active layer (layer.select one, or pass 'layer')")?,
                };
                let cmd = self.current_brush_stroke(layer, samples)?;
                let line = brushye_command::syntax::print_line(&cmd.to_raw());
                let n = cmd_samples(&cmd);
                self.session.execute(cmd).map_err(|e| e.0)?;
                Ok(vec![Out::Text(format!(
                    "ok: {n} samples; {}",
                    line.split(" samples=").next().unwrap_or(&line)
                ))])
            }
            "journal" => {
                check_keys(args, &[])?;
                Ok(vec![Out::Text(self.session.journal_text())])
            }
            "doc_open" => {
                check_keys(args, &["path"])?;
                let path = req_str(args, "path")?;
                let l = open_file(path)?;
                let n = l.doc.layers().len();
                let mut msg = format!(
                    "opened {path}: {}x{} px, {n} layer{}",
                    l.doc.width,
                    l.doc.height,
                    if n == 1 { "" } else { "s" }
                );
                if let Some(j) = &l.journal {
                    msg.push_str(&format!(
                        "; it embeds a journal of {} lines (not replayed)",
                        j.lines().count().saturating_sub(1)
                    ));
                }
                for w in &l.warnings {
                    msg.push_str(&format!("\nwarning: {w}"));
                }
                let tools = self.session.tools().clone();
                self.session = Session::from_document(l.doc, l.active);
                self.session.set_tools(tools);
                Ok(vec![Out::Text(msg)])
            }
            "doc_save" => {
                check_keys(args, &["path", "embed_journal"])?;
                let path = req_str(args, "path")?;
                let embed = match args.get("embed_journal") {
                    None | Some(Json::Null) => false,
                    Some(Json::Bool(b)) => *b,
                    Some(_) => return Err("'embed_journal' must be a boolean".into()),
                };
                let doc = self.doc()?;
                let journal = self.session.journal_text();
                let opts = SaveOptions {
                    active: self.session.active_layer(),
                    journal: embed.then_some(journal.as_str()),
                };
                brsh::save_file(Path::new(path), &doc, &opts, &mut TileCache::default())
                    .map_err(|e| format!("{path}: {e}"))?;
                Ok(vec![Out::Text(format!("saved {path}"))])
            }
            "export_png" => {
                check_keys(args, &["path", "layer"])?;
                let path = req_str(args, "path")?;
                let img = self.image(str_arg(args, "layer")?)?;
                let bytes = png_io::png_bytes(&img).map_err(|e| e.to_string())?;
                std::fs::write(path, bytes).map_err(|e| format!("{path}: {e}"))?;
                Ok(vec![Out::Text(format!(
                    "exported {path} ({}x{} px)",
                    img.width, img.height
                ))])
            }
            "replay" => {
                check_keys(args, &["text", "path", "base"])?;
                let text = match (str_arg(args, "text")?, str_arg(args, "path")?) {
                    (Some(t), None) => t.to_owned(),
                    (None, Some(p)) => {
                        std::fs::read_to_string(p).map_err(|e| format!("{p}: {e}"))?
                    }
                    _ => return Err("give exactly one of 'text' or 'path'".into()),
                };
                let replayed = match str_arg(args, "base")? {
                    Some(b) => {
                        let l = open_file(b)?;
                        Session::replay_on(l.doc, l.active, &text)
                    }
                    None => Session::replay(&text),
                };
                let s = replayed.map_err(|e| format!("journal line {}: {}", e.line, e.msg))?;
                let tools = self.session.tools().clone();
                self.session = s;
                self.session.set_tools(tools);
                Ok(vec![Out::Text(format!(
                    "replayed {} commands",
                    self.session.journal_len()
                ))])
            }
            other => Err(format!("unknown tool '{other}'")),
        }
    }
}

/// PNG bytes as base64 (for MCP image content).
pub fn png_base64(png: &[u8]) -> String {
    B64.encode(png)
}
