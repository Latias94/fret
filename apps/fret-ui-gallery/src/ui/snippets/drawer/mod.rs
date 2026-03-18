//! Snippet-backed Drawer examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling `render(cx)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-drawer-*` `test_id`s stable: diag scripts depend on them.

pub mod demo;
pub mod nested;
pub mod outside_press;
pub mod responsive_dialog;
pub mod rtl;
pub mod scrollable_content;
pub mod sides;
pub mod snap_points;
pub mod usage;
