pub const SOURCE: &str = include_str!("input.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    username: Option<Model<String>>,
    password: Option<Model<String>>,
}

fn ensure_models<H: UiHost>(cx: &mut ElementContext<'_, H>) -> (Model<String>, Model<String>) {
    let state = cx.with_state(Models::default, |st| st.clone());
    match (state.username, state.password) {
        (Some(username), Some(password)) => (username, password),
        _ => {
            let models = cx.app.models_mut();
            let username = models.insert(String::new());
            let password = models.insert(String::new());
            cx.with_state(Models::default, |st| {
                st.username = Some(username.clone());
                st.password = Some(password.clone());
            });
            (username, password)
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (username, password) = ensure_models(cx);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    shadcn::FieldSet::new([shadcn::FieldGroup::new([
        shadcn::Field::new([
            shadcn::FieldLabel::new("Username").into_element(cx),
            shadcn::Input::new(username)
                .placeholder("Max Leiter")
                .a11y_label("Username")
                .into_element(cx),
            shadcn::FieldDescription::new("Choose a unique username.").into_element(cx),
        ])
        .into_element(cx),
        shadcn::Field::new([
            shadcn::FieldLabel::new("Password").into_element(cx),
            shadcn::FieldDescription::new("Must be at least 8 characters long.").into_element(cx),
            shadcn::Input::new(password)
                .placeholder("••••••••")
                .a11y_label("Password")
                .into_element(cx),
        ])
        .into_element(cx),
    ])
    .into_element(cx)])
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-input")
}
// endregion: example
