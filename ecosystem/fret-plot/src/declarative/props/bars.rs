//! Bars plot panel prop builder owner.

use fret_core::Px;
use fret_runtime::Model;
use fret_ui::element::{CanvasProps, LayoutStyle, Length};

use crate::cartesian::AxisScale;
use crate::models::{BarsPlotModel, StepMode};
use crate::plot::axis::AxisLabelFormatter;
use crate::state::{PlotOutput, PlotState};
use crate::style::LinePlotStyle;

use super::BarsPlotPanelProps;

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

    pub fn canvas(mut self, canvas: CanvasProps) -> Self {
        self.canvas = canvas;
        self
    }

    pub fn layout(mut self, layout: LayoutStyle) -> Self {
        self.canvas.layout = layout;
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.canvas.layout.size.width = width;
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.canvas.layout.size.height = height;
        self
    }

    pub fn size(mut self, width: Length, height: Length) -> Self {
        self.canvas.layout.size.width = width;
        self.canvas.layout.size.height = height;
        self
    }

    pub fn width_px(self, width: Px) -> Self {
        self.width(Length::Px(width))
    }

    pub fn height_px(self, height: Px) -> Self {
        self.height(Length::Px(height))
    }

    pub fn size_px(self, width: Px, height: Px) -> Self {
        self.size(Length::Px(width), Length::Px(height))
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
