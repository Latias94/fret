//! GradientEditor angle row owner.

use std::sync::Arc;

use fret_core::Px;
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use super::super::PropertyRow;
use super::super::property_row::{PropertyRowOptions, property_row_label_text};
use crate::controls::{DragValue, DragValueOptions};
use crate::primitives::NumericPresentation;

pub(super) fn gradient_angle_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    angle: Model<f64>,
    angle_test_id: Option<Arc<str>>,
) -> AnyElement {
    PropertyRow::new()
        .options(PropertyRowOptions {
            reset_slot_width: Some(Px(0.0)),
            status_slot_width: Some(Px(0.0)),
            ..Default::default()
        })
        .into_element(
            cx,
            |cx| property_row_label_text(cx, "Angle"),
            |cx| {
                DragValue::from_presentation(angle, NumericPresentation::<f64>::degrees(0))
                    .options(DragValueOptions {
                        test_id: angle_test_id,
                        ..Default::default()
                    })
                    .into_element(cx)
            },
            |_cx| None,
        )
}
