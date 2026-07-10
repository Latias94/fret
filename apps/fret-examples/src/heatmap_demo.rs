#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event};
use fret_launch::{
    FnDriver, WinitEventContext, WinitHotReloadContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_plot::HeatmapPlotPanelBinding;
use fret_plot::cartesian::DataRect;
use fret_plot::declarative::heatmap_plot_panel_in;
use fret_plot::models::HeatmapPlotModel;
use fret_plot::style::LinePlotStyle;
use fret_runtime::PlatformCapabilities;
use fret_ui::{UiTree, declarative};

struct HeatmapDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: HeatmapPlotPanelBinding,
    last_logged_output_revision: u64,
}

#[derive(Default)]
struct HeatmapDemoDriver;

impl HeatmapDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> HeatmapDemoWindowState {
        let cols = 128usize;
        let rows = 128usize;

        let data_bounds = DataRect {
            x_min: 0.0,
            x_max: 10.0,
            y_min: 0.0,
            y_max: 10.0,
        };

        let mut values: Vec<f32> = Vec::with_capacity(cols.saturating_mul(rows));
        for row in 0..rows {
            let y = (row as f64 + 0.5) / (rows as f64) * 10.0;
            for col in 0..cols {
                let x = (col as f64 + 0.5) / (cols as f64) * 10.0;
                let dx = x - 5.0;
                let dy = y - 5.0;
                let r2 = dx * dx + dy * dy;

                // A smooth Gaussian bump plus a small periodic component.
                let g = (-r2 / 6.0).exp();
                let w = (x * 2.2).sin() * (y * 1.7).cos() * 0.15;
                values.push((g + w) as f32);
            }
        }

        let plot = HeatmapPlotPanelBinding::new(
            app,
            HeatmapPlotModel::new(data_bounds, cols, rows, values),
        );

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        HeatmapDemoWindowState {
            ui,
            root: None,
            plot,
            last_logged_output_revision: 0,
        }
    }
}

fn create_window_state(
    _driver: &mut HeatmapDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> HeatmapDemoWindowState {
    HeatmapDemoDriver::build_ui(app, window)
}

fn hot_reload_window(
    _driver: &mut HeatmapDemoDriver,
    context: WinitHotReloadContext<'_, HeatmapDemoWindowState>,
) {
    let WinitHotReloadContext {
        app, window, state, ..
    } = context;

    crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
    state.root = None;
}

fn handle_event(
    _driver: &mut HeatmapDemoDriver,
    context: WinitEventContext<'_, HeatmapDemoWindowState>,
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
    _driver: &mut HeatmapDemoDriver,
    context: WinitRenderContext<'_, HeatmapDemoWindowState>,
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
                .render_root("heatmap-demo", {
                    let plot = state.plot.clone();
                    move |cx| {
                        let mut style = LinePlotStyle::default();
                        style.heatmap_show_colorbar = true;
                        let props = plot.panel_props().style(style);
                        vec![heatmap_plot_panel_in(cx, props)]
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

pub fn build_fn_driver() -> impl fret_launch::WinitAppDriver {
    FnDriver::new(
        HeatmapDemoDriver::default(),
        create_window_state,
        handle_event,
        render,
    )
    .with_hooks(|hooks| {
        hooks.hot_reload_window = Some(hot_reload_window);
    })
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title:
            "fret-demo heatmap_demo (RMB drag zoom, Alt+LMB drag query, LMB double-click fit)"
                .to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
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

    crate::run_native_with_driver(config, app, driver).context("run heatmap_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
