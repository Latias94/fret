//! Snippet-backed Menubar examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling `render(cx)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-menubar-*` `test_id`s stable: diag scripts depend on them.

pub mod checkbox;
pub mod demo;
pub mod parts;
pub mod radio;
pub mod rtl;
pub mod submenu;
pub mod with_icons;
