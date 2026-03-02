pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::time::Duration;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::TooltipProvider::new()
        .delay(Duration::ZERO)
        .timeout_duration(Duration::from_millis(400))
        .with(cx, |cx| {
            vec![
                shadcn::Tooltip::new(
                    shadcn::Button::new("Hover")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-tooltip-demo-trigger")
                        .into_element(cx),
                    shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(
                        cx,
                        "Add to library",
                    )])
                    .into_element(cx),
                )
                .arrow(true)
                .arrow_test_id("ui-gallery-tooltip-demo-arrow")
                .side(shadcn::TooltipSide::Top)
                .panel_test_id("ui-gallery-tooltip-demo-panel")
                .into_element(cx)
                .test_id("ui-gallery-tooltip-demo"),
            ]
        })
        .into_iter()
        .next()
        .expect("tooltip provider returns one root element")
}
// endregion: example
