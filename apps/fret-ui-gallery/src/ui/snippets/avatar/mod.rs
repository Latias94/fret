//! Snippet-backed Avatar examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling the snippet `render(...)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-avatar-*` `test_id`s stable: diag scripts depend on them.

use fret_core::ImageId;
use fret_ui::{ElementContext, UiHost};
use fret_ui_assets::ui::ImageSourceElementContextExt as _;

pub(crate) fn demo_image<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Option<ImageId> {
    let request = crate::driver::demo_assets::ui_gallery_shared_media_preview_request();
    cx.use_image_source_state_from_asset_request(&request).image
}

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
