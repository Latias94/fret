pub const SOURCE: &str = include_str!("input.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    text_input: Option<Model<String>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let text_input = cx.with_state(Models::default, |st| st.text_input.clone());
    let text_input = text_input.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(String::new());
        cx.with_state(Models::default, |st| st.text_input = Some(model.clone()));
        model
    });

    let max_w_md = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(Px(520.0));

    shadcn::Input::new(text_input)
        .a11y_label("Email")
        .placeholder("name@example.com")
        .refine_layout(max_w_md)
        .into_element(cx)
        .test_id("ui-gallery-form-input")
}
// endregion: example
