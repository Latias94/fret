pub const SOURCE: &str = include_str!("long_content.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::time::Duration;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::TooltipProvider::new()
        .delay(Duration::ZERO)
        .timeout_duration(Duration::from_millis(400))
        .with(cx, |cx| {
            let content = shadcn::TooltipContent::build(cx, |_cx| {
                [shadcn::TooltipContent::text(
                    "This tooltip demonstrates long content wrapping at the max width boundary without collapsing to min-content.",
                )
                .test_id("ui-gallery-tooltip-long-content-text")]
            });

            vec![shadcn::Tooltip::new(
                cx,
                shadcn::Button::new("Hover long")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-tooltip-long-content-trigger"),
                content,
            )
            .arrow(true)
            .side(shadcn::TooltipSide::Top)
            .panel_test_id("ui-gallery-tooltip-long-content-panel")
            .into_element(cx)
            .test_id("ui-gallery-tooltip-long-content")]
        })
        .into_iter()
        .next()
        .expect("tooltip provider returns one root element")
}
// endregion: example
