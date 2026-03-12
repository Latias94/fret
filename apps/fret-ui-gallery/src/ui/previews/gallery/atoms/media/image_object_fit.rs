use super::super::super::super::super::*;
use fret::UiCx;

pub(in crate::ui) fn preview_image_object_fit(
    cx: &mut UiCx<'_>,
    theme: &Theme,
    square_image: Model<Option<ImageId>>,
    wide_image: Model<Option<ImageId>>,
    tall_image: Model<Option<ImageId>>,
    streaming_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    pages::preview_image_object_fit(
        cx,
        theme,
        square_image,
        wide_image,
        tall_image,
        streaming_image,
    )
}
