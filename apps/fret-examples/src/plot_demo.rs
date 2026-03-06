#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event};
use fret_launch::{
    FnDriver, WinitAppDriver, WinitEventContext, WinitHotReloadContext, WinitRenderContext,
    WinitRunnerConfig,
};
use fret_plot::cartesian::{AxisScale, DataPoint};
use fret_plot::plot::axis::AxisLabelFormatter;
use fret_plot::retained::{
    LinePlotCanvas, LinePlotStyle, LineSeries, PlotOutput, PlotState, YAxis,
};
use fret_plot::series::Series;
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;

struct PlotDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: fret_runtime::Model<fret_plot::retained::LinePlotModel>,
    plot_state: fret_runtime::Model<PlotState>,
    plot_output: fret_runtime::Model<PlotOutput>,
    last_logged_output_revision: u64,
}

#[derive(Default)]
struct PlotDemoDriver;

impl PlotDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> PlotDemoWindowState {
        let n = 4096usize;

        let mut series0: Vec<DataPoint> = Vec::with_capacity(n);
        let mut series1: Vec<DataPoint> = Vec::with_capacity(n);
        let mut series2: Vec<DataPoint> = Vec::with_capacity(n);
        let mut series3: Vec<DataPoint> = Vec::with_capacity(n);
        let mut series4: Vec<DataPoint> = Vec::with_capacity(n);
        let mut series5: Vec<DataPoint> = Vec::with_capacity(n);

        let push = |series: &mut Vec<DataPoint>, x: f64, y: f64| {
            if !x.is_finite() || !y.is_finite() {
                return;
            }
            series.push(DataPoint { x, y });
        };

        for i in 0..n {
            let t = i as f64 / (n - 1) as f64;
            // Log X domain: 0.1 .. 1000.0 (4 decades).
            let x = 0.1 * 10.0_f64.powf(t * 4.0);
            let u = x.log10() * std::f64::consts::TAU;
            push(
                &mut series0,
                x,
                (u * 1.25).sin() * 0.75 + (u * 0.33).cos() * 0.25,
            );
            push(
                &mut series1,
                x,
                (u * 1.10).sin() * 0.55 + (u * 0.20).cos() * 0.20 + 0.35,
            );
            push(
                &mut series2,
                x,
                (u * 0.75).sin() * 0.35 + (u * 0.15).cos() * 0.10 - 0.35,
            );
            // A "right axis" series with a very different scale (e.g. amps vs volts).
            push(&mut series3, x, (u * 0.22).sin() * 25.0 + 10.0);
            // Additional right-side axes (Y3/Y4) to validate multi-axis support.
            push(&mut series4, x, (u * 0.18).cos() * 250.0 + 500.0);
            push(&mut series5, x, (u * 0.08).sin() * 1_500.0 + 2_000.0);
        }

        let plot = app
            .models_mut()
            .insert(fret_plot::retained::LinePlotModel::from_series(vec![
                LineSeries::new("signal A", Series::from_points_sorted(series0, true)),
                LineSeries::new("signal B", Series::from_points_sorted(series1, true)),
                LineSeries::new("signal C", Series::from_points_sorted(series2, true)),
                LineSeries::new(
                    "signal D (right)",
                    Series::from_points_sorted(series3, true),
                )
                .y_axis(YAxis::Right),
                LineSeries::new(
                    "signal E (right2)",
                    Series::from_points_sorted(series4, true),
                )
                .y_axis(YAxis::Right2),
                LineSeries::new(
                    "signal F (right3)",
                    Series::from_points_sorted(series5, true),
                )
                .y_axis(YAxis::Right3),
            ]));
        let plot_state = app.models_mut().insert(PlotState::default());
        let plot_output = app.models_mut().insert(PlotOutput::default());

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        PlotDemoWindowState {
            ui,
            root: None,
            plot,
            plot_state,
            plot_output,
            last_logged_output_revision: 0,
        }
    }
}

fn create_window_state(
    _driver: &mut PlotDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> PlotDemoWindowState {
    PlotDemoDriver::build_ui(app, window)
}

fn hot_reload_window(
    _driver: &mut PlotDemoDriver,
    context: WinitHotReloadContext<'_, PlotDemoWindowState>,
) {
    let WinitHotReloadContext {
        app, window, state, ..
    } = context;

    crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
    state.root = None;
}

fn configure_fn_driver_hooks(
    hooks: &mut fret_launch::FnDriverHooks<PlotDemoDriver, PlotDemoWindowState>,
) {
    hooks.hot_reload_window = Some(hot_reload_window);
}

fn handle_event(
    _driver: &mut PlotDemoDriver,
    context: WinitEventContext<'_, PlotDemoWindowState>,
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

fn render(_driver: &mut PlotDemoDriver, context: WinitRenderContext<'_, PlotDemoWindowState>) {
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
        let style = LinePlotStyle::default();
        let canvas = LinePlotCanvas::new(state.plot.clone())
            .style(style)
            .x_axis_scale(AxisScale::Log10)
            .y_axis_labels(AxisLabelFormatter::custom(0x554e4954u64, |v, span| {
                if !v.is_finite() {
                    return "NA".to_string();
                }
                if span.abs().is_finite() && span.abs() < 1.0 {
                    format!("{v:.4} V")
                } else if v.abs() < 10.0 {
                    format!("{v:.3} V")
                } else {
                    format!("{v:.2} V")
                }
            }))
            .y2_axis_labels(AxisLabelFormatter::custom(0x5941u64, |v, _span| {
                if !v.is_finite() {
                    return "NA".to_string();
                }
                format!("{v:.1} A")
            }))
            .y3_axis_labels(AxisLabelFormatter::custom(0x5941_3303u64, |v, _span| {
                if !v.is_finite() {
                    return "NA".to_string();
                }
                format!("{v:.0} mA")
            }))
            .y4_axis_labels(AxisLabelFormatter::custom(0x5941_3404u64, |v, _span| {
                if !v.is_finite() {
                    return "NA".to_string();
                }
                format!("{v:.0} Pa")
            }))
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
        main_window_title:
            "fret-demo plot_demo (LogX, RMB drag zoom, Alt+LMB drag query, LMB double-click fit)"
                .to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_fn_driver() -> impl WinitAppDriver {
    FnDriver::new(
        PlotDemoDriver::default(),
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
    crate::run_native_with_fn_driver_with_hooks(
        config,
        app,
        PlotDemoDriver::default(),
        create_window_state,
        handle_event,
        render,
        configure_fn_driver_hooks,
    )
    .context("run plot_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
