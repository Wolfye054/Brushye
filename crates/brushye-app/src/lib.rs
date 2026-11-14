//! The Brushye desktop app (DECISIONS D-010): the only crate with UI, GPU and OS-input
//! dependencies. All document changes go through the command layer (`brushye-command`).
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod app;
pub mod canvas_gpu;
pub mod curve_editor;
pub mod display;
pub mod fonts;
pub mod icons;
mod panels;
pub mod settings;
pub mod view;
