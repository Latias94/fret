use fret_kit::prelude::*;

use delinea::data::{Column, DataTable};
use delinea::ids::{AxisId, FieldId, StackId};
use delinea::{
    AreaBaseline, AxisKind, AxisPointerSpec, AxisPointerTrigger, AxisPointerType, AxisPosition,
    AxisRange, AxisScale, ChartEngine, ChartSpec, DatasetSpec, FieldSpec, GridSpec, SeriesEncode,
    SeriesKind, SeriesSpec, TimeAxisScale,
};
use fret_chart::{ChartCanvasPanelProps, chart_canvas_panel};

struct ChartDeclarativeState {
    engine: Model<ChartEngine>,
    spec: ChartSpec,
}

pub fn run() -> anyhow::Result<()> {
    fret_kit::app("chart-declarative-demo", init_window, view)?
        .with_main_window("chart_declarative_demo", (960.0, 720.0))
        .run()?;
    Ok(())
}

fn init_window(app: &mut App, _window: AppWindowId) -> ChartDeclarativeState {
    let dataset_id = delinea::ids::DatasetId::new(1);
    let grid_id = delinea::ids::GridId::new(1);
    let x_axis = AxisId::new(1);
    let y_left_axis = AxisId::new(2);
    let y_right_axis = AxisId::new(3);
    let stack_id = StackId::new(1);
    let series_a_id = delinea::ids::SeriesId::new(1);
    let series_b_id = delinea::ids::SeriesId::new(2);
    let series_c_id = delinea::ids::SeriesId::new(3);
    let x_field = FieldId::new(1);
    let y_a_field = FieldId::new(2);
    let y_b_field = FieldId::new(3);
    let y_c_field = FieldId::new(4);

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
                FieldSpec {
                    id: y_c_field,
                    column: 3,
                },
            ],
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
                id: y_left_axis,
                name: Some("Left".to_string()),
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: Some(AxisRange::Auto),
            },
            delinea::AxisSpec {
                id: y_right_axis,
                name: Some("Right".to_string()),
                kind: AxisKind::Y,
                grid: grid_id,
                position: Some(AxisPosition::Right),
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
        series: vec![
            SeriesSpec {
                id: series_a_id,
                name: Some("Stack A (area)".to_string()),
                kind: SeriesKind::Area,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_a_field,
                    y2: None,
                },
                x_axis,
                y_axis: y_left_axis,
                stack: Some(stack_id),
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: Some(AreaBaseline::Zero),
                lod: None,
            },
            SeriesSpec {
                id: series_b_id,
                name: Some("Stack B (area)".to_string()),
                kind: SeriesKind::Area,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_b_field,
                    y2: None,
                },
                x_axis,
                y_axis: y_left_axis,
                stack: Some(stack_id),
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: Some(AreaBaseline::Zero),
                lod: None,
            },
            SeriesSpec {
                id: series_c_id,
                name: Some("Right axis (line)".to_string()),
                kind: SeriesKind::Line,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_c_field,
                    y2: None,
                },
                x_axis,
                y_axis: y_right_axis,
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec.clone()).expect("chart spec should be valid");

    // 2025-01-01T00:00:00Z in epoch milliseconds.
    let base_ms = 1_735_689_600_000.0;
    let interval_ms = 60_000.0;

    let n = 8_192usize;
    let mut x: Vec<f64> = Vec::with_capacity(n);
    let mut y_a: Vec<f64> = Vec::with_capacity(n);
    let mut y_b: Vec<f64> = Vec::with_capacity(n);
    let mut y_c: Vec<f64> = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 / (n - 1) as f64;
        let xi = base_ms + interval_ms * i as f64;
        let theta = t * std::f64::consts::TAU;
        let yi_a = (theta * 8.0).sin() * 0.8;
        let yi_b = (theta * 6.0).cos() * 0.6 + 0.1;
        let yi_c = (theta * 1.5).sin() * 50.0 + 100.0;
        x.push(xi);
        y_a.push(yi_a);
        y_b.push(yi_b);
        y_c.push(yi_c);
    }

    let mut table = DataTable::default();
    table.push_column(Column::F64(x));
    table.push_column(Column::F64(y_a));
    table.push_column(Column::F64(y_b));
    table.push_column(Column::F64(y_c));
    engine.datasets_mut().insert(dataset_id, table);

    let engine = app.models_mut().insert(engine);
    ChartDeclarativeState { engine, spec }
}

fn view(
    cx: &mut ElementContext<'_, App>,
    st: &mut ChartDeclarativeState,
) -> fret_kit::ViewElements {
    cx.observe_model(&st.engine, Invalidation::Paint);

    let mut props = ChartCanvasPanelProps::new(st.spec.clone());
    props.engine = Some(st.engine.clone());
    vec![chart_canvas_panel(cx, props)].into()
}
