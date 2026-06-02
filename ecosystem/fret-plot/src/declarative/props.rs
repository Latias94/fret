//! Public plot panel props and builder methods.

use fret_runtime::Model;
use fret_ui::element::CanvasProps;

use crate::cartesian::AxisScale;
use crate::models::{
    AreaPlotModel, BarsPlotModel, CandlestickPlotModel, ErrorBarsPlotModel, HeatmapPlotModel,
    Histogram2DPlotModel, HistogramPlotModel, ShadedPlotModel, StemsPlotModel, StepMode,
};
use crate::plot::axis::AxisLabelFormatter;
use crate::state::{PlotOutput, PlotState};
use crate::style::LinePlotStyle;

mod line;
mod records;

pub use records::{
    AreaPlotPanelProps, BarsPlotPanelProps, CandlestickPlotPanelProps, ErrorBarsPlotPanelProps,
    HeatmapPlotPanelProps, Histogram2DPlotPanelProps, HistogramPlotPanelProps, LinePlotPanelProps,
    ShadedPlotPanelProps, StemsPlotPanelProps,
};

impl ErrorBarsPlotPanelProps {
    pub fn new(model: Model<ErrorBarsPlotModel>) -> Self {
        Self {
            canvas: CanvasProps::default(),
            model,
            state: None,
            output: None,
            style: LinePlotStyle::default(),
            x_axis_labels: None,
            y_axis_labels: None,
            y2_axis_labels: None,
            y3_axis_labels: None,
            y4_axis_labels: None,
            x_scale: AxisScale::Linear,
            y_scale: AxisScale::Linear,
            step_mode: None,
        }
    }

    pub fn output(mut self, output: Model<PlotOutput>) -> Self {
        self.output = Some(output);
        self
    }

    pub fn state(mut self, state: Model<PlotState>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn style(mut self, style: LinePlotStyle) -> Self {
        self.style = style;
        self
    }

    pub fn x_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.x_axis_labels = Some(labels);
        self
    }

    pub fn y_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y_axis_labels = Some(labels);
        self
    }

    pub fn y2_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y2_axis_labels = Some(labels);
        self
    }

    pub fn y3_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y3_axis_labels = Some(labels);
        self
    }

    pub fn y4_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y4_axis_labels = Some(labels);
        self
    }

    pub fn x_scale(mut self, scale: AxisScale) -> Self {
        self.x_scale = scale;
        self
    }

    pub fn y_scale(mut self, scale: AxisScale) -> Self {
        self.y_scale = scale;
        self
    }

    pub fn step_mode(mut self, mode: StepMode) -> Self {
        self.step_mode = Some(mode);
        self
    }
}

impl HistogramPlotPanelProps {
    pub fn new(model: Model<HistogramPlotModel>) -> Self {
        Self {
            canvas: CanvasProps::default(),
            model,
            state: None,
            output: None,
            style: LinePlotStyle::default(),
            x_axis_labels: None,
            y_axis_labels: None,
            y2_axis_labels: None,
            y3_axis_labels: None,
            y4_axis_labels: None,
            x_scale: AxisScale::Linear,
            y_scale: AxisScale::Linear,
            step_mode: None,
        }
    }

    pub fn output(mut self, output: Model<PlotOutput>) -> Self {
        self.output = Some(output);
        self
    }

    pub fn state(mut self, state: Model<PlotState>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn style(mut self, style: LinePlotStyle) -> Self {
        self.style = style;
        self
    }

    pub fn x_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.x_axis_labels = Some(labels);
        self
    }

    pub fn y_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y_axis_labels = Some(labels);
        self
    }

    pub fn y2_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y2_axis_labels = Some(labels);
        self
    }

    pub fn y3_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y3_axis_labels = Some(labels);
        self
    }

    pub fn y4_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y4_axis_labels = Some(labels);
        self
    }

    pub fn x_scale(mut self, scale: AxisScale) -> Self {
        self.x_scale = scale;
        self
    }

    pub fn y_scale(mut self, scale: AxisScale) -> Self {
        self.y_scale = scale;
        self
    }

    pub fn step_mode(mut self, mode: StepMode) -> Self {
        self.step_mode = Some(mode);
        self
    }
}

impl BarsPlotPanelProps {
    pub fn new(model: Model<BarsPlotModel>) -> Self {
        Self {
            canvas: CanvasProps::default(),
            model,
            state: None,
            output: None,
            style: LinePlotStyle::default(),
            x_axis_labels: None,
            y_axis_labels: None,
            y2_axis_labels: None,
            y3_axis_labels: None,
            y4_axis_labels: None,
            x_scale: AxisScale::Linear,
            y_scale: AxisScale::Linear,
            step_mode: None,
        }
    }

    pub fn output(mut self, output: Model<PlotOutput>) -> Self {
        self.output = Some(output);
        self
    }

    pub fn state(mut self, state: Model<PlotState>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn style(mut self, style: LinePlotStyle) -> Self {
        self.style = style;
        self
    }

    pub fn x_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.x_axis_labels = Some(labels);
        self
    }

    pub fn y_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y_axis_labels = Some(labels);
        self
    }

    pub fn y2_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y2_axis_labels = Some(labels);
        self
    }

    pub fn y3_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y3_axis_labels = Some(labels);
        self
    }

    pub fn y4_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y4_axis_labels = Some(labels);
        self
    }

    pub fn x_scale(mut self, scale: AxisScale) -> Self {
        self.x_scale = scale;
        self
    }

    pub fn y_scale(mut self, scale: AxisScale) -> Self {
        self.y_scale = scale;
        self
    }

    pub fn step_mode(mut self, mode: StepMode) -> Self {
        self.step_mode = Some(mode);
        self
    }
}

impl CandlestickPlotPanelProps {
    pub fn new(model: Model<CandlestickPlotModel>) -> Self {
        Self {
            canvas: CanvasProps::default(),
            model,
            state: None,
            output: None,
            style: LinePlotStyle::default(),
            x_axis_labels: None,
            y_axis_labels: None,
            y2_axis_labels: None,
            y3_axis_labels: None,
            y4_axis_labels: None,
            x_scale: AxisScale::Linear,
            y_scale: AxisScale::Linear,
            step_mode: None,
        }
    }

    pub fn output(mut self, output: Model<PlotOutput>) -> Self {
        self.output = Some(output);
        self
    }

    pub fn state(mut self, state: Model<PlotState>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn style(mut self, style: LinePlotStyle) -> Self {
        self.style = style;
        self
    }

    pub fn x_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.x_axis_labels = Some(labels);
        self
    }

    pub fn y_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y_axis_labels = Some(labels);
        self
    }

    pub fn y2_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y2_axis_labels = Some(labels);
        self
    }

    pub fn y3_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y3_axis_labels = Some(labels);
        self
    }

    pub fn y4_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y4_axis_labels = Some(labels);
        self
    }

    pub fn x_scale(mut self, scale: AxisScale) -> Self {
        self.x_scale = scale;
        self
    }

    pub fn y_scale(mut self, scale: AxisScale) -> Self {
        self.y_scale = scale;
        self
    }

    pub fn step_mode(mut self, mode: StepMode) -> Self {
        self.step_mode = Some(mode);
        self
    }
}

impl HeatmapPlotPanelProps {
    pub fn new(model: Model<HeatmapPlotModel>) -> Self {
        let mut style = LinePlotStyle::default();
        style.heatmap_show_colorbar = true;
        Self {
            canvas: CanvasProps::default(),
            model,
            state: None,
            output: None,
            style,
            x_axis_labels: None,
            y_axis_labels: None,
            y2_axis_labels: None,
            y3_axis_labels: None,
            y4_axis_labels: None,
            x_scale: AxisScale::Linear,
            y_scale: AxisScale::Linear,
            step_mode: None,
        }
    }

    pub fn output(mut self, output: Model<PlotOutput>) -> Self {
        self.output = Some(output);
        self
    }

    pub fn state(mut self, state: Model<PlotState>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn style(mut self, style: LinePlotStyle) -> Self {
        self.style = style;
        self
    }

    pub fn x_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.x_axis_labels = Some(labels);
        self
    }

    pub fn y_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y_axis_labels = Some(labels);
        self
    }

    pub fn y2_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y2_axis_labels = Some(labels);
        self
    }

    pub fn y3_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y3_axis_labels = Some(labels);
        self
    }

    pub fn y4_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y4_axis_labels = Some(labels);
        self
    }

    pub fn x_scale(mut self, scale: AxisScale) -> Self {
        self.x_scale = scale;
        self
    }

    pub fn y_scale(mut self, scale: AxisScale) -> Self {
        self.y_scale = scale;
        self
    }

    pub fn step_mode(mut self, mode: StepMode) -> Self {
        self.step_mode = Some(mode);
        self
    }
}

impl Histogram2DPlotPanelProps {
    pub fn new(model: Model<Histogram2DPlotModel>) -> Self {
        let mut style = LinePlotStyle::default();
        style.heatmap_show_colorbar = true;
        Self {
            canvas: CanvasProps::default(),
            model,
            state: None,
            output: None,
            style,
            x_axis_labels: None,
            y_axis_labels: None,
            y2_axis_labels: None,
            y3_axis_labels: None,
            y4_axis_labels: None,
            x_scale: AxisScale::Linear,
            y_scale: AxisScale::Linear,
            step_mode: None,
        }
    }

    pub fn output(mut self, output: Model<PlotOutput>) -> Self {
        self.output = Some(output);
        self
    }

    pub fn state(mut self, state: Model<PlotState>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn style(mut self, style: LinePlotStyle) -> Self {
        self.style = style;
        self
    }

    pub fn x_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.x_axis_labels = Some(labels);
        self
    }

    pub fn y_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y_axis_labels = Some(labels);
        self
    }

    pub fn y2_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y2_axis_labels = Some(labels);
        self
    }

    pub fn y3_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y3_axis_labels = Some(labels);
        self
    }

    pub fn y4_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y4_axis_labels = Some(labels);
        self
    }

    pub fn x_scale(mut self, scale: AxisScale) -> Self {
        self.x_scale = scale;
        self
    }

    pub fn y_scale(mut self, scale: AxisScale) -> Self {
        self.y_scale = scale;
        self
    }

    pub fn step_mode(mut self, mode: StepMode) -> Self {
        self.step_mode = Some(mode);
        self
    }
}

impl AreaPlotPanelProps {
    pub fn new(model: Model<AreaPlotModel>) -> Self {
        Self {
            canvas: CanvasProps::default(),
            model,
            state: None,
            output: None,
            style: LinePlotStyle::default(),
            x_axis_labels: None,
            y_axis_labels: None,
            y2_axis_labels: None,
            y3_axis_labels: None,
            y4_axis_labels: None,
            x_scale: AxisScale::Linear,
            y_scale: AxisScale::Linear,
            step_mode: None,
        }
    }

    pub fn output(mut self, output: Model<PlotOutput>) -> Self {
        self.output = Some(output);
        self
    }

    pub fn state(mut self, state: Model<PlotState>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn style(mut self, style: LinePlotStyle) -> Self {
        self.style = style;
        self
    }

    pub fn x_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.x_axis_labels = Some(labels);
        self
    }

    pub fn y_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y_axis_labels = Some(labels);
        self
    }

    pub fn y2_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y2_axis_labels = Some(labels);
        self
    }

    pub fn y3_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y3_axis_labels = Some(labels);
        self
    }

    pub fn y4_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y4_axis_labels = Some(labels);
        self
    }

    pub fn x_scale(mut self, scale: AxisScale) -> Self {
        self.x_scale = scale;
        self
    }

    pub fn y_scale(mut self, scale: AxisScale) -> Self {
        self.y_scale = scale;
        self
    }

    pub fn step_mode(mut self, mode: StepMode) -> Self {
        self.step_mode = Some(mode);
        self
    }
}

impl ShadedPlotPanelProps {
    pub fn new(model: Model<ShadedPlotModel>) -> Self {
        Self {
            canvas: CanvasProps::default(),
            model,
            state: None,
            output: None,
            style: LinePlotStyle::default(),
            x_axis_labels: None,
            y_axis_labels: None,
            y2_axis_labels: None,
            y3_axis_labels: None,
            y4_axis_labels: None,
            x_scale: AxisScale::Linear,
            y_scale: AxisScale::Linear,
            step_mode: None,
        }
    }

    pub fn output(mut self, output: Model<PlotOutput>) -> Self {
        self.output = Some(output);
        self
    }

    pub fn state(mut self, state: Model<PlotState>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn style(mut self, style: LinePlotStyle) -> Self {
        self.style = style;
        self
    }

    pub fn x_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.x_axis_labels = Some(labels);
        self
    }

    pub fn y_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y_axis_labels = Some(labels);
        self
    }

    pub fn y2_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y2_axis_labels = Some(labels);
        self
    }

    pub fn y3_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y3_axis_labels = Some(labels);
        self
    }

    pub fn y4_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y4_axis_labels = Some(labels);
        self
    }

    pub fn x_scale(mut self, scale: AxisScale) -> Self {
        self.x_scale = scale;
        self
    }

    pub fn y_scale(mut self, scale: AxisScale) -> Self {
        self.y_scale = scale;
        self
    }

    pub fn step_mode(mut self, mode: StepMode) -> Self {
        self.step_mode = Some(mode);
        self
    }
}

impl StemsPlotPanelProps {
    pub fn new(model: Model<StemsPlotModel>) -> Self {
        Self {
            canvas: CanvasProps::default(),
            model,
            state: None,
            output: None,
            style: LinePlotStyle::default(),
            x_axis_labels: None,
            y_axis_labels: None,
            y2_axis_labels: None,
            y3_axis_labels: None,
            y4_axis_labels: None,
            x_scale: AxisScale::Linear,
            y_scale: AxisScale::Linear,
            step_mode: None,
        }
    }

    pub fn output(mut self, output: Model<PlotOutput>) -> Self {
        self.output = Some(output);
        self
    }

    pub fn state(mut self, state: Model<PlotState>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn style(mut self, style: LinePlotStyle) -> Self {
        self.style = style;
        self
    }

    pub fn x_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.x_axis_labels = Some(labels);
        self
    }

    pub fn y_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y_axis_labels = Some(labels);
        self
    }

    pub fn y2_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y2_axis_labels = Some(labels);
        self
    }

    pub fn y3_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y3_axis_labels = Some(labels);
        self
    }

    pub fn y4_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y4_axis_labels = Some(labels);
        self
    }

    pub fn x_scale(mut self, scale: AxisScale) -> Self {
        self.x_scale = scale;
        self
    }

    pub fn y_scale(mut self, scale: AxisScale) -> Self {
        self.y_scale = scale;
        self
    }

    pub fn step_mode(mut self, mode: StepMode) -> Self {
        self.step_mode = Some(mode);
        self
    }
}
