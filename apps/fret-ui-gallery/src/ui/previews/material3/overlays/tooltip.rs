use super::super::super::super::*;

pub(in crate::ui) fn preview_material3_tooltip(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    let content = material3::TooltipProvider::new().with_elements(cx, |cx| {
        let outlined = material3::ButtonVariant::Outlined;

        let top = material3::PlainTooltip::new(
            material3::Button::new("Hover (Top)")
                .variant(outlined)
                .test_id("ui-gallery-material3-tooltip-top-trigger")
                .into_element(cx),
            "Plain tooltip (top)",
        )
        .side(material3::TooltipSide::Top)
        .into_element(cx);

        let right = material3::PlainTooltip::new(
            material3::Button::new("Hover (Right)")
                .variant(outlined)
                .test_id("ui-gallery-material3-tooltip-right-trigger")
                .into_element(cx),
            "Plain tooltip (right)",
        )
        .side(material3::TooltipSide::Right)
        .into_element(cx);

        let bottom = material3::PlainTooltip::new(
            material3::Button::new("Hover (Bottom)")
                .variant(outlined)
                .test_id("ui-gallery-material3-tooltip-bottom-trigger")
                .into_element(cx),
            "Plain tooltip (bottom)",
        )
        .side(material3::TooltipSide::Bottom)
        .into_element(cx);

        let left = material3::PlainTooltip::new(
            material3::Button::new("Hover (Left)")
                .variant(outlined)
                .test_id("ui-gallery-material3-tooltip-left-trigger")
                .into_element(cx),
            "Plain tooltip (left)",
        )
        .side(material3::TooltipSide::Left)
        .into_element(cx);

        let rich = material3::RichTooltip::new(
            material3::Button::new("Hover (Rich)")
                .variant(outlined)
                .test_id("ui-gallery-material3-rich-tooltip-trigger")
                .into_element(cx),
            "Rich tooltip supporting text (body medium).",
        )
        .title("Rich tooltip title")
        .side(material3::TooltipSide::Top)
        .into_element(cx);

        let rich_no_title = material3::RichTooltip::new(
            material3::Button::new("Hover (Rich / no title)")
                .variant(outlined)
                .test_id("ui-gallery-material3-rich-tooltip-no-title-trigger")
                .into_element(cx),
            "Rich tooltip supporting text only.",
        )
        .side(material3::TooltipSide::Bottom)
        .into_element(cx);

        vec![
                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N4)
                        .layout(LayoutRefinement::default().w_full()),
                    |_cx| [top, right, bottom, left],
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N4)
                        .layout(LayoutRefinement::default().w_full()),
                    |_cx| [rich, rich_no_title],
                ),
                cx.text("Note: Tooltip open delay is controlled via Material3 TooltipProvider (delay-group)."),
            ]
    });

    let card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Tooltip").into_element(cx),
            shadcn::CardDescription::new(
                "Tooltip MVP: delay group + hover intent + safe-hover corridor + token-driven styling (plain + rich).",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(content).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    vec![card]
}
