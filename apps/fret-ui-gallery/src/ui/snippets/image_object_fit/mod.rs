//! Snippet-backed `MediaImage` object-fit examples for UI Gallery.

use crate::demo_assets;
use fret::AppComponentCx;
use fret_core::ImageId;
use fret_ui_assets::ui::ImageSourceElementContextExt as _;

pub(crate) fn square_image(cx: &mut AppComponentCx<'_>) -> Option<ImageId> {
    cx.use_image_source_state_from_asset_request(&demo_assets::ui_gallery_profile_square_request())
        .image
}

pub(crate) fn wide_image(cx: &mut AppComponentCx<'_>) -> Option<ImageId> {
    cx.use_image_source_state_from_asset_request(
        &demo_assets::ui_gallery_aspect_ratio_landscape_request(),
    )
    .image
}

pub(crate) fn tall_image(cx: &mut AppComponentCx<'_>) -> Option<ImageId> {
    cx.use_image_source_state_from_asset_request(
        &demo_assets::ui_gallery_aspect_ratio_portrait_request(),
    )
    .image
}

pub(crate) fn sampling_image(cx: &mut AppComponentCx<'_>) -> Option<ImageId> {
    cx.use_image_source_state_from_asset_request(
        &demo_assets::ui_gallery_image_object_fit_sampling_request(),
    )
    .image
}

pub mod mapping;
pub mod sampling;

#[cfg(test)]
mod tests {
    const MODULE_SOURCE: &str = include_str!("mod.rs");

    #[test]
    fn image_object_fit_gallery_images_resolve_bundle_assets() {
        let helper_source = MODULE_SOURCE
            .split("#[cfg(test)]")
            .next()
            .expect("image object-fit snippet module keeps helper section before tests");
        assert!(helper_source.contains("use_image_source_state_from_asset_request"));
        assert!(helper_source.contains("ui_gallery_profile_square_request"));
        assert!(helper_source.contains("ui_gallery_aspect_ratio_landscape_request"));
        assert!(helper_source.contains("ui_gallery_aspect_ratio_portrait_request"));
        assert!(helper_source.contains("ui_gallery_image_object_fit_sampling_request"));
        assert!(!helper_source.contains("ImageSource::rgba8("));
    }
}
