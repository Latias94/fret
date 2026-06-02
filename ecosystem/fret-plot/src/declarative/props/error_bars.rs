//! Error-bars plot panel prop builder owner.

use fret_runtime::Model;
use fret_ui::element::CanvasProps;

use crate::cartesian::AxisScale;
use crate::models::{ErrorBarsPlotModel, StepMode};
use crate::plot::axis::AxisLabelFormatter;
use crate::state::{PlotOutput, PlotState};
use crate::style::LinePlotStyle;

use super::ErrorBarsPlotPanelProps;

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
