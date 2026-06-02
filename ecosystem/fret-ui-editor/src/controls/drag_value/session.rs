use fret_core::Px;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::{InsetStyle, LayoutStyle, Length, Overflow, PositionStyle, SizeStyle};

use super::{DragValueOutcome, OnDragValueOutcome};
use crate::controls::numeric_input::NumericInputOutcome;

pub(super) fn hidden_layout(mut layout: LayoutStyle) -> LayoutStyle {
    layout.size = SizeStyle {
        width: Length::Px(Px(0.0)),
        height: Length::Px(Px(0.0)),
        min_width: Some(Length::Px(Px(0.0))),
        min_height: Some(Length::Px(Px(0.0))),
        ..Default::default()
    };
    layout.position = PositionStyle::Absolute;
    layout.inset = InsetStyle {
        top: Some(Px(0.0)).into(),
        left: Some(Px(0.0)).into(),
        ..Default::default()
    };
    layout.overflow = Overflow::Clip;
    layout
}

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
