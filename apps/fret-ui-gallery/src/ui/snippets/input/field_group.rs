pub const SOURCE: &str = include_str!("field_group.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let name = cx.local_model_keyed("name", String::new);
    let email = cx.local_model_keyed("email", String::new);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    shadcn::field_group(|cx| {
        ui::children![
            cx;
            shadcn::Field::new([
                shadcn::FieldLabel::new("Name").into_element(cx),
                shadcn::Input::new(name)
                    .a11y_label("Name")
                    .placeholder("Jordan Lee")
                    .into_element(cx),
            ]),
            shadcn::Field::new([
                shadcn::FieldLabel::new("Email").into_element(cx),
                shadcn::Input::new(email)
                    .a11y_label("Email")
                    .placeholder("name@example.com")
                    .into_element(cx),
                shadcn::FieldDescription::new("We'll send updates to this address.").into_element(cx),
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
