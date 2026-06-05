use fret::app::prelude::*;
use fret::imui::prelude::*;
use fret::style::Space;
use fret_core::{Color, Px};
use fret_plot::cartesian::{AxisScale, DataPoint};
use fret_plot::declarative::LinePlotPanelProps;
use fret_plot::models::{LinePlotModel, LineSeries};
use fret_plot::series::Series;
use fret_plot::state::{PlotOutput, PlotState};
use fret_plot::style::{LinePlotStyle, SeriesTooltipMode};
use fret_runtime::Model;
use fret_ui::element::{CanvasProps, Length};

const TEST_ID_ROOT: &str = "cookbook.imui_plot_basics.root";
const TEST_ID_PANEL: &str = "cookbook.imui_plot_basics.panel";
const TEST_ID_SUMMARY: &str = "cookbook.imui_plot_basics.summary";
const TEST_ID_VIEW: &str = "cookbook.imui_plot_basics.view";
const TEST_ID_CURSOR: &str = "cookbook.imui_plot_basics.cursor";

struct ImUiPlotBasicsView {
    plot: Model<LinePlotModel>,
    plot_state: Model<PlotState>,
    plot_output: Model<PlotOutput>,
}

impl View for ImUiPlotBasicsView {
    fn init(app: &mut App, _window: WindowId) -> Self {
        let n = 1024usize;
        let mut temperature = Vec::with_capacity(n);
        let mut pressure = Vec::with_capacity(n);

        for i in 0..n {
            let t = i as f64 / (n - 1) as f64;
            let x = t * 24.0;
            let theta = t * std::f64::consts::TAU;
            temperature.push(DataPoint {
                x,
                y: (theta * 4.0).sin() * 0.65 + (theta * 0.75).cos() * 0.18,
            });
            pressure.push(DataPoint {
                x,
                y: (theta * 2.0).cos() * 0.35 + (theta * 6.5).sin() * 0.08 + 0.35,
            });
        }

        let plot = LinePlotModel::from_series(vec![
            LineSeries::new("temperature", Series::from_points_sorted(temperature, true))
                .color(Color::from_srgb_hex_rgb(0x3b_82_f6)),
            LineSeries::new("pressure", Series::from_points_sorted(pressure, true))
                .color(Color::from_srgb_hex_rgb(0x22_c5_5e)),
        ]);

        Self {
            plot: app.models_mut().insert(plot),
            plot_state: app.models_mut().insert(PlotState::default()),
            plot_output: app.models_mut().insert(PlotOutput::default()),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let output = self.plot_output.layout(cx).value_or_default();
        let view = output.snapshot.view_bounds;
        let view_readout = format!(
            "View: x=[{:.1}, {:.1}] y=[{:.2}, {:.2}] rev={}",
            view.x_min, view.x_max, view.y_min, view.y_max, output.revision
        );
        let cursor_readout = output
            .snapshot
            .cursor
            .map(|cursor| format!("Cursor: x={:.2}, y={:.3}", cursor.x, cursor.y))
            .unwrap_or_else(|| String::from("Cursor: outside plot"));

        ui::v_flex(|cx| {
            let plot_panel = cx
                .column(fret_ui::element::ColumnProps::default(), |cx| {
                    imui_raw(cx, |ui| {
                        ui.text("Plot adapter");

                        let mut canvas = CanvasProps::default();
                        canvas.layout.size.width = Length::Fill;
                        canvas.layout.size.height = Length::Px(Px(280.0));

                        let mut props = LinePlotPanelProps::new(self.plot.clone())
                            .style(LinePlotStyle {
                                padding: Px(18.0),
                                stroke_width: Px(2.0),
                                series_tooltip: SeriesTooltipMode::NearestAtCursor,
                                ..Default::default()
                            })
                            .x_scale(AxisScale::Linear)
                            .y_scale(AxisScale::Linear)
                            .state(self.plot_state.clone())
                            .output(self.plot_output.clone());
                        props.canvas = canvas;

                        fret_plot::imui::line_plot_panel(ui, props);
                    })
                })
                .test_id(TEST_ID_PANEL);

            ui::children![
                cx;
                shadcn::Label::new("Immediate-mode plot adapter"),
                cx.text("This lesson hosts fret_plot::imui from the root fret::imui lane.")
                    .test_id(TEST_ID_SUMMARY),
                plot_panel,
                cx.text(view_readout).test_id(TEST_ID_VIEW),
                cx.text(cursor_readout).test_id(TEST_ID_CURSOR),
            ]
        })
        .size_full()
        .gap(Space::N4)
        .test_id(TEST_ID_ROOT)
        .into_element_in(cx)
        .into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-imui-plot-basics")
        .window("cookbook-imui-plot-basics", (760.0, 520.0))
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<ImUiPlotBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
