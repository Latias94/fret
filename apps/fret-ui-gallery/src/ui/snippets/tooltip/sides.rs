pub const SOURCE: &str = include_str!("sides.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::time::Duration;

fn make_tooltip<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    side: shadcn::TooltipSide,
    content: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let tooltip_content =
        shadcn::TooltipContent::build(cx, |_cx| [shadcn::TooltipContent::text(content)]);
    shadcn::Tooltip::new(
        cx,
        shadcn::Button::new(label).variant(shadcn::ButtonVariant::Outline),
        tooltip_content,
    )
    .arrow(true)
    .side(side)
    .into_element(cx)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::TooltipProvider::new()
        .delay(Duration::ZERO)
        .timeout_duration(Duration::from_millis(400))
        .with(cx, |cx| {
            vec![
                ui::h_row(|cx| {
                    vec![
                        make_tooltip(cx, "Left", shadcn::TooltipSide::Left, "Add to library")
                            .into_element(cx),
                        make_tooltip(cx, "Top", shadcn::TooltipSide::Top, "Add to library")
                            .into_element(cx),
                        make_tooltip(cx, "Bottom", shadcn::TooltipSide::Bottom, "Add to library")
                            .into_element(cx),
                        make_tooltip(cx, "Right", shadcn::TooltipSide::Right, "Add to library")
                            .into_element(cx),
                    ]
                })
                .gap(Space::N2)
                .items_center()
                .into_element(cx)
                .test_id("ui-gallery-tooltip-sides"),
            ]
        })
        .into_iter()
        .next()
        .expect("tooltip provider returns one root element")
}
// endregion: example
