pub const SOURCE: &str = include_str!("fieldset.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let street = cx.local_model_keyed("street", String::new);
    let city = cx.local_model_keyed("city", String::new);
    let zip = cx.local_model_keyed("zip", String::new);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));
    let street_id = "ui-gallery-field-fieldset-street";
    let city_id = "ui-gallery-field-fieldset-city";
    let zip_id = "ui-gallery-field-fieldset-zip";

    let row = ui::h_flex(|cx| {
        vec![
            shadcn::Field::new(ui::children![
                cx;
                shadcn::FieldLabel::new("City").for_control(city_id),
                shadcn::Input::new(city)
                    .control_id(city_id)
                    .placeholder("New York")
                    .a11y_label("City"),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
            shadcn::Field::new(ui::children![
                cx;
                shadcn::FieldLabel::new("Postal Code").for_control(zip_id),
                shadcn::Input::new(zip)
                    .control_id(zip_id)
                    .placeholder("90502")
                    .a11y_label("Postal Code"),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .gap(Space::N4)
    .into_element(cx);

    shadcn::field_set(|cx| {
        ui::children![
            cx;
            shadcn::FieldLegend::new("Address Information"),
            shadcn::FieldDescription::new("We need your address to deliver your order."),
            shadcn::field_group(|cx| {
                ui::children![
                    cx;
                    shadcn::Field::new(ui::children![
                        cx;
                        shadcn::FieldLabel::new("Street Address").for_control(street_id),
                        shadcn::Input::new(street)
                            .control_id(street_id)
                            .placeholder("123 Main St")
                            .a11y_label("Street Address"),
                    ]),
                    row,
                ]
            }),
        ]
    })
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-fieldset")
}
// endregion: example
