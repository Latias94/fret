//! Immediate-mode authoring facade for Fret.
//!
//! This crate provides a small, policy-light *authoring frontend* that compiles down to Fret's
//! declarative element tree (`AnyElement` via `ElementContext`).
//!
//! The "egui/imgui-like experience" (richer response signals, widget helpers, floating areas,
//! overlays, etc.) is intentionally hosted in ecosystem facade crates (e.g. `fret-ui-kit` behind its
//! `imui` feature) to keep this crate minimal and third-party-friendly.
//!
//! Notes:
//! - This crate intentionally does not depend on platform or renderer crates.
//! - Styling/recipes should live in separate ecosystem crates (e.g. shadcn/material adapters).

pub use fret_authoring::Response;
mod frontend;

pub use frontend::{ImUi, imui, imui_build, imui_raw};

#[cfg(test)]
use fret_ui::element::Elements;

#[cfg(feature = "state-query")]
pub mod query;
#[cfg(feature = "state-selector")]
pub mod selector;

pub mod prelude {
    #[cfg(feature = "state-query")]
    pub use crate::query::UiWriterQueryExt as _;
    #[cfg(feature = "state-selector")]
    pub use crate::selector::UiWriterSelectorExt as _;
    pub use crate::{ImUi, Response, imui, imui_build, imui_raw};
    pub use fret_authoring::UiWriter;
}

#[cfg(test)]
mod tests;
