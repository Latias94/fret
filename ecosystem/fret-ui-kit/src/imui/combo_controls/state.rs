use fret_ui::{Invalidation, UiHost};

use super::super::{ResponseExt, UiWriterImUiFacadeExt};

pub(super) fn combo_enabled<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    options_enabled: bool,
) -> bool {
    options_enabled && ui.with_cx_mut(|cx| !super::super::imui_is_disabled(cx))
}

pub(super) fn combo_popup_open<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
) -> bool {
    let popup_open = ui.popup_open_model(id);
    ui.with_cx_mut(|cx| {
        cx.read_model(&popup_open, Invalidation::Paint, |_app, value| *value)
            .unwrap_or(false)
    })
}

pub(super) fn toggle_popup_from_trigger<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    enabled: bool,
    open_before: bool,
    trigger: &ResponseExt,
) {
    if !enabled || !trigger.clicked() {
        return;
    }

    if open_before {
        ui.close_popup(id);
    } else if let Some(anchor) = trigger.rect() {
        ui.open_popup_at(id, anchor);
    }
}

pub(super) fn close_disabled_popup<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    enabled: bool,
    popup_opened: bool,
) {
    if !enabled && popup_opened {
        ui.close_popup(id);
    }
}

pub(super) fn combo_toggled<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    trigger: &ResponseExt,
    open_after: bool,
) -> bool {
    trigger.id().is_some_and(|element_id| {
        ui.with_cx_mut(|cx| super::super::model_value_changed_for(cx, element_id, open_after))
    })
}

pub(super) fn apply_trigger_open_response(
    trigger: &mut ResponseExt,
    toggled: bool,
    open_after: bool,
) {
    trigger.set_activated(toggled && open_after);
    trigger.set_deactivated(toggled && !open_after);
    trigger.set_deactivated_after_edit(false);
}
