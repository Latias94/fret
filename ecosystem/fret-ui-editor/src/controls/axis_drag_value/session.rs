use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::{ElementContext, UiHost};

use super::model::{AxisDragValueOutcome, OnAxisDragValueOutcome};

pub(super) fn emit_axis_drag_value_outcome(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    on_outcome: Option<&OnAxisDragValueOutcome>,
    outcome: AxisDragValueOutcome,
) {
    if let Some(cb) = on_outcome {
        cb(host, action_cx, outcome);
    }
}

#[track_caller]
pub(super) fn draft_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    cx.local_model(String::new)
}

#[track_caller]
pub(super) fn error_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<Option<Arc<str>>> {
    cx.local_model(|| None::<Arc<str>>)
}
