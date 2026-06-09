use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::UiHost;

use crate::imui::UiWriterImUiFacadeExt;

pub(super) fn read_combo_model_response_selected<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    model: &Model<Option<Arc<str>>>,
) -> Option<Arc<str>> {
    ui.with_cx_mut(|cx| {
        cx.read_model(model, fret_ui::Invalidation::Paint, |_app, value| {
            value.clone()
        })
        .unwrap_or(None)
    })
}
