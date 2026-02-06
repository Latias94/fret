use super::super::*;

pub(super) fn preview_tooltip(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    shadcn::TooltipProvider::new()
        .with_elements(cx, |cx| {
            let mk = |cx: &mut ElementContext<'_, App>, label: &str, side: shadcn::TooltipSide| {
                shadcn::Tooltip::new(
                    shadcn::Button::new(label)
                        .variant(shadcn::ButtonVariant::Outline)
                        .into_element(cx),
                    shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(
                        cx,
                        format!("Tooltip on {label}"),
                    )])
                    .into_element(cx),
                )
                .arrow(true)
                .side(side)
                .open_delay_frames(10)
                .close_delay_frames(10)
                .into_element(cx)
            };

            vec![
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            mk(cx, "Top", shadcn::TooltipSide::Top),
                            mk(cx, "Right", shadcn::TooltipSide::Right),
                            mk(cx, "Bottom", shadcn::TooltipSide::Bottom),
                            mk(cx, "Left", shadcn::TooltipSide::Left),
                        ]
                    },
                ),
                cx.text(
                    "Hover the buttons to validate hover intent, delay group, and overlay placement.",
                ),
            ]
        })
        .into_vec()
}
