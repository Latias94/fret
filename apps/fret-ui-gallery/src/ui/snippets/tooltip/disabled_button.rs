pub const SOURCE: &str = include_str!("disabled_button.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::time::Duration;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::TooltipProvider::new()
        .delay(Duration::ZERO)
        .timeout_duration(Duration::from_millis(400))
        .with(cx, |cx| {
            let disabled_trigger = ui::h_row(|cx| {
                vec![
                    shadcn::Button::new("Disabled")
                        .variant(shadcn::ButtonVariant::Outline)
                        .disabled(true)
                        .into_element(cx),
                ]
            })
            .items_center()
            .into_element(cx);
            let tooltip_content = shadcn::TooltipContent::build(cx, |_cx| {
                [shadcn::TooltipContent::text(
                    "This feature is currently unavailable",
                )]
            });

            vec![
                shadcn::Tooltip::new(cx, disabled_trigger, tooltip_content)
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
