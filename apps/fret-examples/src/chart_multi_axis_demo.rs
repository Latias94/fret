#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event};
#[cfg(not(target_arch = "wasm32"))]
use fret_launch::run_app;
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;

use delinea::data::{Column, DataTable};
use delinea::ids::{AxisId, FieldId};
use delinea::{AxisKind, AxisPosition, AxisScale, SeriesKind};
use delinea::{ChartSpec, DatasetSpec, FieldSpec, GridSpec, SeriesEncode, SeriesSpec};
use fret_chart::retained::ChartCanvas;

struct ChartMultiAxisDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
}

#[derive(Default)]
struct ChartMultiAxisDemoDriver;

impl ChartMultiAxisDemoDriver {
    fn build_ui(_app: &mut App, window: AppWindowId) -> ChartMultiAxisDemoWindowState {
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        ChartMultiAxisDemoWindowState { ui, root: None }
    }

    fn build_canvas() -> ChartCanvas {
        let dataset_id = delinea::ids::DatasetId::new(1);
        let grid_id = delinea::ids::GridId::new(1);

        let x_bottom = AxisId::new(1);
        let x_top = AxisId::new(2);
        let y_left = AxisId::new(3);
        let y_right = AxisId::new(4);

        let x1_field = FieldId::new(1);
        let x2_field = FieldId::new(2);
        let y_left_a_field = FieldId::new(3);
        let y_left_b_field = FieldId::new(4);
        let y_right_a_field = FieldId::new(5);
        let y_right_b_field = FieldId::new(6);

        let spec = ChartSpec {
            id: delinea::ids::ChartId::new(1),
            viewport: None,
            datasets: vec![DatasetSpec {
                id: dataset_id,
                fields: vec![
                    FieldSpec {
                        id: x1_field,
                        column: 0,
                    },
                    FieldSpec {
                        id: x2_field,
                        column: 1,
                    },
                    FieldSpec {
                        id: y_left_a_field,
                        column: 2,
                    },
                    FieldSpec {
                        id: y_left_b_field,
                        column: 3,
                    },
                    FieldSpec {
                        id: y_right_a_field,
                        column: 4,
                    },
                    FieldSpec {
                        id: y_right_b_field,
                        column: 5,
                    },
                ],
            }],
            grids: vec![GridSpec { id: grid_id }],
            axes: vec![
                delinea::AxisSpec {
                    id: x_bottom,
                    name: Some("X (bottom)".to_string()),
                    kind: AxisKind::X,
                    grid: grid_id,
                    position: Some(AxisPosition::Bottom),
                    scale: AxisScale::Value(Default::default()),
                    range: None,
                },
                delinea::AxisSpec {
                    id: x_top,
                    name: Some("X (top)".to_string()),
                    kind: AxisKind::X,
                    grid: grid_id,
                    position: Some(AxisPosition::Top),
                    scale: AxisScale::Value(Default::default()),
                    range: None,
                },
                delinea::AxisSpec {
                    id: y_left,
                    name: Some("Y (left)".to_string()),
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: Some(AxisPosition::Left),
                    scale: AxisScale::Value(Default::default()),
                    range: None,
                },
                delinea::AxisSpec {
                    id: y_right,
                    name: Some("Y (right)".to_string()),
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: Some(AxisPosition::Right),
                    scale: AxisScale::Value(Default::default()),
                    range: None,
                },
            ],
            data_zoom_x: vec![],
            axis_pointer: Some(delinea::AxisPointerSpec::default()),
            series: vec![
                SeriesSpec {
                    id: delinea::ids::SeriesId::new(1),
                    name: Some("Bottom X / Left Y (line)".to_string()),
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x1_field,
                        y: y_left_a_field,
                        y2: None,
                    },
                    x_axis: x_bottom,
                    y_axis: y_left,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                },
                SeriesSpec {
                    id: delinea::ids::SeriesId::new(2),
                    name: Some("Bottom X / Right Y (line)".to_string()),
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x1_field,
                        y: y_right_a_field,
                        y2: None,
                    },
                    x_axis: x_bottom,
                    y_axis: y_right,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                },
                SeriesSpec {
                    id: delinea::ids::SeriesId::new(3),
                    name: Some("Top X / Left Y (scatter)".to_string()),
                    kind: SeriesKind::Scatter,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x2_field,
                        y: y_left_b_field,
                        y2: None,
                    },
                    x_axis: x_top,
                    y_axis: y_left,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                },
                SeriesSpec {
                    id: delinea::ids::SeriesId::new(4),
                    name: Some("Top X / Right Y (scatter)".to_string()),
                    kind: SeriesKind::Scatter,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x2_field,
                        y: y_right_b_field,
                        y2: None,
                    },
                    x_axis: x_top,
                    y_axis: y_right,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                },
            ],
        };

        let mut canvas = ChartCanvas::new(spec).expect("chart spec should be valid");

        let n = 4096usize;
        let mut x1: Vec<f64> = Vec::with_capacity(n);
        let mut x2: Vec<f64> = Vec::with_capacity(n);
        let mut y_left_a: Vec<f64> = Vec::with_capacity(n);
        let mut y_left_b: Vec<f64> = Vec::with_capacity(n);
        let mut y_right_a: Vec<f64> = Vec::with_capacity(n);
        let mut y_right_b: Vec<f64> = Vec::with_capacity(n);

        for i in 0..n {
            let t = i as f64 / (n - 1).max(1) as f64;
            let theta = t * std::f64::consts::TAU;

            let x = (t * 2000.0) - 1000.0;
            x1.push(x);
            x2.push(x);

            y_left_a.push((theta * 2.0).sin() * 20.0);
            y_left_b.push((theta * 16.0).sin() * 2.0 + (theta * 3.0).cos() * 1.5);

            y_right_a.push((theta * 1.25).cos() * 400.0 + 1000.0);
            y_right_b.push((theta * 8.0).cos() * 75.0 + 200.0);
        }

        let mut table = DataTable::default();
        table.push_column(Column::F64(x1));
        table.push_column(Column::F64(x2));
        table.push_column(Column::F64(y_left_a));
        table.push_column(Column::F64(y_left_b));
        table.push_column(Column::F64(y_right_a));
        table.push_column(Column::F64(y_right_b));
        canvas.engine_mut().datasets_mut().insert(dataset_id, table);

        canvas
    }
}

impl WinitAppDriver for ChartMultiAxisDemoDriver {
    type WindowState = ChartMultiAxisDemoWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
            ..
        } = context;

        match event {
            Event::WindowCloseRequested
            | Event::KeyDown {
                key: fret_core::KeyCode::Escape,
                ..
            } => {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
            }
            _ => {
                state.ui.dispatch_event(app, services, event);
            }
        }
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let WinitRenderContext {
            app,
            services,
            window,
            state,
            bounds,
            scale_factor,
            scene,
        } = context;

        let root = state.root.get_or_insert_with(|| {
            let canvas = Self::build_canvas();
            let node = ChartCanvas::create_node(&mut state.ui, canvas);
            state.ui.set_root(node);
            node
        });

        state.ui.set_root(*root);
        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);

        scene.clear();
        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();
        frame.paint_all(scene);
    }

    fn window_create_spec(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        None
    }

    fn window_created(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
        _new_window: AppWindowId,
    ) {
    }
}

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "fret-demo chart_multi_axis_demo (delinea + fret-chart)".to_string(),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    ChartMultiAxisDemoDriver::default()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    let app = build_app();
    let config = build_runner_config();
    let driver = build_driver();

    run_app(config, app, driver)
        .context("run chart_multi_axis_demo app")
        .map_err(anyhow::Error::from)
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
