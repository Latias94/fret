//! Editor-grade UI primitives, controls, and composites for Fret.
//!
//! This crate intentionally lives in `ecosystem/` and follows ADR 0066:
//! - `crates/fret-ui` stays mechanism-only (no component policy).
//! - editor-specific interaction policy and composition belong in ecosystem crates.
//!
//! v1 scope is tracked in:
//! - `docs/workstreams/ui-editor-v1.md`

#![forbid(unsafe_code)]

pub mod composites;
pub mod controls;
pub mod primitives;

#[cfg(feature = "imui")]
pub mod imui;

#[cfg(feature = "state")]
pub mod state;

pub mod prelude {
    //! Convenient imports for app code using editor controls.

    pub use crate::{composites, controls, primitives};
}

#[cfg(test)]
mod tests {
    #[test]
    fn crate_smoke_compiles() {}
}
