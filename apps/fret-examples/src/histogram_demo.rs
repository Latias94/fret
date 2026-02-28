#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::scene::Color;
use fret_core::{AppWindowId, Event};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_plot::retained::{
    HistogramPlotCanvas, HistogramPlotModel, HistogramSeries, LinePlotStyle, PlotOutput, PlotState,
    SeriesTooltipMode,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
use std::sync::Arc;

struct HistogramDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: fret_runtime::Model<HistogramPlotModel>,
    plot_state: fret_runtime::Model<PlotState>,
    plot_output: fret_runtime::Model<PlotOutput>,
    last_logged_output_revision: u64,
}

#[derive(Default)]
struct HistogramDemoDriver;

#[derive(Debug, Clone, Copy)]
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn next_f64(&mut self) -> f64 {
        // 53-bit precision float in [0, 1).
        let v = self.next_u64() >> 11;
        (v as f64) * (1.0 / ((1u64 << 53) as f64))
    }

    fn normal_f64(&mut self) -> f64 {
        // Box-Muller transform.
        let u1 = self.next_f64().clamp(f64::MIN_POSITIVE, 1.0);
        let u2 = self.next_f64();
        let r = (-2.0 * u1.ln()).sqrt();
        let t = std::f64::consts::TAU * u2;
        r * t.cos()
    }
}

impl HistogramDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> HistogramDemoWindowState {
        let n = 12_000usize;

        let mut rng = XorShift64::new(0xC0FFEE_1234_5678);
        let mut a: Vec<f64> = Vec::with_capacity(n);
        let mut b: Vec<f64> = Vec::with_capacity(n);
        for _ in 0..n {
            a.push(rng.normal_f64() * 0.85);
            b.push(rng.normal_f64() * 0.60 + 1.25);
        }

        let a: Arc<[f64]> = a.into();
        let b: Arc<[f64]> = b.into();

        let series = vec![
            HistogramSeries::new("A (N(0, 0.85))", a)
                .bins(80)
                .bar_gap_fraction(0.12)
                .fill(Color {
                    r: 0.35,
                    g: 0.65,
                    b: 0.95,
                    a: 0.35,
                }),
            HistogramSeries::new("B (N(1.25, 0.60))", b)
                .bins(80)
                .bar_gap_fraction(0.12)
                .fill(Color {
                    r: 0.95,
                    g: 0.45,
                    b: 0.55,
                    a: 0.35,
                }),
        ];

        let plot = app
            .models_mut()
            .insert(HistogramPlotModel::from_series(series));
        let plot_state = app.models_mut().insert(PlotState::default());
        let plot_output = app.models_mut().insert(PlotOutput::default());

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        HistogramDemoWindowState {
            ui,
            root: None,
            plot,
            plot_state,
            plot_output,
            last_logged_output_revision: 0,
        }
    }
}

impl WinitAppDriver for HistogramDemoDriver {
    type WindowState = HistogramDemoWindowState;

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
            let style = LinePlotStyle {
                series_tooltip: SeriesTooltipMode::NearestAtCursor,
                ..Default::default()
            };
            let canvas = HistogramPlotCanvas::new(state.plot.clone())
                .style(style)
                .state(state.plot_state.clone())
                .output(state.plot_output.clone());
            let node = HistogramPlotCanvas::create_node(&mut state.ui, canvas);
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
            "fret-demo histogram_demo (RMB drag zoom, Alt+LMB drag query, LMB double-click fit)"
                .to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    HistogramDemoDriver::default()
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

    crate::run_native_demo(config, app, driver).context("run histogram_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
