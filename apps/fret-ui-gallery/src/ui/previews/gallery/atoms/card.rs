use super::super::super::super::*;
use fret::UiCx;

pub(in crate::ui) fn preview_card(
    cx: &mut UiCx<'_>,
    event_cover_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    pages::preview_card(cx, event_cover_image)
}
