pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_app::App;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default, Clone)]
struct SliderModels {
    values: Option<Model<Vec<f32>>>,
}

fn ensure_values(cx: &mut ElementContext<'_, App>) -> Model<Vec<f32>> {
    let state = cx.with_state(SliderModels::default, |st| st.clone());
    match state.values {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(vec![33.0]);
            cx.with_state(SliderModels::default, |st| st.values = Some(model.clone()));
            model
        }
    }
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    shadcn::Slider::new(ensure_values(cx))
        .range(0.0, 100.0)
        .step(1.0)
        .a11y_label("Slider")
        .into_element(cx)
        .test_id("ui-gallery-slider-usage")
}
// endregion: example
