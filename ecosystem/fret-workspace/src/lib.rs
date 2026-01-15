//! Workspace-shell building blocks (editor-grade app chrome).
//!
//! This crate intentionally lives in `ecosystem/`:
//! - It is policy-heavy and will iterate faster than `crates/fret-ui`.
//! - It should not expand the `fret-ui` runtime contract surface (ADR 0066).

pub mod commands;
mod frame;
pub mod layout;
pub mod menu;
mod tab_strip;
pub mod tabs;

pub use frame::{WorkspaceFrame, WorkspaceStatusBar, WorkspaceTopBar};
pub use tab_strip::{WorkspaceTab, WorkspaceTabStrip};
