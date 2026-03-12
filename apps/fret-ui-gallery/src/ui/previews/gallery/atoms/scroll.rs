use super::super::super::super::*;
use fret::UiCx;

pub(in crate::ui) fn preview_scroll_area(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    pages::preview_scroll_area(cx)
}
