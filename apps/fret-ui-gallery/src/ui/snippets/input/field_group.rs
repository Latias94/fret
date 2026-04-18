pub const SOURCE: &str = include_str!("field_group.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let name = cx.local_model_keyed("name", String::new);
    let email = cx.local_model_keyed("email", String::new);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));
    let name_id = "ui-gallery-input-field-group-name";
    let email_id = "ui-gallery-input-field-group-email";

    shadcn::field_group(|cx| {
        ui::children![
            cx;
            shadcn::Field::new([
                shadcn::FieldLabel::new("Name")
                    .for_control(name_id)
                    .into_element(cx),
                shadcn::Input::new(name)
                    .control_id(name_id)
                    .placeholder("Jordan Lee")
                    .into_element(cx),
            ]),
            shadcn::Field::new([
                shadcn::FieldLabel::new("Email")
                    .for_control(email_id)
                    .into_element(cx),
                shadcn::Input::new(email)
                    .control_id(email_id)
                    .placeholder("name@example.com")
                    .into_element(cx),
                shadcn::FieldDescription::new("We'll send updates to this address.")
                    .for_control(email_id)
                    .into_element(cx),
            ]),
            shadcn::Field::new([
                shadcn::Button::new("Reset")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
                shadcn::Button::new("Submit").into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal),
        ]
    })
    .refine_layout(max_w_xs)
    .into_element(cx)
    .test_id("ui-gallery-input-field-group")
}
// endregion: example
