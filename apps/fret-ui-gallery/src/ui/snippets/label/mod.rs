//! Snippet-backed Label examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling `render(cx)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-label-*` `test_id`s stable: diag scripts depend on them.

pub mod demo;
pub mod label_in_field;
pub mod rtl;
