use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::UiHost;

use super::super::{ComboResponse, ResponseExt, UiWriterImUiFacadeExt};

mod lifecycle;
mod selected;

pub(super) fn finish_combo_model_response<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    model: &Model<Option<Arc<str>>>,
    enabled: bool,
    combo: ComboResponse,
) -> ResponseExt {
    let selected_now = selected::read_combo_model_response_selected(ui, model);
    let mut response = combo.response();
    lifecycle::populate_combo_model_response_lifecycle(
        ui,
        &mut response,
        selected_now,
        lifecycle::ComboModelResponseLifecycleInput {
            enabled,
            toggled: combo.toggled,
            open: combo.open,
        },
    );
    response
}
