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
use fret_ui_plot::cartesian::DataPoint;
use fret_ui_plot::retained::{LinePlotCanvas, LinePlotStyle, LineSeries};
use fret_ui_plot::series::Series;

struct PlotDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: fret_runtime::Model<fret_ui_plot::retained::LinePlotModel>,
}

#[derive(Default)]
struct PlotDemoDriver;

impl PlotDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> PlotDemoWindowState {
        let n = 4096usize;

        let mut series0: Vec<DataPoint> = Vec::with_capacity(n);
        let mut series1: Vec<DataPoint> = Vec::with_capacity(n);
        let mut series2: Vec<DataPoint> = Vec::with_capacity(n);

        let push = |series: &mut Vec<DataPoint>, x: f32, y: f32| {
            if !x.is_finite() || !y.is_finite() {
                return;
            }
            series.push(DataPoint { x, y });
        };

        for i in 0..n {
            let t = i as f32 / (n - 1) as f32;
            let x = t * 10.0;
            push(
                &mut series0,
                x,
                (x * 1.25).sin() * 0.75 + (x * 0.33).cos() * 0.25,
            );
            push(
                &mut series1,
                x,
                (x * 1.10).sin() * 0.55 + (x * 0.20).cos() * 0.20 + 0.35,
            );
            push(
                &mut series2,
                x,
                (x * 0.75).sin() * 0.35 + (x * 0.15).cos() * 0.10 - 0.35,
            );
        }

        let plot = app
            .models_mut()
            .insert(fret_ui_plot::retained::LinePlotModel::from_series(vec![
                LineSeries::new("signal A", Series::from_points_sorted(series0, true)),
                LineSeries::new("signal B", Series::from_points_sorted(series1, true)),
                LineSeries::new("signal C", Series::from_points_sorted(series2, true)),
            ]));

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        PlotDemoWindowState {
            ui,
            root: None,
            plot,
        }
    }
}

impl WinitAppDriver for PlotDemoDriver {
    type WindowState = PlotDemoWindowState;

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
                return;
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
            let theme = fret_ui::Theme::global(&*app).snapshot();
            let style = LinePlotStyle {
                background: Some(theme.colors.panel_background),
                border: Some(theme.colors.panel_border),
                ..Default::default()
            };
            let canvas = LinePlotCanvas::new(state.plot.clone()).style(style);
            let node = LinePlotCanvas::create_node(&mut state.ui, canvas);
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
        main_window_title: "fret-demo plot_demo (Shift+Drag zoom, Alt+Drag query, Q clear query)"
            .to_string(),
        main_window_size: winit::dpi::LogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    PlotDemoDriver::default()
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

    run_app(config, app, driver)
        .context("run plot_demo app")
        .map_err(anyhow::Error::from)
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
