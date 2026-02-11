use super::super::super::super::*;

pub(in crate::ui) fn preview_toggle(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_toggle(cx)
}

pub(in crate::ui) fn preview_toggle_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_toggle_group(cx)
}

pub(in crate::ui) fn preview_typography(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_typography(cx)
}

pub(in crate::ui) fn preview_empty(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_empty(cx)
}
