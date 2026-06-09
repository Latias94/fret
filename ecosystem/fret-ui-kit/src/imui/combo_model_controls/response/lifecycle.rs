use std::sync::Arc;

use fret_ui::UiHost;

use crate::imui::{ResponseExt, UiWriterImUiFacadeExt};

pub(super) struct ComboModelResponseLifecycleInput {
    pub(super) enabled: bool,
    pub(super) toggled: bool,
    pub(super) open: bool,
}

pub(super) fn populate_combo_model_response_lifecycle<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    response: &mut ResponseExt,
    selected_now: Option<Arc<str>>,
    input: ComboModelResponseLifecycleInput,
) {
    let changed = combo_model_response_changed(ui, response, selected_now, input.enabled);
    response.set_core_changed(changed);
    response.merge_edited(changed);
    response.merge_deactivated_after_edit(changed && input.toggled && !input.open);
}

fn combo_model_response_changed<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    response: &ResponseExt,
    selected_now: Option<Arc<str>>,
    enabled: bool,
) -> bool {
    enabled
        && response.id().is_some_and(|element_id| {
            ui.with_cx_mut(|cx| {
                crate::imui::model_value_changed_for(cx, element_id, selected_now.clone())
            })
        })
}
