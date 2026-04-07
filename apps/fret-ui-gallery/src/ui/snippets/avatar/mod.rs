//! Snippet-backed Avatar examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling the snippet `render(...)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-avatar-*` `test_id`s stable: diag scripts depend on them.

use crate::demo_assets;
use fret_core::ImageId;
use fret_ui::{ElementContext, UiHost};
use fret_ui_assets::ui::ImageSourceElementContextExt as _;

pub(crate) fn demo_image<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Option<ImageId> {
    cx.use_image_source_state_from_asset_request(&demo_assets::ui_gallery_profile_square_request())
        .image
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

#[cfg(test)]
mod tests {
    const MODULE_SOURCE: &str = include_str!("mod.rs");

    #[test]
    fn avatar_gallery_demo_images_resolve_bundle_assets() {
        let helper_source = MODULE_SOURCE
            .split("#[cfg(test)]")
            .next()
            .expect("avatar snippet module keeps helper section before tests");
        assert!(helper_source.contains("use_image_source_state_from_asset_request"));
        assert!(helper_source.contains("ui_gallery_profile_square_request"));
        assert!(!helper_source.contains("ImageSource::rgba8("));
    }
}
