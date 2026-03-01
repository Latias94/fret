#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::scene::Color;
use fret_core::{AppWindowId, Event};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_plot::retained::{
    BarsPlotCanvas, BarsPlotModel, CategoryBarSeries, LinePlotStyle, PlotOutput, PlotState,
    SeriesTooltipMode,
};
use fret_plot::series::SeriesId;
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
use std::sync::Arc;

struct StackedBarsDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: fret_runtime::Model<BarsPlotModel>,
    plot_state: fret_runtime::Model<PlotState>,
    plot_output: fret_runtime::Model<PlotOutput>,
    last_logged_output_revision: u64,
}

#[derive(Default)]
struct StackedBarsDemoDriver;

impl StackedBarsDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> StackedBarsDemoWindowState {
        let n = 12usize;

        let mut categories: Vec<f64> = Vec::with_capacity(n);
        let mut a: Vec<f64> = Vec::with_capacity(n);
        let mut b: Vec<f64> = Vec::with_capacity(n);
        let mut c: Vec<f64> = Vec::with_capacity(n);
        for i in 0..n {
            let x = i as f64;
            categories.push(x);

            let t = x * 0.55;
            a.push((t.sin() * 1.4).clamp(-1.2, 2.2));
            b.push(((t + 0.9).cos() * 1.1).clamp(-1.1, 2.0));
            c.push(((t + 1.6).sin() * 0.9).clamp(-0.9, 1.6));
        }

        let categories: Arc<[f64]> = categories.into();
        let series = vec![
            CategoryBarSeries::new("A", a.into())
                .id(SeriesId::from_label("A"))
                .fill(Color {
                    a: 0.65,
                    ..Color::from_srgb_hex_rgb(0x59_a6_f2)
                }),
            CategoryBarSeries::new("B", b.into())
                .id(SeriesId::from_label("B"))
                .fill(Color {
                    a: 0.65,
                    ..Color::from_srgb_hex_rgb(0xf2_73_8c)
                }),
            CategoryBarSeries::new("C", c.into())
                .id(SeriesId::from_label("C"))
                .fill(Color {
                    a: 0.65,
                    ..Color::from_srgb_hex_rgb(0x73_d9_8c)
                }),
        ];

        let model = BarsPlotModel::stacked_categories(categories, series, 0.8);
        let plot = app.models_mut().insert(model);
        let plot_state = app.models_mut().insert(PlotState::default());
        let plot_output = app.models_mut().insert(PlotOutput::default());

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        StackedBarsDemoWindowState {
            ui,
            root: None,
            plot,
            plot_state,
            plot_output,
            last_logged_output_revision: 0,
        }
    }
}

impl WinitAppDriver for StackedBarsDemoDriver {
    type WindowState = StackedBarsDemoWindowState;

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
            let canvas = BarsPlotCanvas::new(state.plot.clone())
                .style(style)
                .state(state.plot_state.clone())
                .output(state.plot_output.clone());
            let node = BarsPlotCanvas::create_node(&mut state.ui, canvas);
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
        main_window_title:
            "fret-demo stacked_bars_demo (RMB drag zoom, Alt+LMB drag query, LMB double-click fit)"
                .to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    StackedBarsDemoDriver::default()
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

    crate::run_native_demo(config, app, driver).context("run stacked_bars_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
