use super::super::*;

pub(super) fn preview_carousel(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).clone();

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
        cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(760.0)),
            ),
            move |_cx| [body],
        )
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

    let slide = |cx: &mut ElementContext<'_, App>, idx: usize, caption: &'static str| {
        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new(format!("Slide {idx}")).into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![cx.text(caption)]).into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
    };

    let carousel = |cx: &mut ElementContext<'_, App>,
                    test_id: &'static str,
                    orientation: shadcn::CarouselOrientation,
                    basis: Px,
                    spacing: Space,
                    max_w: Px,
                    viewport_h: Option<Px>| {
        let items = vec![
            slide(cx, 1, "Drag to swipe or use previous/next controls."),
            slide(cx, 2, "Item sizing uses `item_basis_main_px`."),
            slide(
                cx,
                3,
                "Spacing uses `track_start_neg_margin` + `item_padding_start`.",
            ),
            slide(cx, 4, "Orientation can be horizontal or vertical."),
            slide(cx, 5, "Wrap with direction provider for RTL locales."),
        ];

        let mut base = shadcn::Carousel::new(items)
            .orientation(orientation)
            .item_basis_main_px(basis)
            .track_start_neg_margin(spacing)
            .item_padding_start(spacing)
            .test_id(test_id)
            .refine_layout(LayoutRefinement::default().w_full().max_w(max_w));

        if let Some(viewport_h) = viewport_h {
            base = base.refine_viewport_layout(LayoutRefinement::default().h_px(viewport_h));
        }

        base.into_element(cx)
            .attach_semantics(SemanticsDecoration::default().test_id(test_id))
    };

    let demo_content = carousel(
        cx,
        "ui-gallery-carousel-demo",
        shadcn::CarouselOrientation::Horizontal,
        Px(260.0),
        Space::N4,
        Px(360.0),
        None,
    );
    let demo = section_card(cx, "Demo", demo_content);

    let sizes_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Match docs `Sizes`: smaller basis shows more items, larger basis emphasizes one item.",
                ),
                carousel(
                    cx,
                    "ui-gallery-carousel-size-compact",
                    shadcn::CarouselOrientation::Horizontal,
                    Px(180.0),
                    Space::N4,
                    Px(360.0),
                    None,
                ),
                carousel(
                    cx,
                    "ui-gallery-carousel-size-wide",
                    shadcn::CarouselOrientation::Horizontal,
                    Px(280.0),
                    Space::N4,
                    Px(420.0),
                    None,
                ),
            ]
        },
    );
    let sizes = section_card(cx, "Sizes", sizes_content);

    let spacing_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Match docs `Spacing`: tune track negative margin + item start padding together.",
                ),
                carousel(
                    cx,
                    "ui-gallery-carousel-spacing-tight",
                    shadcn::CarouselOrientation::Horizontal,
                    Px(220.0),
                    Space::N1,
                    Px(360.0),
                    None,
                ),
                carousel(
                    cx,
                    "ui-gallery-carousel-spacing-loose",
                    shadcn::CarouselOrientation::Horizontal,
                    Px(220.0),
                    Space::N6,
                    Px(420.0),
                    None,
                ),
            ]
        },
    );
    let spacing = section_card(cx, "Spacing", spacing_content);

    let orientation_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Match docs `Orientation`: vertical mode uses item basis on the vertical main axis.",
                ),
                carousel(
                    cx,
                    "ui-gallery-carousel-orientation-horizontal",
                    shadcn::CarouselOrientation::Horizontal,
                    Px(220.0),
                    Space::N4,
                    Px(360.0),
                    None,
                ),
                carousel(
                    cx,
                    "ui-gallery-carousel-orientation-vertical",
                    shadcn::CarouselOrientation::Vertical,
                    Px(120.0),
                    Space::N2,
                    Px(360.0),
                    Some(Px(300.0)),
                ),
            ]
        },
    );
    let orientation = section_card(cx, "Orientation", orientation_content);

    let api_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Upstream exposes `setApi` for Embla events/options. Current Fret API focuses on deterministic swipe + buttons.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Follow-up can expose controlled index/event hooks once API contracts are stabilized.",
                ),
            ]
        },
    );
    let api = section_card(cx, "API", api_content);

    let plugins_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Upstream supports Embla plugins (e.g. autoplay). Fret currently does not expose plugin injection.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Keep this section as an explicit gap marker to avoid silent parity drift.",
                ),
            ]
        },
    );
    let plugins = section_card(cx, "Plugins", plugins_content);

    let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            carousel(
                cx,
                "ui-gallery-carousel-rtl",
                shadcn::CarouselOrientation::Horizontal,
                Px(260.0),
                Space::N4,
                Px(360.0),
                None,
            )
        },
    );
    let rtl = section_card(cx, "RTL", rtl_content);

    let preview_hint = shadcn::typography::muted(
        cx,
        "Preview follows shadcn Carousel docs flow: Demo -> Sizes -> Spacing -> Orientation -> API -> Plugins -> RTL.",
    );
    let component_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| {
            vec![
                preview_hint,
                demo,
                sizes,
                spacing,
                orientation,
                api,
                plugins,
                rtl,
            ]
        },
    );
    let component_panel = shell(cx, component_stack)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-carousel-component"));

    let code_block =
        |cx: &mut ElementContext<'_, App>, title: &'static str, snippet: &'static str| {
            shadcn::Card::new(vec![
                shadcn::CardHeader::new(vec![shadcn::CardTitle::new(title).into_element(cx)])
                    .into_element(cx),
                shadcn::CardContent::new(vec![ui::text_block(cx, snippet).into_element(cx)])
                    .into_element(cx),
            ])
            .into_element(cx)
        };

    let code_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                code_block(
                    cx,
                    "Basic",
                    r#"let carousel = shadcn::Carousel::new(items)
    .item_basis_main_px(Px(260.0))
    .refine_layout(LayoutRefinement::default().max_w(Px(360.0)))
    .into_element(cx);"#,
                ),
                code_block(
                    cx,
                    "Spacing + Orientation",
                    r#"shadcn::Carousel::new(items)
    .track_start_neg_margin(Space::N4)
    .item_padding_start(Space::N4)
    .orientation(shadcn::CarouselOrientation::Vertical)
    .refine_viewport_layout(LayoutRefinement::default().h_px(Px(300.0)))"#,
                ),
                code_block(
                    cx,
                    "RTL",
                    r#"with_direction_provider(LayoutDirection::Rtl, |cx| {
    shadcn::Carousel::new(items)
        .orientation(shadcn::CarouselOrientation::Horizontal)
        .into_element(cx)
})"#,
                ),
            ]
        },
    );
    let code_panel = shell(cx, code_stack);

    let notes_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::h4(cx, "Notes"),
                shadcn::typography::muted(
                    cx,
                    "`item_basis_main_px` defines the visible density contract; keep it explicit per page width.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Spacing parity with web examples depends on pairing negative track margin with item start padding.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Vertical orientation should always set viewport height explicitly to prevent clipping ambiguity.",
                ),
                shadcn::typography::muted(
                    cx,
                    "API/plugins gaps are tracked here intentionally so future Embla parity work remains discoverable.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-carousel",
        component_panel,
        code_panel,
        notes_panel,
    )
}
