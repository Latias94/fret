//! Snippet-backed Alert Dialog examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling `render(cx)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-alert-dialog-*` `test_id`s stable: diag scripts depend on them.

pub mod basic;
pub mod demo;
pub mod destructive;
pub mod media;
pub mod parts;
pub mod rtl;
pub mod small;
pub mod small_with_media;
