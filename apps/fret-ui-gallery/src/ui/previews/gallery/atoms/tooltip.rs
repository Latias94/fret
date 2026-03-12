use super::super::super::super::*;
use fret::UiCx;

pub(in crate::ui) fn preview_tooltip(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    pages::preview_tooltip(cx)
}
