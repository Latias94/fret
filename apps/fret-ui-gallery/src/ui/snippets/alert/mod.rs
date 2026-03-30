//! Snippet-backed Alert examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling `render(cx)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-alert-*` `test_id`s stable: diag scripts depend on them.

pub mod action;
pub mod basic;
pub mod custom_colors;
pub mod demo;
pub mod destructive;
pub mod interactive_links;
pub mod rich_description;
pub mod rich_title;
pub mod rtl;
pub mod usage;
