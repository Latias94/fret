//! Snippet-backed Avatar examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling the snippet `render(...)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-avatar-*` `test_id`s stable: diag scripts depend on them.

pub mod badge_icon;
pub mod basic;
pub mod demo;
pub mod dropdown;
pub mod fallback_only;
pub mod group;
pub mod group_count;
pub mod group_count_icon;
pub mod rtl;
pub mod sizes;
pub mod usage;
pub mod with_badge;
