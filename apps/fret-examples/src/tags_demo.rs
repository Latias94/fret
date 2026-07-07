use fret::app::prelude::*;
use fret_plot::LinePlotPanelBinding;
use fret_plot::cartesian::DataPoint;
use fret_plot::declarative::line_plot_panel_in;
use fret_plot::models::{LinePlotModel, LineSeries};
use fret_plot::series::Series;
use fret_plot::state::{PlotOverlays, PlotState};
use fret_plot::style::{LinePlotStyle, SeriesTooltipMode};

mod driver;

pub use driver::{build_app, build_fn_driver, build_runner_config, run};

struct TagsDemoView {
    plot: LinePlotPanelBinding,
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

        let model = LinePlotModel::from_series(vec![LineSeries::new(
            "signal",
            Series::from_points_sorted(series0, true),
        )]);

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
            plot: LinePlotPanelBinding::new_with_state(app, model, state),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let style = LinePlotStyle {
            series_tooltip: SeriesTooltipMode::NearestAtCursor,
            ..Default::default()
        };
        let props = self
            .plot
            .panel_props()
            .style(style)
            .x_scale(fret_plot::cartesian::AxisScale::Linear)
            .y_scale(fret_plot::cartesian::AxisScale::Linear);

        line_plot_panel_in(cx, props).into()
    }
}
