//! Snippet-backed NativeSelect examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling the snippet `render(...)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-native-select-*` `test_id`s stable: diag scripts depend on them.

pub mod demo;
pub mod disabled;
pub mod invalid;
pub mod label;
pub mod rtl;
pub mod with_groups;
