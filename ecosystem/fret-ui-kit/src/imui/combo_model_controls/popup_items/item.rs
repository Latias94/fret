use std::sync::Arc;

use fret_ui::UiHost;

use super::ComboModelItemsInput;
use crate::imui::{SelectableOptions, UiWriterImUiFacadeExt};

pub(super) struct ComboModelItemInput<'a, 'b> {
    pub(super) index: usize,
    pub(super) item: &'a Arc<str>,
    pub(super) owner: &'a ComboModelItemsInput<'b>,
}

pub(super) fn render_combo_model_item<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    input: ComboModelItemInput<'_, '_>,
) {
    let checked = combo_model_item_checked(input.owner.selected_before.as_ref(), input.item);
    let item_test_id = combo_model_item_test_id(input.owner.trigger_test_id.as_ref(), input.index);
    let item_response = ui.selectable_with_options(
        input.item.clone(),
        SelectableOptions {
            selected: checked,
            test_id: item_test_id,
            ..Default::default()
        },
    );

    if item_response.clicked() {
        super::selection::commit_combo_model_item_selection(ui, input.owner, input.item, checked);
    }
}

fn combo_model_item_checked(selected_before: Option<&Arc<str>>, item: &Arc<str>) -> bool {
    selected_before.is_some_and(|current| current.as_ref() == item.as_ref())
}

fn combo_model_item_test_id(trigger_test_id: Option<&Arc<str>>, index: usize) -> Option<Arc<str>> {
    trigger_test_id.map(|trigger_id| Arc::from(format!("{trigger_id}.option.{index}")))
}
