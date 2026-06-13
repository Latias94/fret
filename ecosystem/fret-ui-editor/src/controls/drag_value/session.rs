use fret_ui::action::{ActionCx, UiActionHost};

use super::{DragValueOutcome, OnDragValueOutcome};
use crate::controls::numeric_input::NumericInputOutcome;

pub(super) fn drag_value_outcome_from_numeric_input(
    outcome: NumericInputOutcome,
) -> DragValueOutcome {
    match outcome {
        NumericInputOutcome::Committed => DragValueOutcome::Committed,
        NumericInputOutcome::Canceled => DragValueOutcome::Canceled,
    }
}

pub(super) fn emit_drag_value_outcome(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    on_outcome: Option<&OnDragValueOutcome>,
    outcome: DragValueOutcome,
) {
    if let Some(cb) = on_outcome {
        cb(host, action_cx, outcome);
    }
}
