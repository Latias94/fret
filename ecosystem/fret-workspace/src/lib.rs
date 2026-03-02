//! Workspace-shell building blocks (editor-grade app chrome).
//!
//! This crate intentionally lives in `ecosystem/`:
//! - It is policy-heavy and will iterate faster than `crates/fret-ui`.
//! - It should not expand the `fret-ui` runtime contract surface (ADR 0066).

mod command_scope;
pub mod commands;
mod focus_registry;
mod frame;
pub mod layout;
pub mod menu;
mod pane_content_focus;
pub mod panes;
pub mod tab_drag;
mod tab_strip;
pub mod tabs;

pub use command_scope::WorkspaceCommandScope;
pub use frame::{WorkspaceFrame, WorkspaceStatusBar, WorkspaceTopBar};
pub use pane_content_focus::WorkspacePaneContentFocusTarget;
pub use panes::workspace_pane_tree_element_with_resize;
pub use tab_drag::{DRAG_KIND_WORKSPACE_TAB, WorkspaceTabDragState};
pub use tab_strip::{WorkspaceTab, WorkspaceTabStrip};
