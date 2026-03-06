#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event};
use fret_launch::{
    FnDriver, WinitAppDriver, WinitEventContext, WinitHotReloadContext, WinitRenderContext,
    WinitRunnerConfig,
};
use fret_plot::retained::{
    LinePlotCanvas, LinePlotModel, LinePlotStyle, LineSeries, PlotOutput, PlotOverlays, PlotState,
    PlotText, SeriesTooltipMode, TagX, TagY, YAxis,
};
use fret_plot::series::Series;
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;

struct TagsDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: fret_runtime::Model<LinePlotModel>,
    plot_state: fret_runtime::Model<PlotState>,
    plot_output: fret_runtime::Model<PlotOutput>,
}

#[derive(Default)]
struct TagsDemoDriver;

impl TagsDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> TagsDemoWindowState {
        let n = 2048usize;
        let mut series0 = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f64 / (n - 1) as f64;
            let x = t * 100.0;
            let y = (t * std::f64::consts::TAU * 3.0).sin();
            if !x.is_finite() || !y.is_finite() {
                continue;
            }
            series0.push(fret_plot::cartesian::DataPoint { x, y });
        }

        let plot = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "signal",
                Series::from_points_sorted(series0, true),
            )]));

        let mut state = PlotState::default();
        state.overlays = PlotOverlays {
            tags_x: vec![TagX::new(25.0).label("T1"), TagX::new(75.0).label("T2")],
            tags_y: vec![TagY::new(0.5, YAxis::Left).label("limit")],
            text: vec![PlotText::new(
                50.0,
                -0.75,
                YAxis::Left,
                "PlotText at (50, -0.75)",
            )],
            ..Default::default()
        };

        let plot_state = app.models_mut().insert(state);
        let plot_output = app.models_mut().insert(PlotOutput::default());

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        TagsDemoWindowState {
            ui,
            root: None,
            plot,
            plot_state,
            plot_output,
        }
    }
}

fn create_window_state(
    _driver: &mut TagsDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> TagsDemoWindowState {
    TagsDemoDriver::build_ui(app, window)
}

fn hot_reload_window(
    _driver: &mut TagsDemoDriver,
    context: WinitHotReloadContext<'_, TagsDemoWindowState>,
) {
    let WinitHotReloadContext {
        app, window, state, ..
    } = context;

    crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
    state.root = None;
}

fn handle_event(
    _driver: &mut TagsDemoDriver,
    context: WinitEventContext<'_, TagsDemoWindowState>,
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
        }
    }
}

fn render(_driver: &mut TagsDemoDriver, context: WinitRenderContext<'_, TagsDemoWindowState>) {
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
        let canvas = LinePlotCanvas::new(state.plot.clone())
            .style(style)
            .state(state.plot_state.clone())
            .output(state.plot_output.clone());
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

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "fret-demo tags_demo (TagX/TagY/PlotText overlays)".to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    FnDriver::new(
        TagsDemoDriver::default(),
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
    let driver = build_driver();

    crate::run_native_with_compat_driver(config, app, driver).context("run tags_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
