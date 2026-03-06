//! Snippet-backed Slider examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling the snippet `render(...)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-slider-*` `test_id`s stable: diag scripts depend on them.

pub mod demo;
pub mod extras;
pub mod label;
pub mod usage;
