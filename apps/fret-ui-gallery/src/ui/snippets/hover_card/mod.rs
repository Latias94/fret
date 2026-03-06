//! Snippet-backed Hover Card examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling `render(cx)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-hover-card-*` `test_id`s stable: diag scripts depend on them.

pub mod basic;
pub mod demo;
pub mod positioning;
pub mod rtl;
pub mod sides;
pub mod trigger_delays;
pub mod usage;
