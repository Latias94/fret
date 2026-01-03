//! Portable platform contracts.
//!
//! This crate is intentionally backend-agnostic (no `winit`, no `wgpu`, no `accesskit`).
//!
//! Backend implementations live in crates like:
//! - `fret-platform-native` (native: Windows/macOS/Linux)
//! - `fret-platform-web` (wasm32: browser APIs)
//! - future: `fret-platform-ios` / `fret-platform-android`

pub mod clipboard;
pub mod external_drop;
pub mod file_dialog;
pub mod open_url;
