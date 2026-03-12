pub const SOURCE: &str = include_str!("long_content.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::time::Duration;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::TooltipProvider::new()
        .delay(Duration::ZERO)
        .timeout_duration(Duration::from_millis(400))
        .with(cx, |cx| {
            let text = shadcn::TooltipContent::text(
                cx,
                "This tooltip demonstrates long content wrapping at the max width boundary without collapsing to min-content.",
            )
            .test_id("ui-gallery-tooltip-long-content-text");

            vec![shadcn::Tooltip::new(
                shadcn::Button::new("Hover long")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-tooltip-long-content-trigger")
                    .into_element(cx),
                shadcn::TooltipContent::new(vec![text]).into_element(cx),
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
