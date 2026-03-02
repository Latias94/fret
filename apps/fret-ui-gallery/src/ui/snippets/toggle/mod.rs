//! Snippet-backed Toggle examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling the snippet `render(cx)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-toggle-*` `test_id`s stable: diag scripts depend on them.

pub mod demo;
pub mod disabled;
pub mod outline;
pub mod rtl;
pub mod size;
pub mod with_text;
