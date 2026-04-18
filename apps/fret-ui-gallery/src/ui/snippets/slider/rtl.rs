pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::Slider::new_controllable(cx, None, || vec![75.0])
            .range(0.0, 100.0)
            .step(1.0)
            .a11y_label("RTL slider")
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
            .test_id_prefix("ui-gallery-slider-rtl")
            .into_element(cx)
    })
}
// endregion: example
