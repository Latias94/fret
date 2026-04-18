use super::super::super::super::*;
use fret::AppComponentCx;

pub(in crate::ui) fn preview_toggle(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    pages::preview_toggle(cx)
}

pub(in crate::ui) fn preview_toggle_group(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    pages::preview_toggle_group(cx)
}

pub(in crate::ui) fn preview_typography(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    pages::preview_typography(cx)
}

pub(in crate::ui) fn preview_empty(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    pages::preview_empty(cx)
}
