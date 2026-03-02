// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    email_invalid: Option<Model<String>>,
}

fn email_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.email_invalid {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.email_invalid = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let email_invalid = email_model(cx);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    shadcn::Field::new([
        shadcn::FieldLabel::new("Email").into_element(cx),
        shadcn::Input::new(email_invalid)
            .placeholder("email@example.com")
            .a11y_label("Email")
            .aria_invalid(true)
            .into_element(cx),
        shadcn::FieldError::new("Enter a valid email address.").into_element(cx),
    ])
    .invalid(true)
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-validation-and-errors")
}
// endregion: example
