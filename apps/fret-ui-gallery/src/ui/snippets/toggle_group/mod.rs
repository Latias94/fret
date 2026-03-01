//! Snippet-backed Toggle Group examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling the snippet `render(...)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-toggle-group-*` `test_id`s stable: diag scripts depend on them.

pub mod demo;
pub mod disabled;
pub mod flex_1_items;
pub mod full_width_items;
pub mod outline;
pub mod rtl;
pub mod size;
pub mod spacing;
pub mod vertical;

