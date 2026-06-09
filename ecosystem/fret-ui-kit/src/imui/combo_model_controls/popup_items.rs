use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::UiHost;

use super::super::UiWriterImUiFacadeExt;

mod item;
mod selection;

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
        item::render_combo_model_item(
            ui,
            item::ComboModelItemInput {
                index,
                item,
                owner: &input,
            },
        );
    }
}
