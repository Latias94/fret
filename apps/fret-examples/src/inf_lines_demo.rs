#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::geometry::Px;
use fret_core::{AppWindowId, Event};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_plot::plot::axis::AxisLabelFormatter;
use fret_plot::retained::{
    InfLineX, InfLineY, LinePlotCanvas, LinePlotStyle, LineSeries, PlotOutput, PlotOverlays,
    PlotState, SeriesTooltipMode, YAxis,
};
use fret_plot::series::Series;
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;

struct InfLinesDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: fret_runtime::Model<fret_plot::retained::LinePlotModel>,
    plot_state: fret_runtime::Model<PlotState>,
    plot_output: fret_runtime::Model<PlotOutput>,
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

        let plot = app
            .models_mut()
            .insert(fret_plot::retained::LinePlotModel::from_series(vec![
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
            ]));

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

        let plot_state = app.models_mut().insert(state);
        let plot_output = app.models_mut().insert(PlotOutput::default());

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        InfLinesDemoWindowState {
            ui,
            root: None,
            plot,
            plot_state,
            plot_output,
            last_logged_output_revision: 0,
        }
    }
}

impl WinitAppDriver for InfLinesDemoDriver {
    type WindowState = InfLinesDemoWindowState;

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
            let canvas = LinePlotCanvas::new(state.plot.clone())
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
                ))
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
        main_window_title: "fret-demo inf_lines_demo (caller-owned overlays: InfLines)".to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    InfLinesDemoDriver::default()
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

    crate::run_native_demo(config, app, driver).context("run inf_lines_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
