//! Workspace-shell building blocks (editor-grade app chrome).
//!
//! This crate intentionally lives in `ecosystem/`:
//! - It is policy-heavy and will iterate faster than `crates/fret-ui`.
//! - It should not expand the `fret-ui` runtime contract surface (ADR 0066).

pub mod commands;
mod frame;
pub mod layout;
pub mod menu;
pub mod panes;
pub mod tab_drag;
mod tab_strip;
mod tab_strip_overflow;
pub mod tabs;

pub use frame::{WorkspaceFrame, WorkspaceStatusBar, WorkspaceTopBar};
pub use panes::workspace_pane_tree_element_with_resize;
pub use tab_drag::{DRAG_KIND_WORKSPACE_TAB, WorkspaceTabDragState};
pub use tab_strip::{WorkspaceTab, WorkspaceTabStrip};
