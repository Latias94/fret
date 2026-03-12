pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::UiCx;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    let values = cx.local_model_keyed("values", || vec![33.0]);

    shadcn::Slider::new(values)
        .range(0.0, 100.0)
        .step(1.0)
        .a11y_label("Slider")
        .into_element(cx)
        .test_id("ui-gallery-slider-usage")
}
// endregion: example
