pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    value: Option<Model<Vec<f32>>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.with_state(Models::default, |st| st.value.clone());
    let value = match value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(vec![50.0]);
            cx.with_state(Models::default, |st| st.value = Some(model.clone()));
            model
        }
    };

    let control_id = ControlId::from("ui-gallery-slider-label");
    let slider = shadcn::Slider::new(value)
        .range(0.0, 100.0)
        .control_id(control_id.clone())
        .test_id("ui-gallery-slider-label")
        .into_element(cx);

    shadcn::FieldGroup::new([shadcn::Field::new([
        shadcn::FieldContent::new([
            shadcn::FieldLabel::new("Volume")
                .for_control(control_id.clone())
                .test_id("ui-gallery-slider-label-label")
                .into_element(cx),
            shadcn::FieldDescription::new("Click the label to focus the slider thumb.")
                .for_control(control_id.clone())
                .into_element(cx),
        ])
        .into_element(cx),
        slider,
    ])
    .into_element(cx)])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
}
// endregion: example
