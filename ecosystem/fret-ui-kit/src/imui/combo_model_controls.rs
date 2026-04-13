//! Immediate-mode model-backed combo helpers.

use std::sync::Arc;

use fret_ui::UiHost;

use super::{
    ComboModelOptions, ComboOptions, ResponseExt, SelectableOptions, UiWriterImUiFacadeExt,
};

pub(super) fn combo_model_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    model: &fret_runtime::Model<Option<Arc<str>>>,
    items: &[Arc<str>],
    options: ComboModelOptions,
) -> ResponseExt {
    let model = model.clone();
    let enabled = options.enabled && ui.with_cx_mut(|cx| !super::imui_is_disabled(cx));

    let selected = ui.with_cx_mut(|cx| {
        cx.read_model(&model, fret_ui::Invalidation::Paint, |_app, v| v.clone())
            .unwrap_or(None)
    });

    let preview: Arc<str> = selected
        .clone()
        .or_else(|| options.placeholder.clone())
        .unwrap_or_else(|| Arc::from("Select..."));
    let popup_open = ui.popup_open_model(id);

    let selected_before = selected.clone();
    let model_for_pick = model.clone();
    let trigger_test_id = options.test_id.clone();
    let combo = ui.combo_with_options(
        id,
        label.clone(),
        preview,
        ComboOptions {
            enabled,
            focusable: options.focusable,
            a11y_label: options.a11y_label.clone(),
            test_id: options.test_id.clone(),
            popup: options.popup,
            activate_shortcut: None,
            shortcut_repeat: false,
        },
        move |ui| {
            for (index, item) in items.iter().enumerate() {
                let checked = selected_before
                    .as_ref()
                    .is_some_and(|current| current.as_ref() == item.as_ref());
                let item_test_id = trigger_test_id
                    .as_ref()
                    .map(|trigger_id| Arc::from(format!("{trigger_id}.option.{index}")));
                let item_response = ui.selectable_with_options(
                    item.clone(),
                    SelectableOptions {
                        selected: checked,
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
                        .update(&popup_open, |open| *open = false);
                }
            }
        },
    );

    let selected_now = ui.with_cx_mut(|cx| {
        cx.read_model(&model, fret_ui::Invalidation::Paint, |_app, v| v.clone())
            .unwrap_or(None)
    });

    let mut response = combo.trigger;
    response.core.changed = enabled
        && response.id.is_some_and(|element_id| {
            ui.with_cx_mut(|cx| {
                super::model_value_changed_for(cx, element_id, selected_now.clone())
            })
        });
    response
}
