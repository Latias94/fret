use super::super::super::super::*;

pub(in crate::ui) fn preview_chart_torture(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use delinea::data::{Column, DataTable};
    use delinea::{
        AxisKind, AxisPointerSpec, AxisPointerTrigger, AxisPointerType, AxisRange, AxisScale,
        ChartSpec, DatasetSpec, FieldSpec, GridSpec, SeriesEncode, SeriesKind, SeriesSpec,
        TimeAxisScale,
    };
    use fret_chart::ChartCanvas;
    use fret_ui::element::{LayoutStyle, Length, SemanticsProps};
    use fret_ui::retained_bridge::RetainedSubtreeProps;

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: stress canvas charts with pan/zoom (candidate for prepaint-windowed sampling)."),
                cx.text("Use scripted drag+wheel steps to validate correctness and collect perf bundles."),
            ]
        },
    );

    let chart =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let dataset_id = delinea::ids::DatasetId::new(1);
            let grid_id = delinea::ids::GridId::new(1);
            let x_axis = delinea::AxisId::new(1);
            let y_axis = delinea::AxisId::new(2);
            let series_id = delinea::ids::SeriesId::new(1);
            let x_field = delinea::FieldId::new(1);
            let y_field = delinea::FieldId::new(2);

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
                            id: y_field,
                            column: 1,
                        },
                    ],
                    ..Default::default()
                }],
                grids: vec![GridSpec { id: grid_id }],
                axes: vec![
                    delinea::AxisSpec {
                        id: x_axis,
                        name: Some("Time".to_string()),
                        kind: AxisKind::X,
                        grid: grid_id,
                        position: None,
                        scale: AxisScale::Time(TimeAxisScale),
                        range: Some(AxisRange::Auto),
                    },
                    delinea::AxisSpec {
                        id: y_axis,
                        name: Some("Value".to_string()),
                        kind: AxisKind::Y,
                        grid: grid_id,
                        position: None,
                        scale: Default::default(),
                        range: Some(AxisRange::Auto),
                    },
                ],
                data_zoom_x: vec![],
                data_zoom_y: vec![],
                tooltip: None,
                axis_pointer: Some(AxisPointerSpec {
                    enabled: true,
                    trigger: AxisPointerTrigger::Axis,
                    pointer_type: AxisPointerType::Line,
                    label: Default::default(),
                    snap: false,
                    trigger_distance_px: 12.0,
                    throttle_px: 0.75,
                }),
                visual_maps: vec![],
                series: vec![SeriesSpec {
                    id: series_id,
                    name: Some("Series".to_string()),
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                }],
            };

            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Fill;
            layout.size.height = Length::Px(Px(520.0));

            let props = RetainedSubtreeProps::new::<App>(move |ui| {
                use fret_ui::retained_bridge::UiTreeRetainedExt as _;

                let mut canvas =
                    ChartCanvas::new(spec.clone()).expect("chart spec should be valid");
                canvas.set_input_map(fret_chart::input_map::ChartInputMap::default());

                let base_ms = 1_735_689_600_000.0;
                let interval_ms = 60_000.0;

                let n = 200_000usize;
                let mut x: Vec<f64> = Vec::with_capacity(n);
                let mut y: Vec<f64> = Vec::with_capacity(n);
                for i in 0..n {
                    let t = i as f64 / (n - 1) as f64;
                    let xi = base_ms + interval_ms * i as f64;
                    let theta = t * std::f64::consts::TAU;
                    let yi = (theta * 8.0).sin() * 0.8;
                    x.push(xi);
                    y.push(yi);
                }

                let mut table = DataTable::default();
                table.push_column(Column::F64(x));
                table.push_column(Column::F64(y));
                canvas.engine_mut().datasets_mut().insert(dataset_id, table);

                let node = ui.create_node_retained(canvas);
                ui.set_node_view_cache_flags(node, true, true, false);
                node
            })
            .with_layout(layout);

            let subtree = cx.retained_subtree(props);
            vec![cx.semantics(
                SemanticsProps {
                    role: fret_core::SemanticsRole::Group,
                    test_id: Some(Arc::<str>::from("ui-gallery-chart-torture-root")),
                    ..Default::default()
                },
                |_cx| vec![subtree],
            )]
        });

    vec![header, chart]
}
