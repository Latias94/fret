//! Snippet-backed examples for UI Gallery.
//!
//! These modules are intended to be the single source of truth for:
//! - the compiled preview,
//! - and the copyable code tab (via `include_str!`).
//!
//! Prefer user-facing imports in snippet files (typically `use fret_ui_shadcn::prelude::*;`).

pub mod button_group;
pub mod context_menu;
pub mod dropdown_menu;
pub mod input_group;
pub mod menubar;
pub mod popover;
pub mod select;
pub mod sheet;
pub mod tooltip;
