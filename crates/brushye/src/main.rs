//! Placeholder binary so the workspace and CI build on every platform.
//! Application code starts once the proposals in DECISIONS.md are approved.

fn main() {
    println!("brushye {}", env!("CARGO_PKG_VERSION"));
}
