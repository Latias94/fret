// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::time::Duration;

fn make_tooltip<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    side: shadcn::TooltipSide,
    content: &'static str,
) -> AnyElement {
    shadcn::Tooltip::new(
        shadcn::Button::new(label)
            .variant(shadcn::ButtonVariant::Outline)
            .into_element(cx),
        shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(cx, content)]).into_element(cx),
    )
    .arrow(true)
    .side(side)
    .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::TooltipProvider::new()
        .delay(Duration::ZERO)
        .timeout_duration(Duration::from_millis(400))
        .with(cx, |cx| {
            vec![stack::hstack(
                cx,
                stack::HStackProps::default().gap(Space::N2).items_center(),
                |cx| {
                    vec![
                        make_tooltip(cx, "Left", shadcn::TooltipSide::Left, "Add to library"),
                        make_tooltip(cx, "Top", shadcn::TooltipSide::Top, "Add to library"),
                        make_tooltip(cx, "Bottom", shadcn::TooltipSide::Bottom, "Add to library"),
                        make_tooltip(cx, "Right", shadcn::TooltipSide::Right, "Add to library"),
                    ]
                },
            )
            .test_id("ui-gallery-tooltip-sides")]
        })
        .into_iter()
        .next()
        .expect("tooltip provider returns one root element")
}
// endregion: example

