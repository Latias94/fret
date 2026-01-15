//! Workspace-shell building blocks (editor-grade app chrome).
//!
//! This crate intentionally lives in `ecosystem/`:
//! - It is policy-heavy and will iterate faster than `crates/fret-ui`.
//! - It should not expand the `fret-ui` runtime contract surface (ADR 0066).

mod frame;
mod tab_strip;

pub use frame::{WorkspaceFrame, WorkspaceStatusBar, WorkspaceTopBar};
pub use tab_strip::{WorkspaceTab, WorkspaceTabStrip};
