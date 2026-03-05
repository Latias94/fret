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
    let username_id = "username";
    let password_id = "password";

    shadcn::FieldSet::new([shadcn::FieldGroup::new([
        shadcn::Field::new([
            shadcn::FieldLabel::new("Username")
                .for_control(username_id)
                .into_element(cx),
            shadcn::Input::new(username)
                .control_id(username_id)
                .placeholder("Max Leiter")
                .test_id("ui-gallery-field-input-username")
                .into_element(cx),
            shadcn::FieldDescription::new("Choose a unique username for your account.")
                .for_control(username_id)
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::Field::new([
            shadcn::FieldLabel::new("Password")
                .for_control(password_id)
                .into_element(cx),
            shadcn::FieldDescription::new("Must be at least 8 characters long.")
                .for_control(password_id)
                .into_element(cx),
            shadcn::Input::new(password)
                .control_id(password_id)
                .password()
                .placeholder("••••••••")
                .test_id("ui-gallery-field-input-password")
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
