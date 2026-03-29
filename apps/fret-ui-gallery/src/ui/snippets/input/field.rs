pub const SOURCE: &str = include_str!("field.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));
    let username_id = "ui-gallery-input-field-username";

    shadcn::Field::new([
        shadcn::FieldLabel::new("Username")
            .for_control(username_id)
            .into_element(cx),
        shadcn::Input::new(value)
            .control_id(username_id)
            .placeholder("Enter your username")
            .into_element(cx),
        shadcn::FieldDescription::new("Choose a unique username for your account.")
            .for_control(username_id)
            .into_element(cx),
    ])
    .refine_layout(max_w_xs)
    .into_element(cx)
    .test_id("ui-gallery-input-field")
}
// endregion: example
