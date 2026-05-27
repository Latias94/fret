//! Immediate-mode combo helpers.

mod trigger;

use std::sync::Arc;

use fret_ui::UiHost;

use super::label_identity::parse_label_identity;
use super::{ComboOptions, ComboResponse, UiWriterImUiFacadeExt};

pub(super) fn combo_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    preview: Arc<str>,
    options: ComboOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut super::ImUiFacade<'cx2, 'a2, H>),
) -> ComboResponse {
    let parts = parse_label_identity(label.as_ref());
    let label = Arc::<str>::from(parts.visible);
    let enabled = options.enabled && ui.with_cx_mut(|cx| !super::imui_is_disabled(cx));
    let popup_open = ui.popup_open_model(id);
    let open_before = ui.with_cx_mut(|cx| {
        cx.read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, value| {
            *value
        })
        .unwrap_or(false)
    });
    let trigger_options = trigger::ComboTriggerOptions {
        enabled,
        focusable: options.focusable,
        a11y_label: options.a11y_label.clone(),
        test_id: options.test_id.clone(),
        activate_shortcut: options.activate_shortcut,
        shortcut_repeat: options.shortcut_repeat,
        open: open_before,
    };
    let popup_options = options.popup;

    let mut trigger = trigger::combo_trigger(ui, id, label, preview, trigger_options);

    if enabled && trigger.clicked() {
        if open_before {
            ui.close_popup(id);
        } else if let Some(anchor) = trigger.rect() {
            ui.open_popup_at(id, anchor);
        }
    }

    let popup_opened = super::popup_overlay::begin_popup_menu_with_options(
        ui,
        id,
        trigger.id(),
        popup_options,
        false,
        f,
    );
    if !enabled && popup_opened {
        ui.close_popup(id);
    }

    let open_after = ui.with_cx_mut(|cx| {
        cx.read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, value| {
            *value
        })
        .unwrap_or(false)
    });
    let toggled = trigger.id().is_some_and(|element_id| {
        ui.with_cx_mut(|cx| super::model_value_changed_for(cx, element_id, open_after))
    });
    trigger.set_activated(toggled && open_after);
    trigger.set_deactivated(toggled && !open_after);
    trigger.set_deactivated_after_edit(false);

    ComboResponse {
        trigger,
        open: open_after,
        toggled,
    }
}

#[cfg(test)]
mod tests;
