use fret_ui::UiHost;

use super::super::{
    InputTextPickerResponse, ResponseExt, UiWriterImUiFacadeExt, model_value_changed_for,
};
use super::popup::InputTextPickerPopupResult;

pub(super) fn finish_text_picker_response<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    model: &fret_runtime::Model<String>,
    mut input: ResponseExt,
    popup: InputTextPickerPopupResult,
) -> InputTextPickerResponse {
    let InputTextPickerPopupResult {
        opened,
        picked_index,
        picked,
    } = popup;
    merge_text_picker_pick_response(ui, model, &mut input, picked.is_some());
    InputTextPickerResponse {
        input,
        open: opened,
        picked_index,
        picked,
    }
}

fn merge_text_picker_pick_response<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    model: &fret_runtime::Model<String>,
    input: &mut ResponseExt,
    picked: bool,
) {
    if !picked {
        return;
    }

    let selected_now = ui.with_cx_mut(|cx| {
        cx.read_model(model, fret_ui::Invalidation::Paint, |_app, value| {
            value.clone()
        })
        .unwrap_or_default()
    });
    let picked_changed = input.id().is_some_and(|element_id| {
        ui.with_cx_mut(|cx| model_value_changed_for(cx, element_id, selected_now))
    });
    input.merge_core_changed(picked_changed);
    input.merge_edited(picked_changed);
    input.merge_deactivated_after_edit(picked_changed);
}
