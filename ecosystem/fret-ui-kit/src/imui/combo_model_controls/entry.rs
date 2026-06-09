use std::sync::Arc;

use fret_ui::UiHost;

use super::super::{ComboModelOptions, ResponseExt, UiWriterImUiFacadeExt};
use super::{popup_items, response};

mod options;
mod state;

pub(in crate::imui) fn combo_model_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    model: &fret_runtime::Model<Option<Arc<str>>>,
    items: &[Arc<str>],
    options: ComboModelOptions,
) -> ResponseExt {
    let model = model.clone();
    let state = state::read_combo_model_entry_state(ui, id, &model, &options);

    let combo = ui.combo_with_options(
        id,
        label.clone(),
        state.preview,
        options::combo_model_trigger_options(&options, state.enabled),
        {
            let model = model.clone();
            let selected_before = state.selected.clone();
            let popup_open = state.popup_open.clone();
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

    response::finish_combo_model_response(ui, &model, state.enabled, combo)
}
