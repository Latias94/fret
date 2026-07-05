#[allow(dead_code)]
pub const SOURCE: &str = include_str!("images.rs");

use crate::demo_assets;
use fret::assets::AssetRequest;
use fret::component::ui_assets::{self, ImageLoadingStatus};
use fret_ui::{ElementContext, UiHost};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PreviewImageState {
    pub image: Option<ui_assets::ImageId>,
    pub loading: bool,
}

fn preview_image_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    request: &AssetRequest,
) -> PreviewImageState {
    let state = ui_assets::image_source_state_from_asset_request(cx, request);
    PreviewImageState {
        image: state.image,
        loading: state.image.is_none()
            && matches!(
                state.status,
                ImageLoadingStatus::Idle | ImageLoadingStatus::Loading
            ),
    }
}

pub(crate) fn landscape_image_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> PreviewImageState {
    preview_image_state(
        cx,
        &demo_assets::ui_gallery_aspect_ratio_landscape_request(),
    )
}

pub(crate) fn portrait_image_state<H: UiHost>(cx: &mut ElementContext<'_, H>) -> PreviewImageState {
    preview_image_state(cx, &demo_assets::ui_gallery_aspect_ratio_portrait_request())
}

pub(crate) fn square_image_state<H: UiHost>(cx: &mut ElementContext<'_, H>) -> PreviewImageState {
    preview_image_state(cx, &demo_assets::ui_gallery_profile_square_request())
}
