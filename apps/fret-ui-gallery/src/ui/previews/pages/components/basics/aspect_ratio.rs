use super::super::super::super::super::*;
use fret::UiCx;

pub(in crate::ui) fn preview_aspect_ratio(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    pages::preview_aspect_ratio(cx, None, None, None)
}
