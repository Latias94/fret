pub const SOURCE: &str = include_str!("invalid.rs");

// region: example
use super::{default_month, fixed_today};
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use time::Date;

const CONTROL_ID: &str = "ui-gallery-date-picker-invalid-control";

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let today = fixed_today();
    let open = cx.local_model_keyed("invalid_open", || false);
    let month = cx.local_model_keyed("invalid_month", || default_month(today));
    let selected = cx.local_model_keyed("invalid_selected", || None::<Date>);
    let control_id = ControlId::from(CONTROL_ID);

    let date_picker = shadcn::DatePicker::new(open, month, selected)
        .placeholder("Pick a due date")
        .control_id(control_id.clone())
        .required(true)
        .aria_invalid(true)
        .test_id_prefix("ui-gallery-date-picker-invalid")
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx);

    shadcn::Field::new([
        shadcn::FieldLabel::new("Due date")
            .for_control(control_id.clone())
            .test_id("ui-gallery-date-picker-invalid-label")
            .into_element(cx),
        date_picker,
        shadcn::FieldDescription::new("Required before scheduling.")
            .for_control(control_id.clone())
            .into_element(cx),
        shadcn::FieldError::new("Please select a date.")
            .for_control(control_id.clone())
            .into_element(cx)
            .test_id("ui-gallery-date-picker-invalid-error"),
    ])
    .invalid(true)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-date-picker-invalid")
}
// endregion: example
