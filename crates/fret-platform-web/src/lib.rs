//! Web/wasm32 platform services.
//!
//! This crate provides browser API integrations used by `fret-runtime::Effect`s (timers, file
//! inputs, IME bridge, etc.). It intentionally does **not** implement input/event mapping; use a
//! runner crate (e.g. `fret-runner-web`) for the winit/web event layer.
//!
//! For module ownership and “where should this go?” guidance, see
//! `crates/fret-platform-web/README.md`.

#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;

mod ime_dom_state;

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::*;
