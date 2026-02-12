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
use delinea::ids::{AxisId, FieldId, StackId};
use delinea::{
    AreaBaseline, AxisKind, AxisPosition, AxisRange, AxisScale, SeriesKind, TimeAxisScale,
};
use delinea::{ChartSpec, DatasetSpec, FieldSpec, GridSpec, SeriesEncode, SeriesSpec};
use fret_chart::retained::ChartCanvas;

struct ChartDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
}

#[derive(Default)]
struct ChartDemoDriver;

impl ChartDemoDriver {
    fn build_ui(_app: &mut App, window: AppWindowId) -> ChartDemoWindowState {
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        ChartDemoWindowState { ui, root: None }
    }

    fn build_canvas() -> ChartCanvas {
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
            axis_pointer: Some(delinea::AxisPointerSpec {
                enabled: true,
                trigger: delinea::AxisPointerTrigger::Axis,
                pointer_type: delinea::AxisPointerType::Line,
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

        let mut canvas = ChartCanvas::new(spec).expect("chart spec should be valid");

        // 2025-01-01T00:00:00Z in epoch milliseconds.
        let base_ms = 1_735_689_600_000.0;
        let interval_ms = 60_000.0;

        let n = 65_536usize;
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
        canvas.engine_mut().datasets_mut().insert(dataset_id, table);

        canvas
    }
}

impl WinitAppDriver for ChartDemoDriver {
    type WindowState = ChartDemoWindowState;

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
        main_window_title: "fret-demo chart_demo (delinea + fret-chart)".to_string(),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    ChartDemoDriver::default()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    let app = build_app();
    let config = build_runner_config();
    let driver = build_driver();

    run_app(config, app, driver)
        .context("run chart_demo app")
        .map_err(anyhow::Error::from)
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
