pub const SOURCE: &str = include_str!("rtl.rs");

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
        shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(cx, content)])
            .into_element(cx),
    )
    .arrow(true)
    .side(side)
    .into_element(cx)
}

fn make_tooltip_with_test_ids<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    trigger_test_id: &'static str,
    side: shadcn::TooltipSide,
    content: &'static str,
    panel_test_id: &'static str,
    text_test_id: &'static str,
) -> AnyElement {
    shadcn::Tooltip::new(
        shadcn::Button::new(label)
            .variant(shadcn::ButtonVariant::Outline)
            .test_id(trigger_test_id)
            .into_element(cx),
        shadcn::TooltipContent::new(vec![
            shadcn::TooltipContent::text(cx, content).test_id(text_test_id),
        ])
        .into_element(cx),
    )
    .arrow(true)
    .side(side)
    .panel_test_id(panel_test_id)
    .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::TooltipProvider::new()
        .delay(Duration::ZERO)
        .timeout_duration(Duration::from_millis(400))
        .with(cx, |cx| {
            vec![
                with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
                    ui::h_row(|cx| {
                        vec![
                            make_tooltip(
                                cx,
                                "يسار",
                                shadcn::TooltipSide::Left,
                                "إضافة إلى المكتبة",
                            ),
                            make_tooltip_with_test_ids(
                                cx,
                                "أعلى",
                                "ui-gallery-tooltip-rtl-top-trigger",
                                shadcn::TooltipSide::Top,
                                "إضافة إلى المكتبة",
                                "ui-gallery-tooltip-rtl-top-panel",
                                "ui-gallery-tooltip-rtl-top-text",
                            ),
                            make_tooltip(
                                cx,
                                "أسفل",
                                shadcn::TooltipSide::Bottom,
                                "إضافة إلى المكتبة",
                            ),
                            make_tooltip(
                                cx,
                                "يمين",
                                shadcn::TooltipSide::Right,
                                "إضافة إلى المكتبة",
                            ),
                        ]
                    })
                    .gap(Space::N2)
                    .items_center()
                    .into_element(cx)
                })
                .test_id("ui-gallery-tooltip-rtl"),
            ]
        })
        .into_iter()
        .next()
        .expect("tooltip provider returns one root element")
}
// endregion: example
