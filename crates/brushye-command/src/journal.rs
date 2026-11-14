//! Reading a journal for people (M9 journal viewer): one entry per command, with stroke sample
//! blobs summarised and the undo position worked out, so a viewer can show which steps are
//! currently undone. Used by `brushye journal show` and the app's Journal window.
//!
//! This only parses; it never executes. Use [`crate::Session::replay`] to check that a journal
//! replays.

use serde_json::{Value as Json, json};

use crate::commands::{COMMANDS, Class};
use crate::syntax::{Line, RawLine, Value, parse_line, print_line};

/// A summary of a stroke's input samples.
#[derive(Clone, Debug, PartialEq)]
pub struct StrokeInfo {
    pub samples: usize,
    pub duration_ms: f64,
    /// Bounding box of the sample positions: x0, y0, x1, y1 (canvas px).
    pub bounds: [f64; 4],
    /// Lowest and highest pressure, if the device reported pressure.
    pub pressure: Option<[f32; 2]>,
    pub tilt: bool,
}

impl StrokeInfo {
    fn of(samples: &[brushye_core::input::Sample]) -> Self {
        let mut b = [f64::MAX, f64::MAX, f64::MIN, f64::MIN];
        let mut p: Option<[f32; 2]> = None;
        for s in samples {
            b = [b[0].min(s.x), b[1].min(s.y), b[2].max(s.x), b[3].max(s.y)];
            if let Some(v) = s.pressure {
                p = Some(p.map_or([v, v], |[lo, hi]| [lo.min(v), hi.max(v)]));
            }
        }
        if samples.is_empty() {
            b = [0.0; 4];
        }
        Self {
            samples: samples.len(),
            duration_ms: samples.last().map_or(0.0, |s| f64::from(s.t_us) / 1000.0),
            bounds: b,
            pressure: p,
            tilt: samples.iter().any(|s| s.tilt.is_some()),
        }
    }

    /// Short text, e.g. `42 samples, 0.35 s, (10,20)–(300,180), pressure 0.12–0.98`.
    pub fn text(&self) -> String {
        let [x0, y0, x1, y1] = self.bounds;
        let mut s = format!(
            "{} samples, {:.2} s, ({x0:.0},{y0:.0})–({x1:.0},{y1:.0})",
            self.samples,
            self.duration_ms / 1000.0
        );
        match self.pressure {
            Some([lo, hi]) => s.push_str(&format!(", pressure {lo:.2}–{hi:.2}")),
            None => s.push_str(", no pressure"),
        }
        if self.tilt {
            s.push_str(", tilt");
        }
        s
    }
}

/// One journal command.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    /// 1-based line in the journal text.
    pub line: usize,
    pub seq: Option<u64>,
    /// Joined the previous undo step (`@n+`).
    pub merge: bool,
    pub name: String,
    /// `None` for a command this Brushye doesn't know.
    pub class: Option<Class>,
    /// The line with blobs shortened (stroke samples summarised, other blobs as a byte count).
    pub text: String,
    pub stroke: Option<StrokeInfo>,
    /// The change is undone at the end of the journal (it is past the undo position).
    pub undone: bool,
}

/// A parsed journal.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JournalView {
    /// The `#brushye-journal …` header line, if present.
    pub header: Option<String>,
    pub entries: Vec<Entry>,
    /// Lines that didn't parse: (line, message).
    pub errors: Vec<(usize, String)>,
}

/// The line text with blobs shortened, plus the stroke summary if it has samples.
pub fn describe(raw: &RawLine) -> (String, Option<StrokeInfo>) {
    let mut short = raw.clone();
    let mut stroke = None;
    for (k, v) in &mut short.named {
        if let Value::Blob(b) = v {
            let summary = if k == "samples" {
                match brushye_core::input::decode(b) {
                    Ok(s) => {
                        let info = StrokeInfo::of(&s);
                        let t = info.text();
                        stroke = Some(info);
                        t
                    }
                    Err(e) => format!("unreadable: {e}"),
                }
            } else {
                format!("{} bytes", b.len())
            };
            // Shown as a string so the printer keeps it on the line.
            *v = Value::Str(format!("<{summary}>"));
        }
    }
    let text = print_line(&short).replace("\"<", "<").replace(">\"", ">");
    (text, stroke)
}

/// Parse a journal (or a script) for viewing. Never fails: bad lines go to `errors`.
pub fn view(text: &str) -> JournalView {
    let mut v = JournalView::default();
    // Undo steps, each a list of entry indices; `pos` is the current step (D-013 / exec.rs).
    let mut steps: Vec<Vec<usize>> = vec![Vec::new()];
    let mut pos = 0usize;
    for (i, l) in text.lines().enumerate() {
        let n = i + 1;
        if n == 1 && l.starts_with("#brushye-journal ") {
            v.header = Some(l.to_owned());
            continue;
        }
        let raw = match parse_line(l) {
            Ok(Line::Item(r)) => r,
            Ok(_) => continue,
            Err(e) => {
                v.errors.push((n, e.to_string()));
                continue;
            }
        };
        let class = COMMANDS
            .iter()
            .find(|s| s.name == raw.name)
            .and_then(|s| s.class);
        let idx = v.entries.len();
        match class {
            Some(Class::Session) => {
                steps = vec![vec![idx]];
                pos = 0;
            }
            Some(Class::Undoable) => {
                steps.truncate(pos + 1);
                if raw.merge && pos > 0 {
                    steps[pos].push(idx);
                } else {
                    steps.push(vec![idx]);
                    pos += 1;
                }
            }
            Some(Class::History) if raw.name == "history.undo" => pos = pos.saturating_sub(1),
            Some(Class::History) if raw.name == "history.redo" => {
                if pos + 1 < steps.len() {
                    pos += 1;
                }
            }
            _ => {}
        }
        let (text, stroke) = describe(&raw);
        v.entries.push(Entry {
            line: n,
            seq: raw.seq,
            merge: raw.merge,
            name: raw.name.clone(),
            class,
            text,
            stroke,
            undone: false,
        });
    }
    for step in steps.iter().skip(pos + 1) {
        for &i in step {
            if let Some(e) = v.entries.get_mut(i) {
                e.undone = true;
            }
        }
    }
    v
}

impl JournalView {
    /// JSON form (`brushye journal show --json`).
    pub fn to_json(&self) -> Json {
        json!({
            "header": self.header,
            "entries": self.entries.iter().map(|e| json!({
                "line": e.line,
                "seq": e.seq,
                "merge": e.merge,
                "cmd": e.name,
                "class": e.class.map(|c| format!("{c:?}").to_lowercase()),
                "text": e.text,
                "undone": e.undone,
                "stroke": e.stroke.as_ref().map(|s| json!({
                    "samples": s.samples,
                    "duration_ms": s.duration_ms,
                    "bounds": s.bounds,
                    "pressure": s.pressure,
                    "tilt": s.tilt,
                })),
            })).collect::<Vec<_>>(),
            "errors": self.errors.iter().map(|(l, m)| json!({"line": l, "msg": m})).collect::<Vec<_>>(),
        })
    }
}
