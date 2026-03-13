pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::time::Duration;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::TooltipProvider::new()
        .delay(Duration::ZERO)
        .timeout_duration(Duration::from_millis(400))
        .with(cx, |cx| {
            let content = shadcn::TooltipContent::build(cx, |_cx| {
                [shadcn::TooltipContent::text("Add to library")]
            });

            vec![
                shadcn::Tooltip::new(
                    cx,
                    shadcn::Button::new("Hover")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-tooltip-demo-trigger"),
                    content,
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
