pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::Slider::new_controllable(cx, None, || vec![50.0])
        .range(0.0, 100.0)
        .step(1.0)
        .a11y_label("Slider")
        .refine_layout(LayoutRefinement::default().w_percent(60.0))
        .test_id_prefix("ui-gallery-slider-single")
        .into_element(cx)
}
// endregion: example
