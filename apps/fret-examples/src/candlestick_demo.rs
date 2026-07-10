#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event};
use fret_launch::{
    FnDriver, WinitEventContext, WinitHotReloadContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_plot::CandlestickPlotPanelBinding;
use fret_plot::declarative::candlestick_plot_panel_in;
use fret_plot::models::{CandlestickPlotModel, CandlestickSeries, OhlcPoint};
use fret_plot::style::LinePlotStyle;
use fret_runtime::PlatformCapabilities;
use fret_ui::{UiTree, declarative};
use std::sync::Arc;

struct CandlestickDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: CandlestickPlotPanelBinding,
    last_logged_output_revision: u64,
}

#[derive(Default)]
struct CandlestickDemoDriver;

impl CandlestickDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> CandlestickDemoWindowState {
        let n = 512usize;

        let mut out: Vec<OhlcPoint> = Vec::with_capacity(n);
        let mut last_close = 100.0f64;

        for i in 0..n {
            let x = i as f64;
            let open = last_close;
            let delta = (x * 0.08).sin() * 0.9 + (x * 0.023).cos() * 0.5;
            let close = open + delta;
            let high = open.max(close) + (x * 0.31).sin().abs() * 1.4;
            let low = open.min(close) - (x * 0.27).cos().abs() * 1.2;
            out.push(OhlcPoint {
                x,
                open,
                high,
                low,
                close,
            });
            last_close = close;
        }

        let plot = CandlestickPlotPanelBinding::new(
            app,
            CandlestickPlotModel::from_series(vec![
                CandlestickSeries::new_sorted("ohlc", Arc::from(out), true).width(0.9),
            ]),
        );

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        CandlestickDemoWindowState {
            ui,
            root: None,
            plot,
            last_logged_output_revision: 0,
        }
    }
}

fn create_window_state(
    _driver: &mut CandlestickDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> CandlestickDemoWindowState {
    CandlestickDemoDriver::build_ui(app, window)
}

fn hot_reload_window(
    _driver: &mut CandlestickDemoDriver,
    context: WinitHotReloadContext<'_, CandlestickDemoWindowState>,
) {
    let WinitHotReloadContext {
        app, window, state, ..
    } = context;

    crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
    state.root = None;
}

fn handle_event(
    _driver: &mut CandlestickDemoDriver,
    context: WinitEventContext<'_, CandlestickDemoWindowState>,
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
    _driver: &mut CandlestickDemoDriver,
    context: WinitRenderContext<'_, CandlestickDemoWindowState>,
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
                .render_root("candlestick-demo", {
                    let plot = state.plot.clone();
                    move |cx| {
                        let style = LinePlotStyle::default();
                        let props = plot.panel_props().style(style);
                        vec![candlestick_plot_panel_in(cx, props)]
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
            "fret-demo candlestick_demo (RMB drag zoom, Alt+LMB drag query, LMB double-click fit)"
                .to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_fn_driver() -> impl fret_launch::WinitAppDriver {
    FnDriver::new(
        CandlestickDemoDriver::default(),
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

    crate::run_native_with_driver(config, app, driver).context("run candlestick_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
