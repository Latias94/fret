use super::super::*;

pub(super) fn preview_aspect_ratio(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
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
                LayoutRefinement::default().w_full().max_w(Px(760.0)),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

    let ratio_example = |cx: &mut ElementContext<'_, App>,
                         ratio: f32,
                         max_w: Px,
                         ratio_label: &'static str,
                         caption: &'static str,
                         test_id: &'static str| {
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().h_full())
                .items_center()
                .justify_center()
                .gap(Space::N1),
            move |cx| {
                vec![
                    shadcn::typography::h4(cx, ratio_label),
                    shadcn::typography::muted(cx, caption),
                ]
            },
        );

        let (muted_bg, border) = cx.with_theme(|theme| {
            (
                theme.color_required("muted"),
                theme.color_required("border"),
            )
        });

        shadcn::AspectRatio::new(ratio, content)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .bg(ColorRef::Color(muted_bg))
                    .border_color(ColorRef::Color(border)),
            )
            .refine_layout(LayoutRefinement::default().w_full().max_w(max_w))
            .into_element(cx)
            .test_id(test_id)
    };

    let demo_content = ratio_example(
        cx,
        16.0 / 9.0,
        Px(384.0),
        "16:9",
        "Landscape media",
        "ui-gallery-aspect-ratio-demo",
    );
    let demo = section_card(cx, "Demo", demo_content);

    let square_content = ratio_example(
        cx,
        1.0,
        Px(192.0),
        "1:1",
        "Square media",
        "ui-gallery-aspect-ratio-square",
    );
    let square = section_card(cx, "Square", square_content);

    let portrait_content = ratio_example(
        cx,
        9.0 / 16.0,
        Px(160.0),
        "9:16",
        "Portrait media",
        "ui-gallery-aspect-ratio-portrait",
    );
    let portrait = section_card(cx, "Portrait", portrait_content);

    let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            ratio_example(
                cx,
                16.0 / 9.0,
                Px(384.0),
                "16:9",
                "RTL layout sample",
                "ui-gallery-aspect-ratio-rtl",
            )
        },
    );
    let rtl = section_card(cx, "RTL", rtl_content);

    let preview_hint = shadcn::typography::muted(
        cx,
        "Preview follows shadcn Aspect Ratio docs order (Demo, Square, Portrait, RTL).",
    );
    let component_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| vec![preview_hint, demo, square, portrait, rtl],
    );
    let component_panel = shell(cx, component_stack).test_id("ui-gallery-aspect-ratio-component");

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
                    "Basic Usage",
                    r#"let media = AspectRatio::new(16.0 / 9.0, content)
    .refine_layout(LayoutRefinement::default().max_w(Px(384.0)))
    .into_element(cx);"#,
                ),
                code_block(
                    cx,
                    "Square + Portrait",
                    r#"AspectRatio::new(1.0, square_content)
AspectRatio::new(9.0 / 16.0, portrait_content)"#,
                ),
                code_block(
                    cx,
                    "RTL",
                    r#"with_direction_provider(LayoutDirection::Rtl, |cx| {
    AspectRatio::new(16.0 / 9.0, content).into_element(cx)
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
                    "Use `AspectRatio` to lock geometry first, then style radius/border/background around it.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Pick ratio by content type: 16:9 for landscape previews, 1:1 for avatars/thumbnails, 9:16 for reels or short video cards.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Keep max width explicit on narrow ratios to avoid over-tall layouts in dense editor sidebars.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Validate RTL and constrained width together so captions and controls remain stable during localization.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-aspect-ratio",
        component_panel,
        code_panel,
        notes_panel,
    )
}
