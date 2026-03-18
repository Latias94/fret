use fret_core::ImageId;
use fret_ui::{ElementContext, UiHost};
use fret_ui_assets::image_asset_state::ImageLoadingStatus;
use fret_ui_assets::ui::ImageSourceElementContextExt as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PreviewImageState {
    pub image: Option<ImageId>,
    pub loading: bool,
}

fn preview_image_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    request: fret::assets::AssetRequest,
) -> PreviewImageState {
    let state = cx.use_image_source_state_from_asset_request(&request);
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
    let request = crate::driver::demo_assets::ui_gallery_aspect_ratio_landscape_request();
    preview_image_state(cx, request)
}

pub(crate) fn portrait_image_state<H: UiHost>(cx: &mut ElementContext<'_, H>) -> PreviewImageState {
    let request = crate::driver::demo_assets::ui_gallery_aspect_ratio_portrait_request();
    preview_image_state(cx, request)
}
