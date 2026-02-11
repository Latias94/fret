use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;

use anyhow::Context as _;

use delinea::data::{Column, DataTable};
use delinea::engine::window::DataWindow;
use delinea::ids::{AxisId, DataZoomId, DatasetId, FieldId, GridId, SeriesId};
use delinea::{
    Action, AxisKind, AxisPointerTrigger, AxisPointerType, AxisScale, ChartSpec, DataZoomXSpec,
    DatasetSpec, FieldSpec, FilterMode, GridSpec, SeriesEncode, SeriesKind, SeriesSpec,
};
use fret_chart::retained::ChartCanvas;

struct CategoryLineDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
}

#[derive(Default)]
struct CategoryLineDemoDriver;

impl CategoryLineDemoDriver {
    fn build_ui(_app: &mut App, window: AppWindowId) -> CategoryLineDemoWindowState {
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        CategoryLineDemoWindowState { ui, root: None }
    }

    fn build_canvas() -> ChartCanvas {
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
        let zoom_id = DataZoomId::new(1);

        let series_line_id = SeriesId::new(1);
        let series_scatter_id = SeriesId::new(2);

        let x_field = FieldId::new(1);
        let y_line_field = FieldId::new(2);
        let y_scatter_field = FieldId::new(3);

        let categories: Vec<String> = (0..128).map(|i| format!("C{i:03}")).collect();

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
                        id: y_line_field,
                        column: 1,
                    },
                    FieldSpec {
                        id: y_scatter_field,
                        column: 2,
                    },
                ],
                ..Default::default()
            }],
            grids: vec![GridSpec { id: grid_id }],
            axes: vec![
                delinea::AxisSpec {
                    id: x_axis,
                    name: Some("Category".to_string()),
                    kind: AxisKind::X,
                    grid: grid_id,
                    position: None,
                    scale: AxisScale::Category(delinea::CategoryAxisScale { categories }),
                    range: None,
                },
                delinea::AxisSpec {
                    id: y_axis,
                    name: Some("Value".to_string()),
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: None,
                    scale: Default::default(),
                    range: None,
                },
            ],
            data_zoom_x: vec![DataZoomXSpec {
                id: zoom_id,
                axis: x_axis,
                filter_mode: FilterMode::Filter,
                min_value_span: Some(6.0),
                max_value_span: Some(80.0),
            }],
            data_zoom_y: vec![],
            tooltip: None,
            axis_pointer: Some(delinea::AxisPointerSpec {
                enabled: true,
                trigger: AxisPointerTrigger::Axis,
                pointer_type: AxisPointerType::Line,
                label: Default::default(),
                snap: true,
                trigger_distance_px: 12.0,
                throttle_px: 0.75,
            }),
            visual_maps: vec![],
            series: vec![
                SeriesSpec {
                    id: series_line_id,
                    name: Some("Line".to_string()),
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_line_field,
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
                    id: series_scatter_id,
                    name: Some("Scatter".to_string()),
                    kind: SeriesKind::Scatter,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_scatter_field,
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

        let mut canvas = ChartCanvas::new(spec).expect("chart spec should be valid");

        let n = 128usize;
        let mut x: Vec<f64> = Vec::with_capacity(n);
        let mut y_line: Vec<f64> = Vec::with_capacity(n);
        let mut y_scatter: Vec<f64> = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f64 / (n - 1).max(1) as f64;
            let theta = t * std::f64::consts::TAU;
            x.push(i as f64);
            y_line.push((theta * 2.0).sin() * 1.2 + (theta * 0.5).cos() * 0.2);
            y_scatter.push((theta * 3.0).cos() * 0.9 + (theta * 0.25).sin() * 0.15);
        }

        let mut table = DataTable::default();
        table.push_column(Column::F64(x));
        table.push_column(Column::F64(y_line));
        table.push_column(Column::F64(y_scatter));
        canvas.engine_mut().datasets_mut().insert(dataset_id, table);

        canvas.engine_mut().apply_action(Action::SetDataWindowX {
            axis: x_axis,
            window: Some(DataWindow {
                min: 16.0,
                max: 64.0,
            }),
        });

        canvas
    }
}

impl WinitAppDriver for CategoryLineDemoDriver {
    type WindowState = CategoryLineDemoWindowState;

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
        main_window_title: "fret-demo category_line_demo (delinea + fret-chart)".to_string(),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    CategoryLineDemoDriver::default()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    let app = build_app();
    let config = build_runner_config();
    let driver = build_driver();

    crate::run_native_demo(config, app, driver).context("run category_line_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
