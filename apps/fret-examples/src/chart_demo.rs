#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event, Px};
#[cfg(not(target_arch = "wasm32"))]
use fret_launch::run_app;
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;

use delinea::data::{Column, DataTable};
use delinea::ids::AxisId;
use delinea::{AxisKind, AxisRange, SeriesKind};
use delinea::{ChartSpec, DatasetSpec, GridSpec, SeriesSpec};
use fret_chart::retained::{ChartCanvas, ChartStyle};

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
        let y_axis = AxisId::new(2);
        let series_id = delinea::ids::SeriesId::new(1);

        let spec = ChartSpec {
            id: delinea::ids::ChartId::new(1),
            viewport: None,
            datasets: vec![DatasetSpec { id: dataset_id }],
            grids: vec![GridSpec { id: grid_id }],
            axes: vec![
                delinea::AxisSpec {
                    id: x_axis,
                    kind: AxisKind::X,
                    grid: grid_id,
                    range: Some(AxisRange::Auto),
                },
                delinea::AxisSpec {
                    id: y_axis,
                    kind: AxisKind::Y,
                    grid: grid_id,
                    range: Some(AxisRange::Auto),
                },
            ],
            series: vec![SeriesSpec {
                id: series_id,
                kind: SeriesKind::Area,
                dataset: dataset_id,
                x_col: 0,
                y_col: 1,
                x_axis,
                y_axis,
                area_baseline: None,
            }],
        };

        let mut canvas = ChartCanvas::new(spec).expect("chart spec should be valid");
        canvas.set_style(ChartStyle {
            stroke_width: Px(1.5),
            ..ChartStyle::default()
        });

        let n = 65_536usize;
        let mut x: Vec<f64> = Vec::with_capacity(n);
        let mut y: Vec<f64> = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f64 / (n - 1) as f64;
            let xi = t * 1000.0;
            let yi = (t * std::f64::consts::TAU * 8.0).sin();
            x.push(xi);
            y.push(yi);
        }

        let mut table = DataTable::default();
        table.push_column(Column::F64(x));
        table.push_column(Column::F64(y));
        canvas
            .engine_mut()
            .datasets_mut()
            .datasets
            .push((dataset_id, table));

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
