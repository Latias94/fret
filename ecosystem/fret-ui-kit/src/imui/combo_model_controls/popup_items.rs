use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::UiHost;

use super::super::{SelectableOptions, UiWriterImUiFacadeExt};

pub(super) struct ComboModelItemsInput<'a> {
    pub(super) items: &'a [Arc<str>],
    pub(super) selected_before: Option<Arc<str>>,
    pub(super) model: Model<Option<Arc<str>>>,
    pub(super) popup_open: Model<bool>,
    pub(super) trigger_test_id: Option<Arc<str>>,
}

pub(super) fn render_combo_model_items<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    input: ComboModelItemsInput<'_>,
) {
    for (index, item) in input.items.iter().enumerate() {
        let checked = input
            .selected_before
            .as_ref()
            .is_some_and(|current| current.as_ref() == item.as_ref());
        let item_test_id = input
            .trigger_test_id
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
                let _ = ui.with_cx_mut(|cx| {
                    cx.app
                        .models_mut()
                        .update(&input.model, |value| *value = next_value.clone())
                });
            }
            let _ = ui.with_cx_mut(|cx| {
                cx.app
                    .models_mut()
                    .update(&input.popup_open, |open| *open = false)
            });
        }
    }
}
