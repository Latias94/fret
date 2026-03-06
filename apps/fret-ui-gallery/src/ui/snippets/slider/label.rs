pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let control_id = "ui-gallery-slider-label-control";
    let values = shadcn::Slider::new_controllable(cx, None, || vec![50.0])
        .control_id(control_id)
        .range(0.0, 100.0)
        .step(1.0)
        .a11y_label("Volume")
        .test_id("ui-gallery-slider-label-control")
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx);

    shadcn::Field::new(vec![
        shadcn::FieldLabel::new("Volume")
            .for_control(control_id)
            .test_id("ui-gallery-slider-label-label")
            .into_element(cx),
        values,
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-slider-label")
}
// endregion: example
