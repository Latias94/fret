//! Immediate-mode response-bearing image item helpers.

use fret_core::{ImageId, Size};
use fret_ui::UiHost;

use super::{ImageItemOptions, ResponseExt, UiWriterImUiFacadeExt};

mod behavior;
mod entry;
mod props;
mod visual;

pub(super) fn image_item_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    image: ImageId,
    size: Size,
    options: ImageItemOptions,
) -> ResponseExt {
    ui.push_id(("image-item", id), |ui| {
        entry::image_item_with_options_inner(ui, image, size, options)
    })
}

#[cfg(test)]
mod tests;
