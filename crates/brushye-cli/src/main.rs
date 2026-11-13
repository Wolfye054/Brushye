//! `brushye` — headless Brushye (DECISIONS D-010/D-011).
//!
//! ```text
//! brushye run SCRIPT [--journal OUT] [--png OUT]   run a script (- = stdin); print query results as JSON lines
//! brushye replay JOURNAL [--query Q]...   replay a journal, then answer queries
//! brushye render JOURNAL OUT.png          replay a journal and export the flattened image
//! brushye commands                        print the command/query registry (JSON)
//! brushye --version
//! ```
//! Exit status: 0 on success, 1 on a script/journal error (reported as `FILE:LINE: message`),
//! 2 on usage errors.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::io::Read as _;
use std::process::ExitCode;

use brushye_command::Session;

const USAGE: &str = "usage:
  brushye run SCRIPT [--journal OUT] [--png OUT]
                                          run a script (- = stdin); query results print as JSON lines
  brushye replay JOURNAL [--query Q]...   replay a journal, then answer queries (e.g. --query '?doc.hash')
  brushye render JOURNAL OUT.png          replay a journal and export the flattened image as PNG
  brushye commands                        print the command/query registry
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

/// Flatten the session's current document and write it as PNG.
fn export_png(s: &Session, out: &str) -> Result<(), Fail> {
    let doc = s
        .document()
        .ok_or_else(|| Fail::Run("no document to render (run doc.new first)".into()))?;
    let img = brushye_command::core::render::flatten(&doc);
    let bytes = brushye_format::png_io::png_bytes(&img).map_err(|e| Fail::Run(e.to_string()))?;
    std::fs::write(out, bytes).map_err(|e| Fail::Run(format!("{out}: {e}")))
}

fn print_json(j: &serde_json::Value) {
    println!("{j}");
}

enum Fail {
    Usage(String),
    Run(String),
}

fn real_main(args: &[String]) -> Result<(), Fail> {
    let usage = |m: &str| Fail::Usage(format!("{m}\n{USAGE}"));
    match args.first().map(String::as_str) {
        Some("--version" | "-V") => {
            println!("brushye {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("--help" | "-h") | None => {
            println!("{USAGE}");
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
        Some("run") => {
            let mut path = None;
            let mut journal = None;
            let mut png = None;
            let mut it = args[1..].iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "--journal" => {
                        journal = Some(
                            it.next()
                                .ok_or_else(|| usage("--journal needs a file"))?
                                .clone(),
                        )
                    }
                    "--png" => {
                        png = Some(
                            it.next()
                                .ok_or_else(|| usage("--png needs a file"))?
                                .clone(),
                        )
                    }
                    s if path.is_none() => path = Some(s.to_owned()),
                    s => return Err(usage(&format!("unexpected argument '{s}'"))),
                }
            }
            let path = path.ok_or_else(|| usage("run needs a SCRIPT"))?;
            let text = read_input(&path).map_err(Fail::Run)?;
            let mut s = Session::new();
            for (i, line) in text.lines().enumerate() {
                match s.run_line(line) {
                    Ok(Some(j)) => print_json(&j),
                    Ok(None) => {}
                    Err(e) => return Err(Fail::Run(format!("{path}:{}: {e}", i + 1))),
                }
            }
            if let Some(out) = journal {
                std::fs::write(&out, s.journal_text())
                    .map_err(|e| Fail::Run(format!("{out}: {e}")))?;
            }
            if let Some(out) = png {
                export_png(&s, &out)?;
            }
            Ok(())
        }
        Some("replay") => {
            let mut path = None;
            let mut queries = Vec::new();
            let mut it = args[1..].iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "--query" => queries.push(
                        it.next()
                            .ok_or_else(|| usage("--query needs a query"))?
                            .clone(),
                    ),
                    s if path.is_none() => path = Some(s.to_owned()),
                    s => return Err(usage(&format!("unexpected argument '{s}'"))),
                }
            }
            let path = path.ok_or_else(|| usage("replay needs a JOURNAL"))?;
            let text = read_input(&path).map_err(Fail::Run)?;
            let mut s = Session::replay(&text)
                .map_err(|e| Fail::Run(format!("{path}:{}: {}", e.line, e.msg)))?;
            for q in queries {
                if !q.trim_start().starts_with('?') {
                    return Err(usage(&format!(
                        "--query must be a query (starting with '?'): {q}"
                    )));
                }
                match s.run_line(&q) {
                    Ok(Some(j)) => print_json(&j),
                    Ok(None) => {}
                    Err(e) => return Err(Fail::Run(format!("{q}: {e}"))),
                }
            }
            Ok(())
        }
        Some("render") => {
            let (Some(path), Some(out), None) = (args.get(1), args.get(2), args.get(3)) else {
                return Err(usage("render needs JOURNAL and OUT.png"));
            };
            let text = read_input(path).map_err(Fail::Run)?;
            let s = Session::replay(&text)
                .map_err(|e| Fail::Run(format!("{path}:{}: {}", e.line, e.msg)))?;
            export_png(&s, out)
        }
        Some(other) => Err(usage(&format!("unknown subcommand '{other}'"))),
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
