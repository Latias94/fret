// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    street: Option<Model<String>>,
    city: Option<Model<String>>,
    zip: Option<Model<String>>,
}

fn ensure_models<H: UiHost>(cx: &mut ElementContext<'_, H>) -> (Model<String>, Model<String>, Model<String>) {
    let state = cx.with_state(Models::default, |st| st.clone());
    match (state.street, state.city, state.zip) {
        (Some(street), Some(city), Some(zip)) => (street, city, zip),
        _ => {
            let models = cx.app.models_mut();
            let street = models.insert(String::new());
            let city = models.insert(String::new());
            let zip = models.insert(String::new());
            cx.with_state(Models::default, |st| {
                st.street = Some(street.clone());
                st.city = Some(city.clone());
                st.zip = Some(zip.clone());
            });
            (street, city, zip)
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (street, city, zip) = ensure_models(cx);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    let row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4),
        |cx| {
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
        },
    );

    shadcn::FieldSet::new([
        shadcn::FieldLegend::new("Address Information").into_element(cx),
        shadcn::FieldDescription::new("We need your address to deliver your order.")
            .into_element(cx),
        shadcn::FieldGroup::new([
            shadcn::Field::new([
                shadcn::FieldLabel::new("Street Address").into_element(cx),
                shadcn::Input::new(street)
                    .placeholder("123 Main St")
                    .a11y_label("Street Address")
                    .into_element(cx),
            ])
            .into_element(cx),
            row,
        ])
        .into_element(cx),
    ])
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-fieldset")
}
// endregion: example

