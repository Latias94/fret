pub const SOURCE: &str = include_str!("invalid.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));
    let invalid_id = "ui-gallery-input-invalid-control";

    shadcn::Field::new([
        shadcn::FieldLabel::new("Invalid Input")
            .for_control(invalid_id)
            .into_element(cx),
        shadcn::Input::new(value)
            .control_id(invalid_id)
            .placeholder("Error")
            .aria_invalid(true)
            .into_element(cx),
        shadcn::FieldDescription::new("This field contains validation errors.")
            .for_control(invalid_id)
            .into_element(cx),
    ])
    .invalid(true)
    .refine_layout(max_w_xs)
    .into_element(cx)
    .test_id("ui-gallery-input-invalid")
}
// endregion: example
