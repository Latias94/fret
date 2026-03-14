pub const SOURCE: &str = include_str!("flex_1_items.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::ButtonGroup::new([
        shadcn::Button::new("Overview")
            .variant(shadcn::ButtonVariant::Outline)
            .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
            .test_id("ui-gallery-button-group-flex1-overview")
            .into(),
        shadcn::Button::new("Analytics")
            .variant(shadcn::ButtonVariant::Outline)
            .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
            .test_id("ui-gallery-button-group-flex1-analytics")
            .into(),
        shadcn::Button::new("Reports")
            .variant(shadcn::ButtonVariant::Outline)
            .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
            .test_id("ui-gallery-button-group-flex1-reports")
            .into(),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(560.0)))
    .into_element(cx)
    .test_id("ui-gallery-button-group-flex1")
}

// endregion: example
