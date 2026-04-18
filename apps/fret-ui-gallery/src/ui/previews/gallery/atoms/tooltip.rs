use super::super::super::super::*;
use fret::AppComponentCx;

pub(in crate::ui) fn preview_tooltip(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    pages::preview_tooltip(cx)
}
