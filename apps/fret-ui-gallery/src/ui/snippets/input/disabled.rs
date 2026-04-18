pub const SOURCE: &str = include_str!("disabled.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));
    let email_id = "ui-gallery-input-disabled-email";

    shadcn::Field::new([
        shadcn::FieldLabel::new("Email")
            .for_control(email_id)
            .into_element(cx),
        shadcn::Input::new(value)
            .control_id(email_id)
            .placeholder("Email")
            .disabled(true)
            .into_element(cx),
        shadcn::FieldDescription::new("This field is currently disabled.")
            .for_control(email_id)
            .into_element(cx),
    ])
    .disabled(true)
    .refine_layout(max_w_xs)
    .into_element(cx)
    .test_id("ui-gallery-input-disabled")
}
// endregion: example
