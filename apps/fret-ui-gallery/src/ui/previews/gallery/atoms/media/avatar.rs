use super::super::super::super::super::*;
use fret::UiCx;

pub(in crate::ui) fn preview_avatar(
    cx: &mut UiCx<'_>,
    avatar_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    pages::preview_avatar(cx, avatar_image)
}
