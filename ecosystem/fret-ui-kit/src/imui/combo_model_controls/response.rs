use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::UiHost;

use super::super::{ComboResponse, ResponseExt, UiWriterImUiFacadeExt};

pub(super) fn finish_combo_model_response<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    model: &Model<Option<Arc<str>>>,
    enabled: bool,
    combo: ComboResponse,
) -> ResponseExt {
    let selected_now = ui.with_cx_mut(|cx| {
        cx.read_model(model, fret_ui::Invalidation::Paint, |_app, v| v.clone())
            .unwrap_or(None)
    });

    let mut response = combo.response();
    let changed = enabled
        && response.id().is_some_and(|element_id| {
            ui.with_cx_mut(|cx| {
                super::super::model_value_changed_for(cx, element_id, selected_now.clone())
            })
        });
    response.set_core_changed(changed);
    response.merge_edited(changed);
    response.merge_deactivated_after_edit(changed && combo.toggled && !combo.open);
    response
}
