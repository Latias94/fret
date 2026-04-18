pub const SOURCE: &str = include_str!("disabled.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::Slider::new_controllable(cx, None, || vec![50.0])
        .range(0.0, 100.0)
        .step(1.0)
        .disabled(true)
        .a11y_label("Disabled slider")
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .test_id_prefix("ui-gallery-slider-disabled")
        .into_element(cx)
}
// endregion: example
