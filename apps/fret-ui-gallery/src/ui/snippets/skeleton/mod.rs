//! Snippet-backed Skeleton examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling `render(cx)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-skeleton-*` `test_id`s stable: diag scripts depend on them.

pub mod avatar;
pub mod card;
pub mod demo;
pub mod form;
pub mod rtl;
pub mod table;
pub mod text;
