use super::super::super::super::*;
use fret::AppComponentCx;

pub(in crate::ui) fn preview_scroll_area(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    pages::preview_scroll_area(cx)
}
