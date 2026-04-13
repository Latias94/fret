//! Immediate-mode combo helpers.

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::UiHost;

use super::{ComboOptions, ComboResponse, SelectableOptions, UiWriterImUiFacadeExt};

pub(super) fn combo_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    preview: Arc<str>,
    options: ComboOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut super::ImUiFacade<'cx2, 'a2, H>),
) -> ComboResponse {
    let enabled = options.enabled && ui.with_cx_mut(|cx| !super::imui_is_disabled(cx));
    let popup_open = ui.popup_open_model(id);
    let open_before = ui.with_cx_mut(|cx| {
        cx.read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, v| *v)
            .unwrap_or(false)
    });
    let trigger_text = combo_trigger_text(label.as_ref(), preview.as_ref());

    let mut trigger = ui.push_id(format!("{id}.trigger"), |ui| {
        ui.selectable_with_options(
            trigger_text.clone(),
            SelectableOptions {
                enabled,
                focusable: options.focusable,
                selected: open_before,
                test_id: options.test_id.clone(),
                activate_shortcut: options.activate_shortcut,
                shortcut_repeat: options.shortcut_repeat,
                a11y_label: options
                    .a11y_label
                    .clone()
                    .or_else(|| Some(trigger_text.clone())),
                a11y_role: Some(SemanticsRole::ComboBox),
                ..Default::default()
            },
        )
    });

    if enabled && trigger.clicked() {
        if open_before {
            ui.close_popup(id);
        } else if let Some(anchor) = trigger.core.rect {
            ui.open_popup_at(id, anchor);
        }
    }

    let popup_opened = ui.begin_popup_menu_with_options(id, trigger.id, options.popup, f);
    if !enabled && popup_opened {
        ui.close_popup(id);
    }

    let open_after = ui.with_cx_mut(|cx| {
        cx.read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, v| *v)
            .unwrap_or(false)
    });
    let toggled = trigger.id.is_some_and(|element_id| {
        ui.with_cx_mut(|cx| super::model_value_changed_for(cx, element_id, open_after))
    });
    trigger.activated = toggled && open_after;
    trigger.deactivated = toggled && !open_after;
    trigger.deactivated_after_edit = false;

    ComboResponse {
        trigger,
        open: open_after,
        toggled,
    }
}

fn combo_trigger_text(label: &str, preview: &str) -> Arc<str> {
    if label.is_empty() {
        Arc::from(preview)
    } else {
        Arc::from(format!("{label}: {preview}"))
    }
}

#[cfg(test)]
mod tests {
    use super::combo_trigger_text;

    #[test]
    fn combo_trigger_text_formats_label_and_preview() {
        assert_eq!(&*combo_trigger_text("Theme", "Dark"), "Theme: Dark");
    }

    #[test]
    fn combo_trigger_text_uses_preview_only_when_label_is_empty() {
        assert_eq!(&*combo_trigger_text("", "Dark"), "Dark");
    }
}
