#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event};
use fret_launch::{
    FnDriver, WindowCreateSpec, WinitEventContext, WinitHotReloadContext, WinitRenderContext,
    WinitRunnerConfig,
};
use fret_plot::ShadedPlotPanelBinding;
use fret_plot::cartesian::DataPoint;
use fret_plot::declarative::shaded_plot_panel_in;
use fret_plot::models::{ShadedPlotModel, ShadedSeries};
use fret_plot::plot::axis::{AxisLabelFormatter, TimeAxisFormat, TimeAxisPresentation};
use fret_plot::series::Series;
use fret_plot::style::LinePlotStyle;
use fret_runtime::PlatformCapabilities;
use fret_ui::{UiTree, declarative};

pub struct ShadedDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: ShadedPlotPanelBinding,
    last_logged_output_revision: u64,
}

#[derive(Default)]
pub struct ShadedDemoDriver;

impl ShadedDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> ShadedDemoWindowState {
        let n = 4096usize;

        let mut upper0: Vec<DataPoint> = Vec::with_capacity(n);
        let mut lower0: Vec<DataPoint> = Vec::with_capacity(n);
        let mut upper1: Vec<DataPoint> = Vec::with_capacity(n);
        let mut lower1: Vec<DataPoint> = Vec::with_capacity(n);

        for i in 0..n {
            let t = i as f64 / (n - 1) as f64;
            let x = t * 10.0;

            let base0 = (x * 1.05).sin() * 0.75 + (x * 0.25).cos() * 0.10;
            upper0.push(DataPoint { x, y: base0 + 0.18 });
            lower0.push(DataPoint { x, y: base0 - 0.18 });

            let base1 = (x * 0.70).cos() * 0.55 + (x * 0.18).sin() * 0.10 - 0.35;
            upper1.push(DataPoint { x, y: base1 + 0.12 });
            lower1.push(DataPoint { x, y: base1 - 0.12 });
        }

        let plot = ShadedPlotPanelBinding::new(
            app,
            ShadedPlotModel::from_series(vec![
                ShadedSeries::new(
                    "band A",
                    Series::from_points_sorted(upper0, true),
                    Series::from_points_sorted(lower0, true),
                )
                .fill_alpha(0.18),
                ShadedSeries::new(
                    "band B",
                    Series::from_points_sorted(upper1, true),
                    Series::from_points_sorted(lower1, true),
                )
                .fill_alpha(0.18),
            ]),
        );

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        ShadedDemoWindowState {
            ui,
            root: None,
            plot,
            last_logged_output_revision: 0,
        }
    }
}

fn create_window_state(
    _driver: &mut ShadedDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> ShadedDemoWindowState {
    ShadedDemoDriver::build_ui(app, window)
}

fn hot_reload_window(
    _driver: &mut ShadedDemoDriver,
    context: WinitHotReloadContext<'_, ShadedDemoWindowState>,
) {
    let WinitHotReloadContext {
        app, window, state, ..
    } = context;

    crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
    state.root = None;
}

fn handle_event(
    _driver: &mut ShadedDemoDriver,
    context: WinitEventContext<'_, ShadedDemoWindowState>,
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

fn render(_driver: &mut ShadedDemoDriver, context: WinitRenderContext<'_, ShadedDemoWindowState>) {
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
                .render_root("shaded-demo", {
                    let plot = state.plot.clone();
                    move |cx| {
                        let style = LinePlotStyle::default();
                        let props = plot.panel_props().style(style).x_axis_labels(
                            AxisLabelFormatter::time_seconds(TimeAxisFormat {
                                base_seconds: 1_700_000_000.0,
                                presentation: TimeAxisPresentation::UnixUtc,
                            }),
                        );
                        vec![shaded_plot_panel_in(cx, props)]
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

fn window_create_spec(
    _driver: &mut ShadedDemoDriver,
    _app: &mut App,
    _request: &fret_app::CreateWindowRequest,
) -> Option<WindowCreateSpec> {
    None
}

fn window_created(
    _driver: &mut ShadedDemoDriver,
    _app: &mut App,
    _request: &fret_app::CreateWindowRequest,
    _new_window: AppWindowId,
) {
}

fn configure_fn_driver_hooks(
    hooks: &mut fret_launch::FnDriverHooks<ShadedDemoDriver, ShadedDemoWindowState>,
) {
    hooks.hot_reload_window = Some(hot_reload_window);
    hooks.window_create_spec = Some(window_create_spec);
    hooks.window_created = Some(window_created);
}

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title:
            "fret-demo shaded_demo (RMB drag zoom, Alt+LMB drag query, LMB double-click fit)"
                .to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_fn_driver() -> FnDriver<ShadedDemoDriver, ShadedDemoWindowState> {
    FnDriver::new(
        ShadedDemoDriver::default(),
        create_window_state,
        handle_event,
        render,
    )
    .with_hooks(configure_fn_driver_hooks)
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

    crate::run_native_with_driver(config, app, driver).context("run shaded_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
