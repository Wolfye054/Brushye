//! Brushye command layer (DECISIONS D-011, D-013).
//!
//! Text syntax: `docs/command-format.md`. Clients (CLI, app, MCP) change state only through
//! [`Session::execute`] / [`Session::run_line`] / [`Session::replay`]; they receive immutable
//! document versions for reading.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod commands;
pub mod exec;
pub mod params;
pub mod syntax;

/// The document model, for reading the immutable versions a `Session` hands out.
pub use brushye_core as core;
pub use commands::{Command, Query};
pub use exec::{ExecError, LineError, Session};
