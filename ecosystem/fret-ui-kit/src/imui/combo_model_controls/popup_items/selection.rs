use std::sync::Arc;

use fret_ui::UiHost;

use super::ComboModelItemsInput;
use crate::imui::UiWriterImUiFacadeExt;

pub(super) fn commit_combo_model_item_selection<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    input: &ComboModelItemsInput<'_>,
    item: &Arc<str>,
    checked: bool,
) {
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
