#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event};
use fret_launch::{FnDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig};
use fret_plot::Histogram2DPlotPanelBinding;
use fret_plot::declarative::histogram2d_plot_panel_in;
use fret_plot::models::Histogram2DPlotModel;
use fret_plot::plot::axis::{AxisLabelFormatter, AxisNumberFormat};
use fret_plot::plot::histogram2d::{Histogram2DConfig, histogram2d_counts};
use fret_runtime::PlatformCapabilities;
use fret_ui::{UiTree, declarative};

struct Histogram2DDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: Histogram2DPlotPanelBinding,
}

#[derive(Default)]
struct Histogram2DDemoDriver;

impl Histogram2DDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> Histogram2DDemoWindowState {
        let bounds = fret_plot::cartesian::DataRect {
            x_min: -4.0,
            x_max: 4.0,
            y_min: -3.0,
            y_max: 3.0,
        };

        let mut points = Vec::with_capacity(120_000);
        for i in 0..points.capacity() {
            let t = i as f64 * 0.000_1;
            let x = (t * 7.0).sin() * 2.6 + (t * 2.3).cos() * 0.8;
            let y = (t * 5.0).cos() * 2.1 + (t * 1.7).sin() * 0.7;
            points.push(fret_plot::cartesian::DataPoint { x, y });
        }

        let grid = histogram2d_counts(Histogram2DConfig::new(bounds, 256, 192), points);
        let model = Histogram2DPlotModel::new(grid.data_bounds, grid.cols, grid.rows, grid.values);
        let plot = Histogram2DPlotPanelBinding::new(app, model);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Histogram2DDemoWindowState {
            ui,
            root: None,
            plot,
        }
    }
}

fn create_window_state(
    _driver: &mut Histogram2DDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> Histogram2DDemoWindowState {
    Histogram2DDemoDriver::build_ui(app, window)
}

fn handle_event(
    _driver: &mut Histogram2DDemoDriver,
    context: WinitEventContext<'_, Histogram2DDemoWindowState>,
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

fn render(
    _driver: &mut Histogram2DDemoDriver,
    context: WinitRenderContext<'_, Histogram2DDemoWindowState>,
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
                .render_root("histogram2d-demo", {
                    let plot = state.plot.clone();
                    move |cx| {
                        let props = plot
                            .panel_props()
                            .x_axis_labels(AxisLabelFormatter::number(AxisNumberFormat::Fixed(2)))
                            .y_axis_labels(AxisLabelFormatter::number(AxisNumberFormat::Fixed(2)));
                        vec![histogram2d_plot_panel_in(cx, props)]
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
        main_window_title: "fret-demo histogram2d_demo (Histogram2D + colormap + colorbar)"
            .to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_fn_driver() -> impl fret_launch::WinitAppDriver {
    FnDriver::new(
        Histogram2DDemoDriver::default(),
        create_window_state,
        handle_event,
        render,
    )
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

    crate::run_native_with_driver(config, app, driver).context("run histogram2d_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
