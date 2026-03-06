pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    checked: Option<Model<bool>>,
}

fn checked_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.checked {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.checked = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Switch::new(checked_model(cx))
        .a11y_label("Airplane mode")
        .into_element(cx)
        .test_id("ui-gallery-switch-usage")
}
// endregion: example
