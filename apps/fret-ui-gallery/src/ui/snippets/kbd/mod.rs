//! Snippet-backed Kbd examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling `render(cx)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-kbd-*` `test_id`s stable: diag scripts depend on them.

pub mod button;
pub mod demo;
pub mod group;
pub mod input_group;
pub mod rtl;
pub mod tooltip;

