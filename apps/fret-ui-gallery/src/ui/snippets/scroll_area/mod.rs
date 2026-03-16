//! Copyable ScrollArea snippets for the default UI Gallery docs lane (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling `render(cx)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-scroll-area-*` `test_id`s stable: diag scripts depend on them.

pub mod demo;
pub mod horizontal;
pub mod nested_scroll_routing;
pub mod rtl;
pub mod usage;
