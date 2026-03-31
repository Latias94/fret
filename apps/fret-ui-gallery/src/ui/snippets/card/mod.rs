use crate::driver::demo_assets;
use fret::UiCx;
use fret_core::ImageId;
use fret_ui_assets::ui::ImageSourceElementContextExt as _;

pub(crate) fn demo_cover_image(cx: &mut UiCx<'_>) -> Option<ImageId> {
    cx.use_image_source_state_from_asset_request(&demo_assets::ui_gallery_card_event_cover_request())
        .image
}

pub mod card_content;
pub mod compositions;
pub mod demo;
pub mod description_children;
pub mod image;
pub mod meeting_notes;
pub mod rtl;
pub mod size;
pub mod title_children;
pub mod usage;

#[cfg(test)]
mod tests {
    const MODULE_SOURCE: &str = include_str!("mod.rs");

    #[test]
    fn card_gallery_demo_cover_resolves_bundle_assets() {
        let helper_source = MODULE_SOURCE
            .split("#[cfg(test)]")
            .next()
            .expect("card snippet module keeps helper section before tests");
        assert!(helper_source.contains("use_image_source_state_from_asset_request"));
        assert!(helper_source.contains("ui_gallery_card_event_cover_request"));
        assert!(!helper_source.contains("ImageSource::rgba8("));
    }
}
