use super::super::super::super::*;

pub(in crate::ui) fn preview_card(
    cx: &mut ElementContext<'_, App>,
    event_cover_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    pages::preview_card(cx, event_cover_image)
}
