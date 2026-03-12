use super::super::super::super::*;
use fret::UiCx;

pub(in crate::ui) fn preview_toggle(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    pages::preview_toggle(cx)
}

pub(in crate::ui) fn preview_toggle_group(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    pages::preview_toggle_group(cx)
}

pub(in crate::ui) fn preview_typography(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    pages::preview_typography(cx)
}

pub(in crate::ui) fn preview_empty(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    pages::preview_empty(cx)
}
