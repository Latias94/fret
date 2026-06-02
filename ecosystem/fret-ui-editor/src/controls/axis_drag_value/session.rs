use std::sync::Arc;

use fret_core::Px;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::{InsetStyle, LayoutStyle, Length, Overflow, PositionStyle, SizeStyle};
use fret_ui::{ElementContext, UiHost};

use super::model::{AxisDragValueOutcome, OnAxisDragValueOutcome};

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
