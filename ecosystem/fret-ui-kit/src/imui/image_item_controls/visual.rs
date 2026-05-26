use fret_core::scene::{ImageSamplingHint, UvRect};
use fret_core::{Edges, ImageId, Px, Size, ViewportFit};
use fret_ui::element::{ContainerProps, ImageProps, Length, PressableState};
use fret_ui::{ElementContext, UiHost};

use super::super::{ImageItemVariant, control_chrome};

pub(in crate::imui::image_item_controls) fn image_item_chrome<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    enabled: bool,
    state: PressableState,
    variant: ImageItemVariant,
) -> ContainerProps {
    match variant {
        ImageItemVariant::Image => ContainerProps::default(),
        ImageItemVariant::Button => {
            let (_palette, mut chrome) = control_chrome::button_chrome(cx, enabled, state);
            chrome.padding = Edges::all(Px(2.0)).into();
            chrome
        }
    }
}

pub(in crate::imui::image_item_controls) fn image_props_for_item(
    image: ImageId,
    fit: ViewportFit,
    sampling: ImageSamplingHint,
    opacity: f32,
    uv: Option<UvRect>,
) -> ImageProps {
    let mut props = ImageProps::new(image);
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Fill;
    props.fit = fit;
    props.sampling = sampling;
    props.opacity = normalize_opacity(opacity);
    props.uv = uv.filter(|uv| uv_rect_is_valid(*uv));
    props
}

pub(in crate::imui::image_item_controls) fn sanitize_item_size(size: Size) -> Size {
    Size::new(sanitize_px(size.width), sanitize_px(size.height))
}

fn sanitize_px(px: Px) -> Px {
    if px.0.is_finite() {
        Px(px.0.max(0.0))
    } else {
        Px(0.0)
    }
}

pub(in crate::imui::image_item_controls) fn normalize_opacity(opacity: f32) -> f32 {
    if opacity.is_finite() {
        opacity.clamp(0.0, 1.0)
    } else {
        1.0
    }
}

pub(in crate::imui::image_item_controls) fn uv_rect_is_valid(uv: UvRect) -> bool {
    uv.u0.is_finite()
        && uv.v0.is_finite()
        && uv.u1.is_finite()
        && uv.v1.is_finite()
        && uv.u1 > uv.u0
        && uv.v1 > uv.v0
}
