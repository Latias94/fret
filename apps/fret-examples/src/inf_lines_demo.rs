#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::geometry::Px;
use fret_core::{AppWindowId, Event};
use fret_launch::{
    FnDriver, WinitEventContext, WinitHotReloadContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_plot::LinePlotPanelBinding;
use fret_plot::declarative::line_plot_panel_in;
use fret_plot::models::{LinePlotModel, LineSeries, YAxis};
use fret_plot::plot::axis::AxisLabelFormatter;
use fret_plot::series::Series;
use fret_plot::state::{InfLineX, InfLineY, PlotOverlays, PlotState};
use fret_plot::style::{LinePlotStyle, SeriesTooltipMode};
use fret_runtime::PlatformCapabilities;
use fret_ui::{UiTree, declarative};

struct InfLinesDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: LinePlotPanelBinding,
    last_logged_output_revision: u64,
}

#[derive(Default)]
struct InfLinesDemoDriver;

impl InfLinesDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> InfLinesDemoWindowState {
        let n = 4096usize;

        let mut series0 = Vec::with_capacity(n);
        let mut series1 = Vec::with_capacity(n);
        let mut series2 = Vec::with_capacity(n);
        let mut series3 = Vec::with_capacity(n);

        let push = |series: &mut Vec<fret_plot::cartesian::DataPoint>, x: f64, y: f64| {
            if !x.is_finite() || !y.is_finite() {
                return;
            }
            series.push(fret_plot::cartesian::DataPoint { x, y });
        };

        for i in 0..n {
            let t = i as f64 / (n - 1) as f64;
            let x = t * 100.0;
            let u = t * std::f64::consts::TAU * 3.0;
            push(&mut series0, x, (u * 1.00).sin() * 0.75);
            push(&mut series1, x, (u * 0.85).sin() * 25.0 + 10.0);
            push(&mut series2, x, (u * 0.33).cos() * 250.0 + 500.0);
            push(&mut series3, x, (u * 0.25).sin() * 1_500.0 + 2_000.0);
        }

        let model = LinePlotModel::from_series(vec![
            LineSeries::new("signal A (left)", Series::from_points_sorted(series0, true)),
            LineSeries::new(
                "signal B (right)",
                Series::from_points_sorted(series1, true),
            )
            .y_axis(YAxis::Right),
            LineSeries::new(
                "signal C (right2)",
                Series::from_points_sorted(series2, true),
            )
            .y_axis(YAxis::Right2),
            LineSeries::new(
                "signal D (right3)",
                Series::from_points_sorted(series3, true),
            )
            .y_axis(YAxis::Right3),
        ]);

        let mut state = PlotState::default();
        state.overlays = PlotOverlays {
            inf_lines_x: vec![
                InfLineX::new(25.0),
                InfLineX::new(50.0).width(Px(2.0)),
                InfLineX::new(75.0),
            ],
            inf_lines_y: vec![
                InfLineY::new(0.0, YAxis::Left),
                InfLineY::new(10.0, YAxis::Right).width(Px(2.0)),
                InfLineY::new(500.0, YAxis::Right2),
                InfLineY::new(2_000.0, YAxis::Right3),
            ],
            ..Default::default()
        };

        let plot = LinePlotPanelBinding::new_with_state(app, model, state);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        InfLinesDemoWindowState {
            ui,
            root: None,
            plot,
            last_logged_output_revision: 0,
        }
    }
}

fn create_window_state(
    _driver: &mut InfLinesDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> InfLinesDemoWindowState {
    InfLinesDemoDriver::build_ui(app, window)
}

fn hot_reload_window(
    _driver: &mut InfLinesDemoDriver,
    context: WinitHotReloadContext<'_, InfLinesDemoWindowState>,
) {
    let WinitHotReloadContext {
        app, window, state, ..
    } = context;

    crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
    state.root = None;
}

fn handle_event(
    _driver: &mut InfLinesDemoDriver,
    context: WinitEventContext<'_, InfLinesDemoWindowState>,
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
    _driver: &mut InfLinesDemoDriver,
    context: WinitRenderContext<'_, InfLinesDemoWindowState>,
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

    let plot = state.plot.clone();
    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
        .render_root("inf-lines-demo", move |cx| {
            let style = LinePlotStyle {
                series_tooltip: SeriesTooltipMode::NearestAtCursor,
                ..Default::default()
            };
            let props = plot
                .panel_props()
                .style(style)
                .y_axis_labels(AxisLabelFormatter::custom(
                    0x494e464c_5900u64,
                    |v, _span| {
                        if !v.is_finite() {
                            return "NA".to_string();
                        }
                        format!("{v:.2} V")
                    },
                ))
                .y2_axis_labels(AxisLabelFormatter::custom(
                    0x494e464c_5902u64,
                    |v, _span| {
                        if !v.is_finite() {
                            return "NA".to_string();
                        }
                        format!("{v:.1} A")
                    },
                ))
                .y3_axis_labels(AxisLabelFormatter::custom(
                    0x494e464c_5903u64,
                    |v, _span| {
                        if !v.is_finite() {
                            return "NA".to_string();
                        }
                        format!("{v:.0} mA")
                    },
                ))
                .y4_axis_labels(AxisLabelFormatter::custom(
                    0x494e464c_5904u64,
                    |v, _span| {
                        if !v.is_finite() {
                            return "NA".to_string();
                        }
                        format!("{v:.0} Pa")
                    },
                ));
            vec![line_plot_panel_in(cx, props)]
        });

    state.root = Some(root);
    state.ui.set_root(root);
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
        main_window_title: "fret-demo inf_lines_demo (caller-owned overlays: InfLines)".to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_fn_driver() -> impl fret_launch::WinitAppDriver {
    FnDriver::new(
        InfLinesDemoDriver::default(),
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

    crate::run_native_with_compat_driver(config, app, driver).context("run inf_lines_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
