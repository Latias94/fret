use super::*;

pub(super) fn image_item_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    image: fret_core::ImageId,
    size: Size,
    options: ImageItemOptions,
) -> ResponseExt {
    image_item_controls::image_item_with_options(ui, id, image, size, options)
}

pub(super) fn image_button_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    image: fret_core::ImageId,
    size: Size,
    mut options: ImageItemOptions,
) -> ResponseExt {
    let was_plain_image_options = matches!(options.variant, ImageItemVariant::Image);
    options.variant = ImageItemVariant::Button;
    if was_plain_image_options {
        options.focusable = true;
    }
    image_item_controls::image_item_with_options(ui, id, image, size, options)
}
