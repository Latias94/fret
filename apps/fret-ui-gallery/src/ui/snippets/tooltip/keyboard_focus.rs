pub const SOURCE: &str = include_str!("keyboard_focus.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::time::Duration;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::TooltipProvider::new()
        .delay(Duration::ZERO)
        .timeout_duration(Duration::from_millis(400))
        .with(cx, |cx| {
            let focus_start = shadcn::Button::new("Focus Start")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-tooltip-focus-start")
                .into_element(cx);

            let focus_tooltip = shadcn::Tooltip::new(
                shadcn::Button::new("Focus Trigger")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-tooltip-focus-trigger")
                    .into_element(cx),
                shadcn::TooltipContent::new(vec![
                    shadcn::TooltipContent::text(cx, "Opens on keyboard focus")
                        .test_id("ui-gallery-tooltip-focus-text"),
                ])
                .into_element(cx),
            )
            .arrow(true)
            .arrow_test_id("ui-gallery-tooltip-focus-arrow")
            .side(shadcn::TooltipSide::Top)
            .panel_test_id("ui-gallery-tooltip-focus-panel")
            .into_element(cx)
            .test_id("ui-gallery-tooltip-focus");

            vec![
                ui::h_row(|_cx| vec![focus_start, focus_tooltip])
                    .gap(Space::N2)
                    .items_center()
                    .into_element(cx)
                    .test_id("ui-gallery-tooltip-focus-row"),
            ]
        })
        .into_iter()
        .next()
        .expect("tooltip provider returns one root element")
}
// endregion: example
