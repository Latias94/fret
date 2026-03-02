// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    slider_values: Option<Model<Vec<f32>>>,
}

fn values_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<Vec<f32>> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.slider_values {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(vec![200.0, 800.0]);
            cx.with_state(Models::default, |st| st.slider_values = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let slider_values = values_model(cx);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    shadcn::Field::new([
        shadcn::FieldTitle::new("Price Range").into_element(cx),
        shadcn::FieldDescription::new("Set your budget range ($200-$800).").into_element(cx),
        shadcn::Slider::new(slider_values)
            .range(0.0, 1000.0)
            .step(10.0)
            .a11y_label("Price Range")
            .into_element(cx),
    ])
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-slider")
}
// endregion: example
