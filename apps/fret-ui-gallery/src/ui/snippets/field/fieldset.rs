pub const SOURCE: &str = include_str!("fieldset.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let street = cx.local_model_keyed("street", String::new);
    let city = cx.local_model_keyed("city", String::new);
    let zip = cx.local_model_keyed("zip", String::new);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    let row = ui::h_flex(|cx| {
        vec![
            shadcn::Field::new(ui::children![
                cx;
                shadcn::FieldLabel::new("City"),
                shadcn::Input::new(city)
                    .placeholder("New York")
                    .a11y_label("City"),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
            shadcn::Field::new(ui::children![
                cx;
                shadcn::FieldLabel::new("Postal Code"),
                shadcn::Input::new(zip)
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
                        shadcn::FieldLabel::new("Street Address"),
                        shadcn::Input::new(street)
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
