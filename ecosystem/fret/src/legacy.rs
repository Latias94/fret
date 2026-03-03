//! Legacy/compat surfaces for the `fret` facade.
//!
//! This module exists to keep the golden path (`View` + typed actions) boring and discoverable
//! without deleting compatibility helpers that are still used by in-tree demos or by existing apps.
//!
//! v1 stance (Action-first authoring workstream):
//! - MVU is considered legacy for new templates and cookbook examples.
//! - MVU remains available as a compat surface (for payload routing or older demos).
//!
//! See:
//! - ADR 0307 (typed actions)
//! - ADR 0308 (view runtime + hooks)

/// Common imports for legacy MVU-based example code.
///
/// Prefer `fret::prelude::*` for new code.
pub mod prelude {
    pub use crate::prelude::*;

    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pub use crate::mvu::{KeyedMessageRouter, MessageRouter, Program as MvuProgram};
}
