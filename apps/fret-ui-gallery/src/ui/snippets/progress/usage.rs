pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_app::App;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct ProgressModels {
    value: Option<Model<f32>>,
}

fn ensure_value(cx: &mut ElementContext<'_, App>) -> Model<f32> {
    let state = cx.with_state(ProgressModels::default, |st| st.clone());
    match state.value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(33.0);
            cx.with_state(ProgressModels::default, |st| st.value = Some(model.clone()));
            model
        }
    }
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    shadcn::Progress::new(ensure_value(cx))
        .a11y_label("Progress")
        .into_element(cx)
        .test_id("ui-gallery-progress-usage")
}
// endregion: example
