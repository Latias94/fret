//! Immediate-mode select helpers.

use std::sync::Arc;

use fret_ui::UiHost;

use super::{MenuItemOptions, ResponseExt, SelectOptions, UiWriterImUiFacadeExt};

pub(super) fn select_model_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    model: &fret_runtime::Model<Option<Arc<str>>>,
    items: &[Arc<str>],
    options: SelectOptions,
) -> ResponseExt {
    let model = model.clone();
    let enabled = options.enabled && ui.with_cx_mut(|cx| !super::imui_is_disabled(cx));

    let selected = ui.with_cx_mut(|cx| {
        cx.read_model(&model, fret_ui::Invalidation::Paint, |_app, v| v.clone())
            .unwrap_or(None)
    });

    let selected_label: Arc<str> = selected
        .clone()
        .or_else(|| options.placeholder.clone())
        .unwrap_or_else(|| Arc::from("Select..."));
    let trigger_text: Arc<str> = Arc::from(format!("{label}: {selected_label}"));

    let popup_scope_id: Arc<str> = options.popup_scope_id.clone().unwrap_or_else(|| {
        let base = options
            .test_id
            .as_deref()
            .map(str::to_owned)
            .unwrap_or_else(|| label.as_ref().to_string());
        let mut normalized = String::with_capacity(base.len());
        for ch in base.chars() {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                normalized.push(ch);
            } else {
                normalized.push('-');
            }
        }
        Arc::from(format!("imui-select-popup-{normalized}"))
    });
    let popup_open = ui.popup_open_model(popup_scope_id.as_ref());

    let trigger = ui.push_id(format!("{popup_scope_id}.trigger"), |ui| {
        ui.menu_item_with_options(
            trigger_text,
            MenuItemOptions {
                enabled,
                test_id: options.test_id.clone(),
                ..Default::default()
            },
        )
    });

    if enabled
        && trigger.clicked()
        && let Some(anchor) = trigger.core.rect
    {
        ui.open_popup_at(popup_scope_id.as_ref(), anchor);
    }

    let selected_before = selected.clone();
    let model_for_pick = model.clone();
    let popup_open_for_items = popup_open.clone();
    let trigger_test_id = options.test_id.clone();
    let popup_opened = ui.begin_popup_menu_with_options(
        popup_scope_id.as_ref(),
        trigger.id,
        options.popup,
        move |ui| {
            for (index, item) in items.iter().enumerate() {
                let checked = selected_before
                    .as_ref()
                    .is_some_and(|current| current.as_ref() == item.as_ref());
                let item_test_id = trigger_test_id
                    .as_ref()
                    .map(|id| Arc::from(format!("{id}.option.{index}")));
                let item_response = ui.menu_item_radio_with_options(
                    item.clone(),
                    checked,
                    MenuItemOptions {
                        test_id: item_test_id,
                        ..Default::default()
                    },
                );
                if item_response.clicked() {
                    if !checked {
                        let next_value = Some(item.clone());
                        let _ = ui
                            .cx_mut()
                            .app
                            .models_mut()
                            .update(&model_for_pick, |value| *value = next_value.clone());
                    }
                    let _ = ui
                        .cx_mut()
                        .app
                        .models_mut()
                        .update(&popup_open_for_items, |value| *value = false);
                }
            }
        },
    );

    if !enabled && popup_opened {
        ui.close_popup(popup_scope_id.as_ref());
    }

    let selected_now = ui.with_cx_mut(|cx| {
        cx.read_model(&model, fret_ui::Invalidation::Paint, |_app, v| v.clone())
            .unwrap_or(None)
    });

    let mut response = trigger;
    response.core.changed = enabled
        && response.id.is_some_and(|id| {
            ui.with_cx_mut(|cx| super::model_value_changed_for(cx, id, selected_now.clone()))
        });
    response
}
