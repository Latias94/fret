//! Snippet-backed Button examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling the snippet `render(...)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-button-*` `test_id`s stable: diag scripts depend on them.

pub mod button_group;
pub mod children;
pub mod default;
pub mod demo;
pub mod destructive;
pub mod ghost;
pub mod icon;
pub mod link;
pub mod link_render;
pub mod loading;
pub mod outline;
pub mod rounded;
pub mod rtl;
pub mod secondary;
pub mod size;
pub mod usage;
pub mod variants;
pub mod with_icon;
