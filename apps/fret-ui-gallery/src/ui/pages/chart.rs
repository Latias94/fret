use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_chart(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum DemoChartKind {
        Area,
        BarMultiple,
        BarMixed,
        LineMultiple,
    }

    let chart_canvas =
        |cx: &mut ElementContext<'_, App>, kind: DemoChartKind, test_id: Arc<str>| {
            use delinea::data::{Column, DataTable};
            use delinea::{
                AxisKind, AxisScale, CategoryAxisScale, ChartSpec, DatasetSpec, FieldSpec,
                GridSpec, SeriesEncode, SeriesKind, SeriesSpec,
            };
            use fret_chart::ChartCanvas;
            use fret_ui::element::{LayoutStyle, Length};
            use fret_ui::retained_bridge::RetainedSubtreeProps;

            cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
                use fret_ui::retained_bridge::UiTreeRetainedExt as _;

                let dataset_id = delinea::ids::DatasetId::new(1);
                let grid_id = delinea::ids::GridId::new(1);
                let x_axis = delinea::AxisId::new(1);
                let y_axis = delinea::AxisId::new(2);
                let x_field = delinea::FieldId::new(1);
                let y_field_1 = delinea::FieldId::new(2);
                let y_field_2 = delinea::FieldId::new(3);

                let (categories, x, y1, y2, series) = match kind {
                    DemoChartKind::Area => {
                        let categories = vec![
                            "January".to_string(),
                            "February".to_string(),
                            "March".to_string(),
                            "April".to_string(),
                            "May".to_string(),
                            "June".to_string(),
                        ];
                        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
                        let y1 = vec![186.0, 305.0, 237.0, 73.0, 209.0, 214.0];
                        let series = vec![SeriesSpec {
                            id: delinea::ids::SeriesId::new(1),
                            name: Some("Desktop".to_string()),
                            kind: SeriesKind::Area,
                            dataset: dataset_id,
                            encode: SeriesEncode {
                                x: x_field,
                                y: y_field_1,
                                y2: None,
                            },
                            x_axis,
                            y_axis,
                            stack: None,
                            stack_strategy: Default::default(),
                            bar_layout: Default::default(),
                            area_baseline: None,
                            lod: None,
                        }];
                        (categories, x, y1, None, series)
                    }
                    DemoChartKind::BarMultiple => {
                        let categories = vec![
                            "January".to_string(),
                            "February".to_string(),
                            "March".to_string(),
                            "April".to_string(),
                            "May".to_string(),
                            "June".to_string(),
                        ];
                        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
                        let desktop = vec![186.0, 305.0, 237.0, 73.0, 209.0, 214.0];
                        let mobile = vec![80.0, 200.0, 120.0, 190.0, 130.0, 140.0];
                        let series = vec![
                            SeriesSpec {
                                id: delinea::ids::SeriesId::new(1),
                                name: Some("Desktop".to_string()),
                                kind: SeriesKind::Bar,
                                dataset: dataset_id,
                                encode: SeriesEncode {
                                    x: x_field,
                                    y: y_field_1,
                                    y2: None,
                                },
                                x_axis,
                                y_axis,
                                stack: None,
                                stack_strategy: Default::default(),
                                bar_layout: Default::default(),
                                area_baseline: None,
                                lod: None,
                            },
                            SeriesSpec {
                                id: delinea::ids::SeriesId::new(2),
                                name: Some("Mobile".to_string()),
                                kind: SeriesKind::Bar,
                                dataset: dataset_id,
                                encode: SeriesEncode {
                                    x: x_field,
                                    y: y_field_2,
                                    y2: None,
                                },
                                x_axis,
                                y_axis,
                                stack: None,
                                stack_strategy: Default::default(),
                                bar_layout: Default::default(),
                                area_baseline: None,
                                lod: None,
                            },
                        ];
                        (categories, x, desktop, Some(mobile), series)
                    }
                    DemoChartKind::BarMixed => {
                        let categories = vec![
                            "chrome".to_string(),
                            "safari".to_string(),
                            "firefox".to_string(),
                            "edge".to_string(),
                            "other".to_string(),
                        ];
                        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0];
                        let y1 = vec![275.0, 200.0, 187.0, 173.0, 90.0];
                        let series = vec![SeriesSpec {
                            id: delinea::ids::SeriesId::new(1),
                            name: Some("Visitors".to_string()),
                            kind: SeriesKind::Bar,
                            dataset: dataset_id,
                            encode: SeriesEncode {
                                x: x_field,
                                y: y_field_1,
                                y2: None,
                            },
                            x_axis,
                            y_axis,
                            stack: None,
                            stack_strategy: Default::default(),
                            bar_layout: Default::default(),
                            area_baseline: None,
                            lod: None,
                        }];
                        (categories, x, y1, None, series)
                    }
                    DemoChartKind::LineMultiple => {
                        let categories = vec![
                            "January".to_string(),
                            "February".to_string(),
                            "March".to_string(),
                            "April".to_string(),
                            "May".to_string(),
                            "June".to_string(),
                        ];
                        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
                        let desktop = vec![186.0, 305.0, 237.0, 73.0, 209.0, 214.0];
                        let mobile = vec![80.0, 200.0, 120.0, 190.0, 130.0, 140.0];
                        let series = vec![
                            SeriesSpec {
                                id: delinea::ids::SeriesId::new(1),
                                name: Some("Desktop".to_string()),
                                kind: SeriesKind::Line,
                                dataset: dataset_id,
                                encode: SeriesEncode {
                                    x: x_field,
                                    y: y_field_1,
                                    y2: None,
                                },
                                x_axis,
                                y_axis,
                                stack: None,
                                stack_strategy: Default::default(),
                                bar_layout: Default::default(),
                                area_baseline: None,
                                lod: None,
                            },
                            SeriesSpec {
                                id: delinea::ids::SeriesId::new(2),
                                name: Some("Mobile".to_string()),
                                kind: SeriesKind::Line,
                                dataset: dataset_id,
                                encode: SeriesEncode {
                                    x: x_field,
                                    y: y_field_2,
                                    y2: None,
                                },
                                x_axis,
                                y_axis,
                                stack: None,
                                stack_strategy: Default::default(),
                                bar_layout: Default::default(),
                                area_baseline: None,
                                lod: None,
                            },
                        ];
                        (categories, x, desktop, Some(mobile), series)
                    }
                };
                let y2 = y2.unwrap_or_else(|| vec![0.0; x.len()]);

                let spec = ChartSpec {
                    id: delinea::ids::ChartId::new(1),
                    viewport: None,
                    datasets: vec![DatasetSpec {
                        id: dataset_id,
                        fields: vec![
                            FieldSpec {
                                id: x_field,
                                column: 0,
                            },
                            FieldSpec {
                                id: y_field_1,
                                column: 1,
                            },
                            FieldSpec {
                                id: y_field_2,
                                column: 2,
                            },
                        ],
                        ..Default::default()
                    }],
                    grids: vec![GridSpec { id: grid_id }],
                    axes: vec![
                        delinea::AxisSpec {
                            id: x_axis,
                            name: Some("X".to_string()),
                            kind: AxisKind::X,
                            grid: grid_id,
                            position: None,
                            scale: AxisScale::Category(CategoryAxisScale { categories }),
                            range: Default::default(),
                        },
                        delinea::AxisSpec {
                            id: y_axis,
                            name: Some("Y".to_string()),
                            kind: AxisKind::Y,
                            grid: grid_id,
                            position: None,
                            scale: Default::default(),
                            range: Default::default(),
                        },
                    ],
                    data_zoom_x: vec![],
                    data_zoom_y: vec![],
                    tooltip: None,
                    axis_pointer: None,
                    visual_maps: vec![],
                    series,
                };

                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout.size.height = Length::Px(Px(220.0));

                let canvas_test_id = test_id.to_string();
                let props = RetainedSubtreeProps::new::<App>(move |ui| {
                    let mut canvas = ChartCanvas::new(spec.clone())
                        .expect("chart spec should be valid")
                        .test_id(canvas_test_id.clone());
                    canvas.set_accessibility_layer(true);
                    canvas.set_input_map(fret_chart::input_map::ChartInputMap::default());

                    let mut table = DataTable::default();
                    table.push_column(Column::F64(x.clone()));
                    table.push_column(Column::F64(y1.clone()));
                    table.push_column(Column::F64(y2.clone()));
                    canvas.engine_mut().datasets_mut().insert(dataset_id, table);

                    let node = ui.create_node_retained(canvas);
                    ui.set_node_view_cache_flags(node, true, true, false);
                    node
                })
                .with_layout(layout);

                let subtree = cx.retained_subtree(props);
                vec![subtree]
            })
        };

    let trending_footer = |cx: &mut ElementContext<'_, App>, secondary: &'static str| {
        let icon = doc_layout::icon(cx, "lucide.trending-up");
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            |cx| {
                vec![
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                ui::text(cx, "Trending up by 5.2% this month")
                                    .font_medium()
                                    .into_element(cx),
                                icon,
                            ]
                        },
                    ),
                    shadcn::typography::muted(cx, secondary),
                ]
            },
        )
    };

    let chart_card = |cx: &mut ElementContext<'_, App>,
                      title: &'static str,
                      description: &'static str,
                      kind: DemoChartKind,
                      footer_secondary: &'static str,
                      test_id: &'static str| {
        let canvas = chart_canvas(cx, kind, Arc::<str>::from(format!("{test_id}-canvas")));

        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new(title).into_element(cx),
                shadcn::CardDescription::new(description).into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![canvas]).into_element(cx),
            shadcn::CardFooter::new(vec![trending_footer(cx, footer_secondary)]).into_element(cx),
        ])
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .max_w(Px(520.0)),
        )
        .into_element(cx)
        .test_id(test_id)
    };

    let demo_cards = {
        let area = chart_card(
            cx,
            "Area Chart",
            "Showing total visitors for the last 6 months",
            DemoChartKind::Area,
            "January - June 2024",
            "ui-gallery-chart-demo-area",
        );
        let bar = chart_card(
            cx,
            "Bar Chart - Multiple",
            "January - June 2024",
            DemoChartKind::BarMultiple,
            "Showing total visitors for the last 6 months",
            "ui-gallery-chart-demo-bar",
        );
        let mixed = chart_card(
            cx,
            "Bar Chart - Mixed",
            "January - June 2024",
            DemoChartKind::BarMixed,
            "Showing total visitors for the last 6 months",
            "ui-gallery-chart-demo-mixed",
        );
        let line = chart_card(
            cx,
            "Line Chart - Multiple",
            "January - June 2024",
            DemoChartKind::LineMultiple,
            "Showing total visitors for the last 6 months",
            "ui-gallery-chart-demo-line",
        );

        doc_layout::wrap_row_snapshot(
            cx,
            &Theme::global(&*cx.app).snapshot(),
            Space::N4,
            fret_ui::element::CrossAlign::Start,
            |_cx| vec![area, bar, mixed, line],
        )
        .test_id("ui-gallery-chart-demo")
    };

    let (chart_1, chart_2, chart_3) = cx.with_theme(|theme| {
        (
            theme.color_token("chart-1"),
            theme.color_token("chart-2"),
            theme.color_token("chart-3"),
        )
    });

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

    let contracts_overview = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Chart UI contracts: Tooltip + Legend content recipes.",
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

    let rtl = doc_layout::rtl(cx, |cx| {
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
                        "يناير",
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
    });

    let notes_stack = doc_layout::notes(
        cx,
        [
            "Demo cards are rendered with `delinea` + `fret-chart` (not Recharts); this is a stand-in to keep chart layout real in native builds.",
            "The shadcn `ChartTooltipContent` / `ChartLegendContent` recipes are validated independently (no runtime wire-up yet).",
            "Keep color mapping stable through `chart-*` tokens to avoid dark-theme drift.",
            "`fret-chart::ChartCanvas` exposes an accessibility layer via keyboard focus + arrow navigation, mirroring Recharts `accessibilityLayer` outcomes at a high level.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Chart demo: Area, Bar (Multiple), Bar (Mixed), Line (Multiple).",
        ),
        vec![
            DocSection::new("Demo", demo_cards)
                .no_shell()
                .max_w(Px(1100.0))
                .code(
                    "rust",
                    r#"// Each chart card hosts a small `fret-chart::ChartCanvas` subtree.
// The layout mirrors shadcn's ChartDemo grid at a high level."#,
                ),
            DocSection::new("Contracts", contracts_overview)
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::ChartTooltipContent::new()
    .label("January")
    .items([shadcn::ChartTooltipItem::new("Desktop", "186")])
    .indicator(shadcn::ChartTooltipIndicator::Dot)
    .into_element(cx);"#,
                ),
            DocSection::new("Tooltip", tooltip_content)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-chart-tooltip")
                .code(
                    "rust",
                    r#"let tooltip = shadcn::ChartTooltipContent::new()
    .label("January")
    .items([
        shadcn::ChartTooltipItem::new("Desktop", "186"),
        shadcn::ChartTooltipItem::new("Mobile", "80"),
    ])
    .indicator(shadcn::ChartTooltipIndicator::Line)
    .into_element(cx);"#,
                ),
            DocSection::new("Legend", legend_content)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-chart-legend")
                .code(
                    "rust",
                    r#"let legend = shadcn::ChartLegendContent::new()
    .items([
        shadcn::ChartLegendItem::new("Desktop"),
        shadcn::ChartLegendItem::new("Mobile"),
    ])
    .vertical_align(shadcn::ChartLegendVerticalAlign::Bottom)
    .wrap(true)
    .into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-chart-rtl")
                .code(
                    "rust",
                    r#"doc_layout::rtl(cx, |cx| {
    shadcn::ChartTooltipContent::new().label("يناير").into_element(cx)
});"#,
                ),
            DocSection::new("Notes", notes_stack).max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-chart-component")]
}
