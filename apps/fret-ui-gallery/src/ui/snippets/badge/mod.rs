//! Snippet-backed Badge examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling `render(cx)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-badge-*` `test_id`s stable: diag scripts depend on them.

pub mod colors;
pub mod demo;
pub mod icon;
pub mod link;
pub mod rtl;
pub mod spinner;
pub mod usage;
pub mod variants;
