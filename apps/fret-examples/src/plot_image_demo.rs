#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::geometry::Px;
use fret_core::{AppWindowId, Event, ImageColorSpace};
#[cfg(not(target_arch = "wasm32"))]
use fret_launch::run_app;
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_plot::plot::axis::{AxisLabelFormatter, AxisNumberFormat};
use fret_plot::retained::{
    LinePlotCanvas, LinePlotStyle, LineSeries, PlotImage, PlotImageLayer, PlotOutput, PlotOverlays,
    PlotState, SeriesTooltipMode, YAxis,
};
use fret_plot::series::Series;
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
use fret_ui_assets::image_asset_cache::{ImageAssetCacheHostExt, ImageAssetKey};

struct PlotImageDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: fret_runtime::Model<fret_plot::retained::LinePlotModel>,
    plot_state: fret_runtime::Model<PlotState>,
    plot_output: fret_runtime::Model<PlotOutput>,

    image_bytes: Vec<u8>,
    image_key: ImageAssetKey,
    image: Option<fret_core::ImageId>,
    image_size: (u32, u32),
}

#[derive(Default)]
struct PlotImageDemoDriver;

impl PlotImageDemoDriver {
    fn generate_rgba8_pattern(width: u32, height: u32) -> Vec<u8> {
        let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
        for y in 0..height {
            for x in 0..width {
                let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
                let cell = ((x / 16) ^ (y / 16)) & 1;
                let t = (x as f32) / (width.max(1) as f32);
                let (r, g, b) = if cell == 0 {
                    let r = (40.0 + t * 120.0) as u8;
                    (r, 64u8, 92u8)
                } else {
                    let b = (80.0 + t * 120.0) as u8;
                    (92u8, 92u8, b)
                };
                out[idx] = r;
                out[idx + 1] = g;
                out[idx + 2] = b;
                out[idx + 3] = 255;
            }
        }
        out
    }

    fn use_image_asset(
        app: &mut App,
        window: AppWindowId,
        key: ImageAssetKey,
        size: (u32, u32),
        bytes: &[u8],
    ) -> Option<fret_core::ImageId> {
        app.with_image_asset_cache(|cache, app| {
            cache.use_rgba8_keyed(
                app,
                window,
                key,
                size.0,
                size.1,
                bytes,
                ImageColorSpace::Srgb,
            )
        })
    }

    fn build_ui(app: &mut App, window: AppWindowId) -> PlotImageDemoWindowState {
        let n = 4096usize;
        let mut points = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f64 / (n - 1) as f64;
            let x = t * 100.0;
            let u = t * std::f64::consts::TAU * 3.0;
            points.push(fret_plot::cartesian::DataPoint {
                x,
                y: (u * 1.00).sin(),
            });
        }

        let plot = app
            .models_mut()
            .insert(fret_plot::retained::LinePlotModel::from_series(vec![
                LineSeries::new("signal", Series::from_points_sorted(points, true)),
            ]));

        let mut state = PlotState::default();
        state.overlays = PlotOverlays::default();
        let plot_state = app.models_mut().insert(state);
        let plot_output = app.models_mut().insert(PlotOutput::default());

        let size = (256, 256);
        let bytes = Self::generate_rgba8_pattern(size.0, size.1);
        let key = ImageAssetKey::from_rgba8(size.0, size.1, ImageColorSpace::Srgb, &bytes);
        let image = Self::use_image_asset(app, window, key, size, &bytes);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        PlotImageDemoWindowState {
            ui,
            root: None,
            plot,
            plot_state,
            plot_output,
            image_bytes: bytes,
            image_key: key,
            image,
            image_size: size,
        }
    }
}

impl WinitAppDriver for PlotImageDemoDriver {
    type WindowState = PlotImageDemoWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
            ..
        } = context;

        app.with_image_asset_cache(|cache, app| {
            cache.handle_event(app, window, event);
        });

        match event {
            Event::WindowCloseRequested
            | Event::KeyDown {
                key: fret_core::KeyCode::Escape,
                ..
            } => {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
            }
            _ => {
                state.ui.dispatch_event(app, services, event);
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

        let image = Self::use_image_asset(
            app,
            window,
            state.image_key,
            state.image_size,
            &state.image_bytes,
        );
        if image != state.image {
            state.image = image;
            let _ = app.models_mut().update(&state.plot_state, |s| {
                s.overlays.images.clear();
                if let Some(image) = image {
                    s.overlays.images.push(
                        PlotImage::new(
                            image,
                            fret_plot::cartesian::DataRect {
                                x_min: 10.0,
                                x_max: 90.0,
                                y_min: -1.25,
                                y_max: 1.25,
                            },
                            YAxis::Left,
                        )
                        .opacity(0.85)
                        .layer(PlotImageLayer::BelowGrid),
                    );
                }
            });
        }

        if state.image.is_none() {
            app.push_effect(Effect::RequestAnimationFrame(window));
        }

        let root = state.root.get_or_insert_with(|| {
            let style = LinePlotStyle {
                series_tooltip: SeriesTooltipMode::NearestAtCursor,
                hover_threshold: Px(10.0),
                ..Default::default()
            };
            let canvas = LinePlotCanvas::new(state.plot.clone())
                .style(style)
                .y_axis_labels(AxisLabelFormatter::number(AxisNumberFormat::Fixed(2)))
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
        main_window_title: "fret-demo plot_image_demo (PlotImage underlay)".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    PlotImageDemoDriver::default()
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

    run_app(config, app, driver)
        .context("run plot_image_demo app")
        .map_err(anyhow::Error::from)
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
