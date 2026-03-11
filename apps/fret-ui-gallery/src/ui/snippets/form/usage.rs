pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_kit::headless::form_state::FormState;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    form_state: Option<Model<FormState>>,
    username: Option<Model<String>>,
}

fn ensure_models<H: UiHost>(cx: &mut ElementContext<'_, H>) -> (Model<FormState>, Model<String>) {
    let state = cx.with_state(Models::default, |st| st.clone());

    let form_state = state.form_state.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(FormState::default());
        cx.with_state(Models::default, |st| st.form_state = Some(model.clone()));
        model
    });
    let username = state.username.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(String::new());
        cx.with_state(Models::default, |st| st.username = Some(model.clone()));
        model
    });

    (form_state, username)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (form_state, username) = ensure_models(cx);

    let username_field = shadcn::FormField::new(
        form_state,
        "username",
        [shadcn::Input::new(username)
            .placeholder("shadcn")
            .refine_layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx)],
    )
    .label("Username")
    .description("This is your public display name.")
    .into_element(cx);

    shadcn::Form::new([
        username_field,
        shadcn::Button::new("Submit")
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
    .into_element(cx)
    .test_id("ui-gallery-form-usage")
}
// endregion: example
