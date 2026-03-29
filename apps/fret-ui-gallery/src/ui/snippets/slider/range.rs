pub const SOURCE: &str = include_str!("range.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Slider::new_controllable(cx, None, || vec![25.0, 50.0])
        .range(0.0, 100.0)
        .step(5.0)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .test_id_prefix("ui-gallery-slider-range")
        .into_element(cx)
}
// endregion: example
