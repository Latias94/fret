// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::time::Duration;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::TooltipProvider::new()
        .delay(Duration::ZERO)
        .timeout_duration(Duration::from_millis(400))
        .with(cx, |cx| {
            let disabled_trigger =
                stack::hstack(cx, stack::HStackProps::default().items_center(), |cx| {
                    vec![
                        shadcn::Button::new("Disabled")
                            .variant(shadcn::ButtonVariant::Outline)
                            .disabled(true)
                            .into_element(cx),
                    ]
                });

            vec![
                shadcn::Tooltip::new(
                    disabled_trigger,
                    shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(
                        cx,
                        "This feature is currently unavailable",
                    )])
                    .into_element(cx),
                )
                .side(shadcn::TooltipSide::Top)
                .into_element(cx)
                .test_id("ui-gallery-tooltip-disabled"),
            ]
        })
        .into_iter()
        .next()
        .expect("tooltip provider returns one root element")
}
// endregion: example
