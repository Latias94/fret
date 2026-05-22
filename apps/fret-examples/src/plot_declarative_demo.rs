use anyhow::Context as _;
use fret::{FretApp, advanced::prelude::*, component::prelude::*};
use fret_plot::cartesian::{AxisScale, DataPoint};
use fret_plot::declarative::{LinePlotPanelProps, line_plot_panel_in};
use fret_plot::models::{LinePlotModel, LineSeries};
use fret_plot::series::Series;
use fret_plot::style::LinePlotStyle;

struct PlotDeclarativeView {
    model: Model<LinePlotModel>,
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("plot-declarative-demo")
        .window("plot_declarative_demo", (960.0, 640.0))
        .view::<PlotDeclarativeView>()?
        .run()
        .context("run plot_declarative_demo app")
}

impl View for PlotDeclarativeView {
    fn init(app: &mut KernelApp, _window: AppWindowId) -> Self {
        let n = 2048usize;
        let mut signal_a = Vec::with_capacity(n);
        let mut signal_b = Vec::with_capacity(n);
        let mut signal_c = Vec::with_capacity(n);

        for i in 0..n {
            let t = i as f64 / (n - 1) as f64;
            let x = t * 24.0;
            let theta = t * std::f64::consts::TAU;
            signal_a.push(DataPoint {
                x,
                y: (theta * 6.0).sin() * 0.75 + (theta * 0.8).cos() * 0.15,
            });
            signal_b.push(DataPoint {
                x,
                y: (theta * 3.0).cos() * 0.45 + (theta * 9.0).sin() * 0.08 + 0.45,
            });
            signal_c.push(DataPoint {
                x,
                y: (theta * 1.5).sin() * 0.35 - 0.45,
            });
        }

        let model = LinePlotModel::from_series(vec![
            LineSeries::new("signal A", Series::from_points_sorted(signal_a, true)),
            LineSeries::new("signal B", Series::from_points_sorted(signal_b, true)),
            LineSeries::new("signal C", Series::from_points_sorted(signal_c, true)),
        ]);

        Self {
            model: app.models_mut().insert(model),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let style = LinePlotStyle {
            padding: Px(20.0),
            stroke_width: Px(2.0),
            ..Default::default()
        };
        let props = LinePlotPanelProps::new(self.model.clone())
            .style(style)
            .x_scale(AxisScale::Linear)
            .y_scale(AxisScale::Linear);

        line_plot_panel_in(cx, props).into()
    }
}
