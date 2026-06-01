//! Immediate-mode response-bearing image item helpers.

use fret_core::{ImageId, Size};
use fret_ui::UiHost;

use super::{ImageItemOptions, ResponseExt, UiWriterImUiFacadeExt};
use crate::declarative::chrome::control_chrome_pressable_with_id_props;

mod behavior;
mod props;
mod visual;

use visual::{image_item_chrome, image_props_for_item};

pub(super) fn image_item_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    image: ImageId,
    size: Size,
    options: ImageItemOptions,
) -> ResponseExt {
    ui.push_id(("image-item", id), |ui| {
        image_item_with_options_inner(ui, image, size, options)
    })
}

fn image_item_with_options_inner<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    image: ImageId,
    size: Size,
    options: ImageItemOptions,
) -> ResponseExt {
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        let enabled = options.enabled && !super::imui_is_disabled(cx);
        let focusable = enabled && options.focusable;
        let variant = options.variant;
        let props = props::image_item_pressable_props(size, &options, enabled, focusable, variant);

        control_chrome_pressable_with_id_props(cx, move |cx, state, id| {
            behavior::install_image_item_behavior(cx, id, state, enabled, response);

            let chrome = image_item_chrome(cx, enabled, state, variant);
            let image_props = image_props_for_item(
                image,
                options.fit,
                options.sampling,
                options.opacity,
                options.uv,
            );

            (props, chrome, move |cx| vec![cx.image_props(image_props)])
        })
    });

    ui.add(element);
    response
}

#[cfg(test)]
mod tests;
