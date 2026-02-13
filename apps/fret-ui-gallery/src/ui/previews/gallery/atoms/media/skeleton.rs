use super::super::super::super::super::*;

pub(in crate::ui) fn preview_skeleton(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full(),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let round = |cx: &mut ElementContext<'_, App>, size: f32| {
        shadcn::Skeleton::new()
            .refine_style(ChromeRefinement::default().rounded(Radius::Full))
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(Px(size))
                    .h_px(Px(size))
                    .flex_shrink_0(),
            )
            .into_element(cx)
    };

    let demo = {
        let text_lines = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_px(Px(250.0))),
            |cx| {
                vec![
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(200.0)))
                        .into_element(cx),
                ]
            },
        );

        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            |cx| vec![round(cx, 48.0), text_lines],
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-demo"),
        );

        let framed = shell(cx, row);
        let body = centered(cx, framed);
        section(cx, "Demo", body)
    };

    let avatar = {
        let text_lines = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_px(Px(150.0))),
            |cx| {
                vec![
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(100.0)))
                        .into_element(cx),
                ]
            },
        );

        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            |cx| vec![round(cx, 40.0), text_lines],
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-avatar"),
        );

        let framed = shell(cx, row);
        let body = centered(cx, framed);
        section(cx, "Avatar", body)
    };

    let card = {
        let demo_card = shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_px(Px(170.0)))
                    .into_element(cx),
                shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_px(Px(128.0)))
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![
                shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_full().h_px(Px(144.0)))
                    .into_element(cx),
            ])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-card"),
        );

        let body = centered(cx, demo_card);
        section(cx, "Card", body)
    };

    let text_section = {
        let text = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
            |cx| {
                vec![
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
                        .into_element(cx),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-text"),
        );

        let framed = shell(cx, text);
        let body = centered(cx, framed);
        section(cx, "Text", body)
    };

    let form = {
        let row = |cx: &mut ElementContext<'_, App>, label_w: Px| {
            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .layout(LayoutRefinement::default().w_full()),
                move |cx| {
                    vec![
                        shadcn::Skeleton::new()
                            .refine_layout(LayoutRefinement::default().w_px(label_w))
                            .into_element(cx),
                        shadcn::Skeleton::new()
                            .refine_layout(LayoutRefinement::default().w_full().h_px(Px(32.0)))
                            .into_element(cx),
                    ]
                },
            )
        };

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
            |cx| {
                vec![
                    row(cx, Px(80.0)),
                    row(cx, Px(96.0)),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(96.0)).h_px(Px(32.0)))
                        .into_element(cx),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-form"),
        );

        let framed = shell(cx, content);
        let body = centered(cx, framed);
        section(cx, "Form", body)
    };

    let table = {
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full().max_w(Px(420.0))),
            |cx| {
                (0..5)
                    .map(|_| {
                        stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .gap(Space::N4)
                                .items_center()
                                .layout(LayoutRefinement::default().w_full()),
                            |cx| {
                                vec![
                                    shadcn::Skeleton::new()
                                        .refine_layout(
                                            LayoutRefinement::default().flex_1().min_w_0(),
                                        )
                                        .into_element(cx),
                                    shadcn::Skeleton::new()
                                        .refine_layout(LayoutRefinement::default().w_px(Px(96.0)))
                                        .into_element(cx),
                                    shadcn::Skeleton::new()
                                        .refine_layout(LayoutRefinement::default().w_px(Px(80.0)))
                                        .into_element(cx),
                                ]
                            },
                        )
                    })
                    .collect::<Vec<_>>()
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-table"),
        );

        let framed = shell(cx, content);
        let body = centered(cx, framed);
        section(cx, "Table", body)
    };

    let rtl = {
        let content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let text_lines = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N2)
                        .layout(LayoutRefinement::default().w_px(Px(250.0))),
                    |cx| {
                        vec![
                            shadcn::Skeleton::new()
                                .refine_layout(LayoutRefinement::default().w_full())
                                .into_element(cx),
                            shadcn::Skeleton::new()
                                .refine_layout(LayoutRefinement::default().w_px(Px(200.0)))
                                .into_element(cx),
                        ]
                    },
                );

                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N4).items_center(),
                    |cx| vec![round(cx, 48.0), text_lines],
                )
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-rtl"),
        );

        let framed = shell(cx, content);
        let body = centered(cx, framed);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("Use to show a placeholder while content is loading."),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, avatar, card, text_section, form, table, rtl]
        }),
    ]
}
