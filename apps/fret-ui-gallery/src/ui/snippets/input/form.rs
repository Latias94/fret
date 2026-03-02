pub const SOURCE: &str = include_str!("form.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct Models {
    name: Option<Model<String>>,
    email: Option<Model<String>>,
    phone: Option<Model<String>>,
    address: Option<Model<String>>,
    country: Option<Model<Option<Arc<str>>>>,
    country_open: Option<Model<bool>>,
}

fn ensure_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> (
    Model<String>,
    Model<String>,
    Model<String>,
    Model<String>,
    Model<Option<Arc<str>>>,
    Model<bool>,
) {
    let state = cx.with_state(Models::default, |st| st.clone());
    match (
        state.name,
        state.email,
        state.phone,
        state.address,
        state.country,
        state.country_open,
    ) {
        (
            Some(name),
            Some(email),
            Some(phone),
            Some(address),
            Some(country),
            Some(country_open),
        ) => (name, email, phone, address, country, country_open),
        _ => {
            let models = cx.app.models_mut();
            let name = models.insert(String::new());
            let email = models.insert(String::new());
            let phone = models.insert(String::new());
            let address = models.insert(String::new());
            let country = models.insert(None::<Arc<str>>);
            let country_open = models.insert(false);

            cx.with_state(Models::default, |st| {
                st.name = Some(name.clone());
                st.email = Some(email.clone());
                st.phone = Some(phone.clone());
                st.address = Some(address.clone());
                st.country = Some(country.clone());
                st.country_open = Some(country_open.clone());
            });

            (name, email, phone, address, country, country_open)
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (name, email, phone, address, country, country_open) = ensure_models(cx);
    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(420.0));

    let country = shadcn::Select::new(country, country_open)
        .placeholder("Country")
        .items([
            shadcn::SelectItem::new("us", "United States"),
            shadcn::SelectItem::new("uk", "United Kingdom"),
            shadcn::SelectItem::new("ca", "Canada"),
        ])
        .into_element(cx);

    let row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4)
            .items_start(),
        |cx| {
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
        },
    );

    shadcn::FieldGroup::new([
        shadcn::Field::new([
            shadcn::FieldLabel::new("Name").into_element(cx),
            shadcn::Input::new(name)
                .a11y_label("Name")
                .placeholder("Evil Rabbit")
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::Field::new([
            shadcn::FieldLabel::new("Email").into_element(cx),
            shadcn::Input::new(email)
                .a11y_label("Email")
                .placeholder("john@example.com")
                .into_element(cx),
            shadcn::FieldDescription::new("We'll never share your email.").into_element(cx),
        ])
        .into_element(cx),
        row,
        shadcn::Field::new([
            shadcn::FieldLabel::new("Address").into_element(cx),
            shadcn::Input::new(address)
                .a11y_label("Address")
                .placeholder("123 Main St")
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::Field::new([
            shadcn::Button::new("Cancel")
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx),
            shadcn::Button::new("Submit").into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .into_element(cx),
    ])
    .refine_layout(max_w_sm)
    .into_element(cx)
    .test_id("ui-gallery-input-form")
}
// endregion: example
