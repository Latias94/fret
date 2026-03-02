//! Snippet-backed Button examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling the snippet `render(...)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-button-*` `test_id`s stable: diag scripts depend on them.

pub mod button_group;
pub mod icon;
pub mod link_render;
pub mod loading;
pub mod rounded;
pub mod rtl;
pub mod size;
pub mod variants;
pub mod with_icon;
