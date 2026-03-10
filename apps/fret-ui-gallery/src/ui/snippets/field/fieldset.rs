pub const SOURCE: &str = include_str!("fieldset.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::ui::UiElementSinkExt as _;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    street: Option<Model<String>>,
    city: Option<Model<String>>,
    zip: Option<Model<String>>,
}

fn ensure_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> (Model<String>, Model<String>, Model<String>) {
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
