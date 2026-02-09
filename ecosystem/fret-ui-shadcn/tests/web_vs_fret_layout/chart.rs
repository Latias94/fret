use super::*;

fn assert_chart_tooltip_rect_matches_web(
    web_name: &str,
    indicator: fret_ui_shadcn::ChartTooltipIndicator,
    hide_indicator: bool,
    hide_label: bool,
    kind: fret_ui_shadcn::ChartTooltipContentKind,
    fixed_width_border_box: Option<Px>,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_tooltip = find_first(&theme.root, &|n| {
        n.tag == "div"
            && class_has_token(n, "border-border/50")
            && class_has_token(n, "bg-background")
            && class_has_token(n, "shadow-xl")
            && class_has_token(n, "min-w-[8rem]")
    })
    .expect("web chart tooltip node");

    let advanced_layout = matches!(
        kind,
        fret_ui_shadcn::ChartTooltipContentKind::AdvancedKcalTotal
    ) && web_name == "chart-tooltip-advanced";

    let (web_item_0, web_item_1, web_total_row) = if advanced_layout {
        let item_row = |name: &str| {
            find_first(web_tooltip, &|n| {
                n.tag == "div"
                    && class_has_token(n, "flex")
                    && class_has_token(n, "w-full")
                    && class_has_token(n, "items-center")
                    && class_has_token(n, "gap-2")
                    && contains_text(n, name)
            })
            .unwrap_or_else(|| panic!("web chart tooltip item row for {name}"))
        };

        let total_row = find_first(web_tooltip, &|n| {
            n.tag == "div"
                && class_has_token(n, "flex")
                && class_has_token(n, "basis-full")
                && class_has_token(n, "border-t")
                && class_has_token(n, "mt-1.5")
                && class_has_token(n, "pt-1.5")
                && contains_text(n, "Total")
        })
        .expect("web chart tooltip total row");

        (
            Some(item_row("Running")),
            Some(item_row("Swimming")),
            Some(total_row),
        )
    } else {
        (None, None, None)
    };

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let label = Arc::<str>::from(format!("Golden:{web_name}:tooltip"));

    let snap = run_fret_root(bounds, |cx| {
        let mut tooltip = fret_ui_shadcn::ChartTooltipContent::new()
            .label("Tue")
            .indicator(indicator)
            .hide_indicator(hide_indicator)
            .hide_label(hide_label)
            .kind(kind)
            .items([
                fret_ui_shadcn::ChartTooltipItem::new("Running", "380"),
                fret_ui_shadcn::ChartTooltipItem::new("Swimming", "420"),
            ]);
        if advanced_layout {
            tooltip = tooltip.test_id_prefix(label.clone());
        }
        if let Some(width) = fixed_width_border_box {
            tooltip = tooltip.fixed_width_border_box(width);
        }

        let tooltip = tooltip.into_element(cx);

        let tooltip = cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.position = fret_ui::element::PositionStyle::Absolute;
                    layout.inset.left = Some(Px(web_tooltip.rect.x));
                    layout.inset.top = Some(Px(web_tooltip.rect.y));
                    layout
                },
                role: SemanticsRole::Panel,
                label: Some(label.clone()),
                ..Default::default()
            },
            move |_cx| vec![tooltip],
        );

        vec![tooltip]
    });

    let tooltip = find_semantics(&snap, SemanticsRole::Panel, Some(&label))
        .unwrap_or_else(|| panic!("missing fret chart tooltip semantics for {web_name}"));

    assert_rect_close_px(web_name, tooltip.bounds, web_tooltip.rect, 1.0);

    if advanced_layout {
        let item_0_label = format!("{label}:item-0");
        let item_1_label = format!("{label}:item-1");
        let total_row_label = format!("{label}:total-row");

        let item_0 = find_semantics(&snap, SemanticsRole::Panel, Some(&item_0_label))
            .expect("missing fret chart tooltip item-0 semantics");
        let item_1 = find_semantics(&snap, SemanticsRole::Panel, Some(&item_1_label))
            .expect("missing fret chart tooltip item-1 semantics");
        let total_row = find_semantics(&snap, SemanticsRole::Panel, Some(&total_row_label))
            .expect("missing fret chart tooltip total-row semantics");

        assert_rect_close_px(
            "chart-tooltip-advanced item-0",
            item_0.bounds,
            web_item_0.expect("web item-0").rect,
            1.0,
        );
        assert_rect_close_px(
            "chart-tooltip-advanced item-1",
            item_1.bounds,
            web_item_1.expect("web item-1").rect,
            1.0,
        );
        assert_rect_close_px(
            "chart-tooltip-advanced total-row",
            total_row.bounds,
            web_total_row.expect("web total-row").rect,
            1.0,
        );
    }
}

fn assert_chart_legend_rect_matches_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_legend = find_first(&theme.root, &|n| {
        n.tag == "div"
            && class_has_token(n, "flex")
            && class_has_token(n, "items-center")
            && class_has_token(n, "justify-center")
            && class_has_token(n, "gap-4")
            && (class_has_token(n, "pt-3") || class_has_token(n, "pb-3"))
    })
    .expect("web chart legend node");

    let vertical_align = if class_has_token(web_legend, "pb-3") {
        fret_ui_shadcn::ChartLegendVerticalAlign::Top
    } else {
        fret_ui_shadcn::ChartLegendVerticalAlign::Bottom
    };

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let label = Arc::<str>::from(format!("Golden:{web_name}:legend"));

    let snap = run_fret_root(bounds, |cx| {
        let legend = fret_ui_shadcn::ChartLegendContent::new()
            .vertical_align(vertical_align)
            .items([
                fret_ui_shadcn::ChartLegendItem::new("Desktop"),
                fret_ui_shadcn::ChartLegendItem::new("Mobile"),
            ])
            .into_element(cx);

        let legend = cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.position = fret_ui::element::PositionStyle::Absolute;
                    layout.inset.left = Some(Px(web_legend.rect.x));
                    layout.inset.top = Some(Px(web_legend.rect.y));
                    layout.size.width = Length::Px(Px(web_legend.rect.w));
                    layout
                },
                role: SemanticsRole::Panel,
                label: Some(label.clone()),
                ..Default::default()
            },
            move |_cx| vec![legend],
        );

        vec![legend]
    });

    let legend = find_semantics(&snap, SemanticsRole::Panel, Some(&label))
        .unwrap_or_else(|| panic!("missing fret chart legend semantics for {web_name}"));

    assert_rect_close_px(web_name, legend.bounds, web_legend.rect, 1.0);
}

fn assert_chart_pie_legend_rect_matches_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_legend = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_token(n, "recharts-legend-wrapper")
    })
    .and_then(|wrapper| {
        find_first(wrapper, &|n| {
            n.tag == "div"
                && class_has_token(n, "flex")
                && class_has_token(n, "items-center")
                && class_has_token(n, "justify-center")
                && class_has_token(n, "pt-3")
                && class_has_token(n, "-translate-y-2")
                && class_has_token(n, "flex-wrap")
                && class_has_token(n, "gap-2")
        })
    })
    .expect("web chart pie legend node");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let label = Arc::<str>::from(format!("Golden:{web_name}:pie-legend"));

    let snap = run_fret_root(bounds, |cx| {
        let legend = fret_ui_shadcn::ChartLegendContent::new()
            .gap(Space::N2)
            .wrap(true)
            .item_width_px(Px(72.5))
            .item_justify_center(true)
            .items([
                fret_ui_shadcn::ChartLegendItem::new("Chrome"),
                fret_ui_shadcn::ChartLegendItem::new("Safari"),
                fret_ui_shadcn::ChartLegendItem::new("Firefox"),
                fret_ui_shadcn::ChartLegendItem::new("Edge"),
                fret_ui_shadcn::ChartLegendItem::new("Other"),
            ])
            .into_element(cx);

        let legend = cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.position = fret_ui::element::PositionStyle::Absolute;
                    layout.inset.left = Some(Px(web_legend.rect.x));
                    layout.inset.top = Some(Px(web_legend.rect.y));
                    layout.size.width = Length::Px(Px(web_legend.rect.w));
                    layout
                },
                role: SemanticsRole::Panel,
                label: Some(label.clone()),
                ..Default::default()
            },
            move |_cx| vec![legend],
        );

        vec![legend]
    });

    let legend = find_semantics(&snap, SemanticsRole::Panel, Some(&label))
        .unwrap_or_else(|| panic!("missing fret chart pie legend semantics for {web_name}"));

    assert_rect_close_px(web_name, legend.bounds, web_legend.rect, 1.0);
}

fn chart_tooltip_demo_panel<H: fret_ui::UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    label: &str,
    hide_label: bool,
    hide_indicator: bool,
    indicator: fret_ui_shadcn::ChartTooltipIndicator,
    width_border_box: Px,
    items: impl IntoIterator<Item = (Arc<str>, Arc<str>)>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let label = Arc::<str>::from(label);
    let text_xs_px = theme
        .metric_by_key(fret_ui_kit::theme_tokens::metric::COMPONENT_TEXT_XS_PX)
        .unwrap_or(Px(12.0));
    let text_xs_line_height = theme
        .metric_by_key(fret_ui_kit::theme_tokens::metric::COMPONENT_TEXT_XS_LINE_HEIGHT)
        .unwrap_or(Px(16.0));

    let bg = theme.color_required("background");
    let border = theme
        .color_by_key("border/50")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or_else(|| theme.color_required("border"));
    let muted = theme.color_required("muted-foreground");

    let chrome = ChromeRefinement::default()
        .rounded(Radius::Lg)
        .bg(ColorRef::Color(bg))
        .border_1()
        .border_color(ColorRef::Color(border))
        .px(Space::N2p5)
        .py(Space::N1p5)
        .shadow_xl();

    let gap_1p5 = decl_style::space(&theme, Space::N1p5);
    let gap_2 = decl_style::space(&theme, Space::N2);

    let items: Vec<(Arc<str>, Arc<str>)> = items.into_iter().collect();
    let nest_label =
        !hide_label && items.len() == 1 && indicator != fret_ui_shadcn::ChartTooltipIndicator::Dot;

    let row_height = if nest_label {
        Px(text_xs_px.0 * 2.0 + gap_1p5.0)
    } else {
        text_xs_px
    };

    fn build_row<H: fret_ui::UiHost>(
        cx: &mut fret_ui::ElementContext<'_, H>,
        theme: &Theme,
        tooltip_label: Arc<str>,
        muted: fret_core::Color,
        hide_indicator: bool,
        indicator: fret_ui_shadcn::ChartTooltipIndicator,
        nest_label: bool,
        gap_1p5: Px,
        gap_2: Px,
        row_height: Px,
        text_xs_px: Px,
        item_label: Arc<str>,
        item_value: Arc<str>,
    ) -> AnyElement {
        let mut row_children: Vec<AnyElement> = Vec::new();

        if !hide_indicator {
            let indicator_color = theme.color_required("foreground");
            let (w, h) = match indicator {
                fret_ui_shadcn::ChartTooltipIndicator::Dot => (Px(10.0), Px(10.0)),
                fret_ui_shadcn::ChartTooltipIndicator::Line
                | fret_ui_shadcn::ChartTooltipIndicator::Dashed => (Px(4.0), row_height),
            };

            let mut indicator_props = decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .bg(ColorRef::Color(indicator_color))
                    .border_1()
                    .border_color(ColorRef::Color(indicator_color)),
                LayoutRefinement::default(),
            );
            indicator_props.corner_radii = fret_core::Corners::all(Px(2.0));
            indicator_props.layout.size.width = Length::Px(w);
            indicator_props.layout.size.height = Length::Px(h);

            row_children.push(cx.container(indicator_props, |_cx| Vec::new()));
        }

        let label_col = cx.column(
            ColumnProps {
                gap: gap_1p5,
                align: CrossAlign::Start,
                ..Default::default()
            },
            move |cx| {
                let mut out = Vec::new();
                if nest_label {
                    out.push(
                        ui::text(cx, tooltip_label)
                            .text_xs()
                            .font_medium()
                            .line_height_px(text_xs_px)
                            .h_px(MetricRef::Px(text_xs_px))
                            .nowrap()
                            .into_element(cx),
                    );
                }
                out.push(
                    ui::text(cx, item_label)
                        .text_xs()
                        .text_color(ColorRef::Color(muted))
                        .line_height_px(text_xs_px)
                        .h_px(MetricRef::Px(text_xs_px))
                        .nowrap()
                        .into_element(cx),
                );
                out
            },
        );

        let value_el = ui::text(cx, item_value)
            .text_xs()
            .font_medium()
            .line_height_px(text_xs_px)
            .h_px(MetricRef::Px(text_xs_px))
            .nowrap()
            .into_element(cx);

        let content = cx.flex(
            FlexProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.flex.grow = 1.0;
                    layout.flex.shrink = 1.0;
                    layout
                },
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::SpaceBetween,
                align: if nest_label {
                    CrossAlign::End
                } else {
                    CrossAlign::Center
                },
                wrap: false,
            },
            move |_cx| vec![label_col, value_el],
        );

        row_children.push(content);

        cx.flex(
            FlexProps {
                layout: LayoutStyle::default(),
                direction: fret_core::Axis::Horizontal,
                gap: gap_2,
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: match indicator {
                    fret_ui_shadcn::ChartTooltipIndicator::Dot => CrossAlign::Center,
                    fret_ui_shadcn::ChartTooltipIndicator::Line
                    | fret_ui_shadcn::ChartTooltipIndicator::Dashed => CrossAlign::Stretch,
                },
                wrap: false,
            },
            move |_cx| row_children,
        )
    }

    let theme_for_items = theme.clone();
    let label_for_items = label.clone();

    let items_column = cx.column(
        ColumnProps {
            gap: gap_1p5,
            align: CrossAlign::Stretch,
            ..Default::default()
        },
        move |cx| {
            items
                .iter()
                .cloned()
                .map(|(item_label, item_value)| {
                    build_row(
                        cx,
                        &theme_for_items,
                        label_for_items.clone(),
                        muted,
                        hide_indicator,
                        indicator,
                        nest_label,
                        gap_1p5,
                        gap_2,
                        row_height,
                        text_xs_px,
                        item_label,
                        item_value,
                    )
                })
                .collect::<Vec<_>>()
        },
    );

    let content = if !hide_label && !nest_label {
        cx.column(
            ColumnProps {
                gap: gap_1p5,
                align: CrossAlign::Start,
                ..Default::default()
            },
            move |cx| {
                vec![
                    ui::text(cx, label.clone())
                        .text_xs()
                        .font_medium()
                        .line_height_px(text_xs_line_height)
                        .h_px(MetricRef::Px(text_xs_line_height))
                        .nowrap()
                        .into_element(cx),
                    items_column,
                ]
            },
        )
    } else {
        items_column
    };

    let props = decl_style::container_props(
        &theme,
        chrome,
        LayoutRefinement::default().w_px(MetricRef::Px(width_border_box)),
    );

    cx.container(props, move |_cx| vec![content])
}

#[test]
fn web_vs_fret_layout_chart_tooltip_demo_geometry_matches_web() {
    let web = read_web_golden("chart-tooltip-demo");
    let theme = web_theme(&web);

    let web_root = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_token(n, "max-w-md") && class_has_token(n, "aspect-video")
    })
    .expect("web chart-tooltip-demo root");

    let web_panel = |pred: &dyn Fn(&WebNode) -> bool| -> &WebNode {
        find_all(web_root, &|n| {
            n.tag == "div"
                && class_has_token(n, "min-w-[8rem]")
                && class_has_token(n, "rounded-lg")
                && class_has_token(n, "shadow-xl")
        })
        .into_iter()
        .find(|n| pred(n))
        .expect("web chart-tooltip-demo tooltip panel")
    };

    let web_page_views =
        web_panel(&|n| contains_text(n, "Page Views") && contains_text(n, "Mobile"));
    let web_browser_dashed = web_panel(&|n| contains_text(n, "Firefox"));
    let web_indicator_dot =
        web_panel(&|n| contains_text(n, "Chrome") && !contains_text(n, "Firefox"));
    let web_page_views_line = web_panel(&|n| contains_text(n, "12,486"));

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(web_root.rect.w), Px(web_root.rect.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        fn cell<H: fret_ui::UiHost>(
            cx: &mut fret_ui::ElementContext<'_, H>,
            align: CrossAlign,
            justify: MainAlign,
            child: AnyElement,
        ) -> AnyElement {
            cx.flex(
                FlexProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Px(Px(224.0)),
                            height: Length::Px(Px(137.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0),
                    padding: Edges::all(Px(16.0)),
                    justify,
                    align,
                    wrap: false,
                },
                move |_cx| vec![child],
            )
        }

        let page_views = chart_tooltip_demo_panel(
            cx,
            "Page Views",
            false,
            false,
            fret_ui_shadcn::ChartTooltipIndicator::Dot,
            Px(128.0),
            [
                (Arc::from("Desktop"), Arc::from("186")),
                (Arc::from("Mobile"), Arc::from("80")),
            ],
        );
        let page_views = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:chart-tooltip-demo:page-views")),
                ..Default::default()
            },
            move |_cx| vec![page_views],
        );

        let browser_dashed = chart_tooltip_demo_panel(
            cx,
            "Browser",
            true,
            false,
            fret_ui_shadcn::ChartTooltipIndicator::Dashed,
            Px(128.0),
            [
                (Arc::from("Chrome"), Arc::from("1,286")),
                (Arc::from("Firefox"), Arc::from("1,000")),
            ],
        );
        let browser_dashed = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:chart-tooltip-demo:browser-dashed")),
                ..Default::default()
            },
            move |_cx| vec![browser_dashed],
        );

        let page_views_line = chart_tooltip_demo_panel(
            cx,
            "Page Views",
            false,
            false,
            fret_ui_shadcn::ChartTooltipIndicator::Line,
            Px(144.0),
            [(Arc::from("Desktop"), Arc::from("12,486"))],
        );
        let page_views_line = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:chart-tooltip-demo:page-views-line")),
                ..Default::default()
            },
            move |_cx| vec![page_views_line],
        );

        let indicator_dot = chart_tooltip_demo_panel(
            cx,
            "Browser",
            true,
            false,
            fret_ui_shadcn::ChartTooltipIndicator::Dot,
            Px(128.0),
            [(Arc::from("Chrome"), Arc::from("1,286"))],
        );
        let indicator_dot = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:chart-tooltip-demo:indicator-dot")),
                ..Default::default()
            },
            move |_cx| vec![indicator_dot],
        );

        let grid = cx.grid(
            GridProps {
                cols: 2,
                gap: Px(0.0),
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(web_root.rect.w)),
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx| {
                vec![
                    cell(cx, CrossAlign::Center, MainAlign::Center, page_views),
                    cell(cx, CrossAlign::Center, MainAlign::Center, browser_dashed),
                    cell(cx, CrossAlign::Center, MainAlign::Center, page_views_line),
                    cell(cx, CrossAlign::Start, MainAlign::Start, indicator_dot),
                ]
            },
        );

        vec![grid]
    });

    let page_views = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:chart-tooltip-demo:page-views"),
    )
    .expect("fret page views tooltip");
    assert_rect_close_px(
        "chart-tooltip-demo page views",
        page_views.bounds,
        web_page_views.rect,
        1.0,
    );

    let browser_dashed = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:chart-tooltip-demo:browser-dashed"),
    )
    .expect("fret browser dashed tooltip");
    assert_rect_close_px(
        "chart-tooltip-demo browser dashed",
        browser_dashed.bounds,
        web_browser_dashed.rect,
        1.0,
    );

    let page_views_line = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:chart-tooltip-demo:page-views-line"),
    )
    .expect("fret page views line tooltip");
    assert_rect_close_px(
        "chart-tooltip-demo page views line",
        page_views_line.bounds,
        web_page_views_line.rect,
        1.0,
    );

    let indicator_dot = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:chart-tooltip-demo:indicator-dot"),
    )
    .expect("fret indicator dot tooltip");
    assert_rect_close_px(
        "chart-tooltip-demo indicator dot",
        indicator_dot.bounds,
        web_indicator_dot.rect,
        1.0,
    );
}

pub(super) fn web_find_chart_container<'a>(root: &'a WebNode) -> &'a WebNode {
    find_first(root, &|n| {
        n.tag == "div"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("recharts-cartesian"))
    })
    .expect("web chart container")
}

pub(super) fn web_find_chart_grid<'a>(root: &'a WebNode) -> &'a WebNode {
    find_first(root, &|n| {
        n.tag == "g"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("recharts-cartesian-grid"))
    })
    .expect("web chart grid")
}

fn web_find_chart_plot_rect(root: &WebNode) -> WebRect {
    if let Some(grid) = find_first(root, &|n| {
        n.tag == "g"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("recharts-cartesian-grid"))
    }) {
        return grid.rect;
    }

    let rects = find_all(root, &|n| n.tag == "rect");
    rects
        .into_iter()
        .max_by(|a, b| {
            let aa = a.rect.w * a.rect.h;
            let bb = b.rect.w * b.rect.h;
            aa.partial_cmp(&bb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or_else(|| panic!("web chart plot rect (no grid/rect)"))
        .rect
}

pub(super) fn web_find_chart_x_axis<'a>(root: &'a WebNode) -> &'a WebNode {
    find_first(root, &|n| {
        n.tag == "g"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("recharts-cartesian-axis") && c.contains("xAxis"))
    })
    .expect("web chart xAxis")
}

pub(super) fn web_find_chart_curve<'a>(root: &'a WebNode) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.tag == "path"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("recharts-curve"))
    })
}

fn web_find_chart_series_curves<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let mut out = find_all(root, &|n| {
        n.tag == "path"
            && n.class_name.as_deref().is_some_and(|c| {
                c.contains("recharts-line-curve") || c.contains("recharts-area-curve")
            })
    });
    out.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .x
                    .partial_cmp(&b.rect.x)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    out
}

fn web_find_chart_area_fills<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let mut out = find_all(root, &|n| {
        n.tag == "path"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("recharts-area-area"))
    });
    out.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .x
                    .partial_cmp(&b.rect.x)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    out
}

fn web_find_chart_bar_rects<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let mut out = find_all(root, &|n| {
        n.tag == "path"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c == "recharts-rectangle")
    });
    out.sort_by(|a, b| {
        a.rect
            .x
            .partial_cmp(&b.rect.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .y
                    .partial_cmp(&b.rect.y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    out
}

fn web_find_chart_svg<'a>(root: &'a WebNode) -> &'a WebNode {
    find_first(root, &|n| {
        n.tag == "svg" && n.class_name.as_deref() == Some("recharts-surface")
    })
    .expect("web chart svg")
}

fn web_find_pie_svg<'a>(root: &'a WebNode) -> &'a WebNode {
    find_first(root, &|n| {
        n.tag == "svg" && n.class_name.as_deref() == Some("recharts-surface")
    })
    .expect("web pie svg")
}

fn web_find_pie_sectors<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let mut out = find_all(root, &|n| {
        n.tag == "path" && n.class_name.as_deref() == Some("recharts-sector")
    });
    out.sort_by(|a, b| {
        a.rect
            .x
            .partial_cmp(&b.rect.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .y
                    .partial_cmp(&b.rect.y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| {
                a.rect
                    .w
                    .partial_cmp(&b.rect.w)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| {
                a.rect
                    .h
                    .partial_cmp(&b.rect.h)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    out
}

fn web_find_radial_bar_sectors<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let out = find_all(root, &|n| {
        n.tag == "path"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("recharts-radial-bar-sector"))
    });
    out
}

fn web_find_radial_bar_background_sectors<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let out = find_all(root, &|n| {
        n.tag == "path"
            && n.class_name.as_deref()
                == Some("recharts-sector recharts-radial-bar-background-sector")
    });
    out
}

fn sort_radial_band_nodes_by_outer_radius(svg_rect: Rect, nodes: &mut [&WebNode]) {
    let cx = svg_rect.origin.x.0 + svg_rect.size.width.0 / 2.0;
    let cy = svg_rect.origin.y.0 + svg_rect.size.height.0 / 2.0;

    nodes.sort_by(|a, b| {
        fn outer_radius_estimate(rect: WebRect, cx: f32, cy: f32) -> f32 {
            let left = (rect.x - cx).abs();
            let right = (rect.x + rect.w - cx).abs();
            let top = (rect.y - cy).abs();
            let bottom = (rect.y + rect.h - cy).abs();
            left.max(right).max(top).max(bottom)
        }

        let ar = outer_radius_estimate(a.rect, cx, cy);
        let br = outer_radius_estimate(b.rect, cx, cy);

        ar.partial_cmp(&br)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .y
                    .partial_cmp(&b.rect.y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| {
                a.rect
                    .w
                    .partial_cmp(&b.rect.w)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| {
                a.rect
                    .h
                    .partial_cmp(&b.rect.h)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
}

fn web_find_radar_polygons<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let mut out = find_all(root, &|n| {
        n.class_name
            .as_deref()
            .is_some_and(|c| c.contains("recharts-radar-polygon"))
    });
    out.sort_by(|a, b| {
        a.rect
            .x
            .partial_cmp(&b.rect.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .y
                    .partial_cmp(&b.rect.y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| {
                a.rect
                    .w
                    .partial_cmp(&b.rect.w)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| {
                a.rect
                    .h
                    .partial_cmp(&b.rect.h)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    out
}

fn web_find_polar_grid_concentric_polygons<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let mut out = find_all(root, &|n| {
        n.class_name
            .as_deref()
            .is_some_and(|c| c.contains("recharts-polar-grid-concentric-polygon"))
    });
    out.sort_by(|a, b| {
        a.rect
            .w
            .partial_cmp(&b.rect.w)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .h
                    .partial_cmp(&b.rect.h)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    out
}

fn web_find_polar_grid_concentric_circles<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let mut out = find_all(root, &|n| {
        n.tag == "circle"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("recharts-polar-grid-concentric-circle"))
    });
    out.sort_by(|a, b| {
        a.rect
            .w
            .partial_cmp(&b.rect.w)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .h
                    .partial_cmp(&b.rect.h)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    out
}

fn web_find_polar_grid_angle_lines<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let mut out = find_all(root, &|n| {
        n.class_name
            .as_deref()
            .is_some_and(|c| c == "recharts-polar-grid-angle")
    });
    out.sort_by(|a, b| {
        a.rect
            .x
            .partial_cmp(&b.rect.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .y
                    .partial_cmp(&b.rect.y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    out
}

fn web_find_radar_dots<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let mut out = find_all(root, &|n| {
        n.tag == "circle"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("recharts-radar-dot"))
    });
    out.sort_by(|a, b| {
        a.rect
            .x
            .partial_cmp(&b.rect.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .y
                    .partial_cmp(&b.rect.y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    out
}

#[test]
fn web_vs_fret_layout_chart_bar_default_bar_rects_match_web() {
    let web = read_web_golden("chart-bar-default");
    let theme = web_theme(&web);

    let web_chart = web_find_chart_container(&theme.root);
    let web_plot = web_find_chart_plot_rect(web_chart);
    let web_bars = web_find_chart_bar_rects(web_chart);
    assert_eq!(web_bars.len(), 6, "expected 6 bars in chart-bar-default");

    let values = [186.0_f32, 305.0, 237.0, 73.0, 209.0, 214.0];
    let plot = Rect::new(
        Point::new(Px(web_plot.x), Px(web_plot.y)),
        CoreSize::new(Px(web_plot.w), Px(web_plot.h)),
    );
    let bars = fret_ui_shadcn::recharts_geometry::bar_rects(
        plot,
        &values,
        fret_ui_shadcn::recharts_geometry::BarChartSeriesLayout::default(),
    );
    assert_eq!(bars.len(), web_bars.len());

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let chart_label = Arc::<str>::from("Golden:chart-bar-default:chart");
    let bar_labels: Vec<Arc<str>> = (0..bars.len())
        .map(|i| Arc::<str>::from(format!("Golden:chart-bar-default:bar-{i}")))
        .collect();

    let snap = run_fret_root(bounds, move |cx| {
        let chart = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(chart_label.clone()),
                layout: LayoutStyle {
                    position: fret_ui::element::PositionStyle::Absolute,
                    inset: fret_ui::element::InsetStyle {
                        left: Some(Px(web_chart.rect.x)),
                        top: Some(Px(web_chart.rect.y)),
                        ..Default::default()
                    },
                    size: SizeStyle {
                        width: Length::Px(Px(web_chart.rect.w)),
                        height: Length::Px(Px(web_chart.rect.h)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx| {
                let mut out = Vec::new();
                for (i, bar) in bars.iter().enumerate() {
                    let rect = bar.rect;
                    out.push(cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(bar_labels[i].clone()),
                            layout: LayoutStyle {
                                position: fret_ui::element::PositionStyle::Absolute,
                                inset: fret_ui::element::InsetStyle {
                                    left: Some(Px(rect.origin.x.0 - web_chart.rect.x)),
                                    top: Some(Px(rect.origin.y.0 - web_chart.rect.y)),
                                    ..Default::default()
                                },
                                size: SizeStyle {
                                    width: Length::Px(rect.size.width),
                                    height: Length::Px(rect.size.height),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |cx| vec![cx.container(Default::default(), |_cx| Vec::new())],
                    ));
                }
                out
            },
        );

        vec![chart]
    });

    for (i, web_bar) in web_bars.iter().enumerate() {
        let label = format!("Golden:chart-bar-default:bar-{i}");
        let node = find_semantics(&snap, SemanticsRole::Panel, Some(&label))
            .unwrap_or_else(|| panic!("missing fret semantics for {label}"));
        assert_rect_close_px(&label, node.bounds, web_bar.rect, 1.0);
    }
}

#[test]
fn web_vs_fret_layout_chart_bar_interactive_bar_rects_match_web() {
    let layout = fret_ui_shadcn::recharts_geometry::BarChartSeriesLayout::default();
    let web_name = "chart-bar-interactive";

    let web = read_web_golden(web_name);
    let theme = web_theme(&web);
    let chart = web_find_chart_container(&theme.root);
    let plot = web_find_chart_plot_rect(chart);
    let bars = web_find_chart_bar_rects(chart);

    assert_eq!(
        bars.len(),
        CHART_INTERACTIVE_DESKTOP.len(),
        "{web_name}: expected {} bar rect(s), got {}",
        CHART_INTERACTIVE_DESKTOP.len(),
        bars.len()
    );

    let plot = Rect::new(
        Point::new(Px(plot.x), Px(plot.y)),
        CoreSize::new(Px(plot.w), Px(plot.h)),
    );
    let rects =
        fret_ui_shadcn::recharts_geometry::bar_rects(plot, &CHART_INTERACTIVE_DESKTOP, layout);
    assert_chart_bar_rects_match_web(
        web_name,
        rects,
        &bars.iter().map(|n| n.rect).collect::<Vec<_>>(),
    );
}

#[test]
fn web_vs_fret_layout_chart_bar_interactive_mobile_bar_rects_match_web() {
    let layout = fret_ui_shadcn::recharts_geometry::BarChartSeriesLayout::default();
    let web_name = "chart-bar-interactive.mobile";

    let web = read_web_golden(web_name);
    let theme = web_theme(&web);
    let chart = web_find_chart_container(&theme.root);
    let plot = web_find_chart_plot_rect(chart);
    let bars = web_find_chart_bar_rects(chart);

    assert_eq!(
        bars.len(),
        CHART_INTERACTIVE_MOBILE.len(),
        "{web_name}: expected {} bar rect(s), got {}",
        CHART_INTERACTIVE_MOBILE.len(),
        bars.len()
    );

    let plot = Rect::new(
        Point::new(Px(plot.x), Px(plot.y)),
        CoreSize::new(Px(plot.w), Px(plot.h)),
    );
    let rects =
        fret_ui_shadcn::recharts_geometry::bar_rects(plot, &CHART_INTERACTIVE_MOBILE, layout);
    assert_chart_bar_rects_match_web(
        web_name,
        rects,
        &bars.iter().map(|n| n.rect).collect::<Vec<_>>(),
    );
}

fn assert_chart_bar_rects_match_web(
    web_name: &str,
    rects: Vec<fret_ui_shadcn::recharts_geometry::BarRect>,
    expected: &[WebRect],
) {
    let mut actual: Vec<Rect> = rects.into_iter().map(|b| b.rect).collect();
    actual.sort_by(|a, b| {
        a.origin
            .x
            .0
            .partial_cmp(&b.origin.x.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.origin
                    .y
                    .0
                    .partial_cmp(&b.origin.y.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    let mut expected: Vec<WebRect> = expected.to_vec();
    expected.sort_by(|a, b| {
        a.x.partial_cmp(&b.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
    });

    assert_eq!(
        actual.len(),
        expected.len(),
        "{web_name}: expected {} bar rects, got {}",
        expected.len(),
        actual.len()
    );

    for (i, (actual, expected)) in actual.iter().zip(expected.iter()).enumerate() {
        assert_rect_close_px(&format!("{web_name} bar-{i}"), *actual, *expected, 1.0);
    }
}

#[test]
fn web_vs_fret_layout_chart_bar_variants_bar_rects_match_web() {
    let layout = fret_ui_shadcn::recharts_geometry::BarChartSeriesLayout::default();

    {
        let web_name = "chart-bar-active";
        let web = read_web_golden(web_name);
        let theme = web_theme(&web);
        let chart = web_find_chart_container(&theme.root);
        let plot = web_find_chart_plot_rect(chart);
        let bars = web_find_chart_bar_rects(chart);
        let values = [187.0_f32, 200.0, 275.0, 173.0, 90.0];

        let plot = Rect::new(
            Point::new(Px(plot.x), Px(plot.y)),
            CoreSize::new(Px(plot.w), Px(plot.h)),
        );
        let rects = fret_ui_shadcn::recharts_geometry::bar_rects(plot, &values, layout);
        assert_chart_bar_rects_match_web(
            web_name,
            rects,
            &bars.iter().map(|n| n.rect).collect::<Vec<_>>(),
        );
    }

    {
        let web_name = "chart-bar-multiple";
        let web = read_web_golden(web_name);
        let theme = web_theme(&web);
        let chart = web_find_chart_container(&theme.root);
        let plot = web_find_chart_plot_rect(chart);
        let bars = web_find_chart_bar_rects(chart);
        let desktop = [186.0_f32, 305.0, 237.0, 73.0, 209.0, 214.0];
        let mobile = [80.0_f32, 200.0, 120.0, 190.0, 130.0, 140.0];

        let plot = Rect::new(
            Point::new(Px(plot.x), Px(plot.y)),
            CoreSize::new(Px(plot.w), Px(plot.h)),
        );
        let rects = fret_ui_shadcn::recharts_geometry::grouped_bar_rects(
            plot,
            &[&desktop, &mobile],
            layout,
            4.0,
        );
        assert_chart_bar_rects_match_web(
            web_name,
            rects,
            &bars.iter().map(|n| n.rect).collect::<Vec<_>>(),
        );
    }

    {
        let web_name = "chart-bar-stacked";
        let web = read_web_golden(web_name);
        let theme = web_theme(&web);
        let chart = web_find_chart_container(&theme.root);
        let plot = web_find_chart_plot_rect(chart);
        let bars = web_find_chart_bar_rects(chart);
        let desktop = [186.0_f32, 305.0, 237.0, 73.0, 209.0, 214.0];
        let mobile = [80.0_f32, 200.0, 120.0, 190.0, 130.0, 140.0];

        let plot = Rect::new(
            Point::new(Px(plot.x), Px(plot.y)),
            CoreSize::new(Px(plot.w), Px(plot.h)),
        );
        let rects = fret_ui_shadcn::recharts_geometry::stacked_bar_rects(
            plot,
            &[&desktop, &mobile],
            layout,
        );
        assert_chart_bar_rects_match_web(
            web_name,
            rects,
            &bars.iter().map(|n| n.rect).collect::<Vec<_>>(),
        );
    }

    {
        let web_name = "chart-bar-label";
        let web = read_web_golden(web_name);
        let theme = web_theme(&web);
        let chart = web_find_chart_container(&theme.root);
        let plot = web_find_chart_plot_rect(chart);
        let bars = web_find_chart_bar_rects(chart);
        let values = [186.0_f32, 305.0, 237.0, 73.0, 209.0, 214.0];

        let plot = Rect::new(
            Point::new(Px(plot.x), Px(plot.y)),
            CoreSize::new(Px(plot.w), Px(plot.h)),
        );
        let rects = fret_ui_shadcn::recharts_geometry::bar_rects(plot, &values, layout);
        assert_chart_bar_rects_match_web(
            web_name,
            rects,
            &bars.iter().map(|n| n.rect).collect::<Vec<_>>(),
        );
    }

    {
        let web_name = "chart-bar-demo";
        let web = read_web_golden(web_name);
        let theme = web_theme(&web);
        let chart = web_find_chart_container(&theme.root);
        let plot = web_find_chart_plot_rect(chart);
        let bars = web_find_chart_bar_rects(chart);
        let desktop = [186.0_f32, 305.0, 237.0, 73.0, 209.0, 214.0];
        let mobile = [80.0_f32, 200.0, 120.0, 190.0, 130.0, 140.0];

        let plot = Rect::new(
            Point::new(Px(plot.x), Px(plot.y)),
            CoreSize::new(Px(plot.w), Px(plot.h)),
        );
        let rects = fret_ui_shadcn::recharts_geometry::grouped_bar_rects(
            plot,
            &[&desktop, &mobile],
            layout,
            4.0,
        );
        assert_chart_bar_rects_match_web(
            web_name,
            rects,
            &bars.iter().map(|n| n.rect).collect::<Vec<_>>(),
        );
    }

    for web_name in [
        "chart-bar-demo-grid",
        "chart-bar-demo-axis",
        "chart-bar-demo-tooltip",
    ] {
        let web = read_web_golden(web_name);
        let theme = web_theme(&web);
        let chart = web_find_chart_container(&theme.root);
        let plot = web_find_chart_plot_rect(chart);
        let bars = web_find_chart_bar_rects(chart);
        let desktop = [186.0_f32, 305.0, 237.0, 73.0, 209.0, 214.0];
        let mobile = [80.0_f32, 200.0, 120.0, 190.0, 130.0, 140.0];

        let plot = Rect::new(
            Point::new(Px(plot.x), Px(plot.y)),
            CoreSize::new(Px(plot.w), Px(plot.h)),
        );
        let rects = fret_ui_shadcn::recharts_geometry::grouped_bar_rects(
            plot,
            &[&desktop, &mobile],
            layout,
            4.0,
        );
        assert_chart_bar_rects_match_web(
            web_name,
            rects,
            &bars.iter().map(|n| n.rect).collect::<Vec<_>>(),
        );
    }
}

#[test]
fn web_vs_fret_layout_chart_bar_negative_bar_rects_match_web() {
    let layout = fret_ui_shadcn::recharts_geometry::BarChartSeriesLayout::default();
    let web_name = "chart-bar-negative";
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);
    let chart = web_find_chart_container(&theme.root);
    let plot = web_find_chart_plot_rect(chart);
    let bars = web_find_chart_bar_rects(chart);
    let values = [186.0_f32, 205.0, -207.0, 173.0, -209.0, 214.0];

    let plot = Rect::new(
        Point::new(Px(plot.x), Px(plot.y)),
        CoreSize::new(Px(plot.w), Px(plot.h)),
    );
    let rects = fret_ui_shadcn::recharts_geometry::symmetric_bar_rects(plot, &values, layout);
    assert_chart_bar_rects_match_web(
        web_name,
        rects,
        &bars.iter().map(|n| n.rect).collect::<Vec<_>>(),
    );
}

#[test]
fn web_vs_fret_layout_chart_bar_horizontal_variants_bar_rects_match_web() {
    let layout = fret_ui_shadcn::recharts_geometry::BarChartSeriesLayout::default();

    let month_desktop = [186.0_f32, 305.0, 237.0, 73.0, 209.0, 214.0];
    let visitors = [275.0_f32, 200.0, 187.0, 173.0, 90.0];

    let cases = [
        ("chart-bar-horizontal", month_desktop.as_slice()),
        ("chart-bar-mixed", visitors.as_slice()),
        ("chart-bar-label-custom", month_desktop.as_slice()),
    ];

    for (web_name, values) in cases {
        let web = read_web_golden(web_name);
        let theme = web_theme(&web);
        let chart = web_find_chart_container(&theme.root);
        let plot = web_find_chart_plot_rect(chart);
        let bars = web_find_chart_bar_rects(chart);

        let plot = Rect::new(
            Point::new(Px(plot.x), Px(plot.y)),
            CoreSize::new(Px(plot.w), Px(plot.h)),
        );
        let rects = fret_ui_shadcn::recharts_geometry::horizontal_bar_rects(plot, values, layout);
        assert_chart_bar_rects_match_web(
            web_name,
            rects,
            &bars.iter().map(|n| n.rect).collect::<Vec<_>>(),
        );
    }
}

fn assert_pie_sector_rects_match_web(
    web_name: &str,
    values: &[f32],
    inner_radius: f32,
    outer_radius: Option<f32>,
    outer_overrides: &[(usize, f32)],
    extra_rings: &[(usize, f32, f32)],
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let svg = web_find_pie_svg(&theme.root);
    let web_sectors = web_find_pie_sectors(&theme.root);

    let svg_rect = Rect::new(
        Point::new(Px(svg.rect.x), Px(svg.rect.y)),
        CoreSize::new(Px(svg.rect.w), Px(svg.rect.h)),
    );

    let layout = fret_ui_shadcn::recharts_geometry::PieLayout::default();
    let mut expected = fret_ui_shadcn::recharts_geometry::pie_sectors_with_outer_radius_overrides(
        svg_rect,
        values,
        inner_radius,
        outer_radius,
        layout,
        outer_overrides,
    );

    for (index, ring_inner, ring_outer) in extra_rings {
        let rings = fret_ui_shadcn::recharts_geometry::pie_sectors(
            svg_rect,
            values,
            *ring_inner,
            Some(*ring_outer),
            layout,
        );
        if let Some(ring) = rings.get(*index) {
            expected.push(*ring);
        }
    }

    expected.sort_by(|a, b| {
        a.rect
            .origin
            .x
            .0
            .partial_cmp(&b.rect.origin.x.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .origin
                    .y
                    .0
                    .partial_cmp(&b.rect.origin.y.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| {
                a.rect
                    .size
                    .width
                    .0
                    .partial_cmp(&b.rect.size.width.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| {
                a.rect
                    .size
                    .height
                    .0
                    .partial_cmp(&b.rect.size.height.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    assert_eq!(
        expected.len(),
        web_sectors.len(),
        "{web_name}: expected {} sector rects, got {}",
        expected.len(),
        web_sectors.len()
    );

    for (i, (expected, web)) in expected.iter().zip(web_sectors.iter()).enumerate() {
        assert_rect_close_px(
            &format!("{web_name} sector-{i}"),
            expected.rect,
            web.rect,
            1.0,
        );
    }
}

#[test]
fn web_vs_fret_layout_chart_pie_sector_rects_match_web() {
    let browsers = [275.0_f32, 200.0, 187.0, 173.0, 90.0];
    let donut_text = [275.0_f32, 200.0, 287.0, 173.0, 190.0];

    for web_name in [
        "chart-pie-simple",
        "chart-pie-separator-none",
        "chart-pie-label",
        "chart-pie-label-custom",
        "chart-pie-label-list",
    ] {
        assert_pie_sector_rects_match_web(web_name, &browsers, 0.0, None, &[], &[]);
    }

    assert_pie_sector_rects_match_web("chart-pie-donut", &browsers, 60.0, None, &[], &[]);
    assert_pie_sector_rects_match_web("chart-pie-donut-text", &donut_text, 60.0, None, &[], &[]);

    // Active index 0: outerRadius + 10.
    let svg_outer_250 = 0.8 * ((250.0 - 10.0) / 2.0);
    assert_pie_sector_rects_match_web(
        "chart-pie-donut-active",
        &browsers,
        60.0,
        None,
        &[(0, svg_outer_250 + 10.0)],
        &[],
    );

    // Stacked pies: two rings with explicit radii.
    let desktop = [186.0_f32, 305.0, 237.0, 173.0, 209.0];
    let mobile = [80.0_f32, 200.0, 120.0, 190.0, 130.0];
    // We gate by matching both pies' sector rects as a multiset, so we compute them separately and concatenate.
    let web = read_web_golden("chart-pie-stacked");
    let theme = web_theme(&web);
    let svg = web_find_pie_svg(&theme.root);
    let svg_rect = Rect::new(
        Point::new(Px(svg.rect.x), Px(svg.rect.y)),
        CoreSize::new(Px(svg.rect.w), Px(svg.rect.h)),
    );
    let layout = fret_ui_shadcn::recharts_geometry::PieLayout::default();
    let mut expected =
        fret_ui_shadcn::recharts_geometry::pie_sectors(svg_rect, &desktop, 0.0, Some(60.0), layout);
    expected.extend(fret_ui_shadcn::recharts_geometry::pie_sectors(
        svg_rect,
        &mobile,
        70.0,
        Some(90.0),
        layout,
    ));
    expected.sort_by(|a, b| {
        a.rect
            .origin
            .x
            .0
            .partial_cmp(&b.rect.origin.x.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .origin
                    .y
                    .0
                    .partial_cmp(&b.rect.origin.y.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| {
                a.rect
                    .size
                    .width
                    .0
                    .partial_cmp(&b.rect.size.width.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| {
                a.rect
                    .size
                    .height
                    .0
                    .partial_cmp(&b.rect.size.height.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    let web_sectors = web_find_pie_sectors(&theme.root);
    assert_eq!(
        expected.len(),
        web_sectors.len(),
        "chart-pie-stacked sector count"
    );
    for (i, (expected, web)) in expected.iter().zip(web_sectors.iter()).enumerate() {
        assert_rect_close_px(
            &format!("chart-pie-stacked sector-{i}"),
            expected.rect,
            web.rect,
            1.0,
        );
    }

    // Interactive pie: default outer radius at 300x300 is 116, active index 0 uses +10 plus a second ring.
    let svg_outer_300 = 0.8 * ((300.0 - 10.0) / 2.0);
    let desktop_interactive = [186.0_f32, 305.0, 237.0, 173.0, 209.0];
    assert_pie_sector_rects_match_web(
        "chart-pie-interactive",
        &desktop_interactive,
        60.0,
        Some(svg_outer_300),
        &[(0, svg_outer_300 + 10.0)],
        &[(0, svg_outer_300 + 12.0, svg_outer_300 + 25.0)],
    );

    assert_pie_sector_rects_match_web(
        "chart-pie-interactive.february",
        &desktop_interactive,
        60.0,
        Some(svg_outer_300),
        &[(1, svg_outer_300 + 10.0)],
        &[(1, svg_outer_300 + 12.0, svg_outer_300 + 25.0)],
    );

    assert_pie_sector_rects_match_web(
        "chart-pie-interactive.may",
        &desktop_interactive,
        60.0,
        Some(svg_outer_300),
        &[(4, svg_outer_300 + 10.0)],
        &[(4, svg_outer_300 + 12.0, svg_outer_300 + 25.0)],
    );
}

fn assert_radar_geometry_matches_web(
    web_name: &str,
    series: &[&[f32]],
    layout: fret_ui_shadcn::recharts_geometry::PolarChartLayout,
    grid_polygon: bool,
    grid_circle: bool,
    grid_radii_override: Option<&[f32]>,
    radial_lines: bool,
    dots: bool,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let svg = web_find_pie_svg(&theme.root);
    let svg_rect = Rect::new(
        Point::new(Px(svg.rect.x), Px(svg.rect.y)),
        CoreSize::new(Px(svg.rect.w), Px(svg.rect.h)),
    );

    let mut all_values = Vec::new();
    for values in series {
        all_values.extend_from_slice(values);
    }
    let domain_max =
        fret_ui_shadcn::recharts_geometry::nice_polar_domain_max_for_values(&all_values, 5);

    fn union_rect(rects: &[Rect]) -> Option<Rect> {
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = -f32::INFINITY;
        let mut max_y = -f32::INFINITY;

        for r in rects {
            let x0 = r.origin.x.0;
            let y0 = r.origin.y.0;
            let x1 = x0 + r.size.width.0;
            let y1 = y0 + r.size.height.0;
            min_x = min_x.min(x0);
            min_y = min_y.min(y0);
            max_x = max_x.max(x1);
            max_y = max_y.max(y1);
        }

        if !(min_x.is_finite() && min_y.is_finite() && max_x.is_finite() && max_y.is_finite()) {
            return None;
        }

        Some(Rect::new(
            Point::new(Px(min_x), Px(min_y)),
            CoreSize::new(Px(max_x - min_x), Px(max_y - min_y)),
        ))
    }

    let expected_dots = if dots {
        let values = series.first().copied().unwrap_or(&[]);
        let mut rects = fret_ui_shadcn::recharts_geometry::radar_dot_rects(
            svg_rect, values, domain_max, 4.0, layout,
        );
        rects.sort_by(|a, b| {
            a.origin
                .x
                .0
                .partial_cmp(&b.origin.x.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    a.origin
                        .y
                        .0
                        .partial_cmp(&b.origin.y.0)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        rects
    } else {
        Vec::new()
    };

    let mut expected_polys: Vec<Rect> = if dots {
        union_rect(&expected_dots)
            .map(|r| vec![r])
            .unwrap_or_default()
    } else {
        series
            .iter()
            .enumerate()
            .map(|(i, values)| {
                fret_ui_shadcn::recharts_geometry::radar_polygon_rect(
                    svg_rect, values, domain_max, layout,
                )
                .unwrap_or_else(|| {
                    panic!("{web_name}: failed to compute radar polygon for series {i}")
                })
            })
            .collect()
    };
    expected_polys.sort_by(|a, b| {
        a.origin
            .x
            .0
            .partial_cmp(&b.origin.x.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.origin
                    .y
                    .0
                    .partial_cmp(&b.origin.y.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    let web_polys = web_find_radar_polygons(&theme.root);
    assert_eq!(
        expected_polys.len(),
        web_polys.len(),
        "{web_name}: expected {} radar polygon(s), got {}",
        expected_polys.len(),
        web_polys.len()
    );

    for (i, (expected, web)) in expected_polys.iter().zip(web_polys.iter()).enumerate() {
        assert_rect_close_px(&format!("{web_name} radar-{i}"), *expected, web.rect, 1.0);
    }

    if grid_polygon {
        let sides = series.first().map(|s| s.len()).unwrap_or(0).max(3);
        let expected = if let Some(radii) = grid_radii_override {
            fret_ui_shadcn::recharts_geometry::radar_grid_polygon_rects_with_radii(
                svg_rect, sides, radii,
            )
        } else {
            fret_ui_shadcn::recharts_geometry::radar_grid_polygon_rects(svg_rect, sides, layout)
        };
        let actual = web_find_polar_grid_concentric_polygons(&theme.root);
        assert_eq!(
            expected.len(),
            actual.len(),
            "{web_name}: expected {} concentric polygon(s), got {}",
            expected.len(),
            actual.len()
        );
        for (i, (expected, web)) in expected.iter().zip(actual.iter()).enumerate() {
            assert_rect_close_px(
                &format!("{web_name} grid-poly-{i}"),
                *expected,
                web.rect,
                1.0,
            );
        }
    }

    if grid_circle {
        let expected = fret_ui_shadcn::recharts_geometry::radar_grid_circle_rects(svg_rect, layout);
        let actual = web_find_polar_grid_concentric_circles(&theme.root);
        assert_eq!(
            expected.len(),
            actual.len(),
            "{web_name}: expected {} concentric circle(s), got {}",
            expected.len(),
            actual.len()
        );
        for (i, (expected, web)) in expected.iter().zip(actual.iter()).enumerate() {
            assert_rect_close_px(
                &format!("{web_name} grid-circle-{i}"),
                *expected,
                web.rect,
                1.0,
            );
        }
    }

    let angles = web_find_polar_grid_angle_lines(&theme.root);
    let expected_angle_groups = if radial_lines { 1 } else { 0 };
    assert_eq!(
        angles.len(),
        expected_angle_groups,
        "{web_name}: expected {} polar angle grid group(s), got {}",
        expected_angle_groups,
        angles.len()
    );

    if dots {
        let actual = web_find_radar_dots(&theme.root);
        assert_eq!(
            expected_dots.len(),
            actual.len(),
            "{web_name}: expected {} dot(s), got {}",
            expected_dots.len(),
            actual.len()
        );
        for (i, (expected, web)) in expected_dots.iter().zip(actual.iter()).enumerate() {
            assert_rect_close_px(&format!("{web_name} dot-{i}"), *expected, web.rect, 1.0);
        }
    }
}

#[test]
fn web_vs_fret_layout_chart_radar_geometry_matches_web() {
    let default = [186.0_f32, 305.0, 237.0, 273.0, 209.0, 214.0];
    let default_grid_fill = [186.0_f32, 285.0, 237.0, 203.0, 209.0, 264.0];
    let circle_no_lines = [186.0_f32, 305.0, 237.0, 203.0, 209.0, 214.0];

    let multiple_desktop = [186.0_f32, 305.0, 237.0, 73.0, 209.0, 214.0];
    let multiple_mobile = [80.0_f32, 200.0, 120.0, 190.0, 130.0, 140.0];

    let lines_only_desktop = [186.0_f32, 185.0, 207.0, 173.0, 160.0, 174.0];
    let lines_only_mobile = [160.0_f32, 170.0, 180.0, 160.0, 190.0, 204.0];

    assert_radar_geometry_matches_web(
        "chart-radar-default",
        &[&default],
        fret_ui_shadcn::recharts_geometry::PolarChartLayout::default(),
        true,
        false,
        None,
        true,
        false,
    );

    assert_radar_geometry_matches_web(
        "chart-radar-dots",
        &[&default],
        fret_ui_shadcn::recharts_geometry::PolarChartLayout::default(),
        true,
        false,
        None,
        true,
        true,
    );

    assert_radar_geometry_matches_web(
        "chart-radar-grid-none",
        &[&default],
        fret_ui_shadcn::recharts_geometry::PolarChartLayout::default(),
        false,
        false,
        None,
        false,
        true,
    );

    assert_radar_geometry_matches_web(
        "chart-radar-grid-fill",
        &[&default_grid_fill],
        fret_ui_shadcn::recharts_geometry::PolarChartLayout::default(),
        true,
        false,
        None,
        true,
        false,
    );

    assert_radar_geometry_matches_web(
        "chart-radar-grid-circle",
        &[&default],
        fret_ui_shadcn::recharts_geometry::PolarChartLayout::default(),
        false,
        true,
        None,
        true,
        true,
    );

    assert_radar_geometry_matches_web(
        "chart-radar-grid-circle-fill",
        &[&default_grid_fill],
        fret_ui_shadcn::recharts_geometry::PolarChartLayout::default(),
        false,
        true,
        None,
        true,
        false,
    );

    assert_radar_geometry_matches_web(
        "chart-radar-grid-circle-no-lines",
        &[&circle_no_lines],
        fret_ui_shadcn::recharts_geometry::PolarChartLayout::default(),
        false,
        true,
        None,
        false,
        true,
    );

    assert_radar_geometry_matches_web(
        "chart-radar-grid-custom",
        &[&default],
        fret_ui_shadcn::recharts_geometry::PolarChartLayout::default(),
        true,
        false,
        Some(&[90.0]),
        false,
        false,
    );

    assert_radar_geometry_matches_web(
        "chart-radar-multiple",
        &[&multiple_desktop, &multiple_mobile],
        fret_ui_shadcn::recharts_geometry::PolarChartLayout::default(),
        true,
        false,
        None,
        true,
        false,
    );

    assert_radar_geometry_matches_web(
        "chart-radar-lines-only",
        &[&lines_only_desktop, &lines_only_mobile],
        fret_ui_shadcn::recharts_geometry::PolarChartLayout::default(),
        true,
        false,
        None,
        false,
        false,
    );

    assert_radar_geometry_matches_web(
        "chart-radar-radius",
        &[&multiple_desktop, &multiple_mobile],
        fret_ui_shadcn::recharts_geometry::PolarChartLayout::default(),
        true,
        false,
        None,
        true,
        false,
    );

    {
        let mut layout = fret_ui_shadcn::recharts_geometry::PolarChartLayout::default();
        layout.margin_top_px = 10.0;
        layout.margin_right_px = 10.0;
        layout.margin_bottom_px = 10.0;
        layout.margin_left_px = 10.0;

        assert_radar_geometry_matches_web(
            "chart-radar-label-custom",
            &[&multiple_desktop, &multiple_mobile],
            layout,
            true,
            false,
            None,
            true,
            false,
        );
    }

    {
        let mut layout = fret_ui_shadcn::recharts_geometry::PolarChartLayout::default();
        layout.margin_top_px = -40.0;
        layout.margin_bottom_px = -10.0;

        assert_radar_geometry_matches_web(
            "chart-radar-icons",
            &[&multiple_desktop, &multiple_mobile],
            layout,
            true,
            false,
            None,
            true,
            false,
        );
    }
}

#[test]
fn web_vs_fret_layout_chart_radial_geometry_matches_web() {
    let values = [275.0_f32, 200.0, 187.0, 173.0, 90.0];

    fn svg_rect(theme: &WebGoldenTheme) -> Rect {
        let svg = web_find_pie_svg(&theme.root);
        Rect::new(
            Point::new(Px(svg.rect.x), Px(svg.rect.y)),
            CoreSize::new(Px(svg.rect.w), Px(svg.rect.h)),
        )
    }

    {
        let web = read_web_golden("chart-radial-grid");
        let theme = web_theme(&web);
        let svg_rect = svg_rect(theme);
        let expected_grid = fret_ui_shadcn::recharts_geometry::radial_grid_circle_rects(
            svg_rect,
            30.0,
            100.0,
            values.len(),
        );
        let expected_sectors = fret_ui_shadcn::recharts_geometry::radial_bar_sector_rects(
            svg_rect, &values, 275.0, 0.0, 360.0, 30.0, 100.0, 5.6,
        );

        let actual_grid = web_find_polar_grid_concentric_circles(&theme.root);
        assert_eq!(
            expected_grid.len(),
            actual_grid.len(),
            "chart-radial-grid: expected {} concentric circle(s), got {}",
            expected_grid.len(),
            actual_grid.len()
        );
        for (i, (expected, web)) in expected_grid.iter().zip(actual_grid.iter()).enumerate() {
            assert_rect_close_px(
                &format!("chart-radial-grid grid-circle-{i}"),
                *expected,
                web.rect,
                1.0,
            );
        }

        let mut actual_sectors = web_find_radial_bar_sectors(&theme.root);
        sort_radial_band_nodes_by_outer_radius(svg_rect, &mut actual_sectors);
        assert_eq!(
            expected_sectors.len(),
            actual_sectors.len(),
            "chart-radial-grid: expected {} sector(s), got {}",
            expected_sectors.len(),
            actual_sectors.len()
        );
        for (i, (expected, web)) in expected_sectors
            .iter()
            .zip(actual_sectors.iter())
            .enumerate()
        {
            assert_rect_close_px(
                &format!("chart-radial-grid sector-{i}"),
                *expected,
                web.rect,
                1.0,
            );
        }

        let angles = web_find_polar_grid_angle_lines(&theme.root);
        assert_eq!(
            angles.len(),
            1,
            "chart-radial-grid: expected 1 polar angle grid group, got {}",
            angles.len()
        );
    }

    {
        let web_name = "chart-radial-simple";
        let web = read_web_golden(web_name);
        let theme = web_theme(&web);
        let svg_rect = svg_rect(theme);

        let expected_bg = fret_ui_shadcn::recharts_geometry::radial_bar_background_rects(
            svg_rect,
            values.len(),
            0.0,
            360.0,
            30.0,
            110.0,
            5.6,
        );
        let expected_fg = fret_ui_shadcn::recharts_geometry::radial_bar_sector_rects(
            svg_rect, &values, 1400.0, 0.0, 360.0, 30.0, 110.0, 5.6,
        );

        let mut actual_bg = web_find_radial_bar_background_sectors(&theme.root);
        sort_radial_band_nodes_by_outer_radius(svg_rect, &mut actual_bg);
        assert_eq!(
            expected_bg.len(),
            actual_bg.len(),
            "{web_name}: expected {} background sector(s), got {}",
            expected_bg.len(),
            actual_bg.len()
        );
        for (i, (expected, web)) in expected_bg.iter().zip(actual_bg.iter()).enumerate() {
            assert_rect_close_px(&format!("{web_name} bg-{i}"), *expected, web.rect, 1.0);
        }

        let mut actual_fg = web_find_radial_bar_sectors(&theme.root);
        sort_radial_band_nodes_by_outer_radius(svg_rect, &mut actual_fg);
        assert_eq!(
            expected_fg.len(),
            actual_fg.len(),
            "{web_name}: expected {} sector(s), got {}",
            expected_fg.len(),
            actual_fg.len()
        );
        for (i, (expected, web)) in expected_fg.iter().zip(actual_fg.iter()).enumerate() {
            assert_rect_close_px(&format!("{web_name} fg-{i}"), *expected, web.rect, 1.0);
        }
    }

    {
        let web_name = "chart-radial-label";
        let web = read_web_golden(web_name);
        let theme = web_theme(&web);
        let svg_rect = svg_rect(theme);

        // `endAngle` is intentionally > 360° in the upstream example; Recharts/d3-shape treats
        // angles as raw numbers (no modulo), so we do the same here.
        let start_angle = -90.0;
        let end_angle = 380.0;

        let expected_bg = fret_ui_shadcn::recharts_geometry::radial_bar_background_rects(
            svg_rect,
            values.len(),
            0.0,
            360.0,
            30.0,
            110.0,
            5.6,
        );
        let expected_fg = fret_ui_shadcn::recharts_geometry::radial_bar_sector_rects(
            svg_rect,
            &values,
            1400.0,
            start_angle,
            end_angle,
            30.0,
            110.0,
            5.6,
        );

        let mut actual_bg = web_find_radial_bar_background_sectors(&theme.root);
        sort_radial_band_nodes_by_outer_radius(svg_rect, &mut actual_bg);
        assert_eq!(
            expected_bg.len(),
            actual_bg.len(),
            "{web_name}: expected {} background sector(s), got {}",
            expected_bg.len(),
            actual_bg.len()
        );
        for (i, (expected, web)) in expected_bg.iter().zip(actual_bg.iter()).enumerate() {
            assert_rect_close_px(&format!("{web_name} bg-{i}"), *expected, web.rect, 1.0);
        }

        let mut actual_fg = web_find_radial_bar_sectors(&theme.root);
        sort_radial_band_nodes_by_outer_radius(svg_rect, &mut actual_fg);
        assert_eq!(
            expected_fg.len(),
            actual_fg.len(),
            "{web_name}: expected {} sector(s), got {}",
            expected_fg.len(),
            actual_fg.len()
        );
        for (i, (expected, web)) in expected_fg.iter().zip(actual_fg.iter()).enumerate() {
            assert_rect_close_px(&format!("{web_name} fg-{i}"), *expected, web.rect, 1.0);
        }
    }

    for (web_name, total_span, domain_max, band_inner, band_outer) in [
        (
            "chart-radial-shape",
            100.0_f32,
            7200.0_f32,
            68.0_f32,
            92.0_f32,
        ),
        (
            "chart-radial-text",
            250.0_f32,
            950.0_f32,
            74.0_f32,
            86.0_f32,
        ),
    ] {
        let value = if web_name == "chart-radial-shape" {
            1260.0
        } else {
            200.0
        };
        let web = read_web_golden(web_name);
        let theme = web_theme(&web);
        let svg_rect = svg_rect(theme);

        let circles = web_find_polar_grid_concentric_circles(&theme.root);
        let mut expected_circles =
            fret_ui_shadcn::recharts_geometry::polar_circle_rects(svg_rect, &[86.0, 74.0]);
        expected_circles.sort_by(|a, b| {
            a.size
                .width
                .0
                .partial_cmp(&b.size.width.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    a.size
                        .height
                        .0
                        .partial_cmp(&b.size.height.0)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        assert_eq!(
            expected_circles.len(),
            circles.len(),
            "{web_name}: expected {} concentric circle(s), got {}",
            expected_circles.len(),
            circles.len()
        );
        for (i, (expected, web)) in expected_circles.iter().zip(circles.iter()).enumerate() {
            assert_rect_close_px(
                &format!("{web_name} grid-circle-{i}"),
                *expected,
                web.rect,
                1.0,
            );
        }

        let bg = web_find_radial_bar_background_sectors(&theme.root);
        assert_eq!(
            bg.len(),
            1,
            "{web_name}: expected 1 background sector, got {}",
            bg.len()
        );
        let corner_radius = if web_name == "chart-radial-text" {
            10.0
        } else {
            0.0
        };
        let expected_bg =
            fret_ui_shadcn::recharts_geometry::annular_sector_rect_with_corner_radius(
                svg_rect,
                0.0,
                total_span,
                band_inner,
                band_outer,
                corner_radius,
            )
            .unwrap_or_else(|| panic!("{web_name}: failed to compute background rect"));
        assert_rect_close_px(&format!("{web_name} bg"), expected_bg, bg[0].rect, 1.0);

        let fg = web_find_radial_bar_sectors(&theme.root);
        assert_eq!(
            fg.len(),
            1,
            "{web_name}: expected 1 sector, got {}",
            fg.len()
        );
        let end = (value / domain_max) * total_span;
        let expected_fg = fret_ui_shadcn::recharts_geometry::annular_sector_rect(
            svg_rect, 0.0, end, band_inner, band_outer,
        )
        .unwrap_or_else(|| panic!("{web_name}: failed to compute sector rect"));
        assert_rect_close_px(&format!("{web_name} fg"), expected_fg, fg[0].rect, 1.0);
    }

    {
        let web = read_web_golden("chart-radial-stacked");
        let theme = web_theme(&web);
        let svg_rect = svg_rect(theme);
        let desktop = 1260.0_f32;
        let mobile = 570.0_f32;
        let total = desktop + mobile;
        let span = 180.0_f32;

        let inner = 69.0_f32;
        let outer = 88.7_f32;

        let desktop_end = (desktop / total) * span;
        let expected = [
            fret_ui_shadcn::recharts_geometry::annular_sector_rect(
                svg_rect,
                0.0,
                desktop_end,
                inner,
                outer,
            )
            .unwrap_or_else(|| panic!("chart-radial-stacked: failed to compute desktop rect")),
            fret_ui_shadcn::recharts_geometry::annular_sector_rect(
                svg_rect,
                desktop_end,
                span,
                inner,
                outer,
            )
            .unwrap_or_else(|| panic!("chart-radial-stacked: failed to compute mobile rect")),
        ];

        let mut actual = web_find_radial_bar_sectors(&theme.root);
        actual.sort_by(|a, b| {
            a.rect
                .x
                .partial_cmp(&b.rect.x)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut expected = expected;
        expected.sort_by(|a, b| {
            a.origin
                .x
                .0
                .partial_cmp(&b.origin.x.0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        assert_eq!(
            expected.len(),
            actual.len(),
            "chart-radial-stacked: expected {} sector(s), got {}",
            expected.len(),
            actual.len()
        );

        for (i, (expected, web)) in expected.iter().zip(actual.iter()).enumerate() {
            assert_rect_close_px(
                &format!("chart-radial-stacked sector-{i}"),
                *expected,
                web.rect,
                1.5,
            );
        }
    }
}

fn assert_chart_series_curve_bounds_match_web(
    web_name: &str,
    series: &[(&[f32], fret_ui_shadcn::recharts_geometry::CurveKind)],
    y_tick_count: usize,
    domain_max: Option<f32>,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_chart = web_find_chart_container(&theme.root);
    let web_plot = web_find_chart_grid(web_chart);
    let web_curves = web_find_chart_series_curves(web_chart);
    assert_eq!(
        web_curves.len(),
        series.len(),
        "{web_name}: expected {} series curve(s), got {}",
        series.len(),
        web_curves.len()
    );

    let plot = Rect::new(
        Point::new(Px(web_plot.rect.x), Px(web_plot.rect.y)),
        CoreSize::new(Px(web_plot.rect.w), Px(web_plot.rect.h)),
    );

    let domain_max = domain_max.unwrap_or_else(|| {
        let mut all = Vec::new();
        for (values, _) in series {
            all.extend_from_slice(values);
        }
        fret_ui_shadcn::recharts_geometry::nice_domain_max_for_values(&all, y_tick_count)
    });

    let mut expected: Vec<Rect> = series
        .iter()
        .enumerate()
        .map(|(i, (values, kind))| {
            fret_ui_shadcn::recharts_geometry::line_curve_bounds(plot, values, *kind, domain_max)
                .unwrap_or_else(|| {
                    panic!("{web_name}: failed to compute curve bounds for series {i}")
                })
        })
        .collect();
    expected.sort_by(|a, b| {
        a.origin
            .y
            .0
            .partial_cmp(&b.origin.y.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.origin
                    .x
                    .0
                    .partial_cmp(&b.origin.x.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    for (i, (expected, web_curve)) in expected.iter().zip(web_curves.iter()).enumerate() {
        assert_rect_close_px(
            &format!("{web_name} curve-{i}"),
            *expected,
            web_curve.rect,
            1.0,
        );
    }
}

fn assert_chart_stacked_area_fill_bounds_match_web(
    web_name: &str,
    stacked_series: &[(&[f32], fret_ui_shadcn::recharts_geometry::CurveKind)],
    y_tick_count: usize,
    domain_max: Option<f32>,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_chart = web_find_chart_container(&theme.root);
    let web_plot = web_find_chart_grid(web_chart);
    let web_fills = web_find_chart_area_fills(web_chart);
    assert_eq!(
        web_fills.len(),
        stacked_series.len(),
        "{web_name}: expected {} area fill(s), got {}",
        stacked_series.len(),
        web_fills.len()
    );

    let plot = Rect::new(
        Point::new(Px(web_plot.rect.x), Px(web_plot.rect.y)),
        CoreSize::new(Px(web_plot.rect.w), Px(web_plot.rect.h)),
    );

    let domain_max = domain_max.unwrap_or_else(|| {
        let mut all = Vec::new();
        for (values, _) in stacked_series {
            all.extend_from_slice(values);
        }
        fret_ui_shadcn::recharts_geometry::nice_domain_max_for_values(&all, y_tick_count)
    });

    fn union_rect(a: Rect, b: Rect) -> Rect {
        let min_x = a.origin.x.0.min(b.origin.x.0);
        let min_y = a.origin.y.0.min(b.origin.y.0);
        let max_x = (a.origin.x.0 + a.size.width.0).max(b.origin.x.0 + b.size.width.0);
        let max_y = (a.origin.y.0 + a.size.height.0).max(b.origin.y.0 + b.size.height.0);
        Rect::new(
            Point::new(Px(min_x), Px(min_y)),
            CoreSize::new(Px(max_x - min_x), Px(max_y - min_y)),
        )
    }

    let curve_bounds: Vec<Rect> = stacked_series
        .iter()
        .enumerate()
        .map(|(i, (values, kind))| {
            fret_ui_shadcn::recharts_geometry::line_curve_bounds(plot, values, *kind, domain_max)
                .unwrap_or_else(|| {
                    panic!("{web_name}: failed to compute curve bounds for series {i}")
                })
        })
        .collect();

    let plot_bottom = plot.origin.y.0 + plot.size.height.0;
    let mut expected: Vec<Rect> = curve_bounds
        .iter()
        .enumerate()
        .map(|(i, top)| {
            let baseline = if i == 0 {
                Rect::new(
                    Point::new(Px(top.origin.x.0), Px(plot_bottom)),
                    CoreSize::new(Px(top.size.width.0), Px(0.0)),
                )
            } else {
                curve_bounds[i - 1]
            };
            union_rect(*top, baseline)
        })
        .collect();

    expected.sort_by(|a, b| {
        a.origin
            .y
            .0
            .partial_cmp(&b.origin.y.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.origin
                    .x
                    .0
                    .partial_cmp(&b.origin.x.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    for (i, (expected, web_fill)) in expected.iter().zip(web_fills.iter()).enumerate() {
        assert_rect_close_px(
            &format!("{web_name} fill-{i}"),
            *expected,
            web_fill.rect,
            1.0,
        );
    }
}

#[test]
fn web_vs_fret_layout_chart_line_variants_curve_bounds_match_web() {
    let month_desktop = [186.0_f32, 305.0, 237.0, 73.0, 209.0, 214.0];

    let cases = [
        (
            "chart-line-linear",
            fret_ui_shadcn::recharts_geometry::CurveKind::Linear,
        ),
        (
            "chart-line-step",
            fret_ui_shadcn::recharts_geometry::CurveKind::Step,
        ),
        (
            "chart-line-dots",
            fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
        ),
        (
            "chart-line-dots-custom",
            fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
        ),
        (
            "chart-line-label",
            fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
        ),
    ];

    for (web_name, kind) in cases {
        assert_chart_series_curve_bounds_match_web(web_name, &[(&month_desktop, kind)], 5, None);
    }
}

#[test]
fn web_vs_fret_layout_chart_line_dots_colors_curve_bounds_match_web() {
    let visitors = [275.0_f32, 200.0, 187.0, 173.0, 90.0];
    assert_chart_series_curve_bounds_match_web(
        "chart-line-dots-colors",
        &[(
            &visitors,
            fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
        )],
        5,
        None,
    );
}

#[test]
fn web_vs_fret_layout_chart_line_label_custom_curve_bounds_match_web() {
    let visitors = [275.0_f32, 200.0, 187.0, 173.0, 90.0];
    assert_chart_series_curve_bounds_match_web(
        "chart-line-label-custom",
        &[(
            &visitors,
            fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
        )],
        5,
        None,
    );
}

#[test]
fn web_vs_fret_layout_chart_line_multiple_curve_bounds_match_web() {
    let desktop = [186.0_f32, 305.0, 237.0, 73.0, 209.0, 214.0];
    let mobile = [80.0_f32, 200.0, 120.0, 190.0, 130.0, 140.0];
    assert_chart_series_curve_bounds_match_web(
        "chart-line-multiple",
        &[
            (
                &desktop,
                fret_ui_shadcn::recharts_geometry::CurveKind::Monotone,
            ),
            (
                &mobile,
                fret_ui_shadcn::recharts_geometry::CurveKind::Monotone,
            ),
        ],
        5,
        None,
    );
}

#[test]
fn web_vs_fret_layout_chart_line_interactive_curve_bounds_match_web() {
    assert_chart_series_curve_bounds_match_web(
        "chart-line-interactive",
        &[(
            &CHART_INTERACTIVE_DESKTOP,
            fret_ui_shadcn::recharts_geometry::CurveKind::Monotone,
        )],
        5,
        None,
    );
}

#[test]
fn web_vs_fret_layout_chart_line_interactive_mobile_curve_bounds_match_web() {
    assert_chart_series_curve_bounds_match_web(
        "chart-line-interactive.mobile",
        &[(
            &CHART_INTERACTIVE_MOBILE,
            fret_ui_shadcn::recharts_geometry::CurveKind::Monotone,
        )],
        5,
        None,
    );
}

#[test]
fn web_vs_fret_layout_chart_area_variants_curve_bounds_match_web() {
    let desktop = [186.0_f32, 305.0, 237.0, 73.0, 209.0, 214.0];
    let mobile = [80.0_f32, 200.0, 120.0, 190.0, 130.0, 140.0];
    let stacked: Vec<f32> = desktop
        .iter()
        .zip(mobile.iter())
        .map(|(d, m)| d + m)
        .collect();

    let stacked_series = &[
        (
            &mobile[..],
            fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
        ),
        (
            &stacked[..],
            fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
        ),
    ];

    let cases = [
        "chart-area-axes",
        "chart-area-gradient",
        "chart-area-icons",
        "chart-area-stacked",
    ];

    for web_name in cases {
        let tick_count = if web_name == "chart-area-axes" { 3 } else { 5 };
        assert_chart_series_curve_bounds_match_web(web_name, stacked_series, tick_count, None);
        assert_chart_stacked_area_fill_bounds_match_web(web_name, stacked_series, tick_count, None);
    }

    assert_chart_series_curve_bounds_match_web(
        "chart-area-linear",
        &[(
            &desktop,
            fret_ui_shadcn::recharts_geometry::CurveKind::Linear,
        )],
        5,
        None,
    );

    assert_chart_series_curve_bounds_match_web(
        "chart-area-step",
        &[(&desktop, fret_ui_shadcn::recharts_geometry::CurveKind::Step)],
        5,
        None,
    );
}

#[test]
fn web_vs_fret_layout_chart_area_stacked_expand_curve_bounds_match_web() {
    let desktop = [186.0_f32, 305.0, 237.0, 73.0, 209.0, 214.0];
    let mobile = [80.0_f32, 200.0, 120.0, 190.0, 130.0, 140.0];
    let other = [45.0_f32, 100.0, 150.0, 50.0, 100.0, 160.0];

    let mut other_top = Vec::new();
    let mut mobile_top = Vec::new();
    let mut desktop_top = Vec::new();
    for ((d, m), o) in desktop.iter().zip(mobile.iter()).zip(other.iter()) {
        let total = d + m + o;
        other_top.push(o / total);
        mobile_top.push((o + m) / total);
        desktop_top.push(1.0);
    }

    assert_chart_series_curve_bounds_match_web(
        "chart-area-stacked-expand",
        &[
            (
                &other_top[..],
                fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
            ),
            (
                &mobile_top[..],
                fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
            ),
            (
                &desktop_top[..],
                fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
            ),
        ],
        5,
        Some(1.0),
    );

    assert_chart_stacked_area_fill_bounds_match_web(
        "chart-area-stacked-expand",
        &[
            (
                &other_top[..],
                fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
            ),
            (
                &mobile_top[..],
                fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
            ),
            (
                &desktop_top[..],
                fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
            ),
        ],
        5,
        Some(1.0),
    );
}

#[test]
fn web_vs_fret_layout_chart_area_interactive_curve_bounds_match_web() {
    let stacked: Vec<f32> = CHART_INTERACTIVE_DESKTOP
        .iter()
        .zip(CHART_INTERACTIVE_MOBILE.iter())
        .map(|(d, m)| d + m)
        .collect();

    assert_chart_series_curve_bounds_match_web(
        "chart-area-interactive",
        &[
            (
                &CHART_INTERACTIVE_MOBILE,
                fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
            ),
            (
                &stacked,
                fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
            ),
        ],
        5,
        None,
    );
}

#[test]
fn web_vs_fret_layout_chart_area_interactive_fill_bounds_match_web() {
    let stacked: Vec<f32> = CHART_INTERACTIVE_DESKTOP
        .iter()
        .zip(CHART_INTERACTIVE_MOBILE.iter())
        .map(|(d, m)| d + m)
        .collect();

    assert_chart_stacked_area_fill_bounds_match_web(
        "chart-area-interactive",
        &[
            (
                &CHART_INTERACTIVE_MOBILE,
                fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
            ),
            (
                &stacked,
                fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
            ),
        ],
        5,
        None,
    );
}

#[test]
fn web_vs_fret_layout_chart_area_interactive_30d_curve_bounds_match_web() {
    let desktop = &CHART_INTERACTIVE_DESKTOP[60..];
    let mobile = &CHART_INTERACTIVE_MOBILE[60..];

    let stacked: Vec<f32> = desktop
        .iter()
        .zip(mobile.iter())
        .map(|(d, m)| d + m)
        .collect();

    assert_chart_series_curve_bounds_match_web(
        "chart-area-interactive.30d",
        &[
            (
                mobile,
                fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
            ),
            (
                &stacked,
                fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
            ),
        ],
        5,
        None,
    );
}

#[test]
fn web_vs_fret_layout_chart_area_interactive_30d_fill_bounds_match_web() {
    let desktop = &CHART_INTERACTIVE_DESKTOP[60..];
    let mobile = &CHART_INTERACTIVE_MOBILE[60..];

    let stacked: Vec<f32> = desktop
        .iter()
        .zip(mobile.iter())
        .map(|(d, m)| d + m)
        .collect();

    assert_chart_stacked_area_fill_bounds_match_web(
        "chart-area-interactive.30d",
        &[
            (
                mobile,
                fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
            ),
            (
                &stacked,
                fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
            ),
        ],
        5,
        None,
    );
}

#[test]
fn web_vs_fret_layout_chart_area_interactive_7d_curve_bounds_match_web() {
    let desktop = &CHART_INTERACTIVE_DESKTOP[83..];
    let mobile = &CHART_INTERACTIVE_MOBILE[83..];

    let stacked: Vec<f32> = desktop
        .iter()
        .zip(mobile.iter())
        .map(|(d, m)| d + m)
        .collect();

    assert_chart_series_curve_bounds_match_web(
        "chart-area-interactive.7d",
        &[
            (
                mobile,
                fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
            ),
            (
                &stacked,
                fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
            ),
        ],
        5,
        None,
    );
}

#[test]
fn web_vs_fret_layout_chart_area_interactive_7d_fill_bounds_match_web() {
    let desktop = &CHART_INTERACTIVE_DESKTOP[83..];
    let mobile = &CHART_INTERACTIVE_MOBILE[83..];

    let stacked: Vec<f32> = desktop
        .iter()
        .zip(mobile.iter())
        .map(|(d, m)| d + m)
        .collect();

    assert_chart_stacked_area_fill_bounds_match_web(
        "chart-area-interactive.7d",
        &[
            (
                mobile,
                fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
            ),
            (
                &stacked,
                fret_ui_shadcn::recharts_geometry::CurveKind::Natural,
            ),
        ],
        5,
        None,
    );
}
