//! Snippet-backed Breadcrumb examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling `render(cx)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-breadcrumb-*` `test_id`s stable: diag scripts depend on them.

pub mod basic;
pub mod collapsed;
pub mod custom_separator;
pub mod demo;
pub mod dropdown;
pub mod link_component;
pub mod rtl;
