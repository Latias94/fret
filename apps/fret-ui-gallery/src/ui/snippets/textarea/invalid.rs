pub const SOURCE: &str = include_str!("invalid.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);
    let invalid_id = "ui-gallery-textarea-invalid-control";

    shadcn::Field::new([
        shadcn::FieldLabel::new("Message")
            .for_control(invalid_id)
            .into_element(cx),
        shadcn::Textarea::new(value)
            .control_id(invalid_id)
            .a11y_label("Message")
            .placeholder("Type your message here.")
            .aria_invalid(true)
            .test_id(invalid_id)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
        shadcn::FieldDescription::new("Please enter a valid message.")
            .for_control(invalid_id)
            .into_element(cx),
    ])
    .invalid(true)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-textarea-invalid")
}
// endregion: example
