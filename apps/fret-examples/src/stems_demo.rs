#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event};
use fret_launch::{
    FnDriver, WinitEventContext, WinitHotReloadContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_plot::StemsPlotPanelBinding;
use fret_plot::cartesian::DataPoint;
use fret_plot::declarative::stems_plot_panel_in;
use fret_plot::models::{StemsPlotModel, StemsSeries};
use fret_plot::series::Series;
use fret_plot::style::{LinePlotStyle, SeriesTooltipMode};
use fret_runtime::PlatformCapabilities;
use fret_ui::{UiTree, declarative};

struct StemsDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: StemsPlotPanelBinding,
    last_logged_output_revision: u64,
}

#[derive(Default)]
struct StemsDemoDriver;

impl StemsDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> StemsDemoWindowState {
        let n = 512usize;

        let mut points_a: Vec<DataPoint> = Vec::with_capacity(n);
        let mut points_b: Vec<DataPoint> = Vec::with_capacity(n);
        for i in 0..n {
            let x = i as f64 * 0.1;
            let y0 = (x * 0.7).sin() * 1.25;
            let y1 = (x * 0.35).cos() * 0.85 + 0.75;
            points_a.push(DataPoint { x, y: y0 });
            points_b.push(DataPoint { x, y: y1 });
        }

        let series = vec![
            StemsSeries::new("A", Series::from_points_sorted(points_a, true)).baseline(0.0),
            StemsSeries::new("B", Series::from_points_sorted(points_b, true)).baseline(0.0),
        ];

        let plot = StemsPlotPanelBinding::new(app, StemsPlotModel::from_series(series));

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        StemsDemoWindowState {
            ui,
            root: None,
            plot,
            last_logged_output_revision: 0,
        }
    }
}

fn create_window_state(
    _driver: &mut StemsDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> StemsDemoWindowState {
    StemsDemoDriver::build_ui(app, window)
}

fn hot_reload_window(
    _driver: &mut StemsDemoDriver,
    context: WinitHotReloadContext<'_, StemsDemoWindowState>,
) {
    let WinitHotReloadContext {
        app, window, state, ..
    } = context;

    crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
    state.root = None;
}

fn handle_event(
    _driver: &mut StemsDemoDriver,
    context: WinitEventContext<'_, StemsDemoWindowState>,
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

fn render(_driver: &mut StemsDemoDriver, context: WinitRenderContext<'_, StemsDemoWindowState>) {
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
                .render_root("stems-demo", {
                    let plot = state.plot.clone();
                    move |cx| {
                        let style = LinePlotStyle {
                            series_tooltip: SeriesTooltipMode::NearestAtCursor,
                            ..Default::default()
                        };
                        let props = plot.panel_props().style(style);
                        vec![stems_plot_panel_in(cx, props)]
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
            "fret-demo stems_demo (RMB drag zoom, Alt+LMB drag query, LMB double-click fit)"
                .to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_fn_driver() -> impl fret_launch::WinitAppDriver {
    FnDriver::new(
        StemsDemoDriver::default(),
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

    crate::run_native_with_driver(config, app, driver).context("run stems_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
