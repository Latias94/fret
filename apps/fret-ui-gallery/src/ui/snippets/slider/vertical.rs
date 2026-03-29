pub const SOURCE: &str = include_str!("vertical.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::primitives::slider::SliderOrientation;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(_cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    ui::h_flex(|cx| {
        vec![
            shadcn::Slider::new_controllable(cx, None, || vec![50.0])
                .range(0.0, 100.0)
                .step(1.0)
                .orientation(SliderOrientation::Vertical)
                .refine_layout(LayoutRefinement::default().h_px(Px(160.0)))
                .test_id_prefix("ui-gallery-slider-vertical")
                .into_element(cx),
            shadcn::Slider::new_controllable(cx, None, || vec![25.0])
                .range(0.0, 100.0)
                .step(1.0)
                .orientation(SliderOrientation::Vertical)
                .refine_layout(LayoutRefinement::default().h_px(Px(160.0)))
                .test_id_prefix("ui-gallery-slider-vertical-secondary")
                .into_element(cx),
        ]
    })
    .gap(Space::N6)
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .items_center()
}
// endregion: example
