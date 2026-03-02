pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            shadcn::ButtonGroup::new([
                shadcn::Button::new("التالي")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into(),
                shadcn::Button::new("السابق")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into(),
            ])
            .into_element(cx)
        },
    )
    .test_id("ui-gallery-button-group-rtl")
}

// endregion: example
