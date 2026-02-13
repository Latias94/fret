use super::super::super::super::*;

pub(in crate::ui) fn preview_toast(
    cx: &mut ElementContext<'_, App>,
    _last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let deprecated_card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Toast is deprecated").into_element(cx),
            shadcn::CardDescription::new(
                "The toast component is deprecated in shadcn/ui docs. Use Sonner instead.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![shadcn::typography::muted(
            cx,
            "This page intentionally keeps only the deprecation guidance to match upstream docs.",
        )])
        .into_element(cx),
        shadcn::CardFooter::new(vec![
            shadcn::Button::new("Open Sonner page")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_NAV_SONNER)
                .test_id("ui-gallery-toast-open-sonner")
                .into_element(cx),
        ])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-toast-deprecated");

    let centered_card = centered(cx, deprecated_card);

    vec![
        cx.text("A succinct message that is displayed temporarily."),
        centered_card,
    ]
}
