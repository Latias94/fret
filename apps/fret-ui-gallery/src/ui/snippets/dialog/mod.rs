//! Snippet-backed Dialog examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling `render(cx)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-dialog-*` `test_id`s stable: diag scripts depend on them.

pub mod custom_close_button;
pub mod demo;
pub mod no_close_button;
pub mod parts;
pub mod rtl;
pub mod scrollable_content;
pub mod sticky_footer;
pub mod usage;
