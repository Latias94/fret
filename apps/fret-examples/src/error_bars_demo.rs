#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_plot::cartesian::DataPoint;
use fret_plot::retained::{
    ErrorBar, ErrorBarsPlotCanvas, ErrorBarsPlotModel, ErrorBarsSeries, LinePlotStyle, PlotOutput,
    PlotState, YAxis,
};
use fret_plot::series::Series;
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
use std::sync::Arc;

struct ErrorBarsDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: fret_runtime::Model<ErrorBarsPlotModel>,
    plot_state: fret_runtime::Model<PlotState>,
    plot_output: fret_runtime::Model<PlotOutput>,
    last_logged_output_revision: u64,
}

#[derive(Default)]
struct ErrorBarsDemoDriver;

impl ErrorBarsDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> ErrorBarsDemoWindowState {
        let n = 256usize;

        let mut left_points: Vec<DataPoint> = Vec::with_capacity(n);
        let mut right_points: Vec<DataPoint> = Vec::with_capacity(n);
        let mut left_y_errors: Vec<ErrorBar> = Vec::with_capacity(n);
        let mut right_y_errors: Vec<ErrorBar> = Vec::with_capacity(n);
        let mut left_x_errors: Vec<ErrorBar> = Vec::with_capacity(n);

        for i in 0..n {
            let x = i as f64 * 0.25;
            let y = (x * 0.35).sin() * 0.75 + (x * 0.11).cos() * 0.25;
            left_points.push(DataPoint { x, y });

            let y2 = (x * 0.22).sin() * 25.0 + 10.0;
            right_points.push(DataPoint { x, y: y2 });

            let ey = 0.05 + (x * 0.5).sin().abs() * 0.08;
            left_y_errors.push(ErrorBar::symmetric(ey));
            right_y_errors.push(ErrorBar::new(0.8, 1.6));

            let ex = 0.02 + (x * 0.7).cos().abs() * 0.05;
            left_x_errors.push(ErrorBar::symmetric(ex));
        }

        let plot = app
            .models_mut()
            .insert(ErrorBarsPlotModel::from_series(vec![
                ErrorBarsSeries::new(
                    "measurement (x/y errors)",
                    Series::from_points_sorted(left_points, true),
                )
                .y_errors(Arc::from(left_y_errors))
                .x_errors(Arc::from(left_x_errors))
                .cap_size(fret_core::Px(6.0))
                .marker_radius(fret_core::Px(3.5)),
                ErrorBarsSeries::new(
                    "measurement (right axis)",
                    Series::from_points_sorted(right_points, true),
                )
                .y_axis(YAxis::Right)
                .y_errors(Arc::from(right_y_errors))
                .cap_size(fret_core::Px(6.0))
                .marker_radius(fret_core::Px(3.5)),
            ]));

        let plot_state = app.models_mut().insert(PlotState::default());
        let plot_output = app.models_mut().insert(PlotOutput::default());

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        ErrorBarsDemoWindowState {
            ui,
            root: None,
            plot,
            plot_state,
            plot_output,
            last_logged_output_revision: 0,
        }
    }
}

impl WinitAppDriver for ErrorBarsDemoDriver {
    type WindowState = ErrorBarsDemoWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn hot_reload_window(
        &mut self,
        app: &mut App,
        _services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        state: &mut Self::WindowState,
    ) {
        crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
        state.root = None;
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
                return;
            }
            _ => {
                state.ui.dispatch_event(app, services, event);
                if matches!(
                    event,
                    Event::Pointer(fret_core::PointerEvent::Up { .. }) | Event::KeyDown { .. }
                ) {
                    let output = state
                        .plot_output
                        .read(app, |_app, o| *o)
                        .unwrap_or_default();
                    if output.revision != state.last_logged_output_revision {
                        state.last_logged_output_revision = output.revision;
                        if let Some(query) = output.snapshot.query {
                            tracing::info!(
                                "query: x=[{:.3}, {:.3}], y=[{:.3}, {:.3}]",
                                query.x_min,
                                query.x_max,
                                query.y_min,
                                query.y_max
                            );
                        }
                    }
                }
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
            let style = LinePlotStyle::default();
            let canvas = ErrorBarsPlotCanvas::new(state.plot.clone())
                .style(style)
                .state(state.plot_state.clone())
                .output(state.plot_output.clone());
            let node = ErrorBarsPlotCanvas::create_node(&mut state.ui, canvas);
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
        main_window_title:
            "fret-demo error_bars_demo (RMB drag zoom, Alt+LMB drag query, LMB double-click fit)"
                .to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    ErrorBarsDemoDriver::default()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    let app = build_app();
    let config = build_runner_config();
    let driver = build_driver();

    crate::run_native_demo(config, app, driver).context("run error_bars_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
