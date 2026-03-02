// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    name: Option<Model<String>>,
    email: Option<Model<String>>,
}

fn ensure_models<H: UiHost>(cx: &mut ElementContext<'_, H>) -> (Model<String>, Model<String>) {
    let state = cx.with_state(Models::default, |st| st.clone());
    match (state.name, state.email) {
        (Some(name), Some(email)) => (name, email),
        _ => {
            let models = cx.app.models_mut();
            let name = models.insert(String::new());
            let email = models.insert(String::new());
            cx.with_state(Models::default, |st| {
                st.name = Some(name.clone());
                st.email = Some(email.clone());
            });
            (name, email)
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (name, email) = ensure_models(cx);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    shadcn::FieldGroup::new([
        shadcn::Field::new([
            shadcn::FieldLabel::new("Name").into_element(cx),
            shadcn::Input::new(name)
                .a11y_label("Name")
                .placeholder("Jordan Lee")
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::Field::new([
            shadcn::FieldLabel::new("Email").into_element(cx),
            shadcn::Input::new(email)
                .a11y_label("Email")
                .placeholder("name@example.com")
                .into_element(cx),
            shadcn::FieldDescription::new("We'll send updates to this address.").into_element(cx),
        ])
        .into_element(cx),
        shadcn::Field::new([
            shadcn::Button::new("Reset")
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx),
            shadcn::Button::new("Submit").into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .into_element(cx),
    ])
    .refine_layout(max_w_xs)
    .into_element(cx)
    .test_id("ui-gallery-input-field-group")
}
// endregion: example
