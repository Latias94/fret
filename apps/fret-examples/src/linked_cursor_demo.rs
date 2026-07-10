#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Color, Event, Px};
use fret_launch::{
    FnDriver, WinitEventContext, WinitHotReloadContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_plot::cartesian::DataPoint;
use fret_plot::declarative::{area_plot_panel_in, line_plot_panel_in};
use fret_plot::linking::{LinkedPlotGroup, PlotLinkPolicy};
use fret_plot::models::{AreaPlotModel, AreaSeries, LinePlotModel, LineSeries};
use fret_plot::series::Series;
use fret_plot::style::LinePlotStyle;
use fret_plot::{AreaPlotPanelBinding, LinePlotPanelBinding};
use fret_runtime::PlatformCapabilities;
use fret_ui::{FixedSplit, UiTree, declarative};

struct LinkedCursorDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    top_plot: LinePlotPanelBinding,
    bottom_plot: AreaPlotPanelBinding,
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

        let top_plot = LinePlotPanelBinding::new(
            app,
            LinePlotModel::from_series(vec![
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
            ]),
        );

        let bottom_plot = AreaPlotPanelBinding::new(
            app,
            AreaPlotModel::from_series(vec![
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
            ]),
        );

        let mut linked = LinkedPlotGroup::new(PlotLinkPolicy::default());
        linked.push_binding(&top_plot).push_binding(&bottom_plot);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        LinkedCursorDemoWindowState {
            ui,
            root: None,
            top_plot,
            bottom_plot,
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
        let top_node =
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                .render_root("linked-cursor-demo-top", {
                    let top_plot = state.top_plot.clone();
                    move |cx| {
                        let top_style = LinePlotStyle::default();
                        let props = top_plot.panel_props().style(top_style);
                        vec![line_plot_panel_in(cx, props)]
                    }
                });
        let bottom_node =
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                .render_root("linked-cursor-demo-bottom", {
                    let bottom_plot = state.bottom_plot.clone();
                    move |cx| {
                        let bottom_style = LinePlotStyle::default();
                        let props = bottom_plot.panel_props().style(bottom_style);
                        vec![area_plot_panel_in(cx, props)]
                    }
                });
        let root = FixedSplit::create_node_with_children(
            &mut state.ui,
            FixedSplit::vertical(0.5),
            top_node,
            bottom_node,
        );

        state.ui.set_root(root);
        state.ui.set_focus(Some(top_node));
        state.ui.publish_window_runtime_snapshots(app);
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

pub fn build_fn_driver() -> impl fret_launch::WinitAppDriver {
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

    crate::run_native_with_driver(config, app, driver).context("run linked_cursor_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
