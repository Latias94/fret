use super::super::super::super::super::*;

pub(in crate::ui) fn preview_separator(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
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

    let shell = |cx: &mut ElementContext<'_, App>, layout: LayoutRefinement, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                layout,
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let demo = {
        let top = stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N1).items_start(),
            |cx| {
                vec![
                    shadcn::typography::small(cx, "Radix Primitives"),
                    shadcn::typography::muted(cx, "An open-source UI component library."),
                ]
            },
        );

        let links = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N4)
                .items_center()
                .layout(LayoutRefinement::default().w_full().h_px(Px(20.0))),
            |cx| {
                vec![
                    cx.text("Blog"),
                    shadcn::Separator::new()
                        .orientation(shadcn::SeparatorOrientation::Vertical)
                        .flex_stretch_cross_axis(true)
                        .into_element(cx),
                    cx.text("Docs"),
                    shadcn::Separator::new()
                        .orientation(shadcn::SeparatorOrientation::Vertical)
                        .flex_stretch_cross_axis(true)
                        .into_element(cx),
                    cx.text("Source"),
                ]
            },
        );

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .layout(LayoutRefinement::default().w_full().max_w(Px(384.0))),
            |cx| {
                vec![
                    top,
                    shadcn::Separator::new()
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    links,
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-separator-demo"),
        );

        let card = shell(cx, LayoutRefinement::default(), content);
        let body = centered(cx, card);
        section(cx, "Demo", body)
    };

    let vertical = {
        let content = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N4)
                .items_center()
                .layout(LayoutRefinement::default().h_px(Px(20.0))),
            |cx| {
                vec![
                    cx.text("Blog"),
                    shadcn::Separator::new()
                        .orientation(shadcn::SeparatorOrientation::Vertical)
                        .flex_stretch_cross_axis(true)
                        .into_element(cx),
                    cx.text("Docs"),
                    shadcn::Separator::new()
                        .orientation(shadcn::SeparatorOrientation::Vertical)
                        .flex_stretch_cross_axis(true)
                        .into_element(cx),
                    cx.text("Source"),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-separator-vertical"),
        );

        let card = shell(cx, LayoutRefinement::default(), content);
        let body = centered(cx, card);
        section(cx, "Vertical", body)
    };

    let menu = {
        let menu_item =
            |cx: &mut ElementContext<'_, App>, title: &'static str, desc: &'static str| {
                stack::vstack(
                    cx,
                    stack::VStackProps::default().gap(Space::N1).items_start(),
                    move |cx| {
                        vec![
                            shadcn::typography::small(cx, title),
                            shadcn::typography::muted(cx, desc),
                        ]
                    },
                )
            };

        let content = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N3)
                .items_center()
                .layout(LayoutRefinement::default().w_full().max_w(Px(560.0))),
            |cx| {
                vec![
                    menu_item(cx, "Settings", "Manage preferences"),
                    shadcn::Separator::new()
                        .orientation(shadcn::SeparatorOrientation::Vertical)
                        .flex_stretch_cross_axis(true)
                        .into_element(cx),
                    menu_item(cx, "Account", "Profile & security"),
                    shadcn::Separator::new()
                        .orientation(shadcn::SeparatorOrientation::Vertical)
                        .flex_stretch_cross_axis(true)
                        .into_element(cx),
                    menu_item(cx, "Help", "Support & docs"),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-separator-menu"),
        );

        let card = shell(cx, LayoutRefinement::default(), content);
        let body = centered(cx, card);
        section(cx, "Menu", body)
    };

    let list = {
        let row = |cx: &mut ElementContext<'_, App>, key: &'static str, value: &'static str| {
            stack::hstack(
                cx,
                stack::HStackProps::default()
                    .justify_between()
                    .items_center()
                    .layout(LayoutRefinement::default().w_full()),
                move |cx| vec![cx.text(key), shadcn::typography::muted(cx, value)],
            )
        };

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full().max_w(Px(384.0))),
            |cx| {
                vec![
                    row(cx, "Item 1", "Value 1"),
                    shadcn::Separator::new().into_element(cx),
                    row(cx, "Item 2", "Value 2"),
                    shadcn::Separator::new().into_element(cx),
                    row(cx, "Item 3", "Value 3"),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-separator-list"),
        );

        let card = shell(cx, LayoutRefinement::default(), content);
        let body = centered(cx, card);
        section(cx, "List", body)
    };

    let rtl = {
        let content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N4)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full().max_w(Px(384.0))),
                    |cx| {
                        vec![
                            stack::vstack(
                                cx,
                                stack::VStackProps::default().gap(Space::N1).items_start(),
                                |cx| {
                                    vec![
                                        shadcn::typography::small(cx, "shadcn/ui"),
                                        shadcn::typography::muted(cx, "أساس نظام التصميم الخاص بك"),
                                    ]
                                },
                            ),
                            shadcn::Separator::new().into_element(cx),
                            shadcn::typography::muted(
                                cx,
                                "مجموعة مكونات مصممة بشكل جميل يمكنك تخصيصها وتوسيعها.",
                            ),
                        ]
                    },
                )
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-separator-rtl"),
        );

        let card = shell(cx, LayoutRefinement::default(), content);
        let body = centered(cx, card);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("Visually or semantically separates content."),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, vertical, menu, list, rtl]
        }),
    ]
}
