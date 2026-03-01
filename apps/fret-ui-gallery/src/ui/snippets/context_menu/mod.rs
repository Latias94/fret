//! Snippet-backed Context Menu examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling `render(cx)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-context-menu-*` `test_id`s stable: diag scripts depend on them.

pub mod basic;
pub mod checkboxes;
pub mod destructive;
pub mod groups;
pub mod icons;
pub mod radio;
pub mod rtl;
pub mod shortcuts;
pub mod submenu;

