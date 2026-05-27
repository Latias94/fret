use fret_ui::UiHost;

use super::super::{ResponseExt, UiWriterImUiFacadeExt, model_value_changed_for};

pub(super) fn merge_text_picker_pick_response<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
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
