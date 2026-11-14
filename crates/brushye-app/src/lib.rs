//! The Brushye desktop app (DECISIONS D-010): the only crate with UI, GPU and OS-input
//! dependencies. All document changes go through the command layer (`brushye-command`).
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod app;
pub mod canvas_gpu;
pub mod display;
mod panels;
pub mod view;
