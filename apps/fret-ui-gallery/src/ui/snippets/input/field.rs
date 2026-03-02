// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    value: Option<Model<String>>,
}

fn value_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.value = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = value_model(cx);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    shadcn::Field::new([
        shadcn::FieldLabel::new("Username").into_element(cx),
        shadcn::Input::new(value)
            .a11y_label("Username")
            .placeholder("Enter your username")
            .into_element(cx),
        shadcn::FieldDescription::new("Choose a unique username for your account.")
            .into_element(cx),
    ])
    .refine_layout(max_w_xs)
    .into_element(cx)
    .test_id("ui-gallery-input-field")
}
// endregion: example

