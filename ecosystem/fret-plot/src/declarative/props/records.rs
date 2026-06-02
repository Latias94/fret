//! Public plot panel prop records.

use fret_runtime::Model;
use fret_ui::element::CanvasProps;

use crate::cartesian::AxisScale;
use crate::models::{
    AreaPlotModel, BarsPlotModel, CandlestickPlotModel, ErrorBarsPlotModel, HeatmapPlotModel,
    Histogram2DPlotModel, HistogramPlotModel, LinePlotModel, ShadedPlotModel, StemsPlotModel,
    StepMode,
};
use crate::plot::axis::AxisLabelFormatter;
use crate::state::{PlotOutput, PlotState};
use crate::style::LinePlotStyle;

#[derive(Clone)]
pub struct LinePlotPanelProps {
    pub canvas: CanvasProps,
    pub model: Model<LinePlotModel>,
    pub state: Option<Model<PlotState>>,
    pub output: Option<Model<PlotOutput>>,
    pub style: LinePlotStyle,
    pub x_axis_labels: Option<AxisLabelFormatter>,
    pub y_axis_labels: Option<AxisLabelFormatter>,
    pub y2_axis_labels: Option<AxisLabelFormatter>,
    pub y3_axis_labels: Option<AxisLabelFormatter>,
    pub y4_axis_labels: Option<AxisLabelFormatter>,
    pub x_scale: AxisScale,
    pub y_scale: AxisScale,
    pub step_mode: Option<StepMode>,
}

#[derive(Clone)]
pub struct ErrorBarsPlotPanelProps {
    pub canvas: CanvasProps,
    pub model: Model<ErrorBarsPlotModel>,
    pub state: Option<Model<PlotState>>,
    pub output: Option<Model<PlotOutput>>,
    pub style: LinePlotStyle,
    pub x_axis_labels: Option<AxisLabelFormatter>,
    pub y_axis_labels: Option<AxisLabelFormatter>,
    pub y2_axis_labels: Option<AxisLabelFormatter>,
    pub y3_axis_labels: Option<AxisLabelFormatter>,
    pub y4_axis_labels: Option<AxisLabelFormatter>,
    pub x_scale: AxisScale,
    pub y_scale: AxisScale,
    pub step_mode: Option<StepMode>,
}

#[derive(Clone)]
pub struct HistogramPlotPanelProps {
    pub canvas: CanvasProps,
    pub model: Model<HistogramPlotModel>,
    pub state: Option<Model<PlotState>>,
    pub output: Option<Model<PlotOutput>>,
    pub style: LinePlotStyle,
    pub x_axis_labels: Option<AxisLabelFormatter>,
    pub y_axis_labels: Option<AxisLabelFormatter>,
    pub y2_axis_labels: Option<AxisLabelFormatter>,
    pub y3_axis_labels: Option<AxisLabelFormatter>,
    pub y4_axis_labels: Option<AxisLabelFormatter>,
    pub x_scale: AxisScale,
    pub y_scale: AxisScale,
    pub step_mode: Option<StepMode>,
}

#[derive(Clone)]
pub struct BarsPlotPanelProps {
    pub canvas: CanvasProps,
    pub model: Model<BarsPlotModel>,
    pub state: Option<Model<PlotState>>,
    pub output: Option<Model<PlotOutput>>,
    pub style: LinePlotStyle,
    pub x_axis_labels: Option<AxisLabelFormatter>,
    pub y_axis_labels: Option<AxisLabelFormatter>,
    pub y2_axis_labels: Option<AxisLabelFormatter>,
    pub y3_axis_labels: Option<AxisLabelFormatter>,
    pub y4_axis_labels: Option<AxisLabelFormatter>,
    pub x_scale: AxisScale,
    pub y_scale: AxisScale,
    pub step_mode: Option<StepMode>,
}

#[derive(Clone)]
pub struct CandlestickPlotPanelProps {
    pub canvas: CanvasProps,
    pub model: Model<CandlestickPlotModel>,
    pub state: Option<Model<PlotState>>,
    pub output: Option<Model<PlotOutput>>,
    pub style: LinePlotStyle,
    pub x_axis_labels: Option<AxisLabelFormatter>,
    pub y_axis_labels: Option<AxisLabelFormatter>,
    pub y2_axis_labels: Option<AxisLabelFormatter>,
    pub y3_axis_labels: Option<AxisLabelFormatter>,
    pub y4_axis_labels: Option<AxisLabelFormatter>,
    pub x_scale: AxisScale,
    pub y_scale: AxisScale,
    pub step_mode: Option<StepMode>,
}

#[derive(Clone)]
pub struct HeatmapPlotPanelProps {
    pub canvas: CanvasProps,
    pub model: Model<HeatmapPlotModel>,
    pub state: Option<Model<PlotState>>,
    pub output: Option<Model<PlotOutput>>,
    pub style: LinePlotStyle,
    pub x_axis_labels: Option<AxisLabelFormatter>,
    pub y_axis_labels: Option<AxisLabelFormatter>,
    pub y2_axis_labels: Option<AxisLabelFormatter>,
    pub y3_axis_labels: Option<AxisLabelFormatter>,
    pub y4_axis_labels: Option<AxisLabelFormatter>,
    pub x_scale: AxisScale,
    pub y_scale: AxisScale,
    pub step_mode: Option<StepMode>,
}

#[derive(Clone)]
pub struct Histogram2DPlotPanelProps {
    pub canvas: CanvasProps,
    pub model: Model<Histogram2DPlotModel>,
    pub state: Option<Model<PlotState>>,
    pub output: Option<Model<PlotOutput>>,
    pub style: LinePlotStyle,
    pub x_axis_labels: Option<AxisLabelFormatter>,
    pub y_axis_labels: Option<AxisLabelFormatter>,
    pub y2_axis_labels: Option<AxisLabelFormatter>,
    pub y3_axis_labels: Option<AxisLabelFormatter>,
    pub y4_axis_labels: Option<AxisLabelFormatter>,
    pub x_scale: AxisScale,
    pub y_scale: AxisScale,
    pub step_mode: Option<StepMode>,
}

#[derive(Clone)]
pub struct AreaPlotPanelProps {
    pub canvas: CanvasProps,
    pub model: Model<AreaPlotModel>,
    pub state: Option<Model<PlotState>>,
    pub output: Option<Model<PlotOutput>>,
    pub style: LinePlotStyle,
    pub x_axis_labels: Option<AxisLabelFormatter>,
    pub y_axis_labels: Option<AxisLabelFormatter>,
    pub y2_axis_labels: Option<AxisLabelFormatter>,
    pub y3_axis_labels: Option<AxisLabelFormatter>,
    pub y4_axis_labels: Option<AxisLabelFormatter>,
    pub x_scale: AxisScale,
    pub y_scale: AxisScale,
    pub step_mode: Option<StepMode>,
}

#[derive(Clone)]
pub struct ShadedPlotPanelProps {
    pub canvas: CanvasProps,
    pub model: Model<ShadedPlotModel>,
    pub state: Option<Model<PlotState>>,
    pub output: Option<Model<PlotOutput>>,
    pub style: LinePlotStyle,
    pub x_axis_labels: Option<AxisLabelFormatter>,
    pub y_axis_labels: Option<AxisLabelFormatter>,
    pub y2_axis_labels: Option<AxisLabelFormatter>,
    pub y3_axis_labels: Option<AxisLabelFormatter>,
    pub y4_axis_labels: Option<AxisLabelFormatter>,
    pub x_scale: AxisScale,
    pub y_scale: AxisScale,
    pub step_mode: Option<StepMode>,
}

#[derive(Clone)]
pub struct StemsPlotPanelProps {
    pub canvas: CanvasProps,
    pub model: Model<StemsPlotModel>,
    pub state: Option<Model<PlotState>>,
    pub output: Option<Model<PlotOutput>>,
    pub style: LinePlotStyle,
    pub x_axis_labels: Option<AxisLabelFormatter>,
    pub y_axis_labels: Option<AxisLabelFormatter>,
    pub y2_axis_labels: Option<AxisLabelFormatter>,
    pub y3_axis_labels: Option<AxisLabelFormatter>,
    pub y4_axis_labels: Option<AxisLabelFormatter>,
    pub x_scale: AxisScale,
    pub y_scale: AxisScale,
    pub step_mode: Option<StepMode>,
}
