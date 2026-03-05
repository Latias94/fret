pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_app::App;
use fret_core::Px;
use fret_ui_kit::declarative::{CachedSubtreeExt as _, CachedSubtreeProps};
use fret_ui_kit::ui;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
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
        let icon = shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.trending-up"));
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
                                ui::text("Trending up by 5.2% this month")
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

    fret_ui_kit::ui::h_flex(|_cx| vec![area, bar, mixed, line])
        .gap(Space::N4)
        .wrap()
        .w_full()
        .items_start()
        .into_element(cx)
        .test_id("ui-gallery-chart-demo")
}
// endregion: example
