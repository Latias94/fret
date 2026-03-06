#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Color, Event, Px};
use fret_launch::{
    FnDriver, WinitAppDriver, WinitEventContext, WinitHotReloadContext, WinitRenderContext,
    WinitRunnerConfig,
};
use fret_plot::cartesian::DataPoint;
use fret_plot::linking::{LinkedPlotGroup, LinkedPlotMember, PlotLinkPolicy};
use fret_plot::retained::{
    AreaPlotCanvas, AreaPlotModel, AreaSeries, LinePlotCanvas, LinePlotModel, LinePlotStyle,
    LineSeries, PlotOutput, PlotState,
};
use fret_plot::series::Series;
use fret_runtime::PlatformCapabilities;
use fret_ui::{FixedSplit, UiTree};

struct LinkedCursorDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    top_plot: fret_runtime::Model<LinePlotModel>,
    bottom_plot: fret_runtime::Model<AreaPlotModel>,
    top_state: fret_runtime::Model<PlotState>,
    top_output: fret_runtime::Model<PlotOutput>,
    bottom_state: fret_runtime::Model<PlotState>,
    bottom_output: fret_runtime::Model<PlotOutput>,
    linked: LinkedPlotGroup,
}

#[derive(Default)]
struct LinkedCursorDemoDriver;

impl LinkedCursorDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> LinkedCursorDemoWindowState {
        let n = 4096usize;

        let mut series0: Vec<DataPoint> = Vec::with_capacity(n);
        let mut series1: Vec<DataPoint> = Vec::with_capacity(n);
        let mut series2: Vec<DataPoint> = Vec::with_capacity(n);

        for i in 0..n {
            let t = i as f64 / (n - 1) as f64;
            let x = t * 10.0;
            series0.push(DataPoint {
                x,
                y: (x * 1.25).sin() * 0.75 + (x * 0.33).cos() * 0.25,
            });
            series1.push(DataPoint {
                x,
                y: (x * 1.10).sin() * 0.55 + (x * 0.20).cos() * 0.20 + 0.35,
            });
            series2.push(DataPoint {
                x,
                y: (x * 0.75).sin() * 0.35 + (x * 0.15).cos() * 0.10 - 0.35,
            });
        }

        let top_plot = app.models_mut().insert(LinePlotModel::from_series(vec![
            LineSeries::new("signal A", Series::from_points_sorted(series0, true))
                .color(Color {
                    r: 1.0,
                    g: 0.2,
                    b: 0.2,
                    a: 1.0,
                })
                .stroke_width(Px(2.5)),
            LineSeries::new(
                "signal B",
                Series::from_points_sorted(series1.clone(), true),
            )
            .color(Color {
                r: 0.2,
                g: 0.9,
                b: 0.4,
                a: 1.0,
            })
            .stroke_width(Px(2.0)),
            LineSeries::new(
                "signal C",
                Series::from_points_sorted(series2.clone(), true),
            )
            .color(Color {
                r: 0.25,
                g: 0.55,
                b: 1.0,
                a: 1.0,
            })
            .stroke_width(Px(2.0)),
        ]));

        let bottom_plot = app.models_mut().insert(AreaPlotModel::from_series(vec![
            AreaSeries::new("area B", Series::from_points_sorted(series1, true))
                .fill(Color {
                    r: 0.2,
                    g: 0.9,
                    b: 0.4,
                    a: 1.0,
                })
                .stroke(Color {
                    r: 0.2,
                    g: 0.9,
                    b: 0.4,
                    a: 1.0,
                })
                .stroke_width(Px(2.0))
                .fill_alpha(0.18),
            AreaSeries::new("area C", Series::from_points_sorted(series2, true))
                .fill(Color {
                    r: 0.25,
                    g: 0.55,
                    b: 1.0,
                    a: 1.0,
                })
                .stroke(Color {
                    r: 0.25,
                    g: 0.55,
                    b: 1.0,
                    a: 1.0,
                })
                .stroke_width(Px(2.0))
                .fill_alpha(0.18),
        ]));

        let top_state = app.models_mut().insert(PlotState::default());
        let top_output = app.models_mut().insert(PlotOutput::default());
        let bottom_state = app.models_mut().insert(PlotState::default());
        let bottom_output = app.models_mut().insert(PlotOutput::default());

        let mut linked = LinkedPlotGroup::new(PlotLinkPolicy::default());
        linked
            .push(LinkedPlotMember {
                state: top_state.clone(),
                output: top_output.clone(),
            })
            .push(LinkedPlotMember {
                state: bottom_state.clone(),
                output: bottom_output.clone(),
            });

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        LinkedCursorDemoWindowState {
            ui,
            root: None,
            top_plot,
            bottom_plot,
            top_state,
            top_output,
            bottom_state,
            bottom_output,
            linked,
        }
    }
}

fn create_window_state(
    _driver: &mut LinkedCursorDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> LinkedCursorDemoWindowState {
    LinkedCursorDemoDriver::build_ui(app, window)
}

fn hot_reload_window(
    _driver: &mut LinkedCursorDemoDriver,
    context: WinitHotReloadContext<'_, LinkedCursorDemoWindowState>,
) {
    let WinitHotReloadContext {
        app, window, state, ..
    } = context;

    crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
    state.root = None;
}

fn handle_event(
    _driver: &mut LinkedCursorDemoDriver,
    context: WinitEventContext<'_, LinkedCursorDemoWindowState>,
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
    };

    state.linked.tick(app);
}

fn render(
    _driver: &mut LinkedCursorDemoDriver,
    context: WinitRenderContext<'_, LinkedCursorDemoWindowState>,
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
        let top_style = LinePlotStyle::default();
        let top_canvas = LinePlotCanvas::new(state.top_plot.clone())
            .style(top_style)
            .debug_overlay(true)
            .state(state.top_state.clone())
            .output(state.top_output.clone());

        let bottom_style = LinePlotStyle::default();
        let bottom_canvas = AreaPlotCanvas::new(state.bottom_plot.clone())
            .style(bottom_style)
            .debug_overlay(true)
            .state(state.bottom_state.clone())
            .output(state.bottom_output.clone());

        let top_node = LinePlotCanvas::create_node(&mut state.ui, top_canvas);
        let bottom_node = AreaPlotCanvas::create_node(&mut state.ui, bottom_canvas);
        let root = FixedSplit::create_node_with_children(
            &mut state.ui,
            FixedSplit::vertical(0.5),
            top_node,
            bottom_node,
        );

        state.ui.set_root(root);
        state.ui.set_focus(Some(top_node));
        state.root = Some(root);
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
        main_window_title: "fret-demo linked_cursor_demo (linked view/query/cursor)".to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(960.0, 760.0),
        ..Default::default()
    }
}

pub fn build_fn_driver() -> impl WinitAppDriver {
    FnDriver::new(
        LinkedCursorDemoDriver::default(),
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

    crate::run_native_with_compat_driver(config, app, driver).context("run linked_cursor_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
