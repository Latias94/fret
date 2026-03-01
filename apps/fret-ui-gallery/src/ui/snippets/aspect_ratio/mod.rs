//! Snippet-backed Aspect Ratio examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling `render(cx)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-aspect-ratio-*` `test_id`s stable: diag scripts depend on them.

pub mod demo;
pub mod portrait;
pub mod rtl;
pub mod square;
