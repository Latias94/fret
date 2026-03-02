//! Snippet-backed Tooltip examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling `render(cx)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-tooltip-*` `test_id`s stable: diag scripts depend on them.

pub mod demo;
pub mod disabled_button;
pub mod keyboard_focus;
pub mod keyboard_shortcut;
pub mod rtl;
pub mod sides;
