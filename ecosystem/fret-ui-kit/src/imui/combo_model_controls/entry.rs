use std::sync::Arc;

use fret_ui::UiHost;

use super::super::{ComboModelOptions, ComboOptions, ResponseExt, UiWriterImUiFacadeExt};
use super::{popup_items, response};

pub(in crate::imui) fn combo_model_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    model: &fret_runtime::Model<Option<Arc<str>>>,
    items: &[Arc<str>],
    options: ComboModelOptions,
) -> ResponseExt {
    let model = model.clone();
    let enabled = options.enabled && ui.with_cx_mut(|cx| !super::super::imui_is_disabled(cx));

    let selected = ui.with_cx_mut(|cx| {
        cx.read_model(&model, fret_ui::Invalidation::Paint, |_app, v| v.clone())
            .unwrap_or(None)
    });

    let preview: Arc<str> = selected
        .clone()
        .or_else(|| options.placeholder.clone())
        .unwrap_or_else(|| Arc::from("Select..."));
    let popup_open = ui.popup_open_model(id);

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
            activate_shortcut: options.activate_shortcut,
            shortcut_repeat: options.shortcut_repeat,
        },
        {
            let model = model.clone();
            let selected_before = selected.clone();
            let popup_open = popup_open.clone();
            let trigger_test_id = options.test_id.clone();
            move |ui| {
                popup_items::render_combo_model_items(
                    ui,
                    popup_items::ComboModelItemsInput {
                        items,
                        selected_before: selected_before.clone(),
                        model: model.clone(),
                        popup_open: popup_open.clone(),
                        trigger_test_id: trigger_test_id.clone(),
                    },
                );
            }
        },
    );

    response::finish_combo_model_response(ui, &model, enabled, combo)
}
