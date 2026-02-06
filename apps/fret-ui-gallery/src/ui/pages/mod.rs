use super::*;

mod toggle;
mod toggle_group;
mod tooltip;

pub(super) fn preview_toggle(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    toggle::preview_toggle(cx)
}

pub(super) fn preview_toggle_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    toggle_group::preview_toggle_group(cx)
}

pub(super) fn preview_tooltip(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    tooltip::preview_tooltip(cx)
}
