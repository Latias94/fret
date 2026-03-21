pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let values = cx.local_model_keyed("ui-gallery-slider-usage-values", || vec![33.0]);

    shadcn::slider(values)
        .range(0.0, 100.0)
        .step(1.0)
        .a11y_label("Slider")
        .test_id_prefix("ui-gallery-slider-usage")
        .into_element(cx)
}
// endregion: example
