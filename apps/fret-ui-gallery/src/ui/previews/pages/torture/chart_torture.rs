use super::super::super::super::*;
use crate::harness::{UiGalleryChartTortureOutputHandle, UiGalleryChartTortureOutputStore};
use crate::ui::doc_layout::{self, DocSection};
use fret::AppComponentCx;

pub(in crate::ui) fn preview_chart_torture(
    cx: &mut AppComponentCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use std::collections::BTreeMap;

    use delinea::data::{Column, DataTable};
    use delinea::engine::ChartEngine;
    use delinea::{
        AxisKind, AxisPointerSpec, AxisPointerTrigger, AxisPointerType, AxisRange, AxisScale,
        ChartSpec, DataZoomXSpec, DataZoomYSpec, DatasetSpec, FieldSpec, FilterMode, GridSpec,
        SeriesEncode, SeriesKind, SeriesSpec, TimeAxisScale,
    };
    use fret_chart::{ChartCanvasPanelProps, chart_canvas_panel_in};

    let dataset_id = delinea::ids::DatasetId::new(1);
    let grid_id = delinea::ids::GridId::new(1);
    let x_axis = delinea::AxisId::new(1);
    let y_axis = delinea::AxisId::new(2);
    let x_zoom = delinea::DataZoomId::new(1);
    let y_zoom = delinea::DataZoomId::new(2);
    let series_a = delinea::ids::SeriesId::new(1);
    let series_b = delinea::ids::SeriesId::new(2);
    let x_field = delinea::FieldId::new(1);
    let y_a_field = delinea::FieldId::new(2);
    let y_b_field = delinea::FieldId::new(3);

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
                    id: y_a_field,
                    column: 1,
                },
                FieldSpec {
                    id: y_b_field,
                    column: 2,
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
        data_zoom_x: vec![DataZoomXSpec {
            id: x_zoom,
            axis: x_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: y_zoom,
            axis: y_axis,
            filter_mode: FilterMode::None,
            min_value_span: None,
            max_value_span: None,
        }],
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
        series: vec![
            SeriesSpec {
                id: series_a,
                name: Some("A".to_string()),
                kind: SeriesKind::Line,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_a_field,
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
                id: series_b,
                name: Some("B".to_string()),
                kind: SeriesKind::Line,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_b_field,
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
        ],
    };

    let explicit_y_link_map = std::env::var_os("FRET_UI_GALLERY_CHART_TORTURE_EXPLICIT_Y_LINK_MAP")
        .is_some_and(|value| !value.is_empty() && value.to_string_lossy() != "0");

    let spec_for_engine = spec.clone();
    let engine = cx.local_model_keyed("chart_torture_engine", move || {
        let mut engine = ChartEngine::new(spec_for_engine).expect("chart spec should be valid");
        let base_ms = 1_735_689_600_000.0;
        let interval_ms = 60_000.0;

        let n = 200_000usize;
        let mut x: Vec<f64> = Vec::with_capacity(n);
        let mut y_a: Vec<f64> = Vec::with_capacity(n);
        let mut y_b: Vec<f64> = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f64 / (n - 1) as f64;
            let xi = base_ms + interval_ms * i as f64;
            let theta = t * std::f64::consts::TAU;
            let a = (theta * 8.0).sin() * 0.8;
            let b = (theta * 6.0).cos() * 0.6 + 0.15;
            x.push(xi);
            y_a.push(a);
            y_b.push(b);
        }

        let mut table = DataTable::default();
        table.push_column(Column::F64(x));
        table.push_column(Column::F64(y_a));
        table.push_column(Column::F64(y_b));
        engine.datasets_mut().insert(dataset_id, table);
        if explicit_y_link_map {
            engine.apply_action(delinea::Action::SetDataWindowY {
                axis: y_axis,
                window: Some(delinea::engine::window::DataWindow {
                    min: -0.25,
                    max: 0.75,
                }),
            });
        }

        engine
    });
    let output = cx.local_model_keyed(
        "chart_torture_output",
        fret_chart::ChartCanvasOutput::default,
    );
    cx.app
        .with_global_mut(UiGalleryChartTortureOutputStore::default, |store, _app| {
            store.per_window.insert(
                cx.window,
                UiGalleryChartTortureOutputHandle {
                    output: output.clone(),
                    engine: engine.clone(),
                },
            );
        });

    let chart = cx.cached_subtree_with(
        CachedSubtreeProps::default().contain_layout_when_bounds_known(true),
        move |cx| {
            let engine = engine.clone();
            let output = output.clone();
            let mut link_axis_map = BTreeMap::new();
            if explicit_y_link_map {
                link_axis_map.insert(
                    y_axis,
                    fret_chart::LinkAxisKey {
                        kind: AxisKind::Y,
                        dataset: dataset_id,
                        field: y_a_field,
                    },
                );
            }

            let mut props = ChartCanvasPanelProps::new(spec.clone())
                .output_model(output)
                .input_map(fret_chart::input_map::ChartInputMap::default())
                .link_axis_map(link_axis_map)
                .test_id("ui-gallery-chart-torture-root");
            props.engine = Some(engine);
            props.canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

            vec![
                ui::v_flex(move |cx| vec![chart_canvas_panel_in(cx, props)])
                    .layout(LayoutRefinement::default().w_full().h_px(Px(520.0)))
                    .into_element(cx),
            ]
        },
    );

    let chart = DocSection::build(cx, "Chart", chart)
        .description(
            "Use scripted drag+wheel steps to validate correctness and collect perf bundles.",
        )
        .no_shell()
        .max_w(Px(980.0));

    let page = doc_layout::render_doc_page(
        cx,
        Some(
            "Goal: stress canvas charts with pan/zoom (candidate for prepaint-windowed sampling).",
        ),
        vec![chart],
    );

    vec![page.into_element(cx)]
}
