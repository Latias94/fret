pub const SOURCE: &str = include_str!("fieldset.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::ui::UiElementSinkExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let street = cx.local_model_keyed("street", String::new);
    let city = cx.local_model_keyed("city", String::new);
    let zip = cx.local_model_keyed("zip", String::new);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    let row = ui::h_flex(|cx| {
        vec![
            shadcn::Field::new([
                shadcn::FieldLabel::new("City").into_element(cx),
                shadcn::Input::new(city)
                    .placeholder("New York")
                    .a11y_label("City")
                    .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
            shadcn::Field::new([
                shadcn::FieldLabel::new("Postal Code").into_element(cx),
                shadcn::Input::new(zip)
                    .placeholder("90502")
                    .a11y_label("Postal Code")
                    .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .gap(Space::N4)
    .into_element(cx);

    shadcn::FieldSet::build(|cx, out| {
        out.push_ui(cx, shadcn::FieldLegend::new("Address Information"));
        out.push_ui(
            cx,
            shadcn::FieldDescription::new("We need your address to deliver your order."),
        );
        out.push_ui(
            cx,
            shadcn::FieldGroup::build(|cx, out| {
                out.push_ui(
                    cx,
                    shadcn::Field::build(|cx, out| {
                        out.push_ui(cx, shadcn::FieldLabel::new("Street Address"));
                        out.push_ui(
                            cx,
                            shadcn::Input::new(street)
                                .placeholder("123 Main St")
                                .a11y_label("Street Address"),
                        );
                    }),
                );
                out.push(row);
            }),
        );
    })
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-fieldset")
}
// endregion: example
