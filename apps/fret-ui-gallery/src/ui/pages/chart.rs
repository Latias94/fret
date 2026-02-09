use super::super::*;

pub(super) fn preview_chart(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let (chart_1, chart_2, chart_3) = cx.with_theme(|theme| {
        (
            theme.color_required("chart-1"),
            theme.color_required("chart-2"),
            theme.color_required("chart-3"),
        )
    });

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

    let tooltip = |cx: &mut ElementContext<'_, App>,
                   label: &'static str,
                   indicator: shadcn::ChartTooltipIndicator,
                   hide_label: bool,
                   hide_indicator: bool,
                   test_id: &'static str| {
        shadcn::ChartTooltipContent::new()
            .label(label)
            .items([
                shadcn::ChartTooltipItem::new("Desktop", "186").color(ColorRef::Color(chart_1)),
                shadcn::ChartTooltipItem::new("Mobile", "80").color(ColorRef::Color(chart_2)),
                shadcn::ChartTooltipItem::new("Tablet", "42").color(ColorRef::Color(chart_3)),
            ])
            .indicator(indicator)
            .hide_label(hide_label)
            .hide_indicator(hide_indicator)
            .test_id_prefix(test_id)
            .into_element(cx)
            .test_id(test_id)
    };

    let legend = |cx: &mut ElementContext<'_, App>,
                  align: shadcn::ChartLegendVerticalAlign,
                  wrap: bool,
                  hide_icon: bool,
                  test_id: &'static str| {
        shadcn::ChartLegendContent::new()
            .items([
                shadcn::ChartLegendItem::new("Desktop").color(ColorRef::Color(chart_1)),
                shadcn::ChartLegendItem::new("Mobile").color(ColorRef::Color(chart_2)),
                shadcn::ChartLegendItem::new("Tablet").color(ColorRef::Color(chart_3)),
            ])
            .vertical_align(align)
            .wrap(wrap)
            .hide_icon(hide_icon)
            .into_element(cx)
            .test_id(test_id)
    };

    let demo_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "shadcn Chart is Recharts composition. Fret page focuses on chart tooltip/legend contracts.",
                ),
                tooltip(
                    cx,
                    "January",
                    shadcn::ChartTooltipIndicator::Dot,
                    false,
                    false,
                    "ui-gallery-chart-demo-tooltip",
                ),
                legend(
                    cx,
                    shadcn::ChartLegendVerticalAlign::Bottom,
                    true,
                    false,
                    "ui-gallery-chart-demo-legend",
                ),
            ]
        },
    );
    let demo = section_card(cx, "Demo", demo_content);

    let component_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Map series to `chart-*` tokens so tooltip and legend stay color-consistent across light/dark themes.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Current Fret abstraction intentionally stays lightweight and does not wrap Recharts runtime APIs.",
                ),
            ]
        },
    );
    let component = section_card(cx, "Component", component_content);

    let tooltip_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                tooltip(
                    cx,
                    "January",
                    shadcn::ChartTooltipIndicator::Dot,
                    false,
                    false,
                    "ui-gallery-chart-tooltip-dot",
                ),
                tooltip(
                    cx,
                    "January",
                    shadcn::ChartTooltipIndicator::Line,
                    false,
                    false,
                    "ui-gallery-chart-tooltip-line",
                ),
                tooltip(
                    cx,
                    "January",
                    shadcn::ChartTooltipIndicator::Dashed,
                    true,
                    false,
                    "ui-gallery-chart-tooltip-dashed",
                ),
            ]
        },
    );
    let tooltip_section = section_card(cx, "Tooltip", tooltip_content);

    let legend_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                legend(
                    cx,
                    shadcn::ChartLegendVerticalAlign::Top,
                    false,
                    false,
                    "ui-gallery-chart-legend-top",
                ),
                legend(
                    cx,
                    shadcn::ChartLegendVerticalAlign::Bottom,
                    true,
                    true,
                    "ui-gallery-chart-legend-wrap-no-icon",
                ),
            ]
        },
    );
    let legend_section = section_card(cx, "Legend", legend_content);

    let accessibility_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Upstream Recharts supports `accessibilityLayer` for keyboard and screen-reader navigation.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Fret gallery keeps this as a documented parity gap until chart runtime integration is introduced.",
                ),
            ]
        },
    );
    let accessibility = section_card(cx, "Accessibility", accessibility_content);

    let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full()),
                |cx| {
                    vec![
                        tooltip(
                            cx,
                            "?????",
                            shadcn::ChartTooltipIndicator::Dot,
                            false,
                            false,
                            "ui-gallery-chart-rtl-tooltip",
                        ),
                        legend(
                            cx,
                            shadcn::ChartLegendVerticalAlign::Bottom,
                            true,
                            false,
                            "ui-gallery-chart-rtl-legend",
                        ),
                    ]
                },
            )
        },
    );
    let rtl = section_card(cx, "RTL", rtl_content);

    let preview_hint = shadcn::typography::muted(
        cx,
        "Preview follows shadcn Chart docs flow: Demo -> Component -> Tooltip -> Legend -> Accessibility -> RTL.",
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
                component,
                tooltip_section,
                legend_section,
                accessibility,
                rtl,
            ]
        },
    );
    let component_panel = shell(cx, component_stack).test_id("ui-gallery-chart-component");

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
                    "Tooltip",
                    r#"let tooltip = shadcn::ChartTooltipContent::new()
    .label("January")
    .items([
        shadcn::ChartTooltipItem::new("Desktop", "186"),
        shadcn::ChartTooltipItem::new("Mobile", "80"),
    ])
    .indicator(shadcn::ChartTooltipIndicator::Line)
    .into_element(cx);"#,
                ),
                code_block(
                    cx,
                    "Legend",
                    r#"let legend = shadcn::ChartLegendContent::new()
    .items([
        shadcn::ChartLegendItem::new("Desktop"),
        shadcn::ChartLegendItem::new("Mobile"),
    ])
    .vertical_align(shadcn::ChartLegendVerticalAlign::Bottom)
    .wrap(true)
    .into_element(cx);"#,
                ),
                code_block(
                    cx,
                    "RTL",
                    r#"with_direction_provider(LayoutDirection::Rtl, |cx| {
    shadcn::ChartTooltipContent::new().label("?????").into_element(cx)
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
                    "This page validates tooltip/legend composition parity, not full chart drawing parity.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Keep color mapping stable through `chart-*` tokens to avoid dark-theme drift.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Accessibility and full Recharts API integration are intentionally tracked as follow-up work.",
                ),
                shadcn::typography::muted(
                    cx,
                    "When adding chart runtime later, keep this page order unchanged for quick docs side-by-side checks.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-chart",
        component_panel,
        code_panel,
        notes_panel,
    )
}
