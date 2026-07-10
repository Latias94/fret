#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::scene::Color;
use fret_core::{AppWindowId, Event};
use fret_launch::{
    FnDriver, WinitEventContext, WinitHotReloadContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_plot::HistogramPlotPanelBinding;
use fret_plot::declarative::histogram_plot_panel_in;
use fret_plot::models::{HistogramPlotModel, HistogramSeries};
use fret_plot::style::{LinePlotStyle, SeriesTooltipMode};
use fret_runtime::PlatformCapabilities;
use fret_ui::{UiTree, declarative};
use std::sync::Arc;

struct HistogramDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: HistogramPlotPanelBinding,
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
                    a: 0.35,
                    ..Color::from_srgb_hex_rgb(0x59_a6_f2)
                }),
            HistogramSeries::new("B (N(1.25, 0.60))", b)
                .bins(80)
                .bar_gap_fraction(0.12)
                .fill(Color {
                    a: 0.35,
                    ..Color::from_srgb_hex_rgb(0xf2_73_8c)
                }),
        ];

        let plot = HistogramPlotPanelBinding::new(app, HistogramPlotModel::from_series(series));

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        HistogramDemoWindowState {
            ui,
            root: None,
            plot,
            last_logged_output_revision: 0,
        }
    }
}

fn create_window_state(
    _driver: &mut HistogramDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> HistogramDemoWindowState {
    HistogramDemoDriver::build_ui(app, window)
}

fn hot_reload_window(
    _driver: &mut HistogramDemoDriver,
    context: WinitHotReloadContext<'_, HistogramDemoWindowState>,
) {
    let WinitHotReloadContext {
        app, window, state, ..
    } = context;

    crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
    state.root = None;
}

fn handle_event(
    _driver: &mut HistogramDemoDriver,
    context: WinitEventContext<'_, HistogramDemoWindowState>,
    event: &Event,
) {
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
                let output = state.plot.output_untracked(app);
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

fn render(
    _driver: &mut HistogramDemoDriver,
    context: WinitRenderContext<'_, HistogramDemoWindowState>,
) {
    let WinitRenderContext {
        app,
        services,
        window,
        state,
        bounds,
        scale_factor,
        scene,
    } = context;

    if state.root.is_none() {
        let node =
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                .render_root("histogram-demo", {
                    let plot = state.plot.clone();
                    move |cx| {
                        let style = LinePlotStyle {
                            series_tooltip: SeriesTooltipMode::NearestAtCursor,
                            ..Default::default()
                        };
                        let props = plot.panel_props().style(style);
                        vec![histogram_plot_panel_in(cx, props)]
                    }
                });
        state.ui.set_root(node);
        state.ui.set_focus(Some(node));
        state.ui.publish_window_runtime_snapshots(app);
        state.root = Some(node);
    }

    if let Some(root) = state.root {
        state.ui.set_root(root);
    }
    state.ui.request_semantics_snapshot();
    state.ui.ingest_paint_cache_source(scene);

    scene.clear();
    let mut frame =
        fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
    frame.layout_all();
    frame.paint_all(scene);
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

pub fn build_fn_driver() -> impl fret_launch::WinitAppDriver {
    FnDriver::new(
        HistogramDemoDriver::default(),
        create_window_state,
        handle_event,
        render,
    )
    .with_hooks(|hooks| {
        hooks.hot_reload_window = Some(hot_reload_window);
    })
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
    let driver = build_fn_driver();

    crate::run_native_with_driver(config, app, driver).context("run histogram_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
