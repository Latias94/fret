use super::super::super::super::*;

pub(in crate::ui) fn preview_scroll_area(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_scroll_area(cx)
}

#[cfg(any())]
pub(in crate::ui) fn preview_scroll_area_legacy(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        ui::h_flex(move |_cx| [body])
                .layout(LayoutRefinement::default().w_full())
                .justify_center().into_element(cx)
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        ui::v_flex(move |cx| vec![shadcn::raw::typography::h4(cx, title), body])
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()).into_element(cx)
    };

    let shell = |cx: &mut ElementContext<'_, App>, layout: LayoutRefinement, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default().border_1().rounded(Radius::Md),
                layout,
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let demo = {
        let versions: Vec<Arc<str>> = (1..=50)
            .map(|idx| Arc::<str>::from(format!("v1.2.0-beta.{:02}", 51 - idx)))
            .collect();

        let content = ui::v_flex(|cx| {
                let mut rows: Vec<AnyElement> = Vec::with_capacity(versions.len() * 2 + 1);
                rows.push(shadcn::raw::typography::small(cx, "Tags"));
                for tag in versions {
                    rows.push(cx.text(tag));
                    rows.push(
                        shadcn::Separator::new()
                            .refine_layout(LayoutRefinement::default().w_full())
                            .into_element(cx),
                    );
                }
                rows
            })
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full()).into_element(cx);

        let scroll = shadcn::ScrollArea::new([content])
            .axis(fret_ui::element::ScrollAxis::Y)
            .refine_layout(LayoutRefinement::default().w_px(Px(192.0)).h_px(Px(288.0)))
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-scroll-area-demo"),
            );

        let card = shell(cx, LayoutRefinement::default(), scroll);
        let body = centered(cx, card);
        section(cx, "Demo", body)
    };

    let horizontal = {
        let rail = ui::h_row(|cx| {
                let artists = [
                    "Ornella Binni",
                    "Tom Byrom",
                    "Vladimir Malyavko",
                    "Silvia Serra",
                ];
                artists
                    .iter()
                    .map(|artist| {
                        shadcn::Card::new(vec![
                            shadcn::CardContent::new(vec![
                                {
                                    let photo_props = cx.with_theme(|theme| {
                                        decl_style::container_props(
                                            theme,
                                            ChromeRefinement::default()
                                                .rounded(Radius::Md)
                                                .border_1()
                                                .bg(ColorRef::Color(theme.color_token("muted"))),
                                            LayoutRefinement::default()
                                                .w_px(Px(140.0))
                                                .h_px(Px(180.0)),
                                        )
                                    });
                                    cx.container(photo_props, |_cx| Vec::new())
                                },
                                shadcn::raw::typography::muted(cx, format!("Photo by {artist}")),
                            ])
                            .into_element(cx),
                        ])
                        .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
                        .into_element(cx)
                    })
                    .collect::<Vec<_>>()
            })
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_px(Px(760.0))).into_element(cx);

        let scroll = shadcn::ScrollArea::new([rail])
            .axis(fret_ui::element::ScrollAxis::X)
            .refine_layout(LayoutRefinement::default().w_px(Px(384.0)).h_px(Px(280.0)))
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-scroll-area-horizontal"),
            );

        let card = shell(cx, LayoutRefinement::default(), scroll);
        let body = centered(cx, card);
        section(cx, "Horizontal", body)
    };

    let rtl = {
        let rtl_scroll = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let content = ui::v_flex(|cx| {
                        let mut rows: Vec<AnyElement> =
                            vec![shadcn::raw::typography::small(cx, "العلامات")];
                        for idx in 1..=40 {
                            rows.push(cx.text(format!("v1.2.0-beta.{:02}", 41 - idx)));
                            rows.push(
                                shadcn::Separator::new()
                                    .refine_layout(LayoutRefinement::default().w_full())
                                    .into_element(cx),
                            );
                        }
                        rows
                    })
                        .gap(Space::N2)
                        .layout(LayoutRefinement::default().w_full()).into_element(cx);

                shadcn::ScrollArea::new([content])
                    .axis(fret_ui::element::ScrollAxis::Y)
                    .refine_layout(LayoutRefinement::default().w_px(Px(192.0)).h_px(Px(288.0)))
                    .into_element(cx)
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-scroll-area-rtl"),
        );

        let card = shell(cx, LayoutRefinement::default(), rtl_scroll);
        let body = centered(cx, card);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("Scrollable region with custom scrollbars and nested content."),
        ui::v_stack(|_cx| {
            vec![demo, horizontal, rtl]
        }).gap(Space::N6).into_element(cx),
    ]
}
