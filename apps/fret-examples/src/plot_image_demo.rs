use anyhow::Context as _;
use fret::app::prelude::*;
use fret::app::{RenderContextAccess as _, ui_assets};
use fret_bootstrap::ui_app_driver;
use fret_plot::cartesian::{AxisScale, DataPoint, DataRect};
use fret_plot::declarative::{LinePlotPanelProps, line_plot_panel_in};
use fret_plot::models::{LinePlotModel, LineSeries, YAxis};
use fret_plot::plot::axis::{AxisLabelFormatter, AxisNumberFormat};
use fret_plot::series::Series;
use fret_plot::state::{PlotImage, PlotImageLayer, PlotOutput, PlotOverlays, PlotState};
use fret_plot::style::{LinePlotStyle, SeriesTooltipMode};
use fret_runtime::Model;
use fret_runtime::PlatformCapabilities;

struct PlotImageDemoView {
    model: Model<LinePlotModel>,
    plot_state: Model<PlotState>,
    plot_output: Model<PlotOutput>,
    image_bytes: Vec<u8>,
    image: Option<ui_assets::ImageId>,
    image_size: (u32, u32),
}

impl PlotImageDemoView {
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
}

pub fn build_app() -> fret::app::App {
    let mut app = fret::app::App::new();
    app.set_global(PlatformCapabilities::default());
    app
}

pub fn build_runner_config() -> fret_launch::WinitRunnerConfig {
    fret_launch::WinitRunnerConfig {
        main_window_title: "fret-demo plot_image_demo".to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_fn_driver() -> impl fret_launch::WinitAppDriver {
    ui_app_driver::UiAppDriver::new(
        "plot-image-demo",
        fret::advanced::view::view_init_window::<PlotImageDemoView>,
        fret::advanced::view::view_view::<PlotImageDemoView>,
    )
    .on_preferences(
        ui_app_driver::default_on_preferences::<
            fret::advanced::view::ViewWindowState<PlotImageDemoView>,
        >,
    )
    .into_fn_driver()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    FretApp::new("plot-image-demo")
        .window("plot_image_demo", (960.0, 640.0))
        .view::<PlotImageDemoView>()?
        .run()
        .context("run plot_image_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}

impl View for PlotImageDemoView {
    fn init(app: &mut App, _window: WindowId) -> Self {
        let n = 4096usize;
        let mut points = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f64 / (n - 1) as f64;
            let x = t * 100.0;
            let u = t * std::f64::consts::TAU * 3.0;
            points.push(DataPoint {
                x,
                y: (u * 1.00).sin(),
            });
        }

        let plot = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "signal",
                Series::from_points_sorted(points, true),
            )]));

        let state = PlotState {
            overlays: PlotOverlays::default(),
            ..Default::default()
        };
        let plot_state = app.models_mut().insert(state);
        let plot_output = app.models_mut().insert(PlotOutput::default());

        let size = (256, 256);
        let bytes = Self::generate_rgba8_pattern(size.0, size.1);

        Self {
            model: plot,
            plot_state,
            plot_output,
            image_bytes: bytes,
            image: None,
            image_size: size,
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let (_key, image, _status) = ui_assets::rgba8_image_state(
            cx,
            self.image_size.0,
            self.image_size.1,
            self.image_bytes.as_slice(),
            ui_assets::ImageColorSpace::Srgb,
        );
        if image != self.image {
            self.image = image;
            let _ = cx.app_mut().models_mut().update(&self.plot_state, |state| {
                state.overlays.images.clear();
                if let Some(image) = image {
                    state.overlays.images.push(
                        PlotImage::new(
                            image,
                            DataRect {
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

        if self.image.is_none() {
            cx.request_animation_frame();
        }

        let style = LinePlotStyle {
            series_tooltip: SeriesTooltipMode::NearestAtCursor,
            hover_threshold: Px(10.0),
            ..Default::default()
        };
        let props = LinePlotPanelProps::new(self.model.clone())
            .style(style)
            .y_axis_labels(AxisLabelFormatter::number(AxisNumberFormat::Fixed(2)))
            .state(self.plot_state.clone())
            .output(self.plot_output.clone())
            .x_scale(AxisScale::Linear)
            .y_scale(AxisScale::Linear);

        line_plot_panel_in(cx, props).into()
    }
}
