use anyhow::Context as _;
use fret::advanced::raw::Model;
use fret::app::prelude::*;
use fret_bootstrap::ui_app_driver;
use fret_plot::cartesian::DataPoint;
use fret_plot::declarative::{LinePlotPanelProps, line_plot_panel_in};
use fret_plot::models::{LinePlotModel, LineSeries};
use fret_plot::series::Series;
use fret_plot::state::{PlotOutput, PlotOverlays, PlotState};
use fret_plot::style::{LinePlotStyle, SeriesTooltipMode};
use fret_runtime::PlatformCapabilities;

struct TagsDemoView {
    model: Model<LinePlotModel>,
    plot_state: Model<PlotState>,
    plot_output: Model<PlotOutput>,
}

pub fn build_app() -> fret::app::App {
    let mut app = fret::app::App::new();
    app.set_global(PlatformCapabilities::default());
    app
}

pub fn build_runner_config() -> fret_launch::WinitRunnerConfig {
    fret_launch::WinitRunnerConfig {
        main_window_title: "fret-demo tags_demo".to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_fn_driver() -> impl fret_launch::WinitAppDriver {
    ui_app_driver::UiAppDriver::new(
        "tags-demo",
        fret::advanced::view::view_init_window::<TagsDemoView>,
        fret::advanced::view::view_view::<TagsDemoView>,
    )
    .on_preferences(
        ui_app_driver::default_on_preferences::<fret::advanced::view::ViewWindowState<TagsDemoView>>,
    )
    .into_fn_driver()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    FretApp::new("tags-demo")
        .window("tags_demo", (960.0, 640.0))
        .view::<TagsDemoView>()?
        .run()
        .context("run tags_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}

impl View for TagsDemoView {
    fn init(app: &mut App, _window: WindowId) -> Self {
        let n = 2048usize;
        let mut series0 = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f64 / (n - 1) as f64;
            let x = t * 100.0;
            let y = (t * std::f64::consts::TAU * 3.0).sin();
            if !x.is_finite() || !y.is_finite() {
                continue;
            }
            series0.push(DataPoint { x, y });
        }

        let plot = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "signal",
                Series::from_points_sorted(series0, true),
            )]));

        let mut state = PlotState::default();
        state.overlays = PlotOverlays {
            tags_x: vec![
                fret_plot::state::TagX::new(25.0).label("T1"),
                fret_plot::state::TagX::new(75.0).label("T2"),
            ],
            tags_y: vec![
                fret_plot::state::TagY::new(0.5, fret_plot::models::YAxis::Left).label("limit"),
            ],
            text: vec![fret_plot::state::PlotText::new(
                50.0,
                -0.75,
                fret_plot::models::YAxis::Left,
                "PlotText at (50, -0.75)",
            )],
            ..Default::default()
        };

        Self {
            model: plot,
            plot_state: app.models_mut().insert(state),
            plot_output: app.models_mut().insert(PlotOutput::default()),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let style = LinePlotStyle {
            series_tooltip: SeriesTooltipMode::NearestAtCursor,
            ..Default::default()
        };
        let props = LinePlotPanelProps::new(self.model.clone())
            .style(style)
            .state(self.plot_state.clone())
            .output(self.plot_output.clone())
            .x_scale(fret_plot::cartesian::AxisScale::Linear)
            .y_scale(fret_plot::cartesian::AxisScale::Linear);

        line_plot_panel_in(cx, props).into()
    }
}
