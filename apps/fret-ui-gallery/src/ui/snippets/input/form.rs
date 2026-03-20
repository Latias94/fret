pub const SOURCE: &str = include_str!("form.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let name = cx.local_model_keyed("name", String::new);
    let email = cx.local_model_keyed("email", String::new);
    let phone = cx.local_model_keyed("phone", String::new);
    let address = cx.local_model_keyed("address", String::new);
    let country = cx.local_model_keyed("country", || None::<Arc<str>>);
    let country_open = cx.local_model_keyed("country_open", || false);
    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(420.0));

    let country = shadcn::Select::new(country, country_open)
        .value(shadcn::SelectValue::new().placeholder("Country"))
        .items([
            shadcn::SelectItem::new("us", "United States"),
            shadcn::SelectItem::new("uk", "United Kingdom"),
            shadcn::SelectItem::new("ca", "Canada"),
        ])
        .into_element(cx);

    let row = ui::h_flex(|cx| {
        vec![
            shadcn::Field::new([
                shadcn::FieldLabel::new("Phone").into_element(cx),
                shadcn::Input::new(phone)
                    .a11y_label("Phone")
                    .placeholder("+1 (555) 123-4567")
                    .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
            shadcn::Field::new([shadcn::FieldLabel::new("Country").into_element(cx), country])
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .gap(Space::N4)
    .items_start()
    .into_element(cx);

    shadcn::field_group(|cx| {
        ui::children![
            cx;
            shadcn::Field::new([
                shadcn::FieldLabel::new("Name").into_element(cx),
                shadcn::Input::new(name)
                    .a11y_label("Name")
                    .placeholder("Evil Rabbit")
                    .into_element(cx),
            ]),
            shadcn::Field::new([
                shadcn::FieldLabel::new("Email").into_element(cx),
                shadcn::Input::new(email)
                    .a11y_label("Email")
                    .placeholder("john@example.com")
                    .into_element(cx),
                shadcn::FieldDescription::new("We'll never share your email with anyone.")
                    .into_element(cx),
            ]),
            row,
            shadcn::Field::new([
                shadcn::FieldLabel::new("Address").into_element(cx),
                shadcn::Input::new(address)
                    .a11y_label("Address")
                    .placeholder("123 Main St")
                    .into_element(cx),
            ]),
            shadcn::Field::new([
                shadcn::Button::new("Cancel")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
                shadcn::Button::new("Submit").into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal),
        ]
    })
    .refine_layout(max_w_sm)
    .into_element(cx)
    .test_id("ui-gallery-input-form")
}
// endregion: example
