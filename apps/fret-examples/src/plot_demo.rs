use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
    run_app,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
use fret_ui_plot::chart::line_chart::LineChart;
use fret_ui_plot::retained::{LinePlotCanvas, LinePlotStyle};

struct PlotDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: fret_runtime::Model<fret_ui_plot::retained::LinePlotModel>,
    close_requested: bool,
}

#[derive(Default)]
struct PlotDemoDriver;

impl PlotDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> PlotDemoWindowState {
        let points: Vec<(f32, f32)> = (0..2048)
            .map(|i| {
                let t = i as f32 / 2047.0;
                let x = t * 10.0;
                let y = (x * 1.25).sin() * 0.75 + (x * 0.33).cos() * 0.25;
                (x, y)
            })
            .collect();

        let plot = LineChart::new(points)
            .x(|(x, _y)| Some(*x))
            .y(|(_x, y)| Some(*y))
            .install(app);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        PlotDemoWindowState {
            ui,
            root: None,
            plot,
            close_requested: false,
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
                if !state.close_requested {
                    state.close_requested = true;
                    app.push_effect(Effect::Window(WindowRequest::Close(window)));
                }
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
        main_window_title: "fret-demo plot_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    PlotDemoDriver::default()
}

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
