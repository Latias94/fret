use std::cell::{Cell, RefCell};
use std::rc::Rc;

use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, FontWeight, MouseButton, Paint, PathCommand,
    PathStyle, Point, Px, Rect, Size, StrokeStyle, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::canvas::{CanvasPainter, CanvasTextConstraints};
use fret_ui::element::{AnyElement, CanvasProps, Length, ManagedSurfaceProps};
use fret_ui::{ElementContext, ElementContextAccess, UiHost};

use crate::cartesian::{AxisScale, DataPoint, DataRect, PlotTransform, polyline_commands};
use crate::input_map::{ModifierKey, ModifiersMask, PlotInputMap};
use crate::models::{
    AreaPlotModel, BarsPlotModel, CandlestickPlotModel, ErrorBarsPlotModel, HeatmapPlotModel,
    Histogram2DPlotModel, HistogramPlotModel, LinePlotModel, ShadedPlotModel, StemsPlotModel,
    StepMode, YAxis,
};
use crate::plot::axis::{
    AxisLabelFormatter, AxisTicks, axis_ticks_scaled, log10_tick_label_or_empty,
};
use crate::plot::histogram::histogram_bins;
use crate::plot::readout::{
    PlotCursorReadoutArgs, PlotCursorReadoutRow, PlotCursorReadoutSeries, plot_cursor_readout,
};
use crate::plot::view::{
    clamp_view_to_data_scaled, clamp_zoom_factors, data_rect_from_plot_points_scaled,
    local_from_absolute, sanitize_data_rect_scaled, zoom_view_at_px_scaled,
};
use crate::series::{Series, SeriesId};
use crate::state::{
    PlotDragOutput, PlotDragPhase, PlotImageLayer, PlotOutput, PlotOutputSnapshot, PlotOverlays,
    PlotState,
};
use crate::style::{LinePlotStyle, MouseReadoutMode, OverlayAnchor, ReadoutSeriesPolicy};

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

#[derive(Clone)]
struct PlotPanelProps {
    canvas: CanvasProps,
    model: PlotPanelModel,
    state: Option<Model<PlotState>>,
    output: Option<Model<PlotOutput>>,
    style: LinePlotStyle,
    x_axis_labels: Option<AxisLabelFormatter>,
    y_axis_labels: Option<AxisLabelFormatter>,
    y2_axis_labels: Option<AxisLabelFormatter>,
    y3_axis_labels: Option<AxisLabelFormatter>,
    y4_axis_labels: Option<AxisLabelFormatter>,
    x_scale: AxisScale,
    y_scale: AxisScale,
    step_mode: Option<StepMode>,
}

#[derive(Debug, Clone)]
struct PlotPanelModel {
    data_bounds: DataRect,
    data_bounds_y2: Option<DataRect>,
    data_bounds_y3: Option<DataRect>,
    data_bounds_y4: Option<DataRect>,
    heatmap: Option<PlotPanelHeatmap>,
    series: Vec<PlotPanelSeries>,
}

#[derive(Debug, Clone)]
struct PlotPanelHeatmap {
    data_bounds: DataRect,
    cols: usize,
    rows: usize,
    values: std::sync::Arc<[f32]>,
    value_min: f32,
    value_max: f32,
}

#[derive(Debug, Clone)]
struct PlotPanelSeries {
    id: SeriesId,
    label: std::sync::Arc<str>,
    data: Series,
    lower_data: Option<Series>,
    error_bars: Option<PlotPanelErrorBars>,
    histogram: Option<PlotPanelHistogram>,
    bars: Option<PlotPanelBars>,
    candlestick: Option<PlotPanelCandlestick>,
    y_axis: YAxis,
    stroke_color: Option<Color>,
    stroke_width: Option<Px>,
    fill: Option<PlotPanelFill>,
    stem_baseline: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
struct PlotPanelFill {
    color: Option<Color>,
    alpha: f32,
    baseline: f32,
}

#[derive(Debug, Clone)]
struct PlotPanelErrorBars {
    x_errors: Option<std::sync::Arc<[crate::models::ErrorBar]>>,
    y_errors: Option<std::sync::Arc<[crate::models::ErrorBar]>>,
    cap_size: Px,
    show_caps: bool,
    marker_radius: Px,
    show_markers: bool,
    marker_shape: crate::models::MarkerShape,
}

#[derive(Debug, Clone)]
struct PlotPanelHistogram {
    bin_width: f64,
    bar_gap_fraction: f32,
}

#[derive(Debug, Clone)]
struct PlotPanelBars {
    bar_width: f64,
    baseline: f64,
    baselines: Option<std::sync::Arc<[f64]>>,
}

#[derive(Debug, Clone)]
struct PlotPanelCandlestick {
    points: std::sync::Arc<[crate::models::OhlcPoint]>,
    candle_width: f64,
    up_fill: Option<Color>,
    down_fill: Option<Color>,
    wick_color: Option<Color>,
}

impl From<&HeatmapPlotModel> for PlotPanelModel {
    fn from(model: &HeatmapPlotModel) -> Self {
        Self {
            data_bounds: model.data_bounds,
            data_bounds_y2: None,
            data_bounds_y3: None,
            data_bounds_y4: None,
            heatmap: Some(PlotPanelHeatmap {
                data_bounds: model.data_bounds,
                cols: model.cols,
                rows: model.rows,
                values: model.values.clone(),
                value_min: model.value_min,
                value_max: model.value_max,
            }),
            series: Vec::new(),
        }
    }
}

impl From<&Histogram2DPlotModel> for PlotPanelModel {
    fn from(model: &Histogram2DPlotModel) -> Self {
        Self {
            data_bounds: model.data_bounds,
            data_bounds_y2: None,
            data_bounds_y3: None,
            data_bounds_y4: None,
            heatmap: Some(PlotPanelHeatmap {
                data_bounds: model.data_bounds,
                cols: model.cols,
                rows: model.rows,
                values: model.values.clone(),
                value_min: model.value_min,
                value_max: model.value_max,
            }),
            series: Vec::new(),
        }
    }
}

impl From<&ErrorBarsPlotModel> for PlotPanelModel {
    fn from(model: &ErrorBarsPlotModel) -> Self {
        Self {
            data_bounds: model.data_bounds,
            data_bounds_y2: model.data_bounds_y2,
            data_bounds_y3: model.data_bounds_y3,
            data_bounds_y4: model.data_bounds_y4,
            heatmap: None,
            series: model
                .series
                .iter()
                .map(|series| PlotPanelSeries {
                    id: series.id,
                    label: series.label.clone(),
                    data: series.data.clone(),
                    lower_data: None,
                    error_bars: Some(PlotPanelErrorBars {
                        x_errors: series.x_errors.clone(),
                        y_errors: series.y_errors.clone(),
                        cap_size: series.cap_size,
                        show_caps: series.show_caps,
                        marker_radius: series.marker_radius,
                        show_markers: series.show_markers,
                        marker_shape: series.marker_shape,
                    }),
                    histogram: None,
                    bars: None,
                    candlestick: None,
                    y_axis: series.y_axis,
                    stroke_color: series.stroke_color,
                    stroke_width: series.stroke_width,
                    fill: None,
                    stem_baseline: None,
                })
                .collect(),
        }
    }
}

impl From<&BarsPlotModel> for PlotPanelModel {
    fn from(model: &BarsPlotModel) -> Self {
        Self {
            data_bounds: model.data_bounds,
            data_bounds_y2: model.data_bounds_y2,
            data_bounds_y3: model.data_bounds_y3,
            data_bounds_y4: model.data_bounds_y4,
            heatmap: None,
            series: model
                .series
                .iter()
                .map(|series| PlotPanelSeries {
                    id: series.id,
                    label: series.label.clone(),
                    data: series.data.clone(),
                    lower_data: None,
                    error_bars: None,
                    histogram: None,
                    bars: Some(PlotPanelBars {
                        bar_width: f64::from(series.bar_width),
                        baseline: f64::from(series.baseline),
                        baselines: series.baseline_by_index.clone(),
                    }),
                    candlestick: None,
                    y_axis: series.y_axis,
                    stroke_color: series.fill_color,
                    stroke_width: None,
                    fill: None,
                    stem_baseline: None,
                })
                .collect(),
        }
    }
}

impl From<&HistogramPlotModel> for PlotPanelModel {
    fn from(model: &HistogramPlotModel) -> Self {
        Self {
            data_bounds: model.data_bounds,
            data_bounds_y2: model.data_bounds_y2,
            data_bounds_y3: model.data_bounds_y3,
            data_bounds_y4: model.data_bounds_y4,
            heatmap: None,
            series: model
                .series
                .iter()
                .map(|series| PlotPanelSeries {
                    id: series.id,
                    label: series.label.clone(),
                    data: histogram_series_from_bins(series),
                    lower_data: None,
                    error_bars: None,
                    histogram: Some(PlotPanelHistogram {
                        bin_width: histogram_series_bin_width(series),
                        bar_gap_fraction: series.bar_gap_fraction,
                    }),
                    bars: None,
                    candlestick: None,
                    y_axis: series.y_axis,
                    stroke_color: series.fill_color,
                    stroke_width: None,
                    fill: None,
                    stem_baseline: None,
                })
                .collect(),
        }
    }
}

impl From<&CandlestickPlotModel> for PlotPanelModel {
    fn from(model: &CandlestickPlotModel) -> Self {
        Self {
            data_bounds: model.data_bounds,
            data_bounds_y2: model.data_bounds_y2,
            data_bounds_y3: model.data_bounds_y3,
            data_bounds_y4: model.data_bounds_y4,
            heatmap: None,
            series: model
                .series
                .iter()
                .map(|series| PlotPanelSeries {
                    id: series.id,
                    label: series.label.clone(),
                    data: series.close_series.clone(),
                    lower_data: None,
                    error_bars: None,
                    histogram: None,
                    bars: None,
                    candlestick: Some(PlotPanelCandlestick {
                        points: series.points.clone(),
                        candle_width: f64::from(series.candle_width),
                        up_fill: series.up_fill,
                        down_fill: series.down_fill,
                        wick_color: series.wick_color,
                    }),
                    y_axis: series.y_axis,
                    stroke_color: series.wick_color,
                    stroke_width: series.stroke_width,
                    fill: None,
                    stem_baseline: None,
                })
                .collect(),
        }
    }
}

impl From<&LinePlotModel> for PlotPanelModel {
    fn from(model: &LinePlotModel) -> Self {
        Self {
            data_bounds: model.data_bounds,
            data_bounds_y2: model.data_bounds_y2,
            data_bounds_y3: model.data_bounds_y3,
            data_bounds_y4: model.data_bounds_y4,
            heatmap: None,
            series: model
                .series
                .iter()
                .map(|series| PlotPanelSeries {
                    id: series.id,
                    label: series.label.clone(),
                    data: series.data.clone(),
                    lower_data: None,
                    error_bars: None,
                    histogram: None,
                    bars: None,
                    candlestick: None,
                    y_axis: series.y_axis,
                    stroke_color: series.stroke_color,
                    stroke_width: series.stroke_width,
                    fill: None,
                    stem_baseline: None,
                })
                .collect(),
        }
    }
}

impl From<&AreaPlotModel> for PlotPanelModel {
    fn from(model: &AreaPlotModel) -> Self {
        Self {
            data_bounds: model.data_bounds,
            data_bounds_y2: model.data_bounds_y2,
            data_bounds_y3: model.data_bounds_y3,
            data_bounds_y4: model.data_bounds_y4,
            heatmap: None,
            series: model
                .series
                .iter()
                .map(|series| PlotPanelSeries {
                    id: series.id,
                    label: series.label.clone(),
                    data: series.data.clone(),
                    lower_data: None,
                    error_bars: None,
                    histogram: None,
                    bars: None,
                    candlestick: None,
                    y_axis: series.y_axis,
                    stroke_color: series.stroke_color,
                    stroke_width: series.stroke_width,
                    fill: Some(PlotPanelFill {
                        color: series.fill_color,
                        alpha: series.fill_alpha,
                        baseline: series.baseline,
                    }),
                    stem_baseline: None,
                })
                .collect(),
        }
    }
}

impl From<&ShadedPlotModel> for PlotPanelModel {
    fn from(model: &ShadedPlotModel) -> Self {
        Self {
            data_bounds: model.data_bounds,
            data_bounds_y2: model.data_bounds_y2,
            data_bounds_y3: model.data_bounds_y3,
            data_bounds_y4: model.data_bounds_y4,
            heatmap: None,
            series: model
                .series
                .iter()
                .map(|series| PlotPanelSeries {
                    id: series.id,
                    label: series.label.clone(),
                    data: series.upper.clone(),
                    lower_data: Some(series.lower.clone()),
                    error_bars: None,
                    histogram: None,
                    bars: None,
                    candlestick: None,
                    y_axis: series.y_axis,
                    stroke_color: series.stroke_color.or(series.fill_color),
                    stroke_width: series.stroke_width,
                    fill: Some(PlotPanelFill {
                        color: series.fill_color,
                        alpha: series.fill_alpha,
                        baseline: 0.0,
                    }),
                    stem_baseline: None,
                })
                .collect(),
        }
    }
}

impl From<&StemsPlotModel> for PlotPanelModel {
    fn from(model: &StemsPlotModel) -> Self {
        Self {
            data_bounds: model.data_bounds,
            data_bounds_y2: model.data_bounds_y2,
            data_bounds_y3: model.data_bounds_y3,
            data_bounds_y4: model.data_bounds_y4,
            heatmap: None,
            series: model
                .series
                .iter()
                .map(|series| PlotPanelSeries {
                    id: series.id,
                    label: series.label.clone(),
                    data: series.data.clone(),
                    lower_data: None,
                    error_bars: None,
                    histogram: None,
                    bars: None,
                    candlestick: None,
                    y_axis: series.y_axis,
                    stroke_color: series.stroke_color,
                    stroke_width: series.stroke_width,
                    fill: None,
                    stem_baseline: Some(series.baseline),
                })
                .collect(),
        }
    }
}

impl LinePlotPanelProps {
    pub fn new(model: Model<LinePlotModel>) -> Self {
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

#[track_caller]
pub fn error_bars_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: ErrorBarsPlotPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);
    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("error bars plot model should exist");
    plot_panel(
        cx,
        PlotPanelProps {
            canvas: props.canvas,
            model: PlotPanelModel::from(&model),
            state: props.state,
            output: props.output,
            style: props.style,
            x_axis_labels: props.x_axis_labels,
            y_axis_labels: props.y_axis_labels,
            y2_axis_labels: props.y2_axis_labels,
            y3_axis_labels: props.y3_axis_labels,
            y4_axis_labels: props.y4_axis_labels,
            x_scale: props.x_scale,
            y_scale: props.y_scale,
            step_mode: props.step_mode,
        },
    )
}

#[track_caller]
pub fn histogram_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: HistogramPlotPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);
    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("histogram plot model should exist");
    plot_panel(
        cx,
        PlotPanelProps {
            canvas: props.canvas,
            model: PlotPanelModel::from(&model),
            state: props.state,
            output: props.output,
            style: props.style,
            x_axis_labels: props.x_axis_labels,
            y_axis_labels: props.y_axis_labels,
            y2_axis_labels: props.y2_axis_labels,
            y3_axis_labels: props.y3_axis_labels,
            y4_axis_labels: props.y4_axis_labels,
            x_scale: props.x_scale,
            y_scale: props.y_scale,
            step_mode: props.step_mode,
        },
    )
}

#[track_caller]
pub fn bars_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: BarsPlotPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);
    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("bars plot model should exist");
    plot_panel(
        cx,
        PlotPanelProps {
            canvas: props.canvas,
            model: PlotPanelModel::from(&model),
            state: props.state,
            output: props.output,
            style: props.style,
            x_axis_labels: props.x_axis_labels,
            y_axis_labels: props.y_axis_labels,
            y2_axis_labels: props.y2_axis_labels,
            y3_axis_labels: props.y3_axis_labels,
            y4_axis_labels: props.y4_axis_labels,
            x_scale: props.x_scale,
            y_scale: props.y_scale,
            step_mode: props.step_mode,
        },
    )
}

#[track_caller]
pub fn candlestick_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: CandlestickPlotPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);
    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("candlestick plot model should exist");
    plot_panel(
        cx,
        PlotPanelProps {
            canvas: props.canvas,
            model: PlotPanelModel::from(&model),
            state: props.state,
            output: props.output,
            style: props.style,
            x_axis_labels: props.x_axis_labels,
            y_axis_labels: props.y_axis_labels,
            y2_axis_labels: props.y2_axis_labels,
            y3_axis_labels: props.y3_axis_labels,
            y4_axis_labels: props.y4_axis_labels,
            x_scale: props.x_scale,
            y_scale: props.y_scale,
            step_mode: props.step_mode,
        },
    )
}

#[track_caller]
pub fn heatmap_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: HeatmapPlotPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);
    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("heatmap plot model should exist");
    plot_panel(
        cx,
        PlotPanelProps {
            canvas: props.canvas,
            model: PlotPanelModel::from(&model),
            state: props.state,
            output: props.output,
            style: props.style,
            x_axis_labels: props.x_axis_labels,
            y_axis_labels: props.y_axis_labels,
            y2_axis_labels: props.y2_axis_labels,
            y3_axis_labels: props.y3_axis_labels,
            y4_axis_labels: props.y4_axis_labels,
            x_scale: props.x_scale,
            y_scale: props.y_scale,
            step_mode: props.step_mode,
        },
    )
}

#[track_caller]
pub fn histogram2d_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: Histogram2DPlotPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);
    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("histogram2d plot model should exist");
    plot_panel(
        cx,
        PlotPanelProps {
            canvas: props.canvas,
            model: PlotPanelModel::from(&model),
            state: props.state,
            output: props.output,
            style: props.style,
            x_axis_labels: props.x_axis_labels,
            y_axis_labels: props.y_axis_labels,
            y2_axis_labels: props.y2_axis_labels,
            y3_axis_labels: props.y3_axis_labels,
            y4_axis_labels: props.y4_axis_labels,
            x_scale: props.x_scale,
            y_scale: props.y_scale,
            step_mode: props.step_mode,
        },
    )
}

#[track_caller]
pub fn line_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: LinePlotPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);
    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("line plot model should exist");
    plot_panel(
        cx,
        PlotPanelProps {
            canvas: props.canvas,
            model: PlotPanelModel::from(&model),
            state: props.state,
            output: props.output,
            style: props.style,
            x_axis_labels: props.x_axis_labels,
            y_axis_labels: props.y_axis_labels,
            y2_axis_labels: props.y2_axis_labels,
            y3_axis_labels: props.y3_axis_labels,
            y4_axis_labels: props.y4_axis_labels,
            x_scale: props.x_scale,
            y_scale: props.y_scale,
            step_mode: props.step_mode,
        },
    )
}

#[track_caller]
pub fn area_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: AreaPlotPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);
    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("area plot model should exist");
    plot_panel(
        cx,
        PlotPanelProps {
            canvas: props.canvas,
            model: PlotPanelModel::from(&model),
            state: props.state,
            output: props.output,
            style: props.style,
            x_axis_labels: props.x_axis_labels,
            y_axis_labels: props.y_axis_labels,
            y2_axis_labels: props.y2_axis_labels,
            y3_axis_labels: props.y3_axis_labels,
            y4_axis_labels: props.y4_axis_labels,
            x_scale: props.x_scale,
            y_scale: props.y_scale,
            step_mode: props.step_mode,
        },
    )
}

#[track_caller]
pub fn shaded_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: ShadedPlotPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);
    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("shaded plot model should exist");
    plot_panel(
        cx,
        PlotPanelProps {
            canvas: props.canvas,
            model: PlotPanelModel::from(&model),
            state: props.state,
            output: props.output,
            style: props.style,
            x_axis_labels: props.x_axis_labels,
            y_axis_labels: props.y_axis_labels,
            y2_axis_labels: props.y2_axis_labels,
            y3_axis_labels: props.y3_axis_labels,
            y4_axis_labels: props.y4_axis_labels,
            x_scale: props.x_scale,
            y_scale: props.y_scale,
            step_mode: props.step_mode,
        },
    )
}

#[track_caller]
pub fn stems_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: StemsPlotPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);
    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("stems plot model should exist");
    plot_panel(
        cx,
        PlotPanelProps {
            canvas: props.canvas,
            model: PlotPanelModel::from(&model),
            state: props.state,
            output: props.output,
            style: props.style,
            x_axis_labels: props.x_axis_labels,
            y_axis_labels: props.y_axis_labels,
            y2_axis_labels: props.y2_axis_labels,
            y3_axis_labels: props.y3_axis_labels,
            y4_axis_labels: props.y4_axis_labels,
            x_scale: props.x_scale,
            y_scale: props.y_scale,
            step_mode: props.step_mode,
        },
    )
}

#[track_caller]
fn plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    mut props: PlotPanelProps,
) -> AnyElement {
    props.canvas.layout.size.width = Length::Fill;
    props.canvas.layout.size.height = Length::Fill;
    if let Some(state) = &props.state {
        cx.observe_model(state, fret_ui::Invalidation::Paint);
    }
    let model = props.model;
    let step_mode = props.step_mode;
    let output_snapshot = props.output.as_ref().and_then(|output| {
        cx.read_model_ref(output, fret_ui::Invalidation::Paint, |output| {
            output.snapshot
        })
        .ok()
    });
    let output_snapshot = Rc::new(Cell::new(output_snapshot));
    let linked_cursor_x = Rc::new(Cell::new(None::<f64>));
    let pinned_series = Rc::new(Cell::new(None::<SeriesId>));
    let legend_hover = Rc::new(Cell::new(None::<SeriesId>));
    let hidden_series = Rc::new(RefCell::new(Vec::<SeriesId>::new()));
    let pan_session = Rc::new(RefCell::new(None::<LinePlotPanSession>));
    let box_zoom_session = Rc::new(RefCell::new(None::<LinePlotBoxZoomSession>));
    let query_drag_session = Rc::new(RefCell::new(None::<LinePlotQueryDragSession>));
    let drag_session = Rc::new(RefCell::new(None::<LinePlotDragSession>));
    let active_selection = Rc::new(Cell::new(None::<LinePlotSelectionOverlay>));
    let query_selection = Rc::new(Cell::new(None::<DataRect>));
    let overlays = Rc::new(RefCell::new(PlotOverlays::default()));
    let style = props.style;
    let x_axis_labels = props.x_axis_labels.unwrap_or_default();
    let y_axis_labels = props.y_axis_labels.unwrap_or_default();
    let y2_axis_labels = props.y2_axis_labels.unwrap_or_default();
    let y3_axis_labels = props.y3_axis_labels.unwrap_or_default();
    let y4_axis_labels = props.y4_axis_labels.unwrap_or_default();
    let x_scale = props.x_scale;
    let y_scale = props.y_scale;
    let view_bounds = Rc::new(Cell::new(line_plot_view_bounds_from_state(
        &model, None, style, x_scale, y_scale,
    )));
    let state = props.state.clone();
    let event_state = props.state.clone();
    let output = props.output.clone();
    let event_model = model.clone();
    let event_output = output.clone();
    let event_output_snapshot = output_snapshot.clone();
    let event_style = style;
    let event_x_scale = x_scale;
    let event_y_scale = y_scale;
    let event_legend_hover = legend_hover.clone();
    let event_view_bounds = view_bounds.clone();
    let event_pan_session = pan_session.clone();
    let event_box_zoom_session = box_zoom_session.clone();
    let event_query_drag_session = query_drag_session.clone();
    let event_drag_session = drag_session.clone();
    let event_active_selection = active_selection.clone();

    let mut surface = ManagedSurfaceProps::default();
    surface.layout = props.canvas.layout;
    let canvas = props.canvas;
    let element = cx.managed_surface(
        surface,
        |cx| {
            cx.layout_unplaced_children(cx.bounds());
            cx.set_hit_test_rects([cx.bounds()]);
        },
        {
            let linked_cursor_x = linked_cursor_x.clone();
            let pinned_series = pinned_series.clone();
            let view_bounds = view_bounds.clone();
            let hidden_series = hidden_series.clone();
            let query_selection = query_selection.clone();
            let overlays = overlays.clone();
            let state = state.clone();
            let model = model.clone();
            move |cx| {
                if let Some(state) = state.as_ref() {
                    let (linked_x, pinned, hidden, query, next_overlays, next_view_bounds) = state
                        .read_ref(cx.app(), |state| {
                            (
                                state.linked_cursor_x.filter(|x| x.is_finite()),
                                state
                                    .pinned_series
                                    .filter(|id| !state.hidden_series.contains(id)),
                                state.hidden_series.iter().copied().collect::<Vec<_>>(),
                                state.query,
                                state.overlays.clone(),
                                line_plot_view_bounds_from_state(
                                    &model,
                                    Some(state),
                                    style,
                                    x_scale,
                                    y_scale,
                                ),
                            )
                        })
                        .unwrap_or_else(|_| {
                            (
                                None,
                                None,
                                Vec::new(),
                                None,
                                PlotOverlays::default(),
                                line_plot_view_bounds_from_state(
                                    &model, None, style, x_scale, y_scale,
                                ),
                            )
                        });
                    linked_cursor_x.set(linked_x);
                    pinned_series.set(pinned);
                    query_selection.set(query);
                    view_bounds.set(next_view_bounds);
                    hidden_series.replace(hidden);
                    overlays.replace(next_overlays);
                } else {
                    linked_cursor_x.set(None);
                    pinned_series.set(None);
                    query_selection.set(None);
                    view_bounds.set(line_plot_view_bounds_from_state(
                        &model, None, style, x_scale, y_scale,
                    ));
                    hidden_series.replace(Vec::new());
                    overlays.replace(PlotOverlays::default());
                }

                let bounds = cx.bounds();
                for child in cx.children().to_vec() {
                    cx.paint_child(child, bounds);
                }
            }
        },
        move |cx| {
            let model = model.clone();
            let output_snapshot = output_snapshot.clone();
            let linked_cursor_x = linked_cursor_x.clone();
            let pinned_series = pinned_series.clone();
            let legend_hover = legend_hover.clone();
            let view_bounds = view_bounds.clone();
            let hidden_series = hidden_series.clone();
            let active_selection = active_selection.clone();
            let query_selection = query_selection.clone();
            let overlays = overlays.clone();
            vec![cx.canvas(canvas, move |painter| {
                let hidden_series = hidden_series.borrow();
                let overlays = overlays.borrow();
                paint_line_plot_panel(
                    painter,
                    &model,
                    output_snapshot.get(),
                    linked_cursor_x.get(),
                    pinned_series.get(),
                    legend_hover.get(),
                    view_bounds.get(),
                    query_selection.get(),
                    active_selection.get(),
                    &overlays,
                    &hidden_series,
                    step_mode,
                    style,
                    &x_axis_labels,
                    &y_axis_labels,
                    &y2_axis_labels,
                    &y3_axis_labels,
                    &y4_axis_labels,
                    x_scale,
                    y_scale,
                );
            })]
        },
    );
    let surface_id = element.id;
    cx.managed_surface_on_event_for(surface_id, move |cx, event| {
        let bounds = cx.bounds();
        if let Some(state) = event_state.as_ref()
            && handle_line_plot_wheel_zoom_event(
                cx.app(),
                state,
                event,
                bounds,
                &event_model,
                event_style,
                event_x_scale,
                event_y_scale,
            )
        {
            let current_view_bounds = line_plot_current_view_bounds_for_event(
                cx.app(),
                event_state.as_ref(),
                &event_model,
                event_style,
                event_x_scale,
                event_y_scale,
            );
            event_view_bounds.set(current_view_bounds);
            cx.invalidate_self(fret_ui::Invalidation::Paint);
            cx.request_redraw();
            cx.notify();
            cx.stop_propagation();
            return;
        }

        if let Some(state) = event_state.as_ref()
            && let Some(drag_output) = handle_line_plot_draggable_overlay_event(
                cx.app(),
                state,
                &event_drag_session,
                event,
                bounds,
                &event_model,
                event_style,
                event_x_scale,
                event_y_scale,
            )
        {
            let current_view_bounds = line_plot_current_view_bounds_for_event(
                cx.app(),
                event_state.as_ref(),
                &event_model,
                event_style,
                event_x_scale,
                event_y_scale,
            );
            event_view_bounds.set(current_view_bounds);
            let snapshot = line_plot_output_snapshot_with_drag(
                current_view_bounds,
                None,
                line_plot_query_from_state(cx.app(), event_state.as_ref()),
                Some(drag_output),
            );
            let visual_changed = event_output_snapshot.get() != Some(snapshot);
            event_output_snapshot.set(Some(snapshot));
            let output_changed =
                publish_line_plot_panel_output(cx.app(), event_output.as_ref(), snapshot);
            cx.invalidate_self(fret_ui::Invalidation::Paint);
            cx.request_redraw();
            if visual_changed || output_changed {
                cx.notify();
            }
            cx.stop_propagation();
            return;
        }

        if let Some(state) = event_state.as_ref()
            && handle_line_plot_query_drag_event(
                cx.app(),
                state,
                &event_query_drag_session,
                &event_active_selection,
                event,
                bounds,
                &event_model,
                event_style,
                event_x_scale,
                event_y_scale,
            )
        {
            let current_view_bounds = line_plot_current_view_bounds_for_event(
                cx.app(),
                event_state.as_ref(),
                &event_model,
                event_style,
                event_x_scale,
                event_y_scale,
            );
            event_view_bounds.set(current_view_bounds);
            let snapshot = line_plot_output_snapshot(
                current_view_bounds,
                None,
                line_plot_query_from_state(cx.app(), event_state.as_ref()),
            );
            let visual_changed = event_output_snapshot.get() != Some(snapshot);
            event_output_snapshot.set(Some(snapshot));
            let output_changed =
                publish_line_plot_panel_output(cx.app(), event_output.as_ref(), snapshot);
            cx.invalidate_self(fret_ui::Invalidation::Paint);
            cx.request_redraw();
            if visual_changed || output_changed {
                cx.notify();
            }
            cx.stop_propagation();
            return;
        }

        if let Some(state) = event_state.as_ref()
            && handle_line_plot_box_zoom_event(
                cx.app(),
                state,
                &event_box_zoom_session,
                &event_active_selection,
                event,
                bounds,
                &event_model,
                event_style,
                event_x_scale,
                event_y_scale,
            )
        {
            let current_view_bounds = line_plot_current_view_bounds_for_event(
                cx.app(),
                event_state.as_ref(),
                &event_model,
                event_style,
                event_x_scale,
                event_y_scale,
            );
            event_view_bounds.set(current_view_bounds);
            cx.invalidate_self(fret_ui::Invalidation::Paint);
            cx.request_redraw();
            cx.notify();
            cx.stop_propagation();
            return;
        }

        if let Some(state) = event_state.as_ref()
            && handle_line_plot_pan_event(
                cx.app(),
                state,
                &event_pan_session,
                event,
                bounds,
                &event_model,
                event_style,
                event_x_scale,
                event_y_scale,
            )
        {
            let current_view_bounds = line_plot_current_view_bounds_for_event(
                cx.app(),
                event_state.as_ref(),
                &event_model,
                event_style,
                event_x_scale,
                event_y_scale,
            );
            event_view_bounds.set(current_view_bounds);
            cx.invalidate_self(fret_ui::Invalidation::Paint);
            cx.request_redraw();
            cx.notify();
            cx.stop_propagation();
            return;
        }

        if let Some(state) = event_state.as_ref()
            && handle_line_plot_legend_pointer_event(
                cx.app(),
                state,
                event,
                bounds,
                &event_model,
                event_style,
            )
        {
            cx.invalidate_self(fret_ui::Invalidation::Paint);
            cx.request_redraw();
            cx.notify();
            cx.stop_propagation();
            return;
        }

        if let Some(hovered) =
            line_plot_legend_hover_from_event(event, bounds, &event_model, event_style)
        {
            let changed = event_legend_hover.get() != hovered;
            event_legend_hover.set(hovered);
            if changed {
                cx.invalidate_self(fret_ui::Invalidation::Paint);
                cx.request_redraw();
                cx.notify();
            }
            if hovered.is_some() {
                cx.stop_propagation();
                return;
            }
        }

        let current_view_bounds = line_plot_current_view_bounds_for_event(
            cx.app(),
            event_state.as_ref(),
            &event_model,
            event_style,
            event_x_scale,
            event_y_scale,
        );
        event_view_bounds.set(current_view_bounds);

        let Some(snapshot) = line_plot_panel_event_snapshot(
            event,
            bounds,
            &event_model,
            event_style,
            event_x_scale,
            event_y_scale,
            current_view_bounds,
            line_plot_query_from_state(cx.app(), event_state.as_ref()),
        ) else {
            return;
        };
        let visual_changed = event_output_snapshot.get() != Some(snapshot);
        event_output_snapshot.set(Some(snapshot));
        let output_changed =
            publish_line_plot_panel_output(cx.app(), event_output.as_ref(), snapshot);
        if visual_changed || output_changed {
            cx.invalidate_self(fret_ui::Invalidation::Paint);
            cx.request_redraw();
            cx.notify();
        }
    });
    element
}

#[derive(Debug, Clone, Copy)]
struct LinePlotPanSession {
    last_position: Point,
}

#[derive(Debug, Clone, Copy)]
struct LinePlotBoxZoomSession {
    start: Point,
    current: Point,
    button: MouseButton,
    required_mods: ModifiersMask,
}

#[derive(Debug, Clone, Copy)]
struct LinePlotQueryDragSession {
    start: Point,
    current: Point,
    button: MouseButton,
}

#[derive(Debug, Clone, Copy)]
enum LinePlotDragSession {
    LineX {
        id: u64,
        button: MouseButton,
        offset_x: f64,
        current_x: f64,
    },
    LineY {
        id: u64,
        axis: YAxis,
        button: MouseButton,
        offset_y: f64,
        current_y: f64,
    },
    Point {
        id: u64,
        axis: YAxis,
        button: MouseButton,
        offset: DataPoint,
        current: DataPoint,
    },
    Rect {
        id: u64,
        axis: YAxis,
        button: MouseButton,
        handle: LinePlotDragRectHandle,
        offset: DataPoint,
        start: DataRect,
        current: DataRect,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinePlotDragRectHandle {
    Inside,
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinePlotSelectionKind {
    Query,
    BoxZoom,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LinePlotSelectionOverlay {
    start: Point,
    current: Point,
    kind: LinePlotSelectionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinePlotWheelRegion {
    Plot,
    XAxis,
    YAxis,
}

/// Capability-first adapter for [`error_bars_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn error_bars_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: ErrorBarsPlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    error_bars_plot_panel(cx.elements(), props)
}

/// Capability-first adapter for [`histogram_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn histogram_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: HistogramPlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    histogram_plot_panel(cx.elements(), props)
}

/// Capability-first adapter for [`bars_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn bars_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: BarsPlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    bars_plot_panel(cx.elements(), props)
}

/// Capability-first adapter for [`candlestick_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn candlestick_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: CandlestickPlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    candlestick_plot_panel(cx.elements(), props)
}

/// Capability-first adapter for [`heatmap_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn heatmap_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: HeatmapPlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    heatmap_plot_panel(cx.elements(), props)
}

/// Capability-first adapter for [`histogram2d_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn histogram2d_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: Histogram2DPlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    histogram2d_plot_panel(cx.elements(), props)
}

/// Capability-first adapter for [`line_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn line_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: LinePlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    line_plot_panel(cx.elements(), props)
}

/// Capability-first adapter for [`area_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn area_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: AreaPlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    area_plot_panel(cx.elements(), props)
}

/// Capability-first adapter for [`shaded_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn shaded_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: ShadedPlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    shaded_plot_panel(cx.elements(), props)
}

/// Capability-first adapter for [`stems_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn stems_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: StemsPlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    stems_plot_panel(cx.elements(), props)
}

fn paint_line_plot_panel(
    painter: &mut CanvasPainter<'_>,
    model: &PlotPanelModel,
    output: Option<PlotOutputSnapshot>,
    linked_cursor_x: Option<f64>,
    pinned_series: Option<SeriesId>,
    legend_hover: Option<SeriesId>,
    view_bounds: DataRect,
    query_selection: Option<DataRect>,
    active_selection: Option<LinePlotSelectionOverlay>,
    overlays: &PlotOverlays,
    hidden_series: &[SeriesId],
    step_mode: Option<StepMode>,
    style: LinePlotStyle,
    x_axis_labels: &AxisLabelFormatter,
    y_axis_labels: &AxisLabelFormatter,
    y2_axis_labels: &AxisLabelFormatter,
    y3_axis_labels: &AxisLabelFormatter,
    y4_axis_labels: &AxisLabelFormatter,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    let bounds = painter.bounds();
    let plot = line_plot_inner_rect(bounds, style);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return;
    }

    let background = style
        .background
        .unwrap_or_else(|| painter.theme().snapshot().color_required("surface"));
    painter.scene().push(fret_core::SceneOp::Quad {
        order: DrawOrder(0),
        rect: bounds,
        background: Paint::Solid(background).into(),
        border: if style.border.is_some() {
            Edges::all(style.border_width)
        } else {
            Edges::default()
        },
        border_paint: Paint::Solid(style.border.unwrap_or(Color::TRANSPARENT)).into(),
        corner_radii: Corners::default(),
    });

    let transform = PlotTransform {
        viewport: plot,
        data: view_bounds,
        x_scale,
        y_scale,
    };
    paint_line_plot_images(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        PlotImageLayer::BelowGrid,
        x_scale,
        y_scale,
    );
    paint_line_plot_grid_and_axes(painter, transform, style, &x_axis_labels, &y_axis_labels);
    if let Some(heatmap) = &model.heatmap {
        paint_line_plot_heatmap(painter, transform, heatmap, style);
        paint_line_plot_heatmap_colorbar(painter, plot, heatmap, style);
    }
    paint_line_plot_right_axis_tick_labels(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        style,
        y2_axis_labels,
        y3_axis_labels,
        y4_axis_labels,
        x_scale,
        y_scale,
    );
    paint_line_plot_images(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        PlotImageLayer::AboveGrid,
        x_scale,
        y_scale,
    );
    paint_line_plot_reference_lines(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        style,
        x_scale,
        y_scale,
    );
    paint_line_plot_draggable_shapes(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        style,
        x_scale,
        y_scale,
    );
    paint_line_plot_draggable_overlay_labels(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        style,
        x_scale,
        y_scale,
    );
    paint_line_plot_tag_overlays(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        style,
        x_scale,
        y_scale,
    );
    paint_line_plot_text_overlays(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        style,
        x_scale,
        y_scale,
    );

    let series_count = model.series.len();
    let raster_scale_factor = painter.scale_factor();
    let emphasized_series = if style.emphasize_hovered_series {
        pinned_series.or(legend_hover)
    } else {
        None
    };
    let right_transform = model.data_bounds_y2.map(|axis_bounds| PlotTransform {
        viewport: plot,
        data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
        x_scale,
        y_scale,
    });
    let right2_transform = model.data_bounds_y3.map(|axis_bounds| PlotTransform {
        viewport: plot,
        data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
        x_scale,
        y_scale,
    });
    let right3_transform = model.data_bounds_y4.map(|axis_bounds| PlotTransform {
        viewport: plot,
        data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
        x_scale,
        y_scale,
    });

    for (index, series) in model.series.iter().enumerate() {
        if hidden_series.contains(&series.id) {
            continue;
        }
        let series_transform = match series.y_axis {
            YAxis::Left => transform,
            YAxis::Right => right_transform.unwrap_or(transform),
            YAxis::Right2 => right2_transform.unwrap_or(transform),
            YAxis::Right3 => right3_transform.unwrap_or(transform),
        };
        if let Some(candlestick) = &series.candlestick {
            let stroke_width = series.stroke_width.unwrap_or(style.stroke_width);
            let (wick_commands, up_body_commands, down_body_commands) =
                candlestick_commands_from_series(
                    series_transform,
                    candlestick,
                    stroke_width,
                    raster_scale_factor,
                );
            if wick_commands.is_empty()
                && up_body_commands.is_empty()
                && down_body_commands.is_empty()
            {
                continue;
            }

            let mut wick_color = candlestick
                .wick_color
                .or(series.stroke_color)
                .unwrap_or_else(|| series_color(style, index, series_count));
            let mut up_fill = candlestick.up_fill.unwrap_or(Color {
                r: 0.25,
                g: 0.80,
                b: 0.45,
                a: 0.85,
            });
            let mut down_fill = candlestick.down_fill.unwrap_or(Color {
                r: 0.90,
                g: 0.35,
                b: 0.45,
                a: 0.85,
            });
            if let Some(emphasized) = emphasized_series
                && series.id != emphasized
            {
                let dim = style.dimmed_series_alpha.clamp(0.0, 1.0);
                wick_color.a *= dim;
                up_fill.a *= dim;
                down_fill.a *= dim;
            }

            if !up_body_commands.is_empty() {
                painter.path(
                    line_plot_area_fill_path_key(series.id.0),
                    DrawOrder(19),
                    Point::new(Px(0.0), Px(0.0)),
                    &up_body_commands,
                    PathStyle::Fill(fret_core::FillStyle::default()),
                    up_fill,
                    raster_scale_factor,
                );
            }
            if !down_body_commands.is_empty() {
                painter.path(
                    line_plot_candlestick_down_path_key(series.id.0),
                    DrawOrder(19),
                    Point::new(Px(0.0), Px(0.0)),
                    &down_body_commands,
                    PathStyle::Fill(fret_core::FillStyle::default()),
                    down_fill,
                    raster_scale_factor,
                );
            }
            if wick_commands.len() >= 2 {
                painter.path(
                    line_plot_series_path_key(series.id.0),
                    DrawOrder(20),
                    Point::new(Px(0.0), Px(0.0)),
                    &wick_commands,
                    PathStyle::Stroke(StrokeStyle {
                        width: stroke_width,
                    }),
                    wick_color,
                    raster_scale_factor,
                );
            }
            continue;
        }
        if let Some(bars) = &series.bars {
            let commands = bars_commands_from_series(series_transform, &*series.data, bars);
            if commands.is_empty() {
                continue;
            }

            let mut fill_color = series
                .stroke_color
                .unwrap_or_else(|| series_color(style, index, series_count));
            if let Some(emphasized) = emphasized_series
                && series.id != emphasized
            {
                fill_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
            }
            painter.path(
                line_plot_area_fill_path_key(series.id.0),
                DrawOrder(19),
                Point::new(Px(0.0), Px(0.0)),
                &commands,
                PathStyle::Fill(fret_core::FillStyle::default()),
                fill_color,
                raster_scale_factor,
            );
            continue;
        }
        if let Some(histogram) = &series.histogram {
            let commands =
                histogram_commands_from_series(series_transform, &*series.data, histogram);
            if commands.is_empty() {
                continue;
            }

            let mut fill_color = series
                .stroke_color
                .unwrap_or_else(|| series_color(style, index, series_count));
            if let Some(emphasized) = emphasized_series
                && series.id != emphasized
            {
                fill_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
            }
            painter.path(
                line_plot_area_fill_path_key(series.id.0),
                DrawOrder(19),
                Point::new(Px(0.0), Px(0.0)),
                &commands,
                PathStyle::Fill(fret_core::FillStyle::default()),
                fill_color,
                raster_scale_factor,
            );
            continue;
        }
        if let Some(error_bars) = &series.error_bars {
            let commands =
                error_bars_commands_from_series(series_transform, &*series.data, error_bars);
            if commands.len() < 2 {
                continue;
            }

            let mut stroke_color = series
                .stroke_color
                .unwrap_or_else(|| series_color(style, index, series_count));
            if let Some(emphasized) = emphasized_series
                && series.id != emphasized
            {
                stroke_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
            }
            let stroke_width = series.stroke_width.unwrap_or(style.stroke_width);
            painter.path(
                line_plot_series_path_key(series.id.0),
                DrawOrder(20),
                Point::new(Px(0.0), Px(0.0)),
                &commands,
                PathStyle::Stroke(StrokeStyle {
                    width: stroke_width,
                }),
                stroke_color,
                raster_scale_factor,
            );
            continue;
        }
        if let Some(lower_data) = &series.lower_data {
            let (fill_commands, upper_commands, lower_commands) =
                shaded_band_commands_from_series(series_transform, &*series.data, &**lower_data);

            let mut stroke_color = series
                .stroke_color
                .unwrap_or_else(|| series_color(style, index, series_count));
            if let Some(emphasized) = emphasized_series
                && series.id != emphasized
            {
                stroke_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
            }

            if let Some(fill) = series.fill
                && !fill_commands.is_empty()
            {
                let mut fill_color = fill.color.unwrap_or_else(|| {
                    series
                        .stroke_color
                        .unwrap_or_else(|| series_color(style, index, series_count))
                });
                fill_color.a = (fill_color.a * fill.alpha.clamp(0.0, 1.0)).clamp(0.0, 1.0);
                if let Some(emphasized) = emphasized_series
                    && series.id != emphasized
                {
                    fill_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
                }
                painter.path(
                    line_plot_area_fill_path_key(series.id.0),
                    DrawOrder(19),
                    Point::new(Px(0.0), Px(0.0)),
                    &fill_commands,
                    PathStyle::Fill(fret_core::FillStyle::default()),
                    fill_color,
                    raster_scale_factor,
                );
            }

            let stroke_width = series.stroke_width.unwrap_or(style.stroke_width);
            if upper_commands.len() >= 2 {
                painter.path(
                    line_plot_series_path_key(series.id.0),
                    DrawOrder(20),
                    Point::new(Px(0.0), Px(0.0)),
                    &upper_commands,
                    PathStyle::Stroke(StrokeStyle {
                        width: stroke_width,
                    }),
                    stroke_color,
                    raster_scale_factor,
                );
            }
            if lower_commands.len() >= 2 {
                painter.path(
                    line_plot_shaded_lower_path_key(series.id.0),
                    DrawOrder(20),
                    Point::new(Px(0.0), Px(0.0)),
                    &lower_commands,
                    PathStyle::Stroke(StrokeStyle {
                        width: stroke_width,
                    }),
                    stroke_color,
                    raster_scale_factor,
                );
            }
            continue;
        }
        let Some(points) = series.data.as_slice() else {
            continue;
        };
        let commands = if let Some(baseline) = series.stem_baseline {
            stems_commands_from_points(series_transform, points, baseline)
        } else {
            let commands = polyline_commands(series_transform, points);
            if let Some(step_mode) = step_mode {
                step_commands_from_polyline(&commands, step_mode)
            } else {
                commands
            }
        };
        if commands.len() < 2 {
            continue;
        }

        let mut stroke_color = series
            .stroke_color
            .unwrap_or_else(|| series_color(style, index, series_count));
        if let Some(emphasized) = emphasized_series
            && series.id != emphasized
        {
            stroke_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
        }
        if let Some(fill) = series.fill
            && let Some(baseline_y) = series_transform.data_y_to_px(f64::from(fill.baseline))
        {
            let fill_commands = area_fill_commands_from_polyline(&commands, baseline_y);
            if !fill_commands.is_empty() {
                let mut fill_color = fill.color.unwrap_or_else(|| {
                    series
                        .stroke_color
                        .unwrap_or_else(|| series_color(style, index, series_count))
                });
                fill_color.a = (fill_color.a * fill.alpha.clamp(0.0, 1.0)).clamp(0.0, 1.0);
                if let Some(emphasized) = emphasized_series
                    && series.id != emphasized
                {
                    fill_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
                }
                painter.path(
                    line_plot_area_fill_path_key(series.id.0),
                    DrawOrder(19),
                    Point::new(Px(0.0), Px(0.0)),
                    &fill_commands,
                    PathStyle::Fill(fret_core::FillStyle::default()),
                    fill_color,
                    raster_scale_factor,
                );
            }
        }
        let stroke_width = series.stroke_width.unwrap_or(style.stroke_width);
        painter.path(
            line_plot_series_path_key(series.id.0),
            DrawOrder(20),
            Point::new(Px(0.0), Px(0.0)),
            &commands,
            PathStyle::Stroke(StrokeStyle {
                width: stroke_width,
            }),
            stroke_color,
            raster_scale_factor,
        );
    }

    paint_line_plot_legend(painter, model, plot, pinned_series, legend_hover, style);
    paint_line_plot_query_selection(
        painter,
        plot,
        view_bounds,
        query_selection,
        style,
        x_scale,
        y_scale,
    );
    paint_line_plot_active_selection(painter, plot, active_selection, style);
    if paint_line_plot_selection_tooltip(
        painter,
        bounds,
        plot,
        view_bounds,
        active_selection,
        style,
        x_scale,
        y_scale,
    ) {
        return;
    }
    paint_line_plot_cursor_readout(
        painter,
        model,
        plot,
        output,
        pinned_series,
        hidden_series,
        style,
        y_axis_labels,
        y2_axis_labels,
        y3_axis_labels,
        y4_axis_labels,
        x_scale,
        y_scale,
    );
    paint_line_plot_linked_cursor_readout(
        painter,
        model,
        plot,
        transform.data,
        output.and_then(|snapshot| snapshot.cursor),
        linked_cursor_x,
        pinned_series,
        hidden_series,
        style,
        y_axis_labels,
        y2_axis_labels,
        y3_axis_labels,
        y4_axis_labels,
        x_scale,
        y_scale,
    );
}

fn line_plot_view_bounds_for_y_axis(primary: DataRect, axis_bounds: DataRect) -> DataRect {
    DataRect {
        x_min: primary.x_min,
        x_max: primary.x_max,
        y_min: axis_bounds.y_min,
        y_max: axis_bounds.y_max,
    }
}

fn paint_line_plot_heatmap(
    painter: &mut CanvasPainter<'_>,
    transform: PlotTransform,
    heatmap: &PlotPanelHeatmap,
    style: LinePlotStyle,
) {
    if heatmap.cols == 0 || heatmap.rows == 0 || heatmap.values.is_empty() {
        return;
    }
    let Some(prepared) = transform.prepare() else {
        return;
    };

    let dx = (heatmap.data_bounds.x_max - heatmap.data_bounds.x_min) / heatmap.cols as f64;
    let dy = (heatmap.data_bounds.y_max - heatmap.data_bounds.y_min) / heatmap.rows as f64;
    if !dx.is_finite() || !dy.is_finite() || dx <= 0.0 || dy <= 0.0 {
        return;
    }

    let view_x_min = transform.data.x_min.min(transform.data.x_max);
    let view_x_max = transform.data.x_min.max(transform.data.x_max);
    let view_y_min = transform.data.y_min.min(transform.data.y_max);
    let view_y_max = transform.data.y_min.max(transform.data.y_max);
    let clip_min_x = view_x_min.max(heatmap.data_bounds.x_min);
    let clip_max_x = view_x_max.min(heatmap.data_bounds.x_max);
    let clip_min_y = view_y_min.max(heatmap.data_bounds.y_min);
    let clip_max_y = view_y_max.min(heatmap.data_bounds.y_max);
    if clip_max_x <= clip_min_x || clip_max_y <= clip_min_y {
        return;
    }

    let col0 = (((clip_min_x - heatmap.data_bounds.x_min) / dx).floor() as isize)
        .clamp(0, heatmap.cols.saturating_sub(1) as isize) as usize;
    let col1 = (((clip_max_x - heatmap.data_bounds.x_min) / dx).ceil() as isize)
        .clamp(0, heatmap.cols as isize) as usize;
    let row0 = (((clip_min_y - heatmap.data_bounds.y_min) / dy).floor() as isize)
        .clamp(0, heatmap.rows.saturating_sub(1) as isize) as usize;
    let row1 = (((clip_max_y - heatmap.data_bounds.y_min) / dy).ceil() as isize)
        .clamp(0, heatmap.rows as isize) as usize;

    let denom = (heatmap.value_max - heatmap.value_min).max(1.0e-12);
    for row in row0..row1 {
        let y0 = heatmap.data_bounds.y_min + row as f64 * dy;
        let y1 = heatmap.data_bounds.y_min + row.saturating_add(1) as f64 * dy;
        let (Some(py0), Some(py1)) = (prepared.data_y_to_px(y0), prepared.data_y_to_px(y1)) else {
            continue;
        };
        let top = py0.0.min(py1.0);
        let bottom = py0.0.max(py1.0);
        if !top.is_finite() || !bottom.is_finite() || bottom <= top {
            continue;
        }

        for col in col0..col1 {
            let idx = row.saturating_mul(heatmap.cols).saturating_add(col);
            let Some(value) = heatmap.values.get(idx).copied() else {
                continue;
            };
            if !value.is_finite() {
                continue;
            }

            let x0 = heatmap.data_bounds.x_min + col as f64 * dx;
            let x1 = heatmap.data_bounds.x_min + col.saturating_add(1) as f64 * dx;
            let (Some(px0), Some(px1)) = (prepared.data_x_to_px(x0), prepared.data_x_to_px(x1))
            else {
                continue;
            };
            let left = px0.0.min(px1.0);
            let right = px0.0.max(px1.0);
            if !left.is_finite() || !right.is_finite() || right <= left {
                continue;
            }

            let t = ((value - heatmap.value_min) / denom).clamp(0.0, 1.0);
            let color = crate::plot::colormap::sample(style.heatmap_colormap, t);
            push_filled_rect(
                painter,
                Rect::new(
                    Point::new(Px(left), Px(top)),
                    Size::new(Px(right - left), Px(bottom - top)),
                ),
                DrawOrder(2),
                color,
            );
        }
    }
}

fn format_heatmap_colorbar_value(value: f32) -> String {
    if !value.is_finite() {
        return "NA".to_string();
    }
    let abs = value.abs();
    if abs > 1.0e6 || (abs > 0.0 && abs < 1.0e-3) {
        return format!("{value:.3e}");
    }
    if abs >= 1000.0 {
        return format!("{value:.0}");
    }
    if abs >= 10.0 {
        return format!("{value:.2}");
    }
    format!("{value:.3}")
}

fn paint_line_plot_heatmap_colorbar(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    heatmap: &PlotPanelHeatmap,
    style: LinePlotStyle,
) {
    if !style.heatmap_show_colorbar
        || !heatmap.value_min.is_finite()
        || !heatmap.value_max.is_finite()
        || heatmap.value_max <= heatmap.value_min
    {
        return;
    }

    let padding = style.heatmap_colorbar_padding.0.max(0.0);
    let bar_width = style.heatmap_colorbar_width.0.max(1.0);
    let steps = style.heatmap_colorbar_steps.clamp(8, 512);
    let bar_height = (plot.size.height.0 - padding * 2.0).max(0.0);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 || bar_height < 24.0 {
        return;
    }

    let theme = painter.theme().snapshot();
    let tooltip_background = style
        .tooltip_background
        .unwrap_or_else(|| theme.color_required("popover"));
    let tooltip_border = style
        .tooltip_border
        .unwrap_or_else(|| theme.color_required("border"));
    let text_color = style
        .tooltip_text_color
        .or(style.label_color)
        .unwrap_or_else(|| theme.color_required("popover-foreground"));

    let text_style = TextStyle {
        size: Px(11.0),
        weight: FontWeight::NORMAL,
        ..TextStyle::default()
    };
    let constraints = CanvasTextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let raster_scale_factor = painter.scale_factor();
    let scope = painter.key_scope(&"fret-plot.declarative.heatmap-colorbar");
    let max_label = format_heatmap_colorbar_value(heatmap.value_max);
    let min_label = format_heatmap_colorbar_value(heatmap.value_min);
    let max_key: u64 = painter
        .child_key(scope, &("max", max_label.as_str()))
        .into();
    let min_key: u64 = painter
        .child_key(scope, &("min", min_label.as_str()))
        .into();
    let (_max_blob, max_metrics) = painter.prepare_text_with_blob(
        max_key,
        max_label.clone(),
        text_style.clone(),
        constraints,
        raster_scale_factor,
    );
    let (_min_blob, min_metrics) = painter.prepare_text_with_blob(
        min_key,
        min_label.clone(),
        text_style.clone(),
        constraints,
        raster_scale_factor,
    );

    let label_gap = 6.0_f32;
    let label_width = max_metrics.size.width.0.max(min_metrics.size.width.0);
    let panel_width = (bar_width + label_gap + label_width).max(bar_width);
    let panel_left = (plot.origin.x.0 + plot.size.width.0 - padding - panel_width)
        .max(plot.origin.x.0 + padding);
    let panel_top = plot.origin.y.0 + padding;
    let bar_left = panel_left;
    let bar_top = panel_top;
    let label_x = bar_left + bar_width + label_gap;

    painter.scene().push(fret_core::SceneOp::Quad {
        order: DrawOrder(3),
        rect: Rect::new(
            Point::new(Px(panel_left), Px(panel_top)),
            Size::new(Px(panel_width), Px(bar_height)),
        ),
        background: Paint::Solid(Color {
            a: 0.88,
            ..tooltip_background
        })
        .into(),
        border: Edges::all(Px(1.0)),
        border_paint: Paint::Solid(tooltip_border).into(),
        corner_radii: Corners::all(Px(6.0)),
    });

    for index in 0..steps {
        let t0 = index as f32 / steps as f32;
        let t1 = index.saturating_add(1) as f32 / steps as f32;
        let t = (t0 + t1) * 0.5;
        let y0 = bar_top + (1.0 - t1) * bar_height;
        let height = ((t1 - t0) * bar_height).max(1.0);
        let color = crate::plot::colormap::sample(style.heatmap_colormap, t);
        push_filled_rect(
            painter,
            Rect::new(
                Point::new(Px(bar_left), Px(y0)),
                Size::new(Px(bar_width), Px(height)),
            ),
            DrawOrder(4),
            color,
        );
    }

    painter.scene().push(fret_core::SceneOp::Quad {
        order: DrawOrder(5),
        rect: Rect::new(
            Point::new(Px(bar_left), Px(bar_top)),
            Size::new(Px(bar_width), Px(bar_height)),
        ),
        background: Paint::TRANSPARENT.into(),
        border: Edges::all(Px(1.0)),
        border_paint: Paint::Solid(tooltip_border).into(),
        corner_radii: Corners::default(),
    });

    let text_margin = 2.0_f32;
    let _ = painter.text(
        max_key,
        DrawOrder(6),
        Point::new(
            Px(label_x),
            Px(bar_top + text_margin + max_metrics.baseline.0),
        ),
        max_label,
        text_style.clone(),
        text_color,
        constraints,
        raster_scale_factor,
    );
    let _ = painter.text(
        min_key,
        DrawOrder(6),
        Point::new(
            Px(label_x),
            Px(
                bar_top + bar_height - text_margin - min_metrics.size.height.0
                    + min_metrics.baseline.0,
            ),
        ),
        min_label,
        text_style,
        text_color,
        constraints,
        raster_scale_factor,
    );
}

fn paint_line_plot_reference_lines(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    view_bounds: DataRect,
    view_bounds_y2: Option<DataRect>,
    view_bounds_y3: Option<DataRect>,
    view_bounds_y4: Option<DataRect>,
    overlays: &PlotOverlays,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    if overlays.inf_lines_x.is_empty()
        && overlays.inf_lines_y.is_empty()
        && overlays.drag_lines_x.is_empty()
        && overlays.drag_lines_y.is_empty()
    {
        return;
    }
    let Some(transform) = (PlotTransform {
        viewport: plot,
        data: view_bounds,
        x_scale,
        y_scale,
    })
    .prepare() else {
        return;
    };

    let transform_y2 = view_bounds_y2.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });
    let transform_y3 = view_bounds_y3.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });
    let transform_y4 = view_bounds_y4.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });

    let theme = painter.theme().snapshot();
    let base_color = style
        .crosshair_color
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    let default_color = Color {
        a: (base_color.a * 0.45).clamp(0.05, 1.0),
        ..base_color
    };

    let x_lines = overlays
        .inf_lines_x
        .iter()
        .map(|line| (line.x, line.width, line.color.unwrap_or(default_color)))
        .chain(
            overlays
                .drag_lines_x
                .iter()
                .map(|line| (line.x, line.width, line.color.unwrap_or(default_color))),
        );
    for (x_value, line_width, line_color) in x_lines {
        let Some(x) = transform.data_x_to_px(x_value) else {
            continue;
        };
        let width = line_width.0.max(1.0).min(plot.size.width.0.max(1.0));
        let left =
            (x.0 - width * 0.5).clamp(plot.origin.x.0, plot.origin.x.0 + plot.size.width.0 - width);
        push_filled_rect(
            painter,
            Rect::new(
                Point::new(Px(left.round()), plot.origin.y),
                Size::new(Px(width), plot.size.height),
            ),
            DrawOrder(3),
            line_color,
        );
    }

    let y_lines = overlays
        .inf_lines_y
        .iter()
        .map(|line| {
            (
                line.y,
                line.axis,
                line.width,
                line.color.unwrap_or(default_color),
            )
        })
        .chain(overlays.drag_lines_y.iter().map(|line| {
            (
                line.y,
                line.axis,
                line.width,
                line.color.unwrap_or(default_color),
            )
        }));
    for (y_value, axis, line_width, line_color) in y_lines {
        let transform = match axis {
            crate::models::YAxis::Left => transform,
            crate::models::YAxis::Right => transform_y2.unwrap_or(transform),
            crate::models::YAxis::Right2 => transform_y3.unwrap_or(transform),
            crate::models::YAxis::Right3 => transform_y4.unwrap_or(transform),
        };
        let Some(y) = transform.data_y_to_px(y_value) else {
            continue;
        };
        let height = line_width.0.max(1.0).min(plot.size.height.0.max(1.0));
        let top = (y.0 - height * 0.5).clamp(
            plot.origin.y.0,
            plot.origin.y.0 + plot.size.height.0 - height,
        );
        push_filled_rect(
            painter,
            Rect::new(
                Point::new(plot.origin.x, Px(top.round())),
                Size::new(plot.size.width, Px(height)),
            ),
            DrawOrder(3),
            line_color,
        );
    }
}

fn paint_line_plot_draggable_shapes(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    view_bounds: DataRect,
    view_bounds_y2: Option<DataRect>,
    view_bounds_y3: Option<DataRect>,
    view_bounds_y4: Option<DataRect>,
    overlays: &PlotOverlays,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    if overlays.drag_points.is_empty() && overlays.drag_rects.is_empty() {
        return;
    }
    let Some(transform) = (PlotTransform {
        viewport: plot,
        data: view_bounds,
        x_scale,
        y_scale,
    })
    .prepare() else {
        return;
    };
    let transform_y2 = view_bounds_y2.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });
    let transform_y3 = view_bounds_y3.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });
    let transform_y4 = view_bounds_y4.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });

    let theme = painter.theme().snapshot();
    let base_color = style
        .crosshair_color
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    let stroke = Color {
        a: (base_color.a * 0.45).clamp(0.05, 1.0),
        ..base_color
    };
    let border = style
        .tooltip_border
        .unwrap_or_else(|| theme.color_required("border"));

    for point in overlays.drag_points.iter() {
        let p = point.point;
        if !p.x.is_finite() || !p.y.is_finite() {
            continue;
        }
        let transform = match point.axis {
            crate::models::YAxis::Left => transform,
            crate::models::YAxis::Right => transform_y2.unwrap_or(transform),
            crate::models::YAxis::Right2 => transform_y3.unwrap_or(transform),
            crate::models::YAxis::Right3 => transform_y4.unwrap_or(transform),
        };
        let p_px = transform.data_to_px(p);
        if !p_px.x.0.is_finite() || !p_px.y.0.is_finite() {
            continue;
        }

        let radius = point.radius.0.max(2.0);
        let diameter = (radius * 2.0).max(1.0);
        let max_left = (plot.size.width.0 - diameter).max(0.0);
        let max_top = (plot.size.height.0 - diameter).max(0.0);
        let left = (p_px.x.0 - plot.origin.x.0 - radius).clamp(0.0, max_left);
        let top = (p_px.y.0 - plot.origin.y.0 - radius).clamp(0.0, max_top);
        painter.scene().push(fret_core::SceneOp::Quad {
            order: DrawOrder(3),
            rect: Rect::new(
                Point::new(
                    Px((plot.origin.x.0 + left).round()),
                    Px((plot.origin.y.0 + top).round()),
                ),
                Size::new(Px(diameter), Px(diameter)),
            ),
            background: Paint::Solid(point.color.unwrap_or(stroke)).into(),
            border: Edges::all(Px(1.0)),
            border_paint: Paint::Solid(border).into(),
            corner_radii: Corners::all(Px(radius)),
        });
    }

    for rect in overlays.drag_rects.iter() {
        let transform = match rect.axis {
            crate::models::YAxis::Left => transform,
            crate::models::YAxis::Right => transform_y2.unwrap_or(transform),
            crate::models::YAxis::Right2 => transform_y3.unwrap_or(transform),
            crate::models::YAxis::Right3 => transform_y4.unwrap_or(transform),
        };
        let current = rect.rect;
        if !current.x_min.is_finite()
            || !current.x_max.is_finite()
            || !current.y_min.is_finite()
            || !current.y_max.is_finite()
        {
            continue;
        }

        let a = transform.data_to_px(DataPoint {
            x: current.x_min,
            y: current.y_min,
        });
        let b = transform.data_to_px(DataPoint {
            x: current.x_max,
            y: current.y_max,
        });
        if !a.x.0.is_finite() || !a.y.0.is_finite() || !b.x.0.is_finite() || !b.y.0.is_finite() {
            continue;
        }

        let left =
            a.x.0
                .min(b.x.0)
                .clamp(plot.origin.x.0, plot.origin.x.0 + plot.size.width.0);
        let right =
            a.x.0
                .max(b.x.0)
                .clamp(plot.origin.x.0, plot.origin.x.0 + plot.size.width.0);
        let top =
            a.y.0
                .min(b.y.0)
                .clamp(plot.origin.y.0, plot.origin.y.0 + plot.size.height.0);
        let bottom =
            a.y.0
                .max(b.y.0)
                .clamp(plot.origin.y.0, plot.origin.y.0 + plot.size.height.0);
        let width = (right - left).max(0.0);
        let height = (bottom - top).max(0.0);
        if width <= 0.0 || height <= 0.0 {
            continue;
        }

        let color = rect.color.unwrap_or(stroke);
        let fill = rect.fill.unwrap_or(Color { a: 0.12, ..color });
        painter.scene().push(fret_core::SceneOp::Quad {
            order: DrawOrder(3),
            rect: Rect::new(
                Point::new(Px(left.round()), Px(top.round())),
                Size::new(Px(width), Px(height)),
            ),
            background: Paint::Solid(fill).into(),
            border: Edges::all(Px(rect.border_width.0.max(1.0))),
            border_paint: Paint::Solid(color).into(),
            corner_radii: Corners::default(),
        });
    }
}

#[derive(Debug, Clone, Copy)]
struct LinePlotAnnotationTokens {
    background: Color,
    border: Color,
    text: Color,
    stroke: Color,
    padding: Px,
    radius: Px,
}

fn line_plot_annotation_tokens(
    painter: &mut CanvasPainter<'_>,
    style: LinePlotStyle,
) -> LinePlotAnnotationTokens {
    let theme = painter.theme();
    let tooltip_background = style
        .tooltip_background
        .unwrap_or_else(|| theme.color_required("popover"));
    let tooltip_border = style
        .tooltip_border
        .unwrap_or_else(|| theme.color_required("border"));
    let tooltip_text = style
        .tooltip_text_color
        .or(style.label_color)
        .unwrap_or_else(|| theme.color_required("popover-foreground"));
    let crosshair = style
        .crosshair_color
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    LinePlotAnnotationTokens {
        background: crate::theme_tokens::color(
            theme,
            "fret.plot.annotation.background",
            "plot.annotation.background",
        )
        .unwrap_or(tooltip_background),
        border: crate::theme_tokens::color(
            theme,
            "fret.plot.annotation.border",
            "plot.annotation.border",
        )
        .unwrap_or(tooltip_border),
        text: crate::theme_tokens::color(
            theme,
            "fret.plot.annotation.text",
            "plot.annotation.text",
        )
        .unwrap_or(tooltip_text),
        stroke: crate::theme_tokens::color(
            theme,
            "fret.plot.annotation.stroke",
            "plot.annotation.stroke",
        )
        .unwrap_or(crosshair),
        padding: crate::theme_tokens::metric(
            theme,
            "fret.plot.annotation.padding",
            "plot.annotation.padding",
        )
        .unwrap_or_else(|| theme.metric_token("metric.padding.sm")),
        radius: crate::theme_tokens::metric(
            theme,
            "fret.plot.annotation.radius",
            "plot.annotation.radius",
        )
        .unwrap_or_else(|| theme.metric_token("metric.radius.sm")),
    }
}

fn paint_line_plot_images(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    view_bounds: DataRect,
    view_bounds_y2: Option<DataRect>,
    view_bounds_y3: Option<DataRect>,
    view_bounds_y4: Option<DataRect>,
    overlays: &PlotOverlays,
    layer: PlotImageLayer,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    if overlays.images.is_empty() {
        return;
    }
    let Some(transform) = (PlotTransform {
        viewport: plot,
        data: view_bounds,
        x_scale,
        y_scale,
    })
    .prepare() else {
        return;
    };
    let transform_y2 = view_bounds_y2.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });
    let transform_y3 = view_bounds_y3.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });
    let transform_y4 = view_bounds_y4.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });

    painter.with_clip_rect(plot, |painter| {
        for image in overlays.images.iter().filter(|image| image.layer == layer) {
            let transform = match image.axis {
                YAxis::Left => transform,
                YAxis::Right => transform_y2.unwrap_or(transform),
                YAxis::Right2 => transform_y3.unwrap_or(transform),
                YAxis::Right3 => transform_y4.unwrap_or(transform),
            };
            let rect = image.rect;
            if !rect.x_min.is_finite()
                || !rect.x_max.is_finite()
                || !rect.y_min.is_finite()
                || !rect.y_max.is_finite()
            {
                continue;
            }

            let a = transform.data_to_px(DataPoint {
                x: rect.x_min,
                y: rect.y_min,
            });
            let b = transform.data_to_px(DataPoint {
                x: rect.x_max,
                y: rect.y_max,
            });
            if !a.x.0.is_finite() || !a.y.0.is_finite() || !b.x.0.is_finite() || !b.y.0.is_finite()
            {
                continue;
            }

            let left = a.x.0.min(b.x.0);
            let right = a.x.0.max(b.x.0);
            let top = a.y.0.min(b.y.0);
            let bottom = a.y.0.max(b.y.0);
            let width = (right - left).max(0.0);
            let height = (bottom - top).max(0.0);
            if width <= 0.0 || height <= 0.0 {
                continue;
            }

            let opacity = image.opacity.clamp(0.0, 1.0);
            if opacity <= 0.0 {
                continue;
            }

            painter.scene().push(fret_core::SceneOp::ImageRegion {
                order: DrawOrder(1),
                rect: Rect::new(
                    Point::new(Px(left), Px(top)),
                    Size::new(Px(width), Px(height)),
                ),
                image: image.image,
                uv: image.uv,
                sampling: fret_core::scene::ImageSamplingHint::Default,
                opacity,
            });
        }
    });
}

fn paint_line_plot_draggable_overlay_labels(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    view_bounds: DataRect,
    view_bounds_y2: Option<DataRect>,
    view_bounds_y3: Option<DataRect>,
    view_bounds_y4: Option<DataRect>,
    overlays: &PlotOverlays,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    if overlays.drag_lines_x.is_empty()
        && overlays.drag_lines_y.is_empty()
        && overlays.drag_points.is_empty()
    {
        return;
    }
    let Some(transform) = (PlotTransform {
        viewport: plot,
        data: view_bounds,
        x_scale,
        y_scale,
    })
    .prepare() else {
        return;
    };
    let transform_y2 = view_bounds_y2.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });
    let transform_y3 = view_bounds_y3.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });
    let transform_y4 = view_bounds_y4.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });

    let tokens = line_plot_annotation_tokens(painter, style);
    let text_style = TextStyle {
        size: Px(12.0),
        weight: FontWeight::NORMAL,
        ..TextStyle::default()
    };
    let constraints = CanvasTextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let raster_scale_factor = painter.scale_factor();
    let scope = painter.key_scope(&"fret-plot.declarative.draggable-overlay-labels");
    let formatter = AxisLabelFormatter::default();
    let x_span = (view_bounds.x_max - view_bounds.x_min).abs();
    let y_span = (view_bounds.y_max - view_bounds.y_min).abs();

    for (index, line) in overlays.drag_lines_x.iter().enumerate() {
        if !line.x.is_finite() {
            continue;
        }
        let Some(x_px) = transform.data_x_to_px(line.x) else {
            continue;
        };
        let value = line
            .show_value
            .then(|| axis_tick_label_text(x_scale, &formatter, line.x, x_span));
        let text = line_plot_annotation_label(line.label.as_deref(), value);
        if text.is_empty() {
            continue;
        }
        let key: u64 = painter
            .child_key(
                scope,
                &(
                    "drag-line-x",
                    index,
                    line.id,
                    line.x.to_bits(),
                    line.label.as_deref(),
                    line.show_value,
                    text.as_str(),
                ),
            )
            .into();
        paint_line_plot_tag_x_overlay(
            painter,
            plot,
            Px(x_px.0.round()),
            line.color.unwrap_or(tokens.stroke),
            key,
            text,
            &text_style,
            constraints,
            raster_scale_factor,
            tokens,
        );
    }

    for (index, line) in overlays.drag_lines_y.iter().enumerate() {
        if !line.y.is_finite() {
            continue;
        }
        let (transform, right, span) = match line.axis {
            crate::models::YAxis::Left => (Some(transform), false, y_span),
            crate::models::YAxis::Right => (
                Some(transform_y2.unwrap_or(transform)),
                true,
                view_bounds_y2
                    .map(|b| (b.y_max - b.y_min).abs())
                    .unwrap_or(y_span),
            ),
            crate::models::YAxis::Right2 => (
                Some(transform_y3.unwrap_or(transform)),
                true,
                view_bounds_y3
                    .map(|b| (b.y_max - b.y_min).abs())
                    .unwrap_or(y_span),
            ),
            crate::models::YAxis::Right3 => (
                Some(transform_y4.unwrap_or(transform)),
                true,
                view_bounds_y4
                    .map(|b| (b.y_max - b.y_min).abs())
                    .unwrap_or(y_span),
            ),
        };
        let Some(transform) = transform else {
            continue;
        };
        let Some(y_px) = transform.data_y_to_px(line.y) else {
            continue;
        };
        let value = line
            .show_value
            .then(|| axis_tick_label_text(y_scale, &formatter, line.y, span));
        let text = line_plot_annotation_label(line.label.as_deref(), value);
        if text.is_empty() {
            continue;
        }
        let key: u64 = painter
            .child_key(
                scope,
                &(
                    "drag-line-y",
                    index,
                    line.id,
                    line.y.to_bits(),
                    line.label.as_deref(),
                    line.show_value,
                    line.axis,
                    text.as_str(),
                ),
            )
            .into();
        paint_line_plot_tag_y_overlay(
            painter,
            plot,
            Px(y_px.0.round()),
            right,
            line.color.unwrap_or(tokens.stroke),
            key,
            text,
            &text_style,
            constraints,
            raster_scale_factor,
            tokens,
        );
    }

    for (index, point) in overlays.drag_points.iter().enumerate() {
        let current = point.point;
        if !current.x.is_finite() || !current.y.is_finite() {
            continue;
        }
        let (transform, span) = match point.axis {
            crate::models::YAxis::Left => (Some(transform), y_span),
            crate::models::YAxis::Right => (
                Some(transform_y2.unwrap_or(transform)),
                view_bounds_y2
                    .map(|b| (b.y_max - b.y_min).abs())
                    .unwrap_or(y_span),
            ),
            crate::models::YAxis::Right2 => (
                Some(transform_y3.unwrap_or(transform)),
                view_bounds_y3
                    .map(|b| (b.y_max - b.y_min).abs())
                    .unwrap_or(y_span),
            ),
            crate::models::YAxis::Right3 => (
                Some(transform_y4.unwrap_or(transform)),
                view_bounds_y4
                    .map(|b| (b.y_max - b.y_min).abs())
                    .unwrap_or(y_span),
            ),
        };
        let Some(transform) = transform else {
            continue;
        };
        let p_px = transform.data_to_px(current);
        if !p_px.x.0.is_finite() || !p_px.y.0.is_finite() {
            continue;
        }
        let value = point.show_value.then(|| {
            let x = axis_tick_label_text(x_scale, &formatter, current.x, x_span);
            let y = axis_tick_label_text(y_scale, &formatter, current.y, span);
            format!("({x}, {y})")
        });
        let text = line_plot_annotation_label(point.label.as_deref(), value);
        if text.is_empty() {
            continue;
        }
        let margin = Px(8.0);
        let origin = Point::new(
            Px((p_px.x.0 + margin.0).round()),
            Px((p_px.y.0 - margin.0).round()),
        );
        let key: u64 = painter
            .child_key(
                scope,
                &(
                    "drag-point",
                    index,
                    point.id,
                    current.x.to_bits(),
                    current.y.to_bits(),
                    point.label.as_deref(),
                    point.show_value,
                    point.axis,
                    text.as_str(),
                ),
            )
            .into();
        paint_line_plot_annotation_text_box(
            painter,
            plot,
            origin,
            key,
            text,
            &text_style,
            constraints,
            raster_scale_factor,
            tokens.text,
            Some(tokens.background),
            Some(tokens.border),
            tokens.padding,
            tokens.radius,
        );
    }
}

fn paint_line_plot_tag_overlays(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    view_bounds: DataRect,
    view_bounds_y2: Option<DataRect>,
    view_bounds_y3: Option<DataRect>,
    view_bounds_y4: Option<DataRect>,
    overlays: &PlotOverlays,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    if overlays.tags_x.is_empty() && overlays.tags_y.is_empty() {
        return;
    }
    let Some(transform) = (PlotTransform {
        viewport: plot,
        data: view_bounds,
        x_scale,
        y_scale,
    })
    .prepare() else {
        return;
    };
    let transform_y2 = view_bounds_y2.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });
    let transform_y3 = view_bounds_y3.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });
    let transform_y4 = view_bounds_y4.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });

    let tokens = line_plot_annotation_tokens(painter, style);
    let text_style = TextStyle {
        size: Px(12.0),
        weight: FontWeight::NORMAL,
        ..TextStyle::default()
    };
    let constraints = CanvasTextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let raster_scale_factor = painter.scale_factor();
    let scope = painter.key_scope(&"fret-plot.declarative.tag-overlays");
    let formatter = AxisLabelFormatter::default();
    let x_span = (view_bounds.x_max - view_bounds.x_min).abs();
    let y_span = (view_bounds.y_max - view_bounds.y_min).abs();

    for (index, tag) in overlays.tags_x.iter().enumerate() {
        if !tag.x.is_finite() {
            continue;
        }
        let Some(x_px) = transform.data_x_to_px(tag.x) else {
            continue;
        };
        let value = tag
            .show_value
            .then(|| axis_tick_label_text(x_scale, &formatter, tag.x, x_span));
        let text = line_plot_annotation_label(tag.label.as_deref(), value);
        if text.is_empty() {
            continue;
        }
        let key: u64 = painter
            .child_key(
                scope,
                &(
                    "tag-x",
                    index,
                    tag.x.to_bits(),
                    tag.label.as_deref(),
                    tag.show_value,
                    text.as_str(),
                ),
            )
            .into();
        paint_line_plot_tag_x_overlay(
            painter,
            plot,
            Px(x_px.0.round()),
            tag.color.unwrap_or(tokens.stroke),
            key,
            text,
            &text_style,
            constraints,
            raster_scale_factor,
            tokens,
        );
    }

    for (index, tag) in overlays.tags_y.iter().enumerate() {
        if !tag.y.is_finite() {
            continue;
        }
        let (transform, right, span) = match tag.axis {
            crate::models::YAxis::Left => (Some(transform), false, y_span),
            crate::models::YAxis::Right => (
                Some(transform_y2.unwrap_or(transform)),
                true,
                view_bounds_y2
                    .map(|b| (b.y_max - b.y_min).abs())
                    .unwrap_or(y_span),
            ),
            crate::models::YAxis::Right2 => (
                Some(transform_y3.unwrap_or(transform)),
                true,
                view_bounds_y3
                    .map(|b| (b.y_max - b.y_min).abs())
                    .unwrap_or(y_span),
            ),
            crate::models::YAxis::Right3 => (
                Some(transform_y4.unwrap_or(transform)),
                true,
                view_bounds_y4
                    .map(|b| (b.y_max - b.y_min).abs())
                    .unwrap_or(y_span),
            ),
        };
        let Some(transform) = transform else {
            continue;
        };
        let Some(y_px) = transform.data_y_to_px(tag.y) else {
            continue;
        };
        let value = tag
            .show_value
            .then(|| axis_tick_label_text(y_scale, &formatter, tag.y, span));
        let text = line_plot_annotation_label(tag.label.as_deref(), value);
        if text.is_empty() {
            continue;
        }
        let key: u64 = painter
            .child_key(
                scope,
                &(
                    "tag-y",
                    index,
                    tag.y.to_bits(),
                    tag.label.as_deref(),
                    tag.show_value,
                    tag.axis,
                    text.as_str(),
                ),
            )
            .into();
        paint_line_plot_tag_y_overlay(
            painter,
            plot,
            Px(y_px.0.round()),
            right,
            tag.color.unwrap_or(tokens.stroke),
            key,
            text,
            &text_style,
            constraints,
            raster_scale_factor,
            tokens,
        );
    }
}

fn line_plot_annotation_label(label: Option<&str>, value: Option<String>) -> String {
    match (label, value) {
        (Some(label), Some(value)) => format!("{label}: {value}"),
        (Some(label), None) => label.to_owned(),
        (None, Some(value)) => value,
        (None, None) => String::new(),
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_line_plot_annotation_text_box(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    origin: Point,
    key: u64,
    text: String,
    text_style: &TextStyle,
    constraints: CanvasTextConstraints,
    raster_scale_factor: f32,
    color: Color,
    background: Option<Color>,
    border: Option<Color>,
    padding: Px,
    corner_radius: Px,
) {
    let (_blob, metrics) = painter.prepare_text_with_blob(
        key,
        text.clone(),
        text_style.clone(),
        constraints,
        raster_scale_factor,
    );
    let width = Px(metrics.size.width.0 + padding.0 * 2.0);
    let height = Px(metrics.size.height.0 + padding.0 * 2.0);
    if width.0 < 0.0 || height.0 < 0.0 {
        return;
    }
    let left = line_plot_clamp_plot_left(plot, origin.x.0, width);
    let top = line_plot_clamp_plot_top(plot, origin.y.0, height);
    let rect = Rect::new(Point::new(Px(left), Px(top)), Size::new(width, height));

    if let Some(background) = background {
        painter.scene().push(fret_core::SceneOp::Quad {
            order: DrawOrder(3),
            rect,
            background: Paint::Solid(background).into(),
            border: Edges::all(Px(1.0)),
            border_paint: Paint::Solid(border.unwrap_or(Color::TRANSPARENT)).into(),
            corner_radii: Corners::all(corner_radius),
        });
    }

    let _ = painter.text(
        key,
        DrawOrder(3),
        Point::new(
            Px(rect.origin.x.0 + padding.0),
            Px(rect.origin.y.0 + padding.0 + metrics.baseline.0),
        ),
        text,
        text_style.clone(),
        color,
        constraints,
        raster_scale_factor,
    );
}

#[allow(clippy::too_many_arguments)]
fn paint_line_plot_tag_x_overlay(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    x: Px,
    marker_color: Color,
    key: u64,
    text: String,
    text_style: &TextStyle,
    constraints: CanvasTextConstraints,
    raster_scale_factor: f32,
    tokens: LinePlotAnnotationTokens,
) {
    let (_blob, metrics) = painter.prepare_text_with_blob(
        key,
        text.clone(),
        text_style.clone(),
        constraints,
        raster_scale_factor,
    );
    let pad = tokens.padding;
    let width = Px(metrics.size.width.0 + pad.0 * 2.0);
    let height = Px(metrics.size.height.0 + pad.0 * 2.0);
    let margin = Px(6.0);
    let left = line_plot_clamp_plot_left(plot, x.0 - width.0 * 0.5, width);
    let top = line_plot_clamp_plot_top(
        plot,
        plot.origin.y.0 + plot.size.height.0 - height.0 - margin.0,
        height,
    );
    let rect = Rect::new(Point::new(Px(left), Px(top)), Size::new(width, height));

    painter.scene().push(fret_core::SceneOp::Quad {
        order: DrawOrder(3),
        rect,
        background: Paint::Solid(tokens.background).into(),
        border: Edges::all(Px(1.0)),
        border_paint: Paint::Solid(tokens.border).into(),
        corner_radii: Corners::all(tokens.radius),
    });
    let _ = painter.text(
        key,
        DrawOrder(3),
        Point::new(
            Px(rect.origin.x.0 + pad.0),
            Px(rect.origin.y.0 + pad.0 + metrics.baseline.0),
        ),
        text,
        text_style.clone(),
        tokens.text,
        constraints,
        raster_scale_factor,
    );

    let marker_width = Px(2.0);
    let marker_height = Px(8.0_f32.min(plot.size.height.0.max(0.0)));
    let marker_left = line_plot_clamp_plot_left(plot, x.0 - marker_width.0 * 0.5, marker_width);
    let marker_top = (plot.origin.y.0 + plot.size.height.0 - marker_height.0).max(plot.origin.y.0);
    push_filled_rect(
        painter,
        Rect::new(
            Point::new(Px(marker_left), Px(marker_top)),
            Size::new(marker_width, marker_height),
        ),
        DrawOrder(3),
        marker_color,
    );
}

#[allow(clippy::too_many_arguments)]
fn paint_line_plot_tag_y_overlay(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    y: Px,
    right: bool,
    marker_color: Color,
    key: u64,
    text: String,
    text_style: &TextStyle,
    constraints: CanvasTextConstraints,
    raster_scale_factor: f32,
    tokens: LinePlotAnnotationTokens,
) {
    let (_blob, metrics) = painter.prepare_text_with_blob(
        key,
        text.clone(),
        text_style.clone(),
        constraints,
        raster_scale_factor,
    );
    let pad = tokens.padding;
    let width = Px(metrics.size.width.0 + pad.0 * 2.0);
    let height = Px(metrics.size.height.0 + pad.0 * 2.0);
    let margin = Px(6.0);
    let left = if right {
        (plot.origin.x.0 + plot.size.width.0 - width.0 - margin.0).max(plot.origin.x.0)
    } else {
        plot.origin.x.0 + margin.0
    };
    let top = line_plot_clamp_plot_top(plot, y.0 - height.0 * 0.5, height);
    let rect = Rect::new(Point::new(Px(left), Px(top)), Size::new(width, height));

    painter.scene().push(fret_core::SceneOp::Quad {
        order: DrawOrder(3),
        rect,
        background: Paint::Solid(tokens.background).into(),
        border: Edges::all(Px(1.0)),
        border_paint: Paint::Solid(tokens.border).into(),
        corner_radii: Corners::all(tokens.radius),
    });
    let _ = painter.text(
        key,
        DrawOrder(3),
        Point::new(
            Px(rect.origin.x.0 + pad.0),
            Px(rect.origin.y.0 + pad.0 + metrics.baseline.0),
        ),
        text,
        text_style.clone(),
        tokens.text,
        constraints,
        raster_scale_factor,
    );

    let marker_height = Px(2.0);
    let marker_width = Px(8.0_f32.min(plot.size.width.0.max(0.0)));
    let marker_top = line_plot_clamp_plot_top(plot, y.0 - marker_height.0 * 0.5, marker_height);
    let marker_left = if right {
        (plot.origin.x.0 + plot.size.width.0 - marker_width.0).max(plot.origin.x.0)
    } else {
        plot.origin.x.0
    };
    push_filled_rect(
        painter,
        Rect::new(
            Point::new(Px(marker_left), Px(marker_top)),
            Size::new(marker_width, marker_height),
        ),
        DrawOrder(3),
        marker_color,
    );
}

fn line_plot_clamp_plot_left(plot: Rect, desired_left: f32, width: Px) -> f32 {
    desired_left.clamp(
        plot.origin.x.0,
        plot.origin.x.0 + (plot.size.width.0 - width.0).max(0.0),
    )
}

fn line_plot_clamp_plot_top(plot: Rect, desired_top: f32, height: Px) -> f32 {
    desired_top.clamp(
        plot.origin.y.0,
        plot.origin.y.0 + (plot.size.height.0 - height.0).max(0.0),
    )
}

fn paint_line_plot_text_overlays(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    view_bounds: DataRect,
    view_bounds_y2: Option<DataRect>,
    view_bounds_y3: Option<DataRect>,
    view_bounds_y4: Option<DataRect>,
    overlays: &PlotOverlays,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    if overlays.text.is_empty() {
        return;
    }
    let Some(transform) = (PlotTransform {
        viewport: plot,
        data: view_bounds,
        x_scale,
        y_scale,
    })
    .prepare() else {
        return;
    };
    let transform_y2 = view_bounds_y2.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });
    let transform_y3 = view_bounds_y3.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });
    let transform_y4 = view_bounds_y4.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });

    let tokens = line_plot_annotation_tokens(painter, style);
    let text_style = TextStyle {
        size: Px(12.0),
        weight: FontWeight::NORMAL,
        ..TextStyle::default()
    };
    let constraints = CanvasTextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let raster_scale_factor = painter.scale_factor();
    let scope = painter.key_scope(&"fret-plot.declarative.plot-text-overlays");

    for (index, text) in overlays.text.iter().enumerate() {
        if !text.x.is_finite() || !text.y.is_finite() {
            continue;
        }
        let (transform, right) = match text.axis {
            crate::models::YAxis::Left => (Some(transform), false),
            crate::models::YAxis::Right => (Some(transform_y2.unwrap_or(transform)), true),
            crate::models::YAxis::Right2 => (Some(transform_y3.unwrap_or(transform)), true),
            crate::models::YAxis::Right3 => (Some(transform_y4.unwrap_or(transform)), true),
        };
        let Some(transform) = transform else {
            continue;
        };
        let Some(px_x) = transform.data_x_to_px(text.x) else {
            continue;
        };
        let Some(px_y) = transform.data_y_to_px(text.y) else {
            continue;
        };
        let origin = Point::new(
            Px((px_x.0 + text.offset.x.0).round()),
            Px((px_y.0 + text.offset.y.0).round()),
        );
        let padding = if text.background.is_some() && text.padding.0 <= 0.0 {
            tokens.padding
        } else {
            text.padding
        };
        let corner_radius = if text.background.is_some() && text.corner_radius.0 <= 0.0 {
            tokens.radius
        } else {
            text.corner_radius
        };

        let key: u64 = painter
            .child_key(
                scope,
                &(
                    "plot-text",
                    index,
                    text.x.to_bits(),
                    text.y.to_bits(),
                    text.offset.x.0.to_bits(),
                    text.offset.y.0.to_bits(),
                    text.axis,
                    text.text.as_str(),
                ),
            )
            .into();
        let (_blob, metrics) = painter.prepare_text_with_blob(
            key,
            text.text.clone(),
            text_style.clone(),
            constraints,
            raster_scale_factor,
        );

        let width = Px(metrics.size.width.0 + padding.0 * 2.0);
        let height = Px(metrics.size.height.0 + padding.0 * 2.0);
        if width.0 < 0.0 || height.0 < 0.0 {
            continue;
        }
        let left = if right {
            line_plot_clamp_plot_left(
                plot,
                (plot.origin.x.0 + plot.size.width.0 - width.0 - tokens.padding.0)
                    .max(plot.origin.x.0),
                width,
            )
        } else {
            line_plot_clamp_plot_left(plot, origin.x.0, width)
        };
        let top = line_plot_clamp_plot_top(plot, origin.y.0, height);
        let rect = Rect::new(Point::new(Px(left), Px(top)), Size::new(width, height));

        if let Some(background) = text.background {
            painter.scene().push(fret_core::SceneOp::Quad {
                order: DrawOrder(3),
                rect,
                background: Paint::Solid(background).into(),
                border: Edges::all(Px(1.0)),
                border_paint: Paint::Solid(text.border.unwrap_or(tokens.border)).into(),
                corner_radii: Corners::all(corner_radius),
            });
        }

        let _ = painter.text(
            key,
            DrawOrder(3),
            Point::new(
                Px(rect.origin.x.0 + padding.0),
                Px(rect.origin.y.0 + padding.0 + metrics.baseline.0),
            ),
            text.text.clone(),
            text_style.clone(),
            text.color.unwrap_or(tokens.text),
            constraints,
            raster_scale_factor,
        );
    }
}

fn paint_line_plot_query_selection(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    view_bounds: DataRect,
    query_selection: Option<DataRect>,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    let Some(query) = query_selection else {
        return;
    };
    let transform = PlotTransform {
        viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size),
        data: view_bounds,
        x_scale,
        y_scale,
    };
    let a = transform.data_to_px(DataPoint {
        x: query.x_min,
        y: query.y_min,
    });
    let b = transform.data_to_px(DataPoint {
        x: query.x_max,
        y: query.y_max,
    });
    paint_line_plot_selection_rect_from_local(painter, plot, a, b, style);
}

fn paint_line_plot_active_selection(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    active_selection: Option<LinePlotSelectionOverlay>,
    style: LinePlotStyle,
) {
    let Some(selection) = active_selection else {
        return;
    };
    paint_line_plot_selection_rect_from_local(
        painter,
        plot,
        selection.start,
        selection.current,
        style,
    );
}

#[allow(clippy::too_many_arguments)]
fn paint_line_plot_selection_tooltip(
    painter: &mut CanvasPainter<'_>,
    bounds: Rect,
    plot: Rect,
    view_bounds: DataRect,
    active_selection: Option<LinePlotSelectionOverlay>,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> bool {
    let Some(selection) = active_selection else {
        return false;
    };
    let Some(text) =
        line_plot_selection_tooltip_text(view_bounds, plot.size, selection, x_scale, y_scale)
    else {
        return false;
    };

    let theme = painter.theme().snapshot();
    let tooltip_background = style
        .tooltip_background
        .unwrap_or_else(|| theme.color_required("popover"));
    let tooltip_border = style
        .tooltip_border
        .unwrap_or_else(|| theme.color_required("border"));
    let text_color = style
        .tooltip_text_color
        .or(style.label_color)
        .unwrap_or_else(|| theme.color_required("popover-foreground"));
    let text_style = TextStyle {
        size: Px(12.0),
        weight: FontWeight::NORMAL,
        ..TextStyle::default()
    };
    let constraints = CanvasTextConstraints {
        max_width: Some(Px(bounds.size.width.0.max(24.0))),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let raster_scale_factor = painter.scale_factor();
    let scope = painter.key_scope(&"fret-plot.declarative.selection-tooltip");
    let text_key: u64 = painter
        .child_key(
            scope,
            &(
                "text",
                line_plot_selection_kind_label(selection.kind),
                text.as_str(),
            ),
        )
        .into();
    let (_blob, metrics) = painter.prepare_text_with_blob(
        text_key,
        text.clone(),
        text_style.clone(),
        constraints,
        raster_scale_factor,
    );

    let pad = Px(6.0);
    let tooltip_size = Size::new(
        Px(metrics.size.width.0 + pad.0 * 2.0),
        Px(metrics.size.height.0 + pad.0 * 2.0),
    );
    let Some(rect) =
        line_plot_selection_tooltip_rect(bounds, plot, selection.current, tooltip_size)
    else {
        return false;
    };

    painter.scene().push(fret_core::SceneOp::Quad {
        order: DrawOrder(20),
        rect,
        background: Paint::Solid(tooltip_background).into(),
        border: Edges::all(Px(1.0)),
        border_paint: Paint::Solid(tooltip_border).into(),
        corner_radii: Corners::all(Px(6.0)),
    });
    let _ = painter.text(
        text_key,
        DrawOrder(21),
        Point::new(
            Px(rect.origin.x.0 + pad.0),
            Px(rect.origin.y.0 + pad.0 + metrics.baseline.0),
        ),
        text,
        text_style,
        text_color,
        constraints,
        raster_scale_factor,
    );
    true
}

fn line_plot_selection_tooltip_text(
    view_bounds: DataRect,
    plot_size: Size,
    selection: LinePlotSelectionOverlay,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Option<String> {
    let rect = line_plot_query_rect_from_plot_points_raw(
        view_bounds,
        plot_size,
        selection.start,
        selection.current,
        x_scale,
        y_scale,
    )?;
    let x_span = (view_bounds.x_max - view_bounds.x_min).abs();
    let y_span = (view_bounds.y_max - view_bounds.y_min).abs();
    let formatter = AxisLabelFormatter::default();
    let x0 = axis_tick_label_text(x_scale, &formatter, rect.x_min, x_span);
    let x1 = axis_tick_label_text(x_scale, &formatter, rect.x_max, x_span);
    let y0 = axis_tick_label_text(y_scale, &formatter, rect.y_min, y_span);
    let y1 = axis_tick_label_text(y_scale, &formatter, rect.y_max, y_span);
    Some(format!(
        "{}\nx=[{x0}, {x1}]\ny=[{y0}, {y1}]",
        line_plot_selection_kind_label(selection.kind)
    ))
}

fn line_plot_selection_kind_label(kind: LinePlotSelectionKind) -> &'static str {
    match kind {
        LinePlotSelectionKind::Query => "query",
        LinePlotSelectionKind::BoxZoom => "zoom",
    }
}

fn line_plot_selection_tooltip_rect(
    bounds: Rect,
    plot: Rect,
    anchor_local: Point,
    size: Size,
) -> Option<Rect> {
    if bounds.size.width.0 <= 0.0 || bounds.size.height.0 <= 0.0 {
        return None;
    }
    if size.width.0 <= 0.0 || size.height.0 <= 0.0 {
        return None;
    }

    let anchor = Point::new(
        Px(plot.origin.x.0 + anchor_local.x.0),
        Px(plot.origin.y.0 + anchor_local.y.0),
    );
    let gap = 10.0;
    let mut x = anchor.x.0 + gap;
    let mut y = anchor.y.0 + gap;
    let bounds_right = bounds.origin.x.0 + bounds.size.width.0;
    let bounds_bottom = bounds.origin.y.0 + bounds.size.height.0;
    if x + size.width.0 > bounds_right {
        x = anchor.x.0 - gap - size.width.0;
    }
    if y + size.height.0 > bounds_bottom {
        y = anchor.y.0 - gap - size.height.0;
    }

    let min_x = bounds.origin.x.0;
    let min_y = bounds.origin.y.0;
    let max_x = (bounds_right - size.width.0).max(min_x);
    let max_y = (bounds_bottom - size.height.0).max(min_y);
    Some(Rect::new(
        Point::new(Px(x.clamp(min_x, max_x)), Px(y.clamp(min_y, max_y))),
        size,
    ))
}

fn paint_line_plot_selection_rect_from_local(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    start: Point,
    end: Point,
    style: LinePlotStyle,
) {
    let Some(rect) = line_plot_selection_rect_from_local(plot, start, end) else {
        return;
    };
    let (selection_border, selection_fill) = line_plot_selection_colors(painter, style);
    painter.scene().push(fret_core::SceneOp::Quad {
        order: DrawOrder(5),
        rect,
        background: Paint::Solid(selection_fill).into(),
        border: Edges::all(Px(1.0)),
        border_paint: Paint::Solid(selection_border).into(),
        corner_radii: Corners::default(),
    });
}

fn line_plot_selection_rect_from_local(plot: Rect, start: Point, end: Point) -> Option<Rect> {
    let x0 = start.x.0.min(end.x.0).clamp(0.0, plot.size.width.0);
    let x1 = start.x.0.max(end.x.0).clamp(0.0, plot.size.width.0);
    let y0 = start.y.0.min(end.y.0).clamp(0.0, plot.size.height.0);
    let y1 = start.y.0.max(end.y.0).clamp(0.0, plot.size.height.0);
    let width = x1 - x0;
    let height = y1 - y0;
    (width >= 1.0 && height >= 1.0).then(|| {
        Rect::new(
            Point::new(Px(plot.origin.x.0 + x0), Px(plot.origin.y.0 + y0)),
            Size::new(Px(width), Px(height)),
        )
    })
}

fn line_plot_selection_colors(
    painter: &mut CanvasPainter<'_>,
    style: LinePlotStyle,
) -> (Color, Color) {
    let theme = painter.theme().snapshot();
    let selection_border = style
        .crosshair_color
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    let selection_fill = Color {
        a: (selection_border.a * 0.18).clamp(0.06, 0.22),
        ..selection_border
    };
    (selection_border, selection_fill)
}

fn handle_line_plot_legend_pointer_event<H: UiHost>(
    app: &mut H,
    state: &Model<PlotState>,
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
) -> bool {
    let Event::Pointer(fret_core::PointerEvent::Down {
        position,
        button: MouseButton::Left,
        modifiers,
        ..
    }) = event
    else {
        return false;
    };

    let plot = line_plot_inner_rect(bounds, style);
    let Some((series_id, hit)) = line_plot_legend_hit(model, plot, *position) else {
        return false;
    };

    state
        .update(app, |state, _cx| match hit {
            _ if modifiers.shift => {
                let ids: Vec<SeriesId> = model.series.iter().map(|series| series.id).collect();
                let visible_count = ids
                    .iter()
                    .filter(|series_id| !state.hidden_series.contains(series_id))
                    .count();
                let is_solo = visible_count == 1 && !state.hidden_series.contains(&series_id);
                if is_solo {
                    state.hidden_series.clear();
                } else {
                    state.hidden_series = ids.into_iter().filter(|id| *id != series_id).collect();
                    state.hidden_series.remove(&series_id);
                }
                true
            }
            LinePlotLegendHit::Swatch => {
                let total = model.series.len();
                let hidden_count = model
                    .series
                    .iter()
                    .filter(|series| state.hidden_series.contains(&series.id))
                    .count();
                let visible_count = total.saturating_sub(hidden_count);
                if state.hidden_series.contains(&series_id) {
                    state.hidden_series.remove(&series_id);
                    state.pinned_series = state.pinned_series.filter(|id| *id != series_id);
                    true
                } else if visible_count <= 1 {
                    false
                } else {
                    state.hidden_series.insert(series_id);
                    state.pinned_series = state.pinned_series.filter(|id| *id != series_id);
                    true
                }
            }
            LinePlotLegendHit::Label => {
                if state.pinned_series == Some(series_id) {
                    state.pinned_series = None;
                } else {
                    state.pinned_series = Some(series_id);
                    state.hidden_series.remove(&series_id);
                }
                true
            }
        })
        .ok()
        .unwrap_or(false)
}

fn line_plot_panel_event_snapshot(
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
    view_bounds: DataRect,
    query: Option<DataRect>,
) -> Option<PlotOutputSnapshot> {
    let Event::Pointer(fret_core::PointerEvent::Move { position, .. }) = event else {
        return None;
    };
    Some(line_plot_pointer_output_snapshot(
        *position,
        bounds,
        model,
        style,
        x_scale,
        y_scale,
        view_bounds,
        query,
    ))
}

#[allow(clippy::too_many_arguments)]
fn handle_line_plot_draggable_overlay_event<H: UiHost>(
    app: &mut H,
    state: &Model<PlotState>,
    drag_session: &Rc<RefCell<Option<LinePlotDragSession>>>,
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Option<PlotDragOutput> {
    let plot = line_plot_inner_rect(bounds, style);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return None;
    }

    match event {
        Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button,
            modifiers,
            ..
        }) if plot.contains(*position)
            && PlotInputMap::default().pan.matches(*button, *modifiers)
            && line_plot_legend_hit(model, plot, *position).is_none() =>
        {
            let view_bounds = line_plot_current_view_bounds_for_event(
                app,
                Some(state),
                model,
                style,
                x_scale,
                y_scale,
            );
            let overlays = state
                .read_ref(app, |state| state.overlays.clone())
                .unwrap_or_default();
            let local = local_from_absolute(plot.origin, *position);
            let threshold = style.hover_threshold.0.max(1.0);
            let mut best: Option<(f32, LinePlotDragSession)> = None;

            for point in &overlays.drag_points {
                if !point.point.x.is_finite() || !point.point.y.is_finite() {
                    continue;
                }
                let Some(transform) = line_plot_transform_for_y_axis(
                    plot,
                    view_bounds,
                    model,
                    point.axis,
                    x_scale,
                    y_scale,
                ) else {
                    continue;
                };
                let p_px = transform.data_to_px(point.point);
                if !p_px.x.0.is_finite() || !p_px.y.0.is_finite() {
                    continue;
                }
                let hit_r = point.radius.0.max(threshold);
                let dx = local.x.0 - p_px.x.0;
                let dy = local.y.0 - p_px.y.0;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > hit_r {
                    continue;
                }
                let data = transform.px_to_data(local);
                if !data.x.is_finite() || !data.y.is_finite() {
                    continue;
                };
                let candidate = LinePlotDragSession::Point {
                    id: point.id,
                    axis: point.axis,
                    button: *button,
                    offset: DataPoint {
                        x: data.x - point.point.x,
                        y: data.y - point.point.y,
                    },
                    current: point.point,
                };
                if best.as_ref().is_none_or(|(best_dist, _)| dist < *best_dist) {
                    best = Some((dist, candidate));
                }
            }

            if best.is_none() {
                let transform_x =
                    line_plot_transform_for_x_axis(plot, view_bounds, x_scale, y_scale);
                for line in &overlays.drag_lines_x {
                    if !line.x.is_finite() {
                        continue;
                    }
                    let Some(x_px) = transform_x.data_x_to_px(line.x) else {
                        continue;
                    };
                    let dist = (local.x.0 - x_px.0).abs();
                    if dist > threshold {
                        continue;
                    }
                    let data = transform_x.px_to_data(local);
                    if !data.x.is_finite() {
                        continue;
                    }
                    let candidate = LinePlotDragSession::LineX {
                        id: line.id,
                        button: *button,
                        offset_x: data.x - line.x,
                        current_x: line.x,
                    };
                    if best.as_ref().is_none_or(|(best_dist, _)| dist < *best_dist) {
                        best = Some((dist, candidate));
                    }
                }

                for line in &overlays.drag_lines_y {
                    if !line.y.is_finite() {
                        continue;
                    }
                    let Some(transform) = line_plot_transform_for_y_axis(
                        plot,
                        view_bounds,
                        model,
                        line.axis,
                        x_scale,
                        y_scale,
                    ) else {
                        continue;
                    };
                    let Some(y_px) = transform.data_y_to_px(line.y) else {
                        continue;
                    };
                    let dist = (local.y.0 - y_px.0).abs();
                    if dist > threshold {
                        continue;
                    }
                    let data = transform.px_to_data(local);
                    if !data.y.is_finite() {
                        continue;
                    }
                    let candidate = LinePlotDragSession::LineY {
                        id: line.id,
                        axis: line.axis,
                        button: *button,
                        offset_y: data.y - line.y,
                        current_y: line.y,
                    };
                    if best.as_ref().is_none_or(|(best_dist, _)| dist < *best_dist) {
                        best = Some((dist, candidate));
                    }
                }

                for rect in &overlays.drag_rects {
                    let Some(transform) = line_plot_transform_for_y_axis(
                        plot,
                        view_bounds,
                        model,
                        rect.axis,
                        x_scale,
                        y_scale,
                    ) else {
                        continue;
                    };
                    let a = transform.data_to_px(DataPoint {
                        x: rect.rect.x_min,
                        y: rect.rect.y_min,
                    });
                    let b = transform.data_to_px(DataPoint {
                        x: rect.rect.x_max,
                        y: rect.rect.y_max,
                    });
                    if !a.x.0.is_finite()
                        || !a.y.0.is_finite()
                        || !b.x.0.is_finite()
                        || !b.y.0.is_finite()
                    {
                        continue;
                    }

                    let left = a.x.0.min(b.x.0);
                    let right = a.x.0.max(b.x.0);
                    let top = a.y.0.min(b.y.0);
                    let bottom = a.y.0.max(b.y.0);
                    let inside = local.x.0 >= left
                        && local.x.0 <= right
                        && local.y.0 >= top
                        && local.y.0 <= bottom;
                    if !inside {
                        continue;
                    }

                    let dist_left = (local.x.0 - left).abs();
                    let dist_right = (local.x.0 - right).abs();
                    let dist_top = (local.y.0 - top).abs();
                    let dist_bottom = (local.y.0 - bottom).abs();
                    let mut handle = LinePlotDragRectHandle::Inside;
                    let mut dist = 0.0f32;
                    let mut set_handle = |d: f32, h: LinePlotDragRectHandle| {
                        if d <= threshold && (handle == LinePlotDragRectHandle::Inside || d < dist)
                        {
                            handle = h;
                            dist = d;
                        }
                    };
                    set_handle(dist_left, LinePlotDragRectHandle::Left);
                    set_handle(dist_right, LinePlotDragRectHandle::Right);
                    set_handle(dist_top, LinePlotDragRectHandle::Top);
                    set_handle(dist_bottom, LinePlotDragRectHandle::Bottom);

                    let data = transform.px_to_data(local);
                    if !data.x.is_finite() || !data.y.is_finite() {
                        continue;
                    }
                    let offset = match handle {
                        LinePlotDragRectHandle::Inside => DataPoint {
                            x: data.x - rect.rect.x_min,
                            y: data.y - rect.rect.y_min,
                        },
                        LinePlotDragRectHandle::Left => DataPoint {
                            x: data.x - rect.rect.x_min,
                            y: 0.0,
                        },
                        LinePlotDragRectHandle::Right => DataPoint {
                            x: data.x - rect.rect.x_max,
                            y: 0.0,
                        },
                        LinePlotDragRectHandle::Top => DataPoint {
                            x: 0.0,
                            y: data.y - rect.rect.y_max,
                        },
                        LinePlotDragRectHandle::Bottom => DataPoint {
                            x: 0.0,
                            y: data.y - rect.rect.y_min,
                        },
                    };
                    let candidate = LinePlotDragSession::Rect {
                        id: rect.id,
                        axis: rect.axis,
                        button: *button,
                        handle,
                        offset,
                        start: rect.rect,
                        current: rect.rect,
                    };
                    if best.as_ref().is_none_or(|(best_dist, _)| dist < *best_dist) {
                        best = Some((dist, candidate));
                    }
                }
            }

            let (_, session) = best?;
            *drag_session.borrow_mut() = Some(session);
            Some(line_plot_drag_output(session, PlotDragPhase::Start))
        }
        Event::Pointer(fret_core::PointerEvent::Move {
            position, buttons, ..
        }) => {
            let Some(mut session) = *drag_session.borrow() else {
                return None;
            };
            let button = match session {
                LinePlotDragSession::LineX { button, .. } => button,
                LinePlotDragSession::LineY { button, .. } => button,
                LinePlotDragSession::Point { button, .. } => button,
                LinePlotDragSession::Rect { button, .. } => button,
            };
            let phase = if line_plot_mouse_buttons_contains(*buttons, button) {
                PlotDragPhase::Update
            } else {
                drag_session.borrow_mut().take();
                PlotDragPhase::End
            };
            line_plot_update_drag_session_at_position(
                &mut session,
                *position,
                plot,
                line_plot_current_view_bounds_for_event(
                    app,
                    Some(state),
                    model,
                    style,
                    x_scale,
                    y_scale,
                ),
                model,
                x_scale,
                y_scale,
            );
            if phase != PlotDragPhase::End {
                *drag_session.borrow_mut() = Some(session);
            }
            Some(line_plot_drag_output(session, phase))
        }
        Event::Pointer(fret_core::PointerEvent::Up {
            position, button, ..
        }) => {
            let mut session = drag_session.borrow_mut().take()?;
            let session_button = match session {
                LinePlotDragSession::LineX { button, .. } => button,
                LinePlotDragSession::LineY { button, .. } => button,
                LinePlotDragSession::Point { button, .. } => button,
                LinePlotDragSession::Rect { button, .. } => button,
            };
            if session_button != *button {
                *drag_session.borrow_mut() = Some(session);
                return None;
            }
            line_plot_update_drag_session_at_position(
                &mut session,
                *position,
                plot,
                line_plot_current_view_bounds_for_event(
                    app,
                    Some(state),
                    model,
                    style,
                    x_scale,
                    y_scale,
                ),
                model,
                x_scale,
                y_scale,
            );
            Some(line_plot_drag_output(session, PlotDragPhase::End))
        }
        _ => None,
    }
}

fn line_plot_update_drag_session_at_position(
    session: &mut LinePlotDragSession,
    position: Point,
    plot: Rect,
    view_bounds: DataRect,
    model: &PlotPanelModel,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    match session {
        LinePlotDragSession::LineX {
            offset_x,
            current_x,
            ..
        } => {
            let transform = line_plot_transform_for_x_axis(plot, view_bounds, x_scale, y_scale);
            let local = local_from_absolute(plot.origin, position);
            let data = transform.px_to_data(local);
            if data.x.is_finite() {
                *current_x = data.x - *offset_x;
            }
        }
        LinePlotDragSession::LineY {
            axis,
            offset_y,
            current_y,
            ..
        } => {
            let Some(transform) =
                line_plot_transform_for_y_axis(plot, view_bounds, model, *axis, x_scale, y_scale)
            else {
                return;
            };
            let local = local_from_absolute(plot.origin, position);
            let data = transform.px_to_data(local);
            if data.y.is_finite() {
                *current_y = data.y - *offset_y;
            }
        }
        LinePlotDragSession::Point {
            axis,
            offset,
            current,
            ..
        } => {
            let Some(transform) =
                line_plot_transform_for_y_axis(plot, view_bounds, model, *axis, x_scale, y_scale)
            else {
                return;
            };
            let local = local_from_absolute(plot.origin, position);
            let data = transform.px_to_data(local);
            if data.x.is_finite() && data.y.is_finite() {
                *current = DataPoint {
                    x: data.x - offset.x,
                    y: data.y - offset.y,
                };
            }
        }
        LinePlotDragSession::Rect {
            axis,
            handle,
            offset,
            start,
            current,
            ..
        } => {
            let Some(transform) =
                line_plot_transform_for_y_axis(plot, view_bounds, model, *axis, x_scale, y_scale)
            else {
                return;
            };
            let local = local_from_absolute(plot.origin, position);
            let data = transform.px_to_data(local);
            if !data.x.is_finite() || !data.y.is_finite() {
                return;
            }

            let mut next = *current;
            match handle {
                LinePlotDragRectHandle::Inside => {
                    let w = start.width();
                    let h = start.height();
                    next.x_min = data.x - offset.x;
                    next.x_max = next.x_min + w;
                    next.y_min = data.y - offset.y;
                    next.y_max = next.y_min + h;
                }
                LinePlotDragRectHandle::Left => {
                    next.x_min = data.x - offset.x;
                    if next.x_min > next.x_max {
                        next.x_min = next.x_max;
                    }
                }
                LinePlotDragRectHandle::Right => {
                    next.x_max = data.x - offset.x;
                    if next.x_max < next.x_min {
                        next.x_max = next.x_min;
                    }
                }
                LinePlotDragRectHandle::Top => {
                    next.y_max = data.y - offset.y;
                    if next.y_max < next.y_min {
                        next.y_max = next.y_min;
                    }
                }
                LinePlotDragRectHandle::Bottom => {
                    next.y_min = data.y - offset.y;
                    if next.y_min > next.y_max {
                        next.y_min = next.y_max;
                    }
                }
            }
            *current = next;
        }
    }
}

fn line_plot_drag_output(session: LinePlotDragSession, phase: PlotDragPhase) -> PlotDragOutput {
    match session {
        LinePlotDragSession::LineX { id, current_x, .. } => PlotDragOutput::LineX {
            id,
            x: current_x,
            phase,
        },
        LinePlotDragSession::LineY {
            id,
            axis,
            current_y,
            ..
        } => PlotDragOutput::LineY {
            id,
            axis,
            y: current_y,
            phase,
        },
        LinePlotDragSession::Point {
            id, axis, current, ..
        } => PlotDragOutput::Point {
            id,
            axis,
            point: current,
            phase,
        },
        LinePlotDragSession::Rect {
            id, axis, current, ..
        } => PlotDragOutput::Rect {
            id,
            axis,
            rect: current,
            phase,
        },
    }
}

fn line_plot_transform_for_x_axis(
    plot: Rect,
    view_bounds: DataRect,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> PlotTransform {
    PlotTransform {
        viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size),
        data: view_bounds,
        x_scale,
        y_scale,
    }
}

fn line_plot_transform_for_y_axis(
    plot: Rect,
    primary_view_bounds: DataRect,
    model: &PlotPanelModel,
    axis: YAxis,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Option<PlotTransform> {
    let data = match axis {
        YAxis::Left => primary_view_bounds,
        YAxis::Right => {
            line_plot_view_bounds_for_y_axis(primary_view_bounds, model.data_bounds_y2?)
        }
        YAxis::Right2 => {
            line_plot_view_bounds_for_y_axis(primary_view_bounds, model.data_bounds_y3?)
        }
        YAxis::Right3 => {
            line_plot_view_bounds_for_y_axis(primary_view_bounds, model.data_bounds_y4?)
        }
    };
    Some(PlotTransform {
        viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size),
        data,
        x_scale,
        y_scale,
    })
}

#[allow(clippy::too_many_arguments)]
fn handle_line_plot_query_drag_event<H: UiHost>(
    app: &mut H,
    state: &Model<PlotState>,
    query_drag_session: &Rc<RefCell<Option<LinePlotQueryDragSession>>>,
    active_selection: &Rc<Cell<Option<LinePlotSelectionOverlay>>>,
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> bool {
    let plot = line_plot_inner_rect(bounds, style);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return false;
    }

    let input_map = PlotInputMap::default();
    match event {
        Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button,
            modifiers,
            ..
        }) if plot.contains(*position)
            && input_map
                .query_drag
                .is_some_and(|chord| chord.matches(*button, *modifiers)) =>
        {
            let local = local_from_absolute(plot.origin, *position);
            *query_drag_session.borrow_mut() = Some(LinePlotQueryDragSession {
                start: local,
                current: local,
                button: *button,
            });
            active_selection.set(Some(LinePlotSelectionOverlay {
                start: local,
                current: local,
                kind: LinePlotSelectionKind::Query,
            }));
            true
        }
        Event::Pointer(fret_core::PointerEvent::Move {
            position, buttons, ..
        }) => {
            let Some(mut session) = *query_drag_session.borrow() else {
                return false;
            };
            if !line_plot_mouse_buttons_contains(*buttons, session.button) {
                query_drag_session.borrow_mut().take();
                active_selection.set(None);
                return true;
            }
            session.current = local_from_absolute(plot.origin, *position);
            active_selection.set(Some(LinePlotSelectionOverlay {
                start: session.start,
                current: session.current,
                kind: LinePlotSelectionKind::Query,
            }));
            *query_drag_session.borrow_mut() = Some(session);
            true
        }
        Event::Pointer(fret_core::PointerEvent::Up {
            position, button, ..
        }) => {
            let Some(mut session) = query_drag_session.borrow_mut().take() else {
                return false;
            };
            if session.button != *button {
                *query_drag_session.borrow_mut() = Some(session);
                return false;
            }
            session.current = local_from_absolute(plot.origin, *position);
            active_selection.set(None);
            let w = (session.start.x.0 - session.current.x.0).abs();
            let h = (session.start.y.0 - session.current.y.0).abs();
            if w < 4.0 || h < 4.0 {
                return true;
            }

            let current_view = line_plot_current_view_bounds_for_event(
                app,
                Some(state),
                model,
                style,
                x_scale,
                y_scale,
            );
            let Some(next) = line_plot_query_rect_from_plot_points_raw(
                current_view,
                plot.size,
                session.start,
                session.current,
                x_scale,
                y_scale,
            ) else {
                return true;
            };

            state
                .update(app, |state, _cx| {
                    state.query = Some(next);
                    true
                })
                .ok()
                .unwrap_or(false)
        }
        _ => false,
    }
}

fn line_plot_query_rect_from_plot_points_raw(
    view_bounds: DataRect,
    viewport: Size,
    a: Point,
    b: Point,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Option<DataRect> {
    let viewport_w = viewport.width.0;
    let viewport_h = viewport.height.0;
    if !viewport_w.is_finite() || !viewport_h.is_finite() || viewport_w <= 0.0 || viewport_h <= 0.0
    {
        return None;
    }

    let x0 = a.x.0.min(b.x.0).clamp(0.0, viewport_w);
    let x1 = a.x.0.max(b.x.0).clamp(0.0, viewport_w);
    let y0 = a.y.0.min(b.y.0).clamp(0.0, viewport_h);
    let y1 = a.y.0.max(b.y.0).clamp(0.0, viewport_h);

    let transform = PlotTransform {
        viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), viewport),
        data: view_bounds,
        x_scale,
        y_scale,
    };
    let a = transform.px_to_data(Point::new(Px(x0), Px(y0)));
    let b = transform.px_to_data(Point::new(Px(x1), Px(y1)));
    if !a.x.is_finite() || !a.y.is_finite() || !b.x.is_finite() || !b.y.is_finite() {
        return None;
    }

    Some(DataRect {
        x_min: a.x.min(b.x),
        x_max: a.x.max(b.x),
        y_min: a.y.min(b.y),
        y_max: a.y.max(b.y),
    })
}

#[allow(clippy::too_many_arguments)]
fn handle_line_plot_box_zoom_event<H: UiHost>(
    app: &mut H,
    state: &Model<PlotState>,
    box_zoom_session: &Rc<RefCell<Option<LinePlotBoxZoomSession>>>,
    active_selection: &Rc<Cell<Option<LinePlotSelectionOverlay>>>,
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> bool {
    let plot = line_plot_inner_rect(bounds, style);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return false;
    }

    let input_map = PlotInputMap::default();
    match event {
        Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button,
            modifiers,
            ..
        }) if plot.contains(*position) => {
            let start_box_primary = input_map.box_zoom.matches(*button, *modifiers);
            let start_box_alt = input_map
                .box_zoom_alt
                .is_some_and(|chord| chord.matches(*button, *modifiers));
            if !start_box_primary && !start_box_alt {
                return false;
            }
            if line_plot_legend_hit(model, plot, *position).is_some() {
                return false;
            }

            let local = local_from_absolute(plot.origin, *position);
            *box_zoom_session.borrow_mut() = Some(LinePlotBoxZoomSession {
                start: local,
                current: local,
                button: *button,
                required_mods: if start_box_primary {
                    input_map.box_zoom.modifiers
                } else {
                    input_map
                        .box_zoom_alt
                        .unwrap_or(input_map.box_zoom)
                        .modifiers
                },
            });
            active_selection.set(Some(LinePlotSelectionOverlay {
                start: local,
                current: local,
                kind: LinePlotSelectionKind::BoxZoom,
            }));
            true
        }
        Event::Pointer(fret_core::PointerEvent::Move {
            position, buttons, ..
        }) => {
            let Some(mut session) = *box_zoom_session.borrow() else {
                return false;
            };
            if !line_plot_mouse_buttons_contains(*buttons, session.button) {
                box_zoom_session.borrow_mut().take();
                active_selection.set(None);
                return true;
            }
            session.current = local_from_absolute(plot.origin, *position);
            active_selection.set(Some(LinePlotSelectionOverlay {
                start: session.start,
                current: session.current,
                kind: LinePlotSelectionKind::BoxZoom,
            }));
            *box_zoom_session.borrow_mut() = Some(session);
            true
        }
        Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button,
            modifiers,
            ..
        }) => {
            let Some(mut session) = box_zoom_session.borrow_mut().take() else {
                return false;
            };
            if session.button != *button {
                *box_zoom_session.borrow_mut() = Some(session);
                return false;
            }
            session.current = local_from_absolute(plot.origin, *position);
            active_selection.set(None);
            let (start, end) = line_plot_apply_box_select_modifiers(
                plot.size,
                session.start,
                session.current,
                *modifiers,
                input_map.box_zoom_expand_x,
                input_map.box_zoom_expand_y,
                session.required_mods,
            );
            let w = (start.x.0 - end.x.0).abs();
            let h = (start.y.0 - end.y.0).abs();
            if w < 4.0 || h < 4.0 {
                return true;
            }

            let current_view = line_plot_current_view_bounds_for_event(
                app,
                Some(state),
                model,
                style,
                x_scale,
                y_scale,
            );
            let axis_locks = state
                .read_ref(app, |state| state.axis_locks)
                .unwrap_or_default();
            if axis_locks.x.zoom && axis_locks.y.zoom {
                return true;
            }

            let Some(mut next) = data_rect_from_plot_points_scaled(
                current_view,
                plot.size,
                start,
                end,
                x_scale,
                y_scale,
            ) else {
                return true;
            };
            if style.clamp_to_data_bounds {
                next = clamp_view_to_data_scaled(
                    next,
                    model.data_bounds,
                    style.overscroll_fraction,
                    x_scale,
                    y_scale,
                );
            }
            if axis_locks.x.zoom {
                next.x_min = current_view.x_min;
                next.x_max = current_view.x_max;
            }
            if axis_locks.y.zoom {
                next.y_min = current_view.y_min;
                next.y_max = current_view.y_max;
            }
            next = sanitize_data_rect_scaled(next, x_scale, y_scale);
            if next == current_view {
                return true;
            }

            state
                .update(app, |state, _cx| {
                    state.view_is_auto = false;
                    state.view_bounds = Some(next);
                    true
                })
                .ok()
                .unwrap_or(false)
        }
        _ => false,
    }
}

fn line_plot_mouse_buttons_contains(buttons: fret_core::MouseButtons, button: MouseButton) -> bool {
    match button {
        MouseButton::Left => buttons.left,
        MouseButton::Right => buttons.right,
        MouseButton::Middle => buttons.middle,
        MouseButton::Back | MouseButton::Forward | MouseButton::Other(_) => false,
    }
}

fn line_plot_apply_box_select_modifiers(
    plot_size: Size,
    start: Point,
    end: Point,
    modifiers: fret_core::Modifiers,
    expand_x: Option<ModifierKey>,
    expand_y: Option<ModifierKey>,
    required: ModifiersMask,
) -> (Point, Point) {
    let mut start = start;
    let mut end = end;

    if expand_x.is_some_and(|key| key.is_pressed(modifiers) && !key.is_required_by(required)) {
        start.x = Px(0.0);
        end.x = plot_size.width;
    }
    if expand_y.is_some_and(|key| key.is_pressed(modifiers) && !key.is_required_by(required)) {
        start.y = Px(0.0);
        end.y = plot_size.height;
    }

    (start, end)
}

#[allow(clippy::too_many_arguments)]
fn handle_line_plot_pan_event<H: UiHost>(
    app: &mut H,
    state: &Model<PlotState>,
    pan_session: &Rc<RefCell<Option<LinePlotPanSession>>>,
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> bool {
    let plot = line_plot_inner_rect(bounds, style);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return false;
    }

    match event {
        Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: MouseButton::Left,
            modifiers,
            ..
        }) if !modifiers.shift && !modifiers.alt && !modifiers.ctrl && plot.contains(*position) => {
            if line_plot_legend_hit(model, plot, *position).is_some() {
                return false;
            }
            *pan_session.borrow_mut() = Some(LinePlotPanSession {
                last_position: *position,
            });
            true
        }
        Event::Pointer(fret_core::PointerEvent::Move {
            position, buttons, ..
        }) if buttons.left => {
            let Some(mut session) = *pan_session.borrow() else {
                return false;
            };
            let current_view = line_plot_current_view_bounds_for_event(
                app,
                Some(state),
                model,
                style,
                x_scale,
                y_scale,
            );
            let dx_px = position.x.0 - session.last_position.x.0;
            let dy_px = position.y.0 - session.last_position.y.0;
            if dx_px == 0.0 && dy_px == 0.0 {
                return true;
            }
            let mut next =
                pan_line_plot_view_bounds(current_view, plot, dx_px, dy_px, x_scale, y_scale);
            let axis_locks = state
                .read_ref(app, |state| state.axis_locks)
                .unwrap_or_default();
            if axis_locks.x.pan {
                next.x_min = current_view.x_min;
                next.x_max = current_view.x_max;
            }
            if axis_locks.y.pan {
                next.y_min = current_view.y_min;
                next.y_max = current_view.y_max;
            }
            let _ = state.update(app, |state, _cx| {
                state.view_is_auto = false;
                state.view_bounds = Some(next);
            });
            session.last_position = *position;
            *pan_session.borrow_mut() = Some(session);
            true
        }
        Event::Pointer(fret_core::PointerEvent::Move { buttons, .. }) if !buttons.left => {
            pan_session.borrow_mut().take().is_some()
        }
        Event::Pointer(fret_core::PointerEvent::Up {
            button: MouseButton::Left,
            ..
        }) => pan_session.borrow_mut().take().is_some(),
        _ => false,
    }
}

fn pan_line_plot_view_bounds(
    view: DataRect,
    plot: Rect,
    dx_px: f32,
    dy_px: f32,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> DataRect {
    let pan_axis = |scale: AxisScale, min: f64, max: f64, delta_px: f32, span_px: f32| {
        let Some(axis_min) = scale.to_axis(min) else {
            return (min, max);
        };
        let Some(axis_max) = scale.to_axis(max) else {
            return (min, max);
        };
        if span_px <= 0.0 {
            return (min, max);
        }
        let axis_delta = -(delta_px as f64) / span_px as f64 * (axis_max - axis_min);
        (
            scale.from_axis(axis_min + axis_delta).unwrap_or(min),
            scale.from_axis(axis_max + axis_delta).unwrap_or(max),
        )
    };
    let (x_min, x_max) = pan_axis(x_scale, view.x_min, view.x_max, dx_px, plot.size.width.0);
    let (y_min, y_max) = pan_axis(y_scale, view.y_min, view.y_max, -dy_px, plot.size.height.0);
    sanitize_data_rect_scaled(
        DataRect {
            x_min,
            x_max,
            y_min,
            y_max,
        },
        x_scale,
        y_scale,
    )
}

#[allow(clippy::too_many_arguments)]
fn handle_line_plot_wheel_zoom_event<H: UiHost>(
    app: &mut H,
    state: &Model<PlotState>,
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> bool {
    let Event::Pointer(fret_core::PointerEvent::Wheel {
        position,
        delta,
        modifiers,
        ..
    }) = event
    else {
        return false;
    };

    let Some(region) = line_plot_wheel_region_at(bounds, style, *position) else {
        return false;
    };
    let plot = line_plot_inner_rect(bounds, style);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return false;
    }

    let input_map = PlotInputMap::default();
    if let Some(required) = input_map.wheel_zoom_mod
        && !required.is_pressed(*modifiers)
    {
        return false;
    }

    let delta_y = delta.y.0;
    if !delta_y.is_finite() {
        return false;
    }

    let speed = if input_map.wheel_zoom_log2_per_px.is_finite() {
        input_map.wheel_zoom_log2_per_px
    } else {
        PlotInputMap::default().wheel_zoom_log2_per_px
    };
    let zoom = clamp_zoom_factors(2.0_f32.powf(delta_y * speed));
    let mut zoom_x = zoom;
    let mut zoom_y = zoom;

    match region {
        LinePlotWheelRegion::Plot => {
            let x_only = input_map
                .wheel_zoom_x_only_mod
                .is_some_and(|modifier| modifier.is_pressed(*modifiers));
            let y_only = input_map
                .wheel_zoom_y_only_mod
                .is_some_and(|modifier| modifier.is_pressed(*modifiers));
            if x_only {
                zoom_y = 1.0;
            } else if y_only {
                zoom_x = 1.0;
            }
        }
        LinePlotWheelRegion::XAxis => {
            zoom_y = 1.0;
        }
        LinePlotWheelRegion::YAxis => {
            zoom_x = 1.0;
        }
    }

    let axis_locks = state
        .read_ref(app, |state| state.axis_locks)
        .unwrap_or_default();
    if axis_locks.x.zoom {
        zoom_x = 1.0;
    }
    if axis_locks.y.zoom {
        zoom_y = 1.0;
    }

    if zoom_x == 1.0 && zoom_y == 1.0 {
        return false;
    }

    let current =
        line_plot_current_view_bounds_for_event(app, Some(state), model, style, x_scale, y_scale);
    let local = local_from_absolute(plot.origin, *position);
    let Some(mut next) =
        zoom_view_at_px_scaled(current, plot.size, local, zoom_x, zoom_y, x_scale, y_scale)
    else {
        return false;
    };
    if style.clamp_to_data_bounds {
        next = clamp_view_to_data_scaled(
            next,
            model.data_bounds,
            style.overscroll_fraction,
            x_scale,
            y_scale,
        );
    }
    next = sanitize_data_rect_scaled(next, x_scale, y_scale);
    if next == current {
        return false;
    }

    state
        .update(app, |state, _cx| {
            state.view_is_auto = false;
            state.view_bounds = Some(next);
            true
        })
        .ok()
        .unwrap_or(false)
}

fn line_plot_wheel_region_at(
    bounds: Rect,
    style: LinePlotStyle,
    position: Point,
) -> Option<LinePlotWheelRegion> {
    let plot = line_plot_inner_rect(bounds, style);
    if plot.contains(position) {
        return Some(LinePlotWheelRegion::Plot);
    }

    let pad = style.padding.0.max(0.0);
    let axis_gap = style.axis_gap.0.max(0.0);
    let y_axis = Rect::new(
        Point::new(Px(bounds.origin.x.0 + pad), plot.origin.y),
        Size::new(Px(axis_gap), plot.size.height),
    );
    if y_axis.contains(position) {
        return Some(LinePlotWheelRegion::YAxis);
    }

    let x_axis = Rect::new(
        Point::new(plot.origin.x, Px(plot.origin.y.0 + plot.size.height.0)),
        Size::new(plot.size.width, Px(axis_gap)),
    );
    if x_axis.contains(position) {
        return Some(LinePlotWheelRegion::XAxis);
    }

    None
}

fn line_plot_legend_hover_from_event(
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
) -> Option<Option<SeriesId>> {
    let Event::Pointer(fret_core::PointerEvent::Move { position, .. }) = event else {
        return None;
    };

    let plot = line_plot_inner_rect(bounds, style);
    Some(
        line_plot_legend_hit(model, plot, *position)
            .map(|(series_id, _hit)| series_id)
            .filter(|series_id| model.series.iter().any(|series| series.id == *series_id)),
    )
}

fn publish_line_plot_panel_output<H: UiHost>(
    app: &mut H,
    output: Option<&Model<PlotOutput>>,
    snapshot: PlotOutputSnapshot,
) -> bool {
    let Some(output) = output else {
        return false;
    };
    if output
        .read_ref(app, |state| state.snapshot == snapshot)
        .unwrap_or(false)
    {
        return false;
    }
    output
        .update(app, |state, _cx| {
            state.revision = state.revision.wrapping_add(1);
            state.snapshot = snapshot;
            true
        })
        .ok()
        .unwrap_or(false)
}

fn line_plot_query_from_state<H: UiHost>(
    app: &H,
    state: Option<&Model<PlotState>>,
) -> Option<DataRect> {
    state.and_then(|state| state.read_ref(app, |state| state.query).ok().flatten())
}

fn line_plot_output_snapshot(
    view_bounds: DataRect,
    cursor: Option<DataPoint>,
    query: Option<DataRect>,
) -> PlotOutputSnapshot {
    line_plot_output_snapshot_with_drag(view_bounds, cursor, query, None)
}

fn line_plot_output_snapshot_with_drag(
    view_bounds: DataRect,
    cursor: Option<DataPoint>,
    query: Option<DataRect>,
    drag: Option<PlotDragOutput>,
) -> PlotOutputSnapshot {
    PlotOutputSnapshot {
        view_bounds,
        view_bounds_y2: None,
        view_bounds_y3: None,
        view_bounds_y4: None,
        cursor,
        hover: None,
        query,
        drag,
    }
}

fn line_plot_pointer_output_snapshot(
    pointer: Point,
    bounds: Rect,
    _model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
    view_bounds: DataRect,
    query: Option<DataRect>,
) -> PlotOutputSnapshot {
    let plot = line_plot_inner_rect(bounds, style);
    let cursor = cursor_data_for_line_plot_pointer(pointer, plot, view_bounds, x_scale, y_scale);
    line_plot_output_snapshot(view_bounds, cursor, query)
}

fn line_plot_view_bounds_from_state(
    model: &PlotPanelModel,
    state: Option<&PlotState>,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> DataRect {
    if let Some(state) = state
        && !state.view_is_auto
        && let Some(view) = state.view_bounds
    {
        return sanitize_data_rect_scaled(view, x_scale, y_scale);
    }
    let data_bounds = sanitize_data_rect_scaled(model.data_bounds, x_scale, y_scale);
    if style.clamp_to_data_bounds {
        expand_line_plot_data_bounds(data_bounds, style.overscroll_fraction, x_scale, y_scale)
    } else {
        data_bounds
    }
}

fn line_plot_current_view_bounds_for_event<H: UiHost>(
    app: &H,
    state: Option<&Model<PlotState>>,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> DataRect {
    state
        .and_then(|state| {
            state
                .read_ref(app, |state| {
                    line_plot_view_bounds_from_state(model, Some(state), style, x_scale, y_scale)
                })
                .ok()
        })
        .unwrap_or_else(|| line_plot_view_bounds_from_state(model, None, style, x_scale, y_scale))
}

fn expand_line_plot_data_bounds(
    bounds: DataRect,
    fraction: f32,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> DataRect {
    let fraction = fraction.max(0.0) as f64;
    if fraction <= 0.0 {
        return bounds;
    }
    let expand_axis = |scale: AxisScale, min: f64, max: f64| -> (f64, f64) {
        let Some(axis_min) = scale.to_axis(min) else {
            return (min, max);
        };
        let Some(axis_max) = scale.to_axis(max) else {
            return (min, max);
        };
        let span = axis_max - axis_min;
        if !span.is_finite() || span <= 0.0 {
            return (min, max);
        }
        let pad = span * fraction;
        let next_min = scale.from_axis(axis_min - pad).unwrap_or(min);
        let next_max = scale.from_axis(axis_max + pad).unwrap_or(max);
        (next_min, next_max)
    };
    let (x_min, x_max) = expand_axis(x_scale, bounds.x_min, bounds.x_max);
    let (y_min, y_max) = expand_axis(y_scale, bounds.y_min, bounds.y_max);
    sanitize_data_rect_scaled(
        DataRect {
            x_min,
            x_max,
            y_min,
            y_max,
        },
        x_scale,
        y_scale,
    )
}

fn cursor_data_for_line_plot_pointer(
    pointer: Point,
    plot: Rect,
    view_bounds: crate::cartesian::DataRect,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Option<DataPoint> {
    if !plot.contains(pointer) || plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return None;
    }
    let transform = PlotTransform {
        viewport: plot,
        data: view_bounds,
        x_scale,
        y_scale,
    };
    let data = transform.px_to_data(pointer);
    (data.x.is_finite() && data.y.is_finite()).then_some(data)
}

fn paint_line_plot_grid_and_axes(
    painter: &mut CanvasPainter<'_>,
    transform: PlotTransform,
    style: LinePlotStyle,
    x_axis_labels: &AxisLabelFormatter,
    y_axis_labels: &AxisLabelFormatter,
) {
    let plot = transform.viewport;
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return;
    }

    let theme = painter.theme().snapshot();
    let mut grid_color = style
        .grid_color
        .unwrap_or_else(|| theme.color_required("border"));
    grid_color.a *= 0.45;
    let axis_color = style
        .axis_color
        .unwrap_or_else(|| theme.color_required("border"));
    let tick_count = style.tick_count.max(2);

    let x_ticks = axis_ticks_scaled(
        transform.data.x_min,
        transform.data.x_max,
        tick_count,
        AxisTicks::Nice,
        transform.x_scale,
    );
    let y_ticks = axis_ticks_scaled(
        transform.data.y_min,
        transform.data.y_max,
        tick_count,
        AxisTicks::Nice,
        transform.y_scale,
    );

    for x in x_ticks.iter().copied() {
        let Some(px) = transform.data_x_to_px(x) else {
            continue;
        };
        push_vertical_line(
            painter,
            px,
            plot.origin.y,
            plot.size.height,
            DrawOrder(2),
            grid_color,
        );
    }

    for y in y_ticks.iter().copied() {
        let Some(py) = transform.data_y_to_px(y) else {
            continue;
        };
        push_horizontal_line(
            painter,
            plot.origin.x,
            py,
            plot.size.width,
            DrawOrder(2),
            grid_color,
        );
    }

    let baseline_y = transform
        .data_y_to_px(0.0)
        .filter(|y| y.0 >= plot.origin.y.0 && y.0 <= plot.origin.y.0 + plot.size.height.0)
        .unwrap_or_else(|| Px(plot.origin.y.0 + plot.size.height.0 - 1.0));
    let baseline_x = transform
        .data_x_to_px(0.0)
        .filter(|x| x.0 >= plot.origin.x.0 && x.0 <= plot.origin.x.0 + plot.size.width.0)
        .unwrap_or(plot.origin.x);

    push_horizontal_line(
        painter,
        plot.origin.x,
        baseline_y,
        plot.size.width,
        DrawOrder(10),
        axis_color,
    );
    push_vertical_line(
        painter,
        baseline_x,
        plot.origin.y,
        plot.size.height,
        DrawOrder(10),
        axis_color,
    );

    paint_line_plot_axis_tick_labels(
        painter,
        transform,
        style,
        &x_ticks,
        &y_ticks,
        x_axis_labels,
        y_axis_labels,
    );
}

fn push_vertical_line(
    painter: &mut CanvasPainter<'_>,
    x: Px,
    y: Px,
    height: Px,
    order: DrawOrder,
    color: Color,
) {
    if !x.0.is_finite() || !y.0.is_finite() || !height.0.is_finite() || height.0 <= 0.0 {
        return;
    }
    painter.scene().push(fret_core::SceneOp::Quad {
        order,
        rect: Rect::new(Point::new(x, y), Size::new(Px(1.0), height)),
        background: Paint::Solid(color).into(),
        border: Edges::default(),
        border_paint: Paint::Solid(Color::TRANSPARENT).into(),
        corner_radii: Corners::default(),
    });
}

fn push_horizontal_line(
    painter: &mut CanvasPainter<'_>,
    x: Px,
    y: Px,
    width: Px,
    order: DrawOrder,
    color: Color,
) {
    if !x.0.is_finite() || !y.0.is_finite() || !width.0.is_finite() || width.0 <= 0.0 {
        return;
    }
    painter.scene().push(fret_core::SceneOp::Quad {
        order,
        rect: Rect::new(Point::new(x, y), Size::new(width, Px(1.0))),
        background: Paint::Solid(color).into(),
        border: Edges::default(),
        border_paint: Paint::Solid(Color::TRANSPARENT).into(),
        corner_radii: Corners::default(),
    });
}

fn push_filled_rect(painter: &mut CanvasPainter<'_>, rect: Rect, order: DrawOrder, color: Color) {
    if !rect.origin.x.0.is_finite()
        || !rect.origin.y.0.is_finite()
        || !rect.size.width.0.is_finite()
        || !rect.size.height.0.is_finite()
        || rect.size.width.0 <= 0.0
        || rect.size.height.0 <= 0.0
    {
        return;
    }
    painter.scene().push(fret_core::SceneOp::Quad {
        order,
        rect,
        background: Paint::Solid(color).into(),
        border: Edges::default(),
        border_paint: Paint::Solid(Color::TRANSPARENT).into(),
        corner_radii: Corners::default(),
    });
}

fn paint_line_plot_legend(
    painter: &mut CanvasPainter<'_>,
    model: &PlotPanelModel,
    plot: Rect,
    pinned_series: Option<SeriesId>,
    legend_hover: Option<SeriesId>,
    style: LinePlotStyle,
) {
    if model.series.is_empty() || plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return;
    }

    let theme = painter.theme().snapshot();
    let text_color = style
        .label_color
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    let text_style = TextStyle {
        size: Px(12.0),
        weight: FontWeight::NORMAL,
        ..TextStyle::default()
    };
    let text_constraints = CanvasTextConstraints {
        max_width: Some(Px((plot.size.width.0 - 36.0).max(24.0))),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };

    let series_count = model.series.len();
    let metrics = line_plot_legend_metrics();
    let scope = painter.key_scope(&"fret-plot.declarative.legend");
    let raster_scale_factor = painter.scale_factor();

    for (index, series) in model.series.iter().enumerate() {
        let Some(row) = line_plot_legend_row_rect(plot, index) else {
            break;
        };
        let swatch_rect = line_plot_legend_swatch_rect(row);
        if legend_hover == Some(series.id) || pinned_series == Some(series.id) {
            let mut highlight = style
                .crosshair_color
                .unwrap_or_else(|| theme.color_required("muted-foreground"));
            highlight.a *= if pinned_series == Some(series.id) {
                0.16
            } else {
                0.10
            };
            let inset_x = Px(2.0);
            painter.scene().push(fret_core::SceneOp::Quad {
                order: DrawOrder(29),
                rect: Rect::new(
                    Point::new(Px(row.origin.x.0 + inset_x.0), row.origin.y),
                    Size::new(
                        Px((row.size.width.0 - inset_x.0 * 2.0).max(0.0)),
                        row.size.height,
                    ),
                ),
                background: Paint::Solid(highlight).into(),
                border: Edges::default(),
                border_paint: Paint::Solid(Color::TRANSPARENT).into(),
                corner_radii: Corners::all(Px(4.0)),
            });
        }

        let color = series
            .stroke_color
            .unwrap_or_else(|| series_color(style, index, series_count));
        painter.scene().push(fret_core::SceneOp::Quad {
            order: DrawOrder(30),
            rect: swatch_rect,
            background: Paint::Solid(color).into(),
            border: Edges::default(),
            border_paint: Paint::Solid(Color::TRANSPARENT).into(),
            corner_radii: Corners::default(),
        });

        let key: u64 = painter
            .child_key(scope, &("series", series.id.0, series.label.as_ref()))
            .into();
        let _ = painter.text(
            key,
            DrawOrder(31),
            Point::new(
                Px(swatch_rect.origin.x.0 + swatch_rect.size.width.0 + metrics.gap.0),
                Px(row.origin.y.0 + metrics.text_baseline_offset.0),
            ),
            series.label.clone(),
            text_style.clone(),
            text_color,
            text_constraints,
            raster_scale_factor,
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinePlotLegendHit {
    Swatch,
    Label,
}

#[derive(Debug, Clone, Copy)]
struct LinePlotLegendMetrics {
    row_height: Px,
    swatch: Size,
    gap: Px,
    inset: Px,
    text_baseline_offset: Px,
}

fn line_plot_legend_metrics() -> LinePlotLegendMetrics {
    LinePlotLegendMetrics {
        row_height: Px(18.0),
        swatch: Size::new(Px(12.0), Px(3.0)),
        gap: Px(6.0),
        inset: Px(8.0),
        text_baseline_offset: Px(12.0),
    }
}

fn line_plot_legend_row_rect(plot: Rect, index: usize) -> Option<Rect> {
    let metrics = line_plot_legend_metrics();
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return None;
    }
    let y = Px(plot.origin.y.0 + metrics.inset.0 + index as f32 * metrics.row_height.0);
    let max_y = plot.origin.y.0 + plot.size.height.0 - metrics.inset.0;
    if y.0 + metrics.row_height.0 > max_y {
        return None;
    }
    Some(Rect::new(
        Point::new(Px(plot.origin.x.0 + metrics.inset.0), y),
        Size::new(
            Px((plot.size.width.0 - metrics.inset.0 * 2.0).max(0.0)),
            metrics.row_height,
        ),
    ))
}

fn line_plot_legend_swatch_rect(row: Rect) -> Rect {
    let metrics = line_plot_legend_metrics();
    let row_mid = row.origin.y.0 + row.size.height.0 * 0.5;
    Rect::new(
        Point::new(row.origin.x, Px(row_mid - metrics.swatch.height.0 * 0.5)),
        metrics.swatch,
    )
}

fn line_plot_legend_swatch_hit_rect(row: Rect) -> Rect {
    let metrics = line_plot_legend_metrics();
    Rect::new(row.origin, Size::new(metrics.swatch.width, row.size.height))
}

fn line_plot_legend_hit(
    model: &PlotPanelModel,
    plot: Rect,
    position: Point,
) -> Option<(SeriesId, LinePlotLegendHit)> {
    if model.series.is_empty() {
        return None;
    }
    for (index, series) in model.series.iter().enumerate() {
        let row = line_plot_legend_row_rect(plot, index)?;
        if !row.contains(position) {
            continue;
        }
        let hit = if line_plot_legend_swatch_hit_rect(row).contains(position) {
            LinePlotLegendHit::Swatch
        } else {
            LinePlotLegendHit::Label
        };
        return Some((series.id, hit));
    }
    None
}

fn paint_line_plot_axis_tick_labels(
    painter: &mut CanvasPainter<'_>,
    transform: PlotTransform,
    style: LinePlotStyle,
    x_ticks: &[f64],
    y_ticks: &[f64],
    x_formatter: &AxisLabelFormatter,
    y_formatter: &AxisLabelFormatter,
) {
    if x_ticks.is_empty() && y_ticks.is_empty() {
        return;
    }

    let plot = transform.viewport;
    let theme = painter.theme().snapshot();
    let text_color = style
        .label_color
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    let text_style = TextStyle {
        size: Px(12.0),
        weight: FontWeight::NORMAL,
        ..TextStyle::default()
    };
    let constraints = CanvasTextConstraints {
        max_width: Some(Px(72.0)),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let x_span = (transform.data.x_max - transform.data.x_min).abs();
    let y_span = (transform.data.y_max - transform.data.y_min).abs();
    let scope = painter.key_scope(&"fret-plot.declarative.axis-labels");
    let raster_scale_factor = painter.scale_factor();

    let x_label_y = Px(plot.origin.y.0 + plot.size.height.0 + 2.0);
    for (index, value) in x_ticks.iter().copied().enumerate() {
        let Some(x) = transform.data_x_to_px(value) else {
            continue;
        };
        let text = axis_tick_label_text(transform.x_scale, x_formatter, value, x_span);
        if text.is_empty() {
            continue;
        }
        let key: u64 = painter
            .child_key(scope, &("x", index, value.to_bits()))
            .into();
        let _ = painter.text(
            key,
            DrawOrder(11),
            Point::new(Px(x.0 - 12.0), x_label_y),
            text,
            text_style.clone(),
            text_color,
            constraints,
            raster_scale_factor,
        );
    }

    let y_label_x = Px((plot.origin.x.0 - style.axis_gap.0 + 4.0).max(0.0));
    for (index, value) in y_ticks.iter().copied().enumerate() {
        let Some(y) = transform.data_y_to_px(value) else {
            continue;
        };
        let text = axis_tick_label_text(transform.y_scale, y_formatter, value, y_span);
        if text.is_empty() {
            continue;
        }
        let key: u64 = painter
            .child_key(scope, &("y", index, value.to_bits()))
            .into();
        let _ = painter.text(
            key,
            DrawOrder(11),
            Point::new(y_label_x, y),
            text,
            text_style.clone(),
            text_color,
            constraints,
            raster_scale_factor,
        );
    }
}

fn paint_line_plot_right_axis_tick_labels(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    primary_view_bounds: DataRect,
    view_bounds_y2: Option<DataRect>,
    view_bounds_y3: Option<DataRect>,
    view_bounds_y4: Option<DataRect>,
    style: LinePlotStyle,
    y2_formatter: &AxisLabelFormatter,
    y3_formatter: &AxisLabelFormatter,
    y4_formatter: &AxisLabelFormatter,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return;
    }

    for (axis_index, axis_key, axis_bounds, formatter) in [
        (0usize, "y2", view_bounds_y2, y2_formatter),
        (1usize, "y3", view_bounds_y3, y3_formatter),
        (2usize, "y4", view_bounds_y4, y4_formatter),
    ] {
        let Some(axis_bounds) = axis_bounds else {
            continue;
        };
        let transform = PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(primary_view_bounds, axis_bounds),
            x_scale,
            y_scale,
        };
        let y_ticks = axis_ticks_scaled(
            transform.data.y_min,
            transform.data.y_max,
            style.tick_count.max(2),
            AxisTicks::Nice,
            transform.y_scale,
        );
        paint_line_plot_right_axis_tick_labels_for_axis(
            painter, transform, style, &y_ticks, formatter, axis_index, axis_key,
        );
    }
}

fn paint_line_plot_right_axis_tick_labels_for_axis(
    painter: &mut CanvasPainter<'_>,
    transform: PlotTransform,
    style: LinePlotStyle,
    y_ticks: &[f64],
    formatter: &AxisLabelFormatter,
    axis_index: usize,
    axis_key: &'static str,
) {
    if y_ticks.is_empty() {
        return;
    }

    let plot = transform.viewport;
    let theme = painter.theme().snapshot();
    let text_color = style
        .label_color
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    let text_style = TextStyle {
        size: Px(12.0),
        weight: FontWeight::NORMAL,
        ..TextStyle::default()
    };
    let constraints = CanvasTextConstraints {
        max_width: Some(Px(72.0)),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let span = (transform.data.y_max - transform.data.y_min).abs();
    let scope = painter.key_scope(&"fret-plot.declarative.right-axis-labels");
    let raster_scale_factor = painter.scale_factor();
    let lane_gap = style.axis_gap.0.max(18.0);
    let label_x = Px(plot.origin.x.0 + plot.size.width.0 + 4.0 + axis_index as f32 * lane_gap);

    for (index, value) in y_ticks.iter().copied().enumerate() {
        let Some(y) = transform.data_y_to_px(value) else {
            continue;
        };
        let text = axis_tick_label_text(transform.y_scale, formatter, value, span);
        if text.is_empty() {
            continue;
        }
        let key: u64 = painter
            .child_key(scope, &(axis_key, index, value.to_bits()))
            .into();
        let _ = painter.text(
            key,
            DrawOrder(11),
            Point::new(label_x, y),
            text,
            text_style.clone(),
            text_color,
            constraints,
            raster_scale_factor,
        );
    }
}

fn paint_line_plot_cursor_readout(
    painter: &mut CanvasPainter<'_>,
    model: &PlotPanelModel,
    plot: Rect,
    output: Option<PlotOutputSnapshot>,
    pinned_series: Option<SeriesId>,
    hidden_series: &[SeriesId],
    style: LinePlotStyle,
    y_axis_labels: &AxisLabelFormatter,
    y2_axis_labels: &AxisLabelFormatter,
    y3_axis_labels: &AxisLabelFormatter,
    y4_axis_labels: &AxisLabelFormatter,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    let Some(snapshot) = output else {
        return;
    };
    let Some(cursor) = snapshot.cursor else {
        return;
    };
    if style.mouse_readout == MouseReadoutMode::Disabled {
        return;
    }

    let transform = PlotTransform {
        viewport: plot,
        data: snapshot.view_bounds,
        x_scale,
        y_scale,
    };
    let cursor_px = transform.data_to_px(cursor);
    if !plot.contains(cursor_px) {
        return;
    }

    let theme = painter.theme().snapshot();
    let mut crosshair_color = style
        .crosshair_color
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    crosshair_color.a = (crosshair_color.a * 0.45).clamp(0.05, 1.0);
    push_vertical_line(
        painter,
        Px(cursor_px.x.0.round()),
        plot.origin.y,
        plot.size.height,
        DrawOrder(3),
        crosshair_color,
    );
    push_horizontal_line(
        painter,
        plot.origin.x,
        Px(cursor_px.y.0.round()),
        plot.size.width,
        DrawOrder(3),
        crosshair_color,
    );

    if style.mouse_readout != MouseReadoutMode::Overlay {
        return;
    }

    let tooltip_background = style
        .tooltip_background
        .unwrap_or_else(|| theme.color_required("popover"));
    let tooltip_border = style
        .tooltip_border
        .unwrap_or_else(|| theme.color_required("border"));
    let text_color = style
        .tooltip_text_color
        .or(style.label_color)
        .unwrap_or_else(|| theme.color_required("popover-foreground"));

    let x_span = (snapshot.view_bounds.x_max - snapshot.view_bounds.x_min).abs();
    let y_span = (snapshot.view_bounds.y_max - snapshot.view_bounds.y_min).abs();
    let formatter = AxisLabelFormatter::default();
    let x_text = axis_tick_label_text(x_scale, &formatter, cursor.x, x_span);
    let y_text = axis_tick_label_text(y_scale, &formatter, cursor.y, y_span);
    let rows = line_plot_readout_rows(
        model,
        cursor.x,
        plot.size,
        snapshot.view_bounds,
        x_scale,
        y_scale,
        painter.scale_factor(),
        hidden_series,
    );
    let rows = filter_line_plot_readout_rows(rows, pinned_series, ReadoutSeriesPolicy::PinnedOrAll);
    let text = format_line_plot_readout_text(
        format!("x={x_text}  y={y_text}"),
        rows,
        y_axis_labels,
        y2_axis_labels,
        y3_axis_labels,
        y4_axis_labels,
        y_scale,
    );

    let text_style = TextStyle {
        size: Px(12.0),
        weight: FontWeight::NORMAL,
        ..TextStyle::default()
    };
    let constraints = CanvasTextConstraints {
        max_width: Some(Px(plot.size.width.0.max(24.0))),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let raster_scale_factor = painter.scale_factor();
    let scope = painter.key_scope(&"fret-plot.declarative.cursor-readout");
    let text_key: u64 = painter.child_key(scope, &("text", text.as_str())).into();
    let (_blob, metrics) = painter.prepare_text_with_blob(
        text_key,
        text.clone(),
        text_style.clone(),
        constraints,
        raster_scale_factor,
    );

    let pad = Px(6.0);
    let margin = Px(6.0);
    let overlay_size = Size::new(
        Px(metrics.size.width.0 + pad.0 * 2.0),
        Px(metrics.size.height.0 + pad.0 * 2.0),
    );
    let Some(rect) =
        overlay_rect_in_line_plot(plot, overlay_size, style.mouse_readout_anchor, margin)
    else {
        return;
    };
    painter.scene().push(fret_core::SceneOp::Quad {
        order: DrawOrder(12),
        rect,
        background: Paint::Solid(tooltip_background).into(),
        border: Edges::all(Px(1.0)),
        border_paint: Paint::Solid(tooltip_border).into(),
        corner_radii: Corners::all(Px(6.0)),
    });

    let _ = painter.text(
        text_key,
        DrawOrder(13),
        Point::new(
            Px(rect.origin.x.0 + pad.0),
            Px(rect.origin.y.0 + pad.0 + metrics.baseline.0),
        ),
        text,
        text_style,
        text_color,
        constraints,
        raster_scale_factor,
    );
}

fn paint_line_plot_linked_cursor_readout(
    painter: &mut CanvasPainter<'_>,
    model: &PlotPanelModel,
    plot: Rect,
    view_bounds: crate::cartesian::DataRect,
    local_cursor: Option<DataPoint>,
    linked_cursor_x: Option<f64>,
    pinned_series: Option<SeriesId>,
    hidden_series: &[SeriesId],
    style: LinePlotStyle,
    y_axis_labels: &AxisLabelFormatter,
    y2_axis_labels: &AxisLabelFormatter,
    y3_axis_labels: &AxisLabelFormatter,
    y4_axis_labels: &AxisLabelFormatter,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    if local_cursor.is_some() {
        return;
    }
    let Some(linked_x) = linked_cursor_x.filter(|x| x.is_finite()) else {
        return;
    };

    let transform = PlotTransform {
        viewport: plot,
        data: view_bounds,
        x_scale,
        y_scale,
    };
    let Some(cursor_x) = transform.data_x_to_px(linked_x) else {
        return;
    };

    let theme = painter.theme().snapshot();
    let mut crosshair_color = style
        .crosshair_color
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    crosshair_color.a = (crosshair_color.a * 0.55).clamp(0.05, 1.0);
    push_vertical_line(
        painter,
        Px(cursor_x
            .0
            .clamp(plot.origin.x.0, plot.origin.x.0 + plot.size.width.0)
            .round()),
        plot.origin.y,
        plot.size.height,
        DrawOrder(3),
        crosshair_color,
    );

    if style.linked_cursor_readout != MouseReadoutMode::Overlay {
        return;
    }

    let tooltip_background = style
        .tooltip_background
        .unwrap_or_else(|| theme.color_required("popover"));
    let tooltip_border = style
        .tooltip_border
        .unwrap_or_else(|| theme.color_required("border"));
    let text_color = style
        .tooltip_text_color
        .or(style.label_color)
        .unwrap_or_else(|| theme.color_required("popover-foreground"));

    let x_span = (view_bounds.x_max - view_bounds.x_min).abs();
    let formatter = AxisLabelFormatter::default();
    let x_text = axis_tick_label_text(x_scale, &formatter, linked_x, x_span);
    let rows = line_plot_readout_rows(
        model,
        linked_x,
        plot.size,
        view_bounds,
        x_scale,
        y_scale,
        painter.scale_factor(),
        hidden_series,
    );
    let rows =
        filter_line_plot_readout_rows(rows, pinned_series, style.linked_cursor_readout_policy);
    let text = format_line_plot_readout_text(
        format!("x={x_text}"),
        rows,
        y_axis_labels,
        y2_axis_labels,
        y3_axis_labels,
        y4_axis_labels,
        y_scale,
    );

    let text_style = TextStyle {
        size: Px(12.0),
        weight: FontWeight::NORMAL,
        ..TextStyle::default()
    };
    let constraints = CanvasTextConstraints {
        max_width: Some(Px(plot.size.width.0.max(24.0))),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let raster_scale_factor = painter.scale_factor();
    let scope = painter.key_scope(&"fret-plot.declarative.linked-cursor-readout");
    let text_key: u64 = painter.child_key(scope, &("text", text.as_str())).into();
    let (_blob, metrics) = painter.prepare_text_with_blob(
        text_key,
        text.clone(),
        text_style.clone(),
        constraints,
        raster_scale_factor,
    );

    let pad = Px(6.0);
    let margin = Px(6.0);
    let overlay_size = Size::new(
        Px(metrics.size.width.0 + pad.0 * 2.0),
        Px(metrics.size.height.0 + pad.0 * 2.0),
    );
    let Some(rect) = overlay_rect_in_line_plot(
        plot,
        overlay_size,
        style.linked_cursor_readout_anchor,
        margin,
    ) else {
        return;
    };
    painter.scene().push(fret_core::SceneOp::Quad {
        order: DrawOrder(12),
        rect,
        background: Paint::Solid(tooltip_background).into(),
        border: Edges::all(Px(1.0)),
        border_paint: Paint::Solid(tooltip_border).into(),
        corner_radii: Corners::all(Px(6.0)),
    });

    let _ = painter.text(
        text_key,
        DrawOrder(13),
        Point::new(
            Px(rect.origin.x.0 + pad.0),
            Px(rect.origin.y.0 + pad.0 + metrics.baseline.0),
        ),
        text,
        text_style,
        text_color,
        constraints,
        raster_scale_factor,
    );
}

fn line_plot_readout_rows<'a>(
    model: &PlotPanelModel,
    x: f64,
    plot_size: Size,
    view_bounds: crate::cartesian::DataRect,
    x_scale: AxisScale,
    y_scale: AxisScale,
    scale_factor: f32,
    hidden_series: &'a [SeriesId],
) -> Vec<PlotCursorReadoutRow> {
    let hidden: std::collections::HashSet<SeriesId> = hidden_series.iter().copied().collect();
    let mut readout_series: Vec<PlotCursorReadoutSeries<'_>> = Vec::new();
    for series in &model.series {
        if let Some(lower_data) = &series.lower_data {
            readout_series.push(PlotCursorReadoutSeries {
                id: series.id,
                label: std::sync::Arc::from(format!("{} (upper)", series.label)),
                y_axis: series.y_axis,
                data: &*series.data,
            });
            readout_series.push(PlotCursorReadoutSeries {
                id: series.id,
                label: std::sync::Arc::from(format!("{} (lower)", series.label)),
                y_axis: series.y_axis,
                data: &**lower_data,
            });
        } else {
            readout_series.push(PlotCursorReadoutSeries {
                id: series.id,
                label: series.label.clone(),
                y_axis: series.y_axis,
                data: &*series.data,
            });
        }
    }
    plot_cursor_readout(
        readout_series,
        PlotCursorReadoutArgs {
            x,
            plot_size,
            view_bounds,
            x_scale,
            y_scale,
            scale_factor,
            hidden: &hidden,
        },
    )
}

fn format_line_plot_readout_text(
    mut text: String,
    rows: Vec<PlotCursorReadoutRow>,
    y_axis_labels: &AxisLabelFormatter,
    y2_axis_labels: &AxisLabelFormatter,
    y3_axis_labels: &AxisLabelFormatter,
    y4_axis_labels: &AxisLabelFormatter,
    y_scale: AxisScale,
) -> String {
    for row in rows {
        let (formatter, axis_label) = match row.y_axis {
            crate::models::YAxis::Left => (y_axis_labels, "y"),
            crate::models::YAxis::Right => (y2_axis_labels, "y2"),
            crate::models::YAxis::Right2 => (y3_axis_labels, "y3"),
            crate::models::YAxis::Right3 => (y4_axis_labels, "y4"),
        };
        let y_text = row
            .y
            .filter(|y| y.is_finite())
            .map(|y| axis_tick_label_text(y_scale, &formatter, y, 1.0))
            .unwrap_or_else(|| "NA".to_string());
        text.push_str(&format!("\n{}: {axis_label}={y_text}", row.label));
    }
    text
}

fn filter_line_plot_readout_rows(
    rows: Vec<PlotCursorReadoutRow>,
    pinned: Option<SeriesId>,
    policy: ReadoutSeriesPolicy,
) -> Vec<PlotCursorReadoutRow> {
    match (policy, pinned) {
        (ReadoutSeriesPolicy::PinnedOrAll, Some(pinned))
        | (ReadoutSeriesPolicy::PinnedOnly, Some(pinned))
        | (ReadoutSeriesPolicy::PinnedOrLegendHoverOrAll, Some(pinned)) => rows
            .into_iter()
            .filter(|row| row.series_id == pinned)
            .collect(),
        (ReadoutSeriesPolicy::PinnedOnly, None) => Vec::new(),
        (ReadoutSeriesPolicy::PinnedOrAll, None)
        | (ReadoutSeriesPolicy::PinnedOrLegendHoverOrAll, None) => rows,
    }
}

fn axis_tick_label_text(
    scale: AxisScale,
    formatter: &AxisLabelFormatter,
    value: f64,
    span: f64,
) -> String {
    if scale == AxisScale::Log10 && formatter.is_number_auto() {
        return log10_tick_label_or_empty(value);
    }
    formatter.format(value, span)
}

fn overlay_rect_in_line_plot(
    plot: Rect,
    size: Size,
    anchor: OverlayAnchor,
    margin: Px,
) -> Option<Rect> {
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return None;
    }
    if size.width.0 <= 0.0 || size.height.0 <= 0.0 {
        return None;
    }

    let w = size.width.0;
    let h = size.height.0;
    let m = margin.0.max(0.0);
    let x = match anchor {
        OverlayAnchor::TopLeft | OverlayAnchor::BottomLeft => plot.origin.x.0 + m,
        OverlayAnchor::TopRight | OverlayAnchor::BottomRight => {
            plot.origin.x.0 + plot.size.width.0 - m - w
        }
    };
    let y = match anchor {
        OverlayAnchor::TopLeft | OverlayAnchor::TopRight => plot.origin.y.0 + m,
        OverlayAnchor::BottomLeft | OverlayAnchor::BottomRight => {
            plot.origin.y.0 + plot.size.height.0 - m - h
        }
    };

    let max_x = plot.origin.x.0 + plot.size.width.0 - w;
    let max_y = plot.origin.y.0 + plot.size.height.0 - h;
    Some(Rect::new(
        Point::new(
            Px(x.clamp(plot.origin.x.0, max_x)),
            Px(y.clamp(plot.origin.y.0, max_y)),
        ),
        size,
    ))
}

fn line_plot_inner_rect(bounds: Rect, style: LinePlotStyle) -> Rect {
    let pad = style.padding.0.max(0.0);
    let axis_gap = style.axis_gap.0.max(0.0);
    Rect::new(
        Point::new(
            Px(bounds.origin.x.0 + pad + axis_gap),
            Px(bounds.origin.y.0 + pad),
        ),
        Size::new(
            Px((bounds.size.width.0 - pad * 2.0 - axis_gap).max(0.0)),
            Px((bounds.size.height.0 - pad * 2.0 - axis_gap).max(0.0)),
        ),
    )
}

fn series_color(style: LinePlotStyle, series_index: usize, series_count: usize) -> Color {
    if series_count <= 1 {
        return style.stroke_color;
    }
    style.series_palette[series_index % style.series_palette.len()]
}

fn line_plot_series_path_key(series_id: u64) -> u64 {
    0x706c_6f74_6c69_6e65_u64 ^ series_id
}

fn line_plot_area_fill_path_key(series_id: u64) -> u64 {
    0x706c_6f74_6172_6561_u64 ^ series_id
}

fn line_plot_shaded_lower_path_key(series_id: u64) -> u64 {
    0x706c_6f74_7368_6164_u64 ^ series_id
}

fn line_plot_candlestick_down_path_key(series_id: u64) -> u64 {
    0x706c_6f74_6364_6f77_u64 ^ series_id
}

fn histogram_series_from_bins(series: &crate::models::HistogramSeries) -> Series {
    let Some(bins) = histogram_bins(&series.values, series.bin_count, series.range) else {
        return Series::from_points_sorted(Vec::new(), true);
    };

    let points: Vec<DataPoint> = bins
        .counts
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(index, count)| {
            (count.is_finite() && count > 0.0).then(|| DataPoint {
                x: bins.center_x(index),
                y: count,
            })
        })
        .collect();
    Series::from_points_sorted(points, true)
}

fn histogram_series_bin_width(series: &crate::models::HistogramSeries) -> f64 {
    histogram_bins(&series.values, series.bin_count, series.range)
        .map(|bins| bins.bin_width)
        .unwrap_or(0.0)
}

fn line_plot_device_point_budget(transform: PlotTransform, scale_factor: f32) -> usize {
    let width = transform.viewport.size.width.0.max(0.0);
    let device_width = (width * scale_factor.max(1.0)).max(1.0);
    device_width as usize * 2
}

fn candlestick_commands_from_series(
    transform: PlotTransform,
    candlestick: &PlotPanelCandlestick,
    stroke_width: Px,
    scale_factor: f32,
) -> (Vec<PathCommand>, Vec<PathCommand>, Vec<PathCommand>) {
    let points = &candlestick.points;
    if points.is_empty() {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    let candle_width = candlestick.candle_width.abs();
    if !candle_width.is_finite() {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    let view_x_min = transform.data.x_min.min(transform.data.x_max);
    let view_x_max = transform.data.x_min.max(transform.data.x_max);
    let half_w = candle_width * 0.5;
    let max_count = line_plot_device_point_budget(transform, scale_factor).max(8);

    let mut wick = Vec::new();
    let mut body_up = Vec::new();
    let mut body_down = Vec::new();

    let push_rect = |out: &mut Vec<PathCommand>, x0: Px, x1: Px, y0: Px, y1: Px| {
        let left = x0.0.min(x1.0);
        let right = x0.0.max(x1.0);
        let top = y0.0.min(y1.0);
        let bottom = y0.0.max(y1.0);
        if !left.is_finite() || !right.is_finite() || !top.is_finite() || !bottom.is_finite() {
            return;
        }

        let width = (right - left).max(stroke_width.0.max(1.0));
        let height = (bottom - top).max(stroke_width.0.max(1.0));
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            return;
        }

        let p0 = Point::new(Px(left), Px(top));
        let p1 = Point::new(Px(left + width), Px(top));
        let p2 = Point::new(Px(left + width), Px(top + height));
        let p3 = Point::new(Px(left), Px(top + height));

        out.push(PathCommand::MoveTo(p0));
        out.push(PathCommand::LineTo(p1));
        out.push(PathCommand::LineTo(p2));
        out.push(PathCommand::LineTo(p3));
        out.push(PathCommand::Close);
    };

    let mut push_point = |point: crate::models::OhlcPoint| {
        if !point.is_finite() || point.x < view_x_min || point.x > view_x_max {
            return;
        }

        let Some(x_px) = transform.data_x_to_px(point.x) else {
            return;
        };
        let Some(high_px) = transform.data_y_to_px(point.high) else {
            return;
        };
        let Some(low_px) = transform.data_y_to_px(point.low) else {
            return;
        };
        wick.push(PathCommand::MoveTo(Point::new(x_px, high_px)));
        wick.push(PathCommand::LineTo(Point::new(x_px, low_px)));

        let Some(x0_px) = transform.data_x_to_px(point.x - half_w) else {
            return;
        };
        let Some(x1_px) = transform.data_x_to_px(point.x + half_w) else {
            return;
        };
        let Some(open_px) = transform.data_y_to_px(point.open) else {
            return;
        };
        let Some(close_px) = transform.data_y_to_px(point.close) else {
            return;
        };

        if point.close >= point.open {
            push_rect(&mut body_up, x0_px, x1_px, open_px, close_px);
        } else {
            push_rect(&mut body_down, x0_px, x1_px, open_px, close_px);
        }
    };

    if points.len() <= max_count {
        for point in points.iter().copied() {
            push_point(point);
        }
    } else {
        let stride = points.len().div_ceil(max_count).max(1);
        for point in points.iter().copied().step_by(stride) {
            push_point(point);
        }
    }

    (wick, body_up, body_down)
}

fn histogram_commands_from_series(
    transform: PlotTransform,
    data: &dyn crate::series::SeriesData,
    histogram: &PlotPanelHistogram,
) -> Vec<PathCommand> {
    let bin_width = histogram.bin_width;
    if !bin_width.is_finite() || bin_width <= 0.0 {
        return Vec::new();
    }

    let gap = histogram.bar_gap_fraction.clamp(0.0, 0.95);
    let bar_width = bin_width * f64::from(1.0 - gap);
    if !bar_width.is_finite() || bar_width <= 0.0 {
        return Vec::new();
    }

    let mut out = Vec::new();
    for index in 0..data.len() {
        let Some(point) = data.get(index) else {
            continue;
        };
        if !point.x.is_finite() || !point.y.is_finite() || point.y <= 0.0 {
            continue;
        }

        let x0 = point.x - bar_width * 0.5;
        let x1 = point.x + bar_width * 0.5;
        let p00 = transform.data_to_px(DataPoint { x: x0, y: 0.0 });
        let p10 = transform.data_to_px(DataPoint { x: x1, y: 0.0 });
        let p01 = transform.data_to_px(DataPoint { x: x0, y: point.y });
        let p11 = transform.data_to_px(DataPoint { x: x1, y: point.y });

        if !p00.x.0.is_finite()
            || !p00.y.0.is_finite()
            || !p10.x.0.is_finite()
            || !p10.y.0.is_finite()
            || !p01.x.0.is_finite()
            || !p01.y.0.is_finite()
            || !p11.x.0.is_finite()
            || !p11.y.0.is_finite()
        {
            continue;
        }

        let left = p00.x.0.min(p10.x.0).min(p01.x.0).min(p11.x.0);
        let right = p00.x.0.max(p10.x.0).max(p01.x.0).max(p11.x.0);
        let top = p00.y.0.min(p10.y.0).min(p01.y.0).min(p11.y.0);
        let bottom = p00.y.0.max(p10.y.0).max(p01.y.0).max(p11.y.0);

        if !left.is_finite()
            || !right.is_finite()
            || !top.is_finite()
            || !bottom.is_finite()
            || right <= left
            || bottom <= top
        {
            continue;
        }

        let a = Point::new(Px(left), Px(top));
        let b = Point::new(Px(right), Px(top));
        let c = Point::new(Px(right), Px(bottom));
        let d = Point::new(Px(left), Px(bottom));

        out.push(PathCommand::MoveTo(a));
        out.push(PathCommand::LineTo(b));
        out.push(PathCommand::LineTo(c));
        out.push(PathCommand::LineTo(d));
        out.push(PathCommand::Close);
    }

    out
}

fn bars_commands_from_series(
    transform: PlotTransform,
    data: &dyn crate::series::SeriesData,
    bars: &PlotPanelBars,
) -> Vec<PathCommand> {
    let bar_width = bars.bar_width.abs();
    if !bar_width.is_finite() || bar_width <= 0.0 {
        return Vec::new();
    }

    let mut out = Vec::new();
    for index in 0..data.len() {
        let Some(point) = data.get(index) else {
            continue;
        };
        let baseline = bars
            .baselines
            .as_deref()
            .and_then(|baselines| baselines.get(index).copied())
            .unwrap_or(bars.baseline);
        if !point.x.is_finite()
            || !point.y.is_finite()
            || !baseline.is_finite()
            || point.y == baseline
        {
            continue;
        }

        let x0 = point.x - bar_width * 0.5;
        let x1 = point.x + bar_width * 0.5;
        let p00 = transform.data_to_px(DataPoint { x: x0, y: baseline });
        let p10 = transform.data_to_px(DataPoint { x: x1, y: baseline });
        let p01 = transform.data_to_px(DataPoint { x: x0, y: point.y });
        let p11 = transform.data_to_px(DataPoint { x: x1, y: point.y });

        if !p00.x.0.is_finite()
            || !p00.y.0.is_finite()
            || !p10.x.0.is_finite()
            || !p10.y.0.is_finite()
            || !p01.x.0.is_finite()
            || !p01.y.0.is_finite()
            || !p11.x.0.is_finite()
            || !p11.y.0.is_finite()
        {
            continue;
        }

        let left = p00.x.0.min(p10.x.0).min(p01.x.0).min(p11.x.0);
        let right = p00.x.0.max(p10.x.0).max(p01.x.0).max(p11.x.0);
        let top = p00.y.0.min(p10.y.0).min(p01.y.0).min(p11.y.0);
        let bottom = p00.y.0.max(p10.y.0).max(p01.y.0).max(p11.y.0);

        if !left.is_finite()
            || !right.is_finite()
            || !top.is_finite()
            || !bottom.is_finite()
            || right <= left
            || bottom <= top
        {
            continue;
        }

        let a = Point::new(Px(left), Px(top));
        let b = Point::new(Px(right), Px(top));
        let c = Point::new(Px(right), Px(bottom));
        let d = Point::new(Px(left), Px(bottom));

        out.push(PathCommand::MoveTo(a));
        out.push(PathCommand::LineTo(b));
        out.push(PathCommand::LineTo(c));
        out.push(PathCommand::LineTo(d));
        out.push(PathCommand::Close);
    }

    out
}

fn error_bars_commands_from_series(
    transform: PlotTransform,
    data: &dyn crate::series::SeriesData,
    error_bars: &PlotPanelErrorBars,
) -> Vec<PathCommand> {
    let cap = if error_bars.show_caps {
        error_bars.cap_size.0.max(0.0)
    } else {
        0.0
    };
    let marker = if error_bars.show_markers {
        error_bars.marker_radius.0.max(0.0)
    } else {
        0.0
    };
    let mut out = Vec::new();

    let mut push_point = |idx: usize, point: DataPoint| {
        if !point.x.is_finite() || !point.y.is_finite() {
            return;
        }

        let Some(x_px) = transform.data_x_to_px(point.x) else {
            return;
        };

        if let Some(y_err) = error_bars
            .y_errors
            .as_ref()
            .and_then(|errors| errors.get(idx))
        {
            let y0 = point.y - y_err.neg.abs();
            let y1 = point.y + y_err.pos.abs();
            if let (Some(y0_px), Some(y1_px)) =
                (transform.data_y_to_px(y0), transform.data_y_to_px(y1))
            {
                out.push(PathCommand::MoveTo(Point::new(x_px, y0_px)));
                out.push(PathCommand::LineTo(Point::new(x_px, y1_px)));

                if cap > 0.0 {
                    let x0 = Px(x_px.0 - cap);
                    let x1 = Px(x_px.0 + cap);
                    out.push(PathCommand::MoveTo(Point::new(x0, y0_px)));
                    out.push(PathCommand::LineTo(Point::new(x1, y0_px)));
                    out.push(PathCommand::MoveTo(Point::new(x0, y1_px)));
                    out.push(PathCommand::LineTo(Point::new(x1, y1_px)));
                }
            }
        }

        if let Some(x_err) = error_bars
            .x_errors
            .as_ref()
            .and_then(|errors| errors.get(idx))
            && let Some(y_px) = transform.data_y_to_px(point.y)
        {
            let x0 = point.x - x_err.neg.abs();
            let x1 = point.x + x_err.pos.abs();
            if let (Some(x0_px), Some(x1_px)) =
                (transform.data_x_to_px(x0), transform.data_x_to_px(x1))
            {
                out.push(PathCommand::MoveTo(Point::new(x0_px, y_px)));
                out.push(PathCommand::LineTo(Point::new(x1_px, y_px)));

                if cap > 0.0 {
                    let y0 = Px(y_px.0 - cap);
                    let y1 = Px(y_px.0 + cap);
                    out.push(PathCommand::MoveTo(Point::new(x0_px, y0)));
                    out.push(PathCommand::LineTo(Point::new(x0_px, y1)));
                    out.push(PathCommand::MoveTo(Point::new(x1_px, y0)));
                    out.push(PathCommand::LineTo(Point::new(x1_px, y1)));
                }
            }
        }

        if marker > 0.0 {
            let Some(y_px) = transform.data_y_to_px(point.y) else {
                return;
            };
            push_line_plot_marker_commands(
                &mut out,
                x_px,
                y_px,
                Px(marker),
                error_bars.marker_shape,
            );
        }
    };

    if let Some(points) = data.as_slice() {
        for (idx, point) in points.iter().copied().enumerate() {
            push_point(idx, point);
        }
    } else {
        for idx in 0..data.len() {
            let Some(point) = data.get(idx) else {
                continue;
            };
            push_point(idx, point);
        }
    }

    out
}

fn push_line_plot_marker_commands(
    out: &mut Vec<PathCommand>,
    x: Px,
    y: Px,
    radius: Px,
    shape: crate::models::MarkerShape,
) {
    let x = x.0;
    let y = y.0;
    let r = radius.0.max(0.0);
    if !x.is_finite() || !y.is_finite() || !r.is_finite() || r <= 0.0 {
        return;
    }

    match shape {
        crate::models::MarkerShape::Plus => {
            out.push(PathCommand::MoveTo(Point::new(Px(x - r), Px(y))));
            out.push(PathCommand::LineTo(Point::new(Px(x + r), Px(y))));
            out.push(PathCommand::MoveTo(Point::new(Px(x), Px(y - r))));
            out.push(PathCommand::LineTo(Point::new(Px(x), Px(y + r))));
        }
        crate::models::MarkerShape::X => {
            out.push(PathCommand::MoveTo(Point::new(Px(x - r), Px(y - r))));
            out.push(PathCommand::LineTo(Point::new(Px(x + r), Px(y + r))));
            out.push(PathCommand::MoveTo(Point::new(Px(x - r), Px(y + r))));
            out.push(PathCommand::LineTo(Point::new(Px(x + r), Px(y - r))));
        }
        crate::models::MarkerShape::Square => {
            let p0 = Point::new(Px(x - r), Px(y - r));
            let p1 = Point::new(Px(x + r), Px(y - r));
            let p2 = Point::new(Px(x + r), Px(y + r));
            let p3 = Point::new(Px(x - r), Px(y + r));
            out.push(PathCommand::MoveTo(p0));
            out.push(PathCommand::LineTo(p1));
            out.push(PathCommand::LineTo(p2));
            out.push(PathCommand::LineTo(p3));
            out.push(PathCommand::LineTo(p0));
        }
        crate::models::MarkerShape::Diamond => {
            let p0 = Point::new(Px(x), Px(y - r));
            let p1 = Point::new(Px(x + r), Px(y));
            let p2 = Point::new(Px(x), Px(y + r));
            let p3 = Point::new(Px(x - r), Px(y));
            out.push(PathCommand::MoveTo(p0));
            out.push(PathCommand::LineTo(p1));
            out.push(PathCommand::LineTo(p2));
            out.push(PathCommand::LineTo(p3));
            out.push(PathCommand::LineTo(p0));
        }
        crate::models::MarkerShape::TriangleUp => {
            let p0 = Point::new(Px(x), Px(y - r));
            let p1 = Point::new(Px(x + r), Px(y + r));
            let p2 = Point::new(Px(x - r), Px(y + r));
            out.push(PathCommand::MoveTo(p0));
            out.push(PathCommand::LineTo(p1));
            out.push(PathCommand::LineTo(p2));
            out.push(PathCommand::LineTo(p0));
        }
        crate::models::MarkerShape::TriangleDown => {
            let p0 = Point::new(Px(x), Px(y + r));
            let p1 = Point::new(Px(x + r), Px(y - r));
            let p2 = Point::new(Px(x - r), Px(y - r));
            out.push(PathCommand::MoveTo(p0));
            out.push(PathCommand::LineTo(p1));
            out.push(PathCommand::LineTo(p2));
            out.push(PathCommand::LineTo(p0));
        }
        crate::models::MarkerShape::Circle => {
            let segments = 12usize;
            let step = (std::f32::consts::PI * 2.0) / segments as f32;
            let p0 = Point::new(Px(x + r), Px(y));
            out.push(PathCommand::MoveTo(p0));
            for i in 1..=segments {
                let t = step * i as f32;
                let px = x + r * t.cos();
                let py = y + r * t.sin();
                if !px.is_finite() || !py.is_finite() {
                    continue;
                }
                out.push(PathCommand::LineTo(Point::new(Px(px), Px(py))));
            }
        }
    }
}

fn area_fill_commands_from_polyline(polyline: &[PathCommand], baseline_y: Px) -> Vec<PathCommand> {
    if polyline.is_empty() || !baseline_y.0.is_finite() {
        return Vec::new();
    }

    let mut out: Vec<PathCommand> = Vec::new();
    let mut segment: Vec<Point> = Vec::new();

    let mut flush_segment = |segment: &mut Vec<Point>| {
        if segment.len() < 2 {
            segment.clear();
            return;
        }

        let first = segment[0];
        let last = *segment.last().expect("len>=2");
        out.push(PathCommand::MoveTo(Point::new(first.x, baseline_y)));
        out.push(PathCommand::LineTo(first));
        for point in segment.iter().copied().skip(1) {
            out.push(PathCommand::LineTo(point));
        }
        out.push(PathCommand::LineTo(Point::new(last.x, baseline_y)));
        out.push(PathCommand::Close);
        segment.clear();
    };

    for command in polyline {
        match *command {
            PathCommand::MoveTo(point) => {
                flush_segment(&mut segment);
                segment.push(point);
            }
            PathCommand::LineTo(point) => {
                segment.push(point);
            }
            _ => {}
        }
    }

    flush_segment(&mut segment);
    out
}

fn shaded_band_commands_from_series(
    transform: PlotTransform,
    upper: &dyn crate::series::SeriesData,
    lower: &dyn crate::series::SeriesData,
) -> (Vec<PathCommand>, Vec<PathCommand>, Vec<PathCommand>) {
    let mut upper_commands = Vec::<PathCommand>::new();
    let mut lower_commands = Vec::<PathCommand>::new();
    let mut fill_commands = Vec::<PathCommand>::new();
    let mut segment = Vec::<(Point, Point)>::new();

    let mut flush_segment = |segment: &mut Vec<(Point, Point)>| {
        if segment.len() < 2 {
            segment.clear();
            return;
        }

        let first = segment[0];
        upper_commands.push(PathCommand::MoveTo(first.0));
        lower_commands.push(PathCommand::MoveTo(first.1));
        for (upper, lower) in segment.iter().copied().skip(1) {
            upper_commands.push(PathCommand::LineTo(upper));
            lower_commands.push(PathCommand::LineTo(lower));
        }

        fill_commands.push(PathCommand::MoveTo(first.0));
        for (upper, _) in segment.iter().copied().skip(1) {
            fill_commands.push(PathCommand::LineTo(upper));
        }
        for (_, lower) in segment.iter().rev().copied() {
            fill_commands.push(PathCommand::LineTo(lower));
        }
        fill_commands.push(PathCommand::Close);
        segment.clear();
    };

    if upper.is_sorted_by_x() && lower.is_sorted_by_x() {
        #[derive(Clone, Copy)]
        struct Cursor {
            idx: usize,
            prev: Option<DataPoint>,
            next: Option<DataPoint>,
        }

        impl Cursor {
            fn new() -> Self {
                Self {
                    idx: 0,
                    prev: None,
                    next: None,
                }
            }

            fn fetch_next(&mut self, series: &dyn crate::series::SeriesData) -> Option<DataPoint> {
                while self.idx < series.len() {
                    let idx = self.idx;
                    self.idx = self.idx.saturating_add(1);
                    let Some(point) = series.get(idx) else {
                        self.prev = None;
                        continue;
                    };
                    if !point.x.is_finite() || !point.y.is_finite() {
                        self.prev = None;
                        continue;
                    }
                    return Some(point);
                }
                None
            }

            fn ensure_next(&mut self, series: &dyn crate::series::SeriesData) {
                if self.next.is_none() {
                    self.next = self.fetch_next(series);
                }
            }

            fn next_x(&self) -> Option<f64> {
                self.next.map(|point| point.x)
            }

            fn starts_segment_at(&self, x: f64) -> bool {
                self.prev.is_none() && self.next_x().is_some_and(|next_x| next_x == x)
            }

            fn advance_if_at_x(&mut self, series: &dyn crate::series::SeriesData, x: f64) {
                if self.next_x().is_some_and(|next_x| next_x == x) {
                    self.prev = self.next;
                    self.next = self.fetch_next(series);
                }
            }

            fn sample_y(&self, x: f64) -> Option<f64> {
                if !x.is_finite() {
                    return None;
                }

                if let Some(next) = self.next
                    && next.x == x
                {
                    return Some(next.y);
                }

                match (self.prev, self.next) {
                    (Some(a), Some(b)) => {
                        if x < a.x || x > b.x {
                            return None;
                        }
                        let dx = b.x - a.x;
                        if dx == 0.0 || !dx.is_finite() {
                            return Some(b.y);
                        }
                        let t = (x - a.x) / dx;
                        if !t.is_finite() {
                            return None;
                        }
                        let y = a.y + (b.y - a.y) * t;
                        y.is_finite().then_some(y)
                    }
                    (Some(a), None) => (a.x == x).then_some(a.y),
                    (None, Some(b)) => (b.x == x).then_some(b.y),
                    (None, None) => None,
                }
            }
        }

        let mut upper_cursor = Cursor::new();
        let mut lower_cursor = Cursor::new();
        upper_cursor.ensure_next(upper);
        lower_cursor.ensure_next(lower);

        loop {
            let x = match (upper_cursor.next_x(), lower_cursor.next_x()) {
                (Some(a), Some(b)) => a.min(b),
                (Some(a), None) => a,
                (None, Some(b)) => b,
                (None, None) => break,
            };

            if !x.is_finite() {
                flush_segment(&mut segment);
                break;
            }

            if x < transform.data.x_min || x > transform.data.x_max {
                upper_cursor.advance_if_at_x(upper, x);
                lower_cursor.advance_if_at_x(lower, x);
                upper_cursor.ensure_next(upper);
                lower_cursor.ensure_next(lower);
                continue;
            }

            let starts_new_segment =
                upper_cursor.starts_segment_at(x) || lower_cursor.starts_segment_at(x);
            if starts_new_segment && !segment.is_empty() {
                flush_segment(&mut segment);
            }

            let (Some(upper_y), Some(lower_y)) =
                (upper_cursor.sample_y(x), lower_cursor.sample_y(x))
            else {
                flush_segment(&mut segment);
                upper_cursor.advance_if_at_x(upper, x);
                lower_cursor.advance_if_at_x(lower, x);
                upper_cursor.ensure_next(upper);
                lower_cursor.ensure_next(lower);
                continue;
            };
            let upper_px = transform.data_to_px(DataPoint { x, y: upper_y });
            let lower_px = transform.data_to_px(DataPoint { x, y: lower_y });
            if upper_px.x.0.is_finite()
                && upper_px.y.0.is_finite()
                && lower_px.x.0.is_finite()
                && lower_px.y.0.is_finite()
            {
                segment.push((upper_px, lower_px));
            } else {
                flush_segment(&mut segment);
            }

            upper_cursor.advance_if_at_x(upper, x);
            lower_cursor.advance_if_at_x(lower, x);
            upper_cursor.ensure_next(upper);
            lower_cursor.ensure_next(lower);
        }

        flush_segment(&mut segment);
        return (fill_commands, upper_commands, lower_commands);
    }

    let len = upper.len().min(lower.len());
    for index in 0..len {
        let (Some(upper_point), Some(lower_point)) = (upper.get(index), lower.get(index)) else {
            flush_segment(&mut segment);
            continue;
        };
        if !upper_point.x.is_finite()
            || !upper_point.y.is_finite()
            || !lower_point.x.is_finite()
            || !lower_point.y.is_finite()
            || upper_point.x != lower_point.x
        {
            flush_segment(&mut segment);
            continue;
        }
        let upper_px = transform.data_to_px(upper_point);
        let lower_px = transform.data_to_px(lower_point);
        if upper_px.x.0.is_finite()
            && upper_px.y.0.is_finite()
            && lower_px.x.0.is_finite()
            && lower_px.y.0.is_finite()
        {
            segment.push((upper_px, lower_px));
        } else {
            flush_segment(&mut segment);
        }
    }
    flush_segment(&mut segment);

    (fill_commands, upper_commands, lower_commands)
}

fn stems_commands_from_points(
    transform: PlotTransform,
    points: &[DataPoint],
    baseline: f32,
) -> Vec<PathCommand> {
    let Some(baseline_y) = transform.data_y_to_px(f64::from(baseline)) else {
        return Vec::new();
    };
    if !baseline_y.0.is_finite() {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(points.len().saturating_mul(2));
    for point in points {
        if !point.x.is_finite() || !point.y.is_finite() {
            continue;
        }
        let px = transform.data_to_px(*point);
        if !px.x.0.is_finite() || !px.y.0.is_finite() {
            continue;
        }
        out.push(PathCommand::MoveTo(Point::new(px.x, baseline_y)));
        out.push(PathCommand::LineTo(px));
    }
    out
}

fn step_commands_from_polyline(polyline: &[PathCommand], step_mode: StepMode) -> Vec<PathCommand> {
    if polyline.is_empty() {
        return Vec::new();
    }

    let mut out: Vec<PathCommand> = Vec::with_capacity(polyline.len().saturating_mul(2));
    let mut last: Option<Point> = None;

    for cmd in polyline {
        match *cmd {
            PathCommand::MoveTo(p) => {
                out.push(PathCommand::MoveTo(p));
                last = Some(p);
            }
            PathCommand::LineTo(p) => {
                let Some(prev) = last else {
                    out.push(PathCommand::MoveTo(p));
                    last = Some(p);
                    continue;
                };

                let mid = match step_mode {
                    StepMode::Pre => Point::new(prev.x, p.y),
                    StepMode::Post => Point::new(p.x, prev.y),
                };

                if mid != prev {
                    out.push(PathCommand::LineTo(mid));
                }
                if p != mid {
                    out.push(PathCommand::LineTo(p));
                }
                last = Some(p);
            }
            _ => {}
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cartesian::DataPoint;
    use crate::models::{
        AreaPlotModel, AreaSeries, BarSeries, BarsPlotModel, CandlestickPlotModel,
        CandlestickSeries, ErrorBar, ErrorBarsPlotModel, ErrorBarsSeries, HeatmapPlotModel,
        Histogram2DPlotModel, HistogramPlotModel, HistogramSeries, LinePlotModel, LineSeries,
        OhlcPoint, StemsPlotModel, StemsSeries, YAxis,
    };
    use crate::series::Series;
    use crate::state::{
        DragLineX, DragLineY, DragPoint, DragRect, InfLineX, InfLineY, PlotDragOutput,
        PlotDragPhase, PlotImage, PlotOutput, PlotState, PlotText, TagX, TagY,
    };
    use fret_core::{
        AppWindowId, Event, FrameId, ImageId, MaterialDescriptor, MaterialId,
        MaterialRegistrationError, MaterialService, Modifiers, MouseButton, MouseButtons,
        PathCommand, PathConstraints, PathId, PathMetrics, PathService, PointerEvent, PointerId,
        PointerType, Scene, SvgId, SvgService, TextBlobId, TextConstraints, TextInput, TextMetrics,
        TextService, UvRect,
    };
    use fret_runtime::{
        ClipboardToken, CommandRegistry, CommandsHost, DragHost, DragKindId, DragSession,
        DragSessionId, Effect, EffectSink, GlobalsHost, ImageUploadToken, ModelHost, ModelId,
        ModelStore, ModelsHost, ShareSheetToken, TickId, TimeHost, TimerToken,
    };
    use fret_ui::UiTree;
    use fret_ui::declarative::render_root;
    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    #[derive(Default)]
    struct TestHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
        models: ModelStore,
        commands: CommandRegistry,
        drags: HashMap<fret_core::PointerId, DragSession>,
        frame_id: FrameId,
        tick_id: TickId,
        next_timer_token: u64,
        next_clipboard_token: u64,
        next_share_sheet_token: u64,
        next_image_upload_token: u64,
        next_drag_session_id: u64,
    }

    impl TestHost {
        fn set_frame_id(&mut self, frame_id: FrameId) {
            self.frame_id = frame_id;
        }
    }

    impl GlobalsHost for TestHost {
        fn set_global<T: Any>(&mut self, value: T) {
            self.globals.insert(TypeId::of::<T>(), Box::new(value));
        }

        fn global<T: Any>(&self) -> Option<&T> {
            self.globals
                .get(&TypeId::of::<T>())
                .and_then(|value| value.downcast_ref::<T>())
        }

        fn with_global_mut<T: Any, R>(
            &mut self,
            init: impl FnOnce() -> T,
            f: impl FnOnce(&mut T, &mut Self) -> R,
        ) -> R {
            let type_id = TypeId::of::<T>();
            let mut value = self
                .globals
                .remove(&type_id)
                .map(|value| *value.downcast::<T>().expect("global type id should match"))
                .unwrap_or_else(init);
            let out = f(&mut value, self);
            self.globals.insert(type_id, Box::new(value));
            out
        }
    }

    impl ModelHost for TestHost {
        fn models(&self) -> &ModelStore {
            &self.models
        }

        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }
    }

    impl ModelsHost for TestHost {
        fn take_changed_models(&mut self) -> Vec<ModelId> {
            self.models.take_changed_models()
        }
    }

    impl CommandsHost for TestHost {
        fn commands(&self) -> &CommandRegistry {
            &self.commands
        }
    }

    impl EffectSink for TestHost {
        fn request_redraw(&mut self, _window: AppWindowId) {}

        fn push_effect(&mut self, _effect: Effect) {}
    }

    impl TimeHost for TestHost {
        fn tick_id(&self) -> TickId {
            self.tick_id
        }

        fn frame_id(&self) -> FrameId {
            self.frame_id
        }

        fn next_timer_token(&mut self) -> TimerToken {
            let token = TimerToken(self.next_timer_token);
            self.next_timer_token = self.next_timer_token.saturating_add(1);
            token
        }

        fn next_clipboard_token(&mut self) -> ClipboardToken {
            let token = ClipboardToken(self.next_clipboard_token);
            self.next_clipboard_token = self.next_clipboard_token.saturating_add(1);
            token
        }

        fn next_share_sheet_token(&mut self) -> ShareSheetToken {
            let token = ShareSheetToken(self.next_share_sheet_token);
            self.next_share_sheet_token = self.next_share_sheet_token.saturating_add(1);
            token
        }

        fn next_image_upload_token(&mut self) -> ImageUploadToken {
            let token = ImageUploadToken(self.next_image_upload_token);
            self.next_image_upload_token = self.next_image_upload_token.saturating_add(1);
            token
        }
    }

    impl DragHost for TestHost {
        fn drag(&self, pointer_id: fret_core::PointerId) -> Option<&DragSession> {
            self.drags.get(&pointer_id)
        }

        fn drag_mut(&mut self, pointer_id: fret_core::PointerId) -> Option<&mut DragSession> {
            self.drags.get_mut(&pointer_id)
        }

        fn cancel_drag(&mut self, pointer_id: fret_core::PointerId) {
            self.drags.remove(&pointer_id);
        }

        fn any_drag_session(&self, predicate: impl FnMut(&DragSession) -> bool) -> bool {
            self.drags.values().any(predicate)
        }

        fn find_drag_pointer_id(
            &self,
            mut predicate: impl FnMut(&DragSession) -> bool,
        ) -> Option<fret_core::PointerId> {
            self.drags
                .values()
                .find(|session| predicate(session))
                .map(|session| session.pointer_id)
        }

        fn cancel_drag_sessions(
            &mut self,
            mut predicate: impl FnMut(&DragSession) -> bool,
        ) -> Vec<fret_core::PointerId> {
            let pointer_ids: Vec<_> = self
                .drags
                .values()
                .filter(|session| predicate(session))
                .map(|session| session.pointer_id)
                .collect();
            for pointer_id in &pointer_ids {
                self.drags.remove(pointer_id);
            }
            pointer_ids
        }

        fn begin_drag_with_kind<T: Any>(
            &mut self,
            pointer_id: fret_core::PointerId,
            kind: DragKindId,
            source_window: AppWindowId,
            start: Point,
            payload: T,
        ) {
            let session_id = DragSessionId(self.next_drag_session_id);
            self.next_drag_session_id = self.next_drag_session_id.saturating_add(1);
            self.drags.insert(
                pointer_id,
                DragSession::new(session_id, pointer_id, source_window, kind, start, payload),
            );
        }

        fn begin_cross_window_drag_with_kind<T: Any>(
            &mut self,
            pointer_id: fret_core::PointerId,
            kind: DragKindId,
            source_window: AppWindowId,
            start: Point,
            payload: T,
        ) {
            let session_id = DragSessionId(self.next_drag_session_id);
            self.next_drag_session_id = self.next_drag_session_id.saturating_add(1);
            self.drags.insert(
                pointer_id,
                DragSession::new_cross_window(
                    session_id,
                    pointer_id,
                    source_window,
                    kind,
                    start,
                    payload,
                ),
            );
        }
    }

    #[derive(Default)]
    struct FakeServices {
        prepared_text: Vec<String>,
        prepared_paths: Vec<Vec<PathCommand>>,
    }

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            self.prepared_text.push(input.text().to_string());
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::default(),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            self.prepared_paths.push(commands.to_vec());
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    impl MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: MaterialDescriptor,
        ) -> Result<MaterialId, MaterialRegistrationError> {
            Err(MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: MaterialId) -> bool {
            true
        }
    }

    fn line_plot_selection_rects(scene: &Scene) -> Vec<Rect> {
        scene
            .ops()
            .iter()
            .filter_map(|op| match op {
                fret_core::SceneOp::Quad {
                    order: DrawOrder(5),
                    rect,
                    ..
                } => Some(*rect),
                _ => None,
            })
            .collect()
    }

    fn line_plot_reference_line_rects(scene: &Scene) -> Vec<Rect> {
        scene
            .ops()
            .iter()
            .filter_map(|op| match op {
                fret_core::SceneOp::Quad {
                    order: DrawOrder(3),
                    rect,
                    ..
                } => Some(*rect),
                _ => None,
            })
            .collect()
    }

    fn line_plot_image_regions(scene: &Scene) -> Vec<(Rect, UvRect, f32)> {
        scene
            .ops()
            .iter()
            .filter_map(|op| match op {
                fret_core::SceneOp::ImageRegion {
                    rect, uv, opacity, ..
                } => Some((*rect, *uv, *opacity)),
                _ => None,
            })
            .collect()
    }

    fn assert_line_plot_selection_rect(rect: Rect, x: f32, y: f32, width: f32, height: f32) {
        assert!(
            (rect.origin.x.0 - x).abs() < 0.01,
            "unexpected selection rect x: expected {x}, got {rect:?}"
        );
        assert!(
            (rect.origin.y.0 - y).abs() < 0.01,
            "unexpected selection rect y: expected {y}, got {rect:?}"
        );
        assert!(
            (rect.size.width.0 - width).abs() < 0.01,
            "unexpected selection rect width: expected {width}, got {rect:?}"
        );
        assert!(
            (rect.size.height.0 - height).abs() < 0.01,
            "unexpected selection rect height: expected {height}, got {rect:?}"
        );
    }

    #[test]
    fn line_plot_panel_paints_seeded_line_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 1.0 },
                        DataPoint { x: 1.0, y: 4.0 },
                        DataPoint { x: 2.0, y: 2.0 },
                    ],
                    true,
                ),
            )]));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-line-panel",
            |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let line_paths = scene
            .ops()
            .iter()
            .filter(|op| matches!(op, fret_core::SceneOp::Path { order, .. } if order.0 >= 1))
            .count();
        assert!(
            line_paths > 0,
            "declarative line plot panel should emit at least one path"
        );

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn area_plot_panel_paints_area_fill_and_stroke_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app.models_mut().insert(AreaPlotModel::from_series(vec![
            AreaSeries::new(
                "Area",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.2 },
                        DataPoint { x: 1.0, y: 0.8 },
                        DataPoint { x: 2.0, y: 0.4 },
                    ],
                    true,
                ),
            )
            .fill_alpha(0.25),
        ]));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-area-panel",
            |cx| vec![area_plot_panel(cx, AreaPlotPanelProps::new(model.clone()))],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let fill_paths = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Path {
                        order: DrawOrder(19),
                        ..
                    }
                )
            })
            .count();
        let stroke_paths = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Path {
                        order: DrawOrder(20),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(
            fill_paths, 1,
            "declarative area plot should emit one filled area path"
        );
        assert_eq!(
            stroke_paths, 1,
            "declarative area plot should keep the area stroke path"
        );
        assert!(
            services
                .prepared_paths
                .iter()
                .any(|path| path.iter().any(|cmd| matches!(cmd, PathCommand::Close))),
            "area fill path should close back to the baseline"
        );

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn error_bars_plot_panel_paints_x_y_errors_caps_and_markers_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(ErrorBarsPlotModel::from_series(vec![
                ErrorBarsSeries::new(
                    "measurement",
                    Series::from_points_sorted(vec![DataPoint { x: 1.0, y: 1.0 }], true),
                )
                .x_errors(std::sync::Arc::from([ErrorBar::symmetric(0.25)]))
                .y_errors(std::sync::Arc::from([ErrorBar::symmetric(0.5)]))
                .cap_size(Px(5.0))
                .marker_radius(Px(3.0)),
            ]));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-error-bars-panel",
            |cx| {
                vec![error_bars_plot_panel(
                    cx,
                    ErrorBarsPlotPanelProps::new(model.clone()),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let error_paths = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Path {
                        order: DrawOrder(20),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(
            error_paths, 1,
            "declarative error-bars plot should emit one path for the series error bars"
        );

        let error_path = services
            .prepared_paths
            .iter()
            .find(|path| path.len() >= 16)
            .expect("error-bars path should include y-error, x-error, caps, and plus marker");
        assert!(
            !error_path
                .iter()
                .any(|command| matches!(command, PathCommand::Close)),
            "default error-bars markers and caps should be open stroke commands"
        );

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn histogram_plot_panel_paints_closed_bin_fill_paths_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(HistogramPlotModel::from_series(vec![
                HistogramSeries::new("histogram", std::sync::Arc::from([0.1, 0.2, 0.8, 1.2, 1.8]))
                    .bins(2)
                    .range(0.0, 2.0)
                    .bar_gap_fraction(0.0),
            ]));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-histogram-panel",
            |cx| {
                vec![histogram_plot_panel(
                    cx,
                    HistogramPlotPanelProps::new(model.clone()),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let fill_paths = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Path {
                        order: DrawOrder(19),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(
            fill_paths, 1,
            "declarative histogram should emit one fill path for the series bins"
        );

        let histogram_path = services
            .prepared_paths
            .iter()
            .find(|path| {
                path.iter()
                    .filter(|cmd| matches!(cmd, PathCommand::Close))
                    .count()
                    >= 2
            })
            .expect("histogram fill path should close each non-empty bin");
        assert_eq!(
            histogram_path
                .iter()
                .filter(|cmd| matches!(cmd, PathCommand::Close))
                .count(),
            2,
            "the fixture should produce two closed histogram bin rectangles"
        );

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn heatmap_plot_panel_paints_grid_cells_as_declarative_quads() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app.models_mut().insert(HeatmapPlotModel::new(
            DataRect {
                x_min: 0.0,
                x_max: 2.0,
                y_min: 0.0,
                y_max: 2.0,
            },
            2,
            2,
            [0.0_f32, 0.5, 0.75, 1.0],
        ));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-heatmap-panel",
            |cx| {
                vec![heatmap_plot_panel(
                    cx,
                    HeatmapPlotPanelProps::new(model.clone()),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let heatmap_quads: Vec<_> = scene
            .ops()
            .iter()
            .filter_map(|op| match op {
                fret_core::SceneOp::Quad {
                    order: DrawOrder(2),
                    rect,
                    ..
                } => Some(*rect),
                _ => None,
            })
            .filter(|rect| rect.size.width.0 > 20.0 && rect.size.height.0 > 20.0)
            .collect();
        assert_eq!(
            heatmap_quads.len(),
            4,
            "declarative heatmap should emit one visible quad per finite grid cell"
        );

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn histogram2d_plot_panel_paints_grid_cells_and_default_colorbar_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app.models_mut().insert(Histogram2DPlotModel::new(
            DataRect {
                x_min: 0.0,
                x_max: 2.0,
                y_min: 0.0,
                y_max: 2.0,
            },
            2,
            2,
            [0.0_f32, 2.0, 3.0, 4.0],
        ));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-histogram2d-panel",
            |cx| {
                vec![histogram2d_plot_panel(
                    cx,
                    Histogram2DPlotPanelProps::new(model.clone()),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let histogram2d_quads: Vec<_> = scene
            .ops()
            .iter()
            .filter_map(|op| match op {
                fret_core::SceneOp::Quad {
                    order: DrawOrder(2),
                    rect,
                    ..
                } => Some(*rect),
                _ => None,
            })
            .filter(|rect| rect.size.width.0 > 20.0 && rect.size.height.0 > 20.0)
            .collect();
        assert_eq!(
            histogram2d_quads.len(),
            4,
            "declarative histogram2d should emit one visible quad per finite grid cell"
        );

        let gradient_steps = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Quad {
                        order: DrawOrder(4),
                        ..
                    }
                )
            })
            .count();
        assert!(
            gradient_steps >= 8,
            "declarative histogram2d should paint a default colorbar gradient"
        );

        assert!(
            services.prepared_text.iter().any(|text| text == "4.000"),
            "declarative histogram2d colorbar should label the finite maximum value"
        );
        assert!(
            services.prepared_text.iter().any(|text| text == "0.000"),
            "declarative histogram2d colorbar should label the finite minimum value"
        );

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn heatmap_plot_panel_paints_default_colorbar_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app.models_mut().insert(HeatmapPlotModel::new(
            DataRect {
                x_min: 0.0,
                x_max: 2.0,
                y_min: 0.0,
                y_max: 2.0,
            },
            2,
            2,
            [0.0_f32, 0.5, 0.75, 1.0],
        ));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-heatmap-colorbar-panel",
            |cx| {
                vec![heatmap_plot_panel(
                    cx,
                    HeatmapPlotPanelProps::new(model.clone()),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let gradient_steps = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Quad {
                        order: DrawOrder(4),
                        ..
                    }
                )
            })
            .count();
        assert!(
            gradient_steps >= 8,
            "declarative heatmap should paint a default colorbar gradient"
        );

        assert!(
            services.prepared_text.iter().any(|text| text == "1.000"),
            "declarative heatmap colorbar should label the finite maximum value"
        );
        assert!(
            services.prepared_text.iter().any(|text| text == "0.000"),
            "declarative heatmap colorbar should label the finite minimum value"
        );

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn candlestick_plot_panel_paints_wicks_and_up_down_bodies_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(CandlestickPlotModel::from_series(vec![
                CandlestickSeries::new_sorted(
                    "ohlc",
                    std::sync::Arc::from([
                        OhlcPoint {
                            x: 0.0,
                            open: 1.0,
                            high: 2.0,
                            low: 0.5,
                            close: 1.5,
                        },
                        OhlcPoint {
                            x: 1.0,
                            open: 2.0,
                            high: 2.5,
                            low: 1.0,
                            close: 1.25,
                        },
                    ]),
                    true,
                )
                .width(0.8),
            ]));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-candlestick-panel",
            |cx| {
                vec![candlestick_plot_panel(
                    cx,
                    CandlestickPlotPanelProps::new(model.clone()),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let wick_paths = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Path {
                        order: DrawOrder(20),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(
            wick_paths, 1,
            "declarative candlestick should emit one wick stroke path"
        );

        let body_paths = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Path {
                        order: DrawOrder(19),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(
            body_paths, 2,
            "declarative candlestick should emit separate up and down body fill paths"
        );

        let closed_body_paths = services
            .prepared_paths
            .iter()
            .filter(|path| path.iter().any(|cmd| matches!(cmd, PathCommand::Close)))
            .count();
        assert_eq!(
            closed_body_paths, 2,
            "up and down candle bodies should be closed fill rectangles"
        );

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn bars_plot_panel_paints_grouped_and_stacked_closed_fill_paths_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let grouped = BarSeries::new(
            "grouped",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 1.0 }, DataPoint { x: 1.0, y: 2.0 }],
                true,
            ),
        )
        .width(0.8);
        let stacked = BarSeries::new(
            "stacked",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 2.5 }, DataPoint { x: 1.0, y: -1.5 }],
                true,
            ),
        )
        .width(0.8)
        .baseline_by_index(std::sync::Arc::from([1.0, -0.5]));
        let model = app
            .models_mut()
            .insert(BarsPlotModel::from_series(vec![grouped, stacked]));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-bars-panel",
            |cx| vec![bars_plot_panel(cx, BarsPlotPanelProps::new(model.clone()))],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let fill_paths = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Path {
                        order: DrawOrder(19),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(
            fill_paths, 2,
            "declarative bars should emit one fill path per visible series"
        );

        let closed_bar_rects = services
            .prepared_paths
            .iter()
            .filter(|path| {
                path.iter()
                    .filter(|cmd| matches!(cmd, PathCommand::Close))
                    .count()
                    >= 2
            })
            .count();
        assert_eq!(
            closed_bar_rects, 2,
            "grouped and stacked series should each close both bar rectangles"
        );

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn shaded_plot_panel_paints_band_fill_and_two_strokes_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app.models_mut().insert(ShadedPlotModel::from_series(vec![
            crate::models::ShadedSeries::new(
                "Band",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.8 },
                        DataPoint { x: 1.0, y: 1.2 },
                        DataPoint { x: 2.0, y: 0.9 },
                    ],
                    true,
                ),
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.2 },
                        DataPoint { x: 1.0, y: 0.4 },
                        DataPoint { x: 2.0, y: 0.3 },
                    ],
                    true,
                ),
            )
            .fill_alpha(0.25),
        ]));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-shaded-panel",
            |cx| {
                vec![shaded_plot_panel(
                    cx,
                    ShadedPlotPanelProps::new(model.clone()),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let fill_paths = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Path {
                        order: DrawOrder(19),
                        ..
                    }
                )
            })
            .count();
        let stroke_paths = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Path {
                        order: DrawOrder(20),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(
            fill_paths, 1,
            "declarative shaded plot should emit one filled band path"
        );
        assert_eq!(
            stroke_paths, 2,
            "declarative shaded plot should emit upper and lower stroke paths"
        );
        assert!(
            services
                .prepared_paths
                .iter()
                .any(|path| path.iter().any(|cmd| matches!(cmd, PathCommand::Close))),
            "shaded fill path should close the upper/lower band"
        );

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn stems_plot_panel_paints_stems_from_baseline_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app.models_mut().insert(StemsPlotModel::from_series(vec![
            StemsSeries::new(
                "Stems",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.2 },
                        DataPoint { x: 1.0, y: 0.8 },
                        DataPoint { x: 2.0, y: 0.4 },
                    ],
                    true,
                ),
            )
            .baseline(0.0),
        ]));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-stems-panel",
            |cx| {
                vec![stems_plot_panel(
                    cx,
                    StemsPlotPanelProps::new(model.clone()),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let stem_paths = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Path {
                        order: DrawOrder(20),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(
            stem_paths, 1,
            "declarative stems plot should emit one stem path"
        );

        let stem_path = services
            .prepared_paths
            .iter()
            .find(|path| {
                path.windows(2).any(|commands| {
                    matches!(
                        (&commands[0], &commands[1]),
                        (PathCommand::MoveTo(_), PathCommand::LineTo(_))
                    )
                })
            })
            .expect("stems panel should prepare move/line stem commands");
        assert!(
            stem_path.len() >= 6,
            "three sampled stems should produce at least six path commands; got {stem_path:?}"
        );
        assert!(
            !stem_path
                .iter()
                .any(|cmd| matches!(cmd, PathCommand::Close)),
            "stems should be strokes from the baseline, not closed fills"
        );

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn line_plot_panel_paints_axes_and_grid_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.25 },
                    ],
                    true,
                ),
            )]));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-axes-grid",
            |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let axis_quads = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Quad {
                        order: DrawOrder(10),
                        ..
                    }
                )
            })
            .count();
        assert!(
            axis_quads >= 2,
            "declarative line plot should paint x/y axis lines"
        );

        let grid_quads = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Quad {
                        order: DrawOrder(2),
                        ..
                    }
                )
            })
            .count();
        assert!(
            grid_quads >= 2,
            "declarative line plot should paint tick-derived grid lines"
        );

        let line_paths = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Path {
                        order: DrawOrder(20),
                        ..
                    }
                )
            })
            .count();
        assert!(
            line_paths > 0,
            "declarative line plot should keep series paths above grid/axes"
        );

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn line_plot_panel_paints_axis_tick_labels_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(220.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.25 },
                    ],
                    true,
                ),
            )]));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-axis-labels",
            |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let axis_labels = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Text {
                        order: DrawOrder(11),
                        ..
                    }
                )
            })
            .count();
        assert!(
            axis_labels >= 4,
            "declarative line plot should paint x/y tick labels"
        );

        let series_paths = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Path {
                        order: DrawOrder(20),
                        ..
                    }
                )
            })
            .count();
        assert!(
            series_paths > 0,
            "axis label painting should not replace seeded series paths"
        );

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn line_plot_panel_paints_right_axis_tick_labels_with_custom_formatters_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(220.0)),
        );
        let mut services = FakeServices::default();
        let left = LineSeries::new(
            "left",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 4.0 }],
                true,
            ),
        );
        let right = LineSeries::new(
            "right",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 100.0 }],
                true,
            ),
        )
        .y_axis(YAxis::Right);
        let right2 = LineSeries::new(
            "right2",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.0 },
                    DataPoint { x: 4.0, y: 1_000.0 },
                ],
                true,
            ),
        )
        .y_axis(YAxis::Right2);
        let right3 = LineSeries::new(
            "right3",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.0 },
                    DataPoint { x: 4.0, y: 2_000.0 },
                ],
                true,
            ),
        )
        .y_axis(YAxis::Right3);
        let model = app.models_mut().insert(LinePlotModel::from_series(vec![
            left, right, right2, right3,
        ]));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-right-axis-labels",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .y2_axis_labels(AxisLabelFormatter::custom(0x5231u64, |v, _span| {
                            format!("R1:{v:.0}")
                        }))
                        .y3_axis_labels(AxisLabelFormatter::custom(0x5232u64, |v, _span| {
                            format!("R2:{v:.0}")
                        }))
                        .y4_axis_labels(AxisLabelFormatter::custom(0x5233u64, |v, _span| {
                            format!("R3:{v:.0}")
                        })),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        assert!(
            services
                .prepared_text
                .iter()
                .any(|text| text.starts_with("R1:")),
            "declarative line plot should use the y2 formatter for right-axis tick labels, got {:?}",
            services.prepared_text
        );
        assert!(
            services
                .prepared_text
                .iter()
                .any(|text| text.starts_with("R2:")),
            "declarative line plot should use the y3 formatter for right2-axis tick labels, got {:?}",
            services.prepared_text
        );
        assert!(
            services
                .prepared_text
                .iter()
                .any(|text| text.starts_with("R3:")),
            "declarative line plot should use the y4 formatter for right3-axis tick labels, got {:?}",
            services.prepared_text
        );

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn line_plot_panel_paints_series_legend_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(220.0)),
        );
        let mut services = FakeServices::default();
        let series = vec![
            LineSeries::new(
                "Alpha",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 1.0 },
                        DataPoint { x: 1.0, y: 2.0 },
                        DataPoint { x: 2.0, y: 1.5 },
                    ],
                    true,
                ),
            ),
            LineSeries::new(
                "Beta",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.5 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 2.5 },
                    ],
                    true,
                ),
            ),
        ];
        let model = app.models_mut().insert(LinePlotModel::from_series(series));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-legend",
            |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let legend_swatches = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Quad {
                        order: DrawOrder(30),
                        ..
                    }
                )
            })
            .count();
        assert!(
            legend_swatches >= 2,
            "declarative line plot should paint one legend swatch per series"
        );

        let legend_labels = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Text {
                        order: DrawOrder(31),
                        ..
                    }
                )
            })
            .count();
        assert!(
            legend_labels >= 2,
            "declarative line plot should paint one legend label per series"
        );

        let series_paths = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Path {
                        order: DrawOrder(20),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(
            series_paths, 2,
            "legend painting should not replace seeded series paths"
        );

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn line_plot_panel_paints_right_axis_series_with_right_axis_bounds_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let left_series = LineSeries::new(
            "left",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 1.0, y: 1.0 }],
                true,
            ),
        );
        let right_series = LineSeries::new(
            "right",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 1.0, y: 100.0 }],
                true,
            ),
        )
        .y_axis(YAxis::Right);
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![left_series, right_series]));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-right-axis-line-panel",
            |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        services.prepared_paths.clear();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let plot = line_plot_inner_rect(bounds, LinePlotStyle::default());
        let right_path = services
            .prepared_paths
            .iter()
            .find(|commands| {
                commands.iter().any(|command| match command {
                    PathCommand::LineTo(point) => (point.y.0 - plot.origin.y.0).abs() < 0.5,
                    _ => false,
                })
            })
            .cloned();
        assert!(
            right_path.is_some(),
            "declarative right-axis series should use right-axis y bounds and reach the plot top; paths={:?}",
            services.prepared_paths
        );

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn line_plot_panel_paints_right2_and_right3_axis_series_with_axis_bounds_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let left_series = LineSeries::new(
            "left",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 1.0, y: 1.0 }],
                true,
            ),
        );
        let right2_series = LineSeries::new(
            "right2",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 1.0, y: 200.0 }],
                true,
            ),
        )
        .y_axis(YAxis::Right2);
        let right3_series = LineSeries::new(
            "right3",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.0 },
                    DataPoint { x: 1.0, y: 3000.0 },
                ],
                true,
            ),
        )
        .y_axis(YAxis::Right3);
        let model = app.models_mut().insert(LinePlotModel::from_series(vec![
            left_series,
            right2_series,
            right3_series,
        ]));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-right23-axis-line-panel",
            |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        services.prepared_paths.clear();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let endpoint_y: Vec<f32> = services
            .prepared_paths
            .iter()
            .filter_map(|commands| {
                commands.iter().find_map(|command| match command {
                    PathCommand::LineTo(point) => Some(point.y.0),
                    _ => None,
                })
            })
            .collect();
        assert_eq!(
            endpoint_y.len(),
            3,
            "left, right2, and right3 series should each emit a line endpoint; paths={:?}",
            services.prepared_paths
        );
        let right2_endpoint_y = endpoint_y[1];
        assert_eq!(
            endpoint_y
                .iter()
                .skip(1)
                .filter(|y| (**y - right2_endpoint_y).abs() < 0.5)
                .count(),
            2,
            "right2 and right3 series should project their max y values to the same plot-space endpoint through their own y bounds; endpoint_y={endpoint_y:?}, paths={:?}",
            services.prepared_paths
        );

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn line_plot_panel_legend_swatch_click_toggles_series_visibility_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(220.0)),
        );
        let mut services = FakeServices::default();
        let series = vec![
            LineSeries::new(
                "Alpha",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 1.0 },
                        DataPoint { x: 1.0, y: 2.0 },
                        DataPoint { x: 2.0, y: 1.5 },
                    ],
                    true,
                ),
            ),
            LineSeries::new(
                "Beta",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.5 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 2.5 },
                    ],
                    true,
                ),
            ),
        ];
        let alpha_id = series[0].id;
        let model = app.models_mut().insert(LinePlotModel::from_series(series));
        let state = app.models_mut().insert(PlotState::default());

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-legend-toggle",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone()).state(state.clone()),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        let series_paths = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Path {
                        order: DrawOrder(20),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(series_paths, 2);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(42.0), Px(32.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let hidden = state
            .read_ref(&app, |state| state.hidden_series.clone())
            .expect("plot state should be readable");
        assert!(
            hidden.contains(&alpha_id),
            "clicking a declarative legend swatch should hide that series"
        );

        scene.clear();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        let series_paths = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Path {
                        order: DrawOrder(20),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(
            series_paths, 1,
            "hidden declarative legend series should be omitted from line painting"
        );
    }

    #[test]
    fn line_plot_panel_legend_label_click_pins_and_unpins_series_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(220.0)),
        );
        let mut services = FakeServices::default();
        let series = vec![
            LineSeries::new(
                "Alpha",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 1.0 },
                        DataPoint { x: 1.0, y: 2.0 },
                        DataPoint { x: 2.0, y: 1.5 },
                    ],
                    true,
                ),
            ),
            LineSeries::new(
                "Beta",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.5 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 2.5 },
                    ],
                    true,
                ),
            ),
        ];
        let beta_id = series[1].id;
        let model = app.models_mut().insert(LinePlotModel::from_series(series));
        let state = app.models_mut().insert(PlotState::default());

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-legend-pin",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone()).state(state.clone()),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(64.0), Px(48.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let pinned = state
            .read_ref(&app, |state| state.pinned_series)
            .expect("plot state should be readable");
        assert_eq!(
            pinned,
            Some(beta_id),
            "clicking a declarative legend label should pin that series"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(169.0), Px(81.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        services.prepared_text.clear();
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        let prepared_text = services.prepared_text.join("\n");
        assert!(
            prepared_text.contains("Beta: y="),
            "pinned declarative legend series should be kept in cursor readout rows: {prepared_text:?}"
        );
        assert!(
            !prepared_text.contains("Alpha: y="),
            "pinning Beta should filter other declarative cursor readout rows: {prepared_text:?}"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(64.0), Px(48.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let pinned = state
            .read_ref(&app, |state| state.pinned_series)
            .expect("plot state should be readable");
        assert_eq!(
            pinned, None,
            "clicking a pinned declarative legend label should unpin it"
        );

        services.prepared_text.clear();
        scene.clear();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        let prepared_text = services.prepared_text.join("\n");
        assert!(
            prepared_text.contains("Alpha: y=") && prepared_text.contains("Beta: y="),
            "unpinning should restore all visible declarative cursor readout rows: {prepared_text:?}"
        );
    }

    #[test]
    fn line_plot_panel_legend_shift_click_solos_and_restores_series_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(220.0)),
        );
        let mut services = FakeServices::default();
        let series = vec![
            LineSeries::new(
                "Alpha",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 1.0 },
                        DataPoint { x: 1.0, y: 2.0 },
                        DataPoint { x: 2.0, y: 1.5 },
                    ],
                    true,
                ),
            ),
            LineSeries::new(
                "Beta",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.5 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 2.5 },
                    ],
                    true,
                ),
            ),
            LineSeries::new(
                "Gamma",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 1.5 },
                        DataPoint { x: 1.0, y: 1.25 },
                        DataPoint { x: 2.0, y: 0.75 },
                    ],
                    true,
                ),
            ),
        ];
        let alpha_id = series[0].id;
        let beta_id = series[1].id;
        let gamma_id = series[2].id;
        let model = app.models_mut().insert(LinePlotModel::from_series(series));
        let state = app.models_mut().insert(PlotState::default());

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-legend-solo",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone()).state(state.clone()),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(42.0), Px(48.0)),
                button: MouseButton::Left,
                modifiers: Modifiers {
                    shift: true,
                    ..Modifiers::default()
                },
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let hidden = state
            .read_ref(&app, |state| state.hidden_series.clone())
            .expect("plot state should be readable");
        assert!(
            hidden.contains(&alpha_id) && hidden.contains(&gamma_id) && !hidden.contains(&beta_id),
            "shift-clicking a declarative legend row should solo that series"
        );

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        let series_paths = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Path {
                        order: DrawOrder(20),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(
            series_paths, 1,
            "soloed declarative legend series should be the only painted line"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(42.0), Px(48.0)),
                button: MouseButton::Left,
                modifiers: Modifiers {
                    shift: true,
                    ..Modifiers::default()
                },
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let hidden = state
            .read_ref(&app, |state| state.hidden_series.clone())
            .expect("plot state should be readable");
        assert!(
            hidden.is_empty(),
            "shift-clicking an already-solo declarative legend row should restore all series"
        );

        scene.clear();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        let series_paths = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Path {
                        order: DrawOrder(20),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(
            series_paths, 3,
            "restoring declarative legend solo mode should paint every line series again"
        );
    }

    #[test]
    fn line_plot_panel_legend_hover_emphasizes_series_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(220.0)),
        );
        let mut services = FakeServices::default();
        let alpha_color = Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let beta_color = Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        };
        let model = app.models_mut().insert(LinePlotModel::from_series(vec![
            LineSeries::new(
                "Alpha",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 1.0 },
                        DataPoint { x: 1.0, y: 2.0 },
                        DataPoint { x: 2.0, y: 1.5 },
                    ],
                    true,
                ),
            )
            .color(alpha_color),
            LineSeries::new(
                "Beta",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.5 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 2.5 },
                    ],
                    true,
                ),
            )
            .color(beta_color),
        ]));
        let state = app.models_mut().insert(PlotState::default());

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-legend-hover",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone()).state(state.clone()),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(64.0), Px(32.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let legend_highlights = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Quad {
                        order: DrawOrder(29),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(
            legend_highlights, 1,
            "hovering a declarative legend row should paint a legend highlight"
        );

        let mut alpha_path_alpha = None;
        let mut beta_path_alpha = None;
        for op in scene.ops() {
            let fret_core::SceneOp::Path {
                order: DrawOrder(20),
                paint,
                ..
            } = op
            else {
                continue;
            };
            if let Paint::Solid(color) = paint.paint {
                if (color.r - alpha_color.r).abs() < 0.001
                    && (color.g - alpha_color.g).abs() < 0.001
                    && (color.b - alpha_color.b).abs() < 0.001
                {
                    alpha_path_alpha = Some(color.a);
                } else if (color.g - beta_color.g).abs() < 0.001 {
                    beta_path_alpha = Some(color.a);
                }
            }
        }

        assert_eq!(
            alpha_path_alpha,
            Some(1.0),
            "hovered declarative legend series should keep full opacity"
        );
        assert!(
            beta_path_alpha.is_some_and(|alpha| alpha < 0.5),
            "non-hovered declarative line series should be dimmed while a legend row is hovered"
        );
    }

    #[test]
    fn line_plot_panel_uses_controlled_view_bounds_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        let state = app.models_mut().insert(plot_state);
        let output = app.models_mut().insert(PlotOutput::default());

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-controlled-view",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .output(output.clone()),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(169.0), Px(81.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let published = output
            .read_ref(&app, |output| *output)
            .expect("plot output model should be readable");
        assert_eq!(
            published.snapshot.view_bounds,
            DataRect {
                x_min: 0.0,
                x_max: 4.0,
                y_min: 0.0,
                y_max: 4.0,
            },
            "declarative line plot output should publish caller-controlled view bounds"
        );
        let cursor = published
            .snapshot
            .cursor
            .expect("pointer inside the controlled plot region should publish cursor data");
        assert!(
            (cursor.x - 2.0).abs() < 0.04,
            "expected pointer x to map through controlled view bounds, got {:?}",
            cursor
        );
        assert!(
            (cursor.y - 2.0).abs() < 0.08,
            "expected pointer y to map through controlled view bounds, got {:?}",
            cursor
        );
    }

    #[test]
    fn line_plot_panel_pans_controlled_view_bounds_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        let state = app.models_mut().insert(plot_state);

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-pan-view",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone()).state(state.clone()),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(169.0), Px(81.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(189.0), Px(81.0)),
                buttons: MouseButtons {
                    left: true,
                    ..MouseButtons::default()
                },
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position: Point::new(Px(189.0), Px(81.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: false,
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let updated = state
            .read_ref(&app, |state| (state.view_is_auto, state.view_bounds))
            .expect("plot state should be readable");
        let view = updated
            .1
            .expect("declarative panning should leave an explicit view bounds");
        assert!(
            !updated.0,
            "declarative panning should switch/keep plot view in controlled mode"
        );
        assert!(
            view.x_min < -0.20 && view.x_max < 3.80,
            "dragging right should pan the declarative view left in data space, got {view:?}"
        );
        assert!(
            (view.y_min - 0.0).abs() < 0.001 && (view.y_max - 4.0).abs() < 0.001,
            "horizontal pan should preserve y range, got {view:?}"
        );
    }

    #[test]
    fn line_plot_panel_pan_respects_x_pan_lock_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        plot_state.axis_locks.x.pan = true;
        let state = app.models_mut().insert(plot_state);

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-pan-x-lock-view",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone()).state(state.clone()),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(169.0), Px(81.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(189.0), Px(101.0)),
                buttons: MouseButtons {
                    left: true,
                    ..MouseButtons::default()
                },
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let view = state
            .read_ref(&app, |state| state.view_bounds)
            .expect("plot state should be readable")
            .expect("declarative panning should leave an explicit view bounds");
        assert!(
            (view.x_min - 0.0).abs() < 0.001 && (view.x_max - 4.0).abs() < 0.001,
            "X pan lock should preserve the declarative X range, got {view:?}"
        );
        assert!(
            view.y_min > 0.2 && view.y_max > 4.2,
            "X pan lock should still allow declarative Y panning, got {view:?}"
        );
    }

    #[test]
    fn line_plot_panel_pan_respects_y_pan_lock_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        plot_state.axis_locks.y.pan = true;
        let state = app.models_mut().insert(plot_state);

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-pan-y-lock-view",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone()).state(state.clone()),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(169.0), Px(81.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(189.0), Px(101.0)),
                buttons: MouseButtons {
                    left: true,
                    ..MouseButtons::default()
                },
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let view = state
            .read_ref(&app, |state| state.view_bounds)
            .expect("plot state should be readable")
            .expect("declarative panning should leave an explicit view bounds");
        assert!(
            view.x_min < -0.20 && view.x_max < 3.80,
            "Y pan lock should still allow declarative X panning, got {view:?}"
        );
        assert!(
            (view.y_min - 0.0).abs() < 0.001 && (view.y_max - 4.0).abs() < 0.001,
            "Y pan lock should preserve the declarative Y range, got {view:?}"
        );
    }

    #[test]
    fn line_plot_panel_pan_noops_when_both_axes_locked_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        plot_state.axis_locks.x.pan = true;
        plot_state.axis_locks.y.pan = true;
        let state = app.models_mut().insert(plot_state);

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-pan-both-lock-view",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone()).state(state.clone()),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(169.0), Px(81.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(189.0), Px(101.0)),
                buttons: MouseButtons {
                    left: true,
                    ..MouseButtons::default()
                },
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let view = state
            .read_ref(&app, |state| state.view_bounds)
            .expect("plot state should be readable")
            .expect("declarative panning should preserve explicit view bounds");
        assert_eq!(
            view,
            DataRect {
                x_min: 0.0,
                x_max: 4.0,
                y_min: 0.0,
                y_max: 4.0,
            },
            "panning should not change declarative view bounds when both axes are pan-locked"
        );
    }

    #[test]
    fn line_plot_panel_box_zoom_updates_controlled_view_bounds_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-box-zoom-view",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(100.0), Px(50.0)),
                button: MouseButton::Right,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(200.0), Px(120.0)),
                buttons: MouseButtons {
                    right: true,
                    ..MouseButtons::default()
                },
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position: Point::new(Px(200.0), Px(120.0)),
                button: MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: false,
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let updated = state
            .read_ref(&app, |state| (state.view_is_auto, state.view_bounds))
            .expect("plot state should be readable");
        let view = updated
            .1
            .expect("declarative box zoom should leave an explicit view bounds");
        assert!(
            !updated.0,
            "declarative box zoom should switch/keep plot view in controlled mode"
        );
        assert!(
            view.x_min > 0.9 && view.x_max < 2.6,
            "right-button box zoom should narrow the declarative X range, got {view:?}"
        );
        assert!(
            view.y_min > 0.8 && view.y_max < 3.0,
            "right-button box zoom should narrow the declarative Y range, got {view:?}"
        );
    }

    #[test]
    fn line_plot_panel_query_drag_updates_query_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-query-drag",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let alt = Modifiers {
            alt: true,
            ..Modifiers::default()
        };
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(100.0), Px(50.0)),
                button: MouseButton::Left,
                modifiers: alt,
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(200.0), Px(120.0)),
                buttons: MouseButtons {
                    left: true,
                    ..MouseButtons::default()
                },
                modifiers: alt,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position: Point::new(Px(200.0), Px(120.0)),
                button: MouseButton::Left,
                modifiers: alt,
                is_click: false,
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let query = state
            .read_ref(&app, |state| state.query)
            .expect("plot state should be readable")
            .expect("declarative query drag should write a query rect");
        assert!(
            query.x_min > 0.9 && query.x_max < 2.6,
            "Alt+left query drag should map the selected X range into data space, got {query:?}"
        );
        assert!(
            query.y_min > 0.8 && query.y_max < 3.1,
            "Alt+left query drag should map the selected Y range into data space, got {query:?}"
        );
    }

    #[test]
    fn line_plot_panel_query_drag_updates_output_query_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        let state = app.models_mut().insert(plot_state);
        let output = app.models_mut().insert(PlotOutput::default());
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-query-output",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .output(output.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let alt = Modifiers {
            alt: true,
            ..Modifiers::default()
        };
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(100.0), Px(50.0)),
                button: MouseButton::Left,
                modifiers: alt,
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(200.0), Px(120.0)),
                buttons: MouseButtons {
                    left: true,
                    ..MouseButtons::default()
                },
                modifiers: alt,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position: Point::new(Px(200.0), Px(120.0)),
                button: MouseButton::Left,
                modifiers: alt,
                is_click: false,
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let output_snapshot = output
            .read_ref(&app, |output| output.snapshot)
            .expect("plot output should be readable");
        let query = output_snapshot
            .query
            .expect("declarative query drag should publish query output");
        assert!(
            query.x_min > 0.9 && query.x_max < 2.6,
            "query output should include the selected X data range, got {query:?}"
        );
        assert!(
            query.y_min > 0.8 && query.y_max < 3.1,
            "query output should include the selected Y data range, got {query:?}"
        );
        assert_eq!(
            output_snapshot.view_bounds,
            DataRect {
                x_min: 0.0,
                x_max: 4.0,
                y_min: 0.0,
                y_max: 4.0,
            },
            "query output should keep reporting the current declarative view bounds"
        );
    }

    #[test]
    fn line_plot_panel_drags_right_axis_y_line_output_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let left_series = LineSeries::new(
            "left",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 4.0 }],
                true,
            ),
        );
        let right_series = LineSeries::new(
            "right",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 100.0 }],
                true,
            ),
        )
        .y_axis(YAxis::Right);
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![left_series, right_series]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        plot_state
            .overlays
            .drag_lines_y
            .push(DragLineY::new(50, 100.0, YAxis::Right));
        let state = app.models_mut().insert(plot_state);
        let output = app.models_mut().insert(PlotOutput::default());
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-right-axis-drag-line-y-output",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .output(output.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(169.0), Px(8.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(169.0), Px(81.0)),
                buttons: MouseButtons {
                    left: true,
                    ..MouseButtons::default()
                },
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let output_snapshot = output
            .read_ref(&app, |output| output.snapshot)
            .expect("plot output should be readable");
        let drag = output_snapshot
            .drag
            .expect("right-axis draggable Y line should publish drag output");
        match drag {
            PlotDragOutput::LineY { id, axis, y, phase } => {
                assert_eq!(id, 50);
                assert_eq!(axis, YAxis::Right);
                assert_eq!(phase, PlotDragPhase::Update);
                assert!(
                    (y - 50.0).abs() < 0.2,
                    "dragging to the plot middle should map through right-axis bounds, got {y}"
                );
            }
            other => panic!("expected right-axis LineY drag output, got {other:?}"),
        }

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position: Point::new(Px(169.0), Px(81.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: false,
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let output_snapshot = output
            .read_ref(&app, |output| output.snapshot)
            .expect("plot output should be readable");
        let drag = output_snapshot
            .drag
            .expect("right-axis draggable Y line should publish drag end output");
        match drag {
            PlotDragOutput::LineY { id, axis, y, phase } => {
                assert_eq!(id, 50);
                assert_eq!(axis, YAxis::Right);
                assert_eq!(phase, PlotDragPhase::End);
                assert!(
                    (y - 50.0).abs() < 0.2,
                    "drag end should preserve the right-axis mapped value, got {y}"
                );
            }
            other => panic!("expected right-axis LineY drag end output, got {other:?}"),
        }
    }

    #[test]
    fn line_plot_panel_drags_x_line_output_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 4.0 }],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        plot_state
            .overlays
            .drag_lines_x
            .push(DragLineX::new(60, 1.0));
        let state = app.models_mut().insert(plot_state);
        let output = app.models_mut().insert(PlotOutput::default());
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-drag-line-x-output",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .output(output.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(98.0), Px(81.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(169.0), Px(81.0)),
                buttons: MouseButtons {
                    left: true,
                    ..MouseButtons::default()
                },
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let output_snapshot = output
            .read_ref(&app, |output| output.snapshot)
            .expect("plot output should be readable");
        let drag = output_snapshot
            .drag
            .expect("draggable X line should publish drag output");
        match drag {
            PlotDragOutput::LineX { id, x, phase } => {
                assert_eq!(id, 60);
                assert_eq!(phase, PlotDragPhase::Update);
                assert!(
                    (x - 2.0).abs() < 0.03,
                    "dragging to the plot middle should map through the X view bounds, got {x}"
                );
            }
            other => panic!("expected LineX drag output, got {other:?}"),
        }

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position: Point::new(Px(169.0), Px(81.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: false,
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let output_snapshot = output
            .read_ref(&app, |output| output.snapshot)
            .expect("plot output should be readable");
        let drag = output_snapshot
            .drag
            .expect("draggable X line should publish drag end output");
        match drag {
            PlotDragOutput::LineX { id, x, phase } => {
                assert_eq!(id, 60);
                assert_eq!(phase, PlotDragPhase::End);
                assert!(
                    (x - 2.0).abs() < 0.03,
                    "drag end should preserve the X mapped value, got {x}"
                );
            }
            other => panic!("expected LineX drag end output, got {other:?}"),
        }
    }

    #[test]
    fn line_plot_panel_drags_right_axis_point_output_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let left_series = LineSeries::new(
            "left",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 4.0 }],
                true,
            ),
        );
        let right_series = LineSeries::new(
            "right",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 100.0 }],
                true,
            ),
        )
        .y_axis(YAxis::Right);
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![left_series, right_series]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        plot_state.overlays.drag_points.push(DragPoint::new(
            70,
            DataPoint { x: 2.0, y: 50.0 },
            YAxis::Right,
        ));
        let state = app.models_mut().insert(plot_state);
        let output = app.models_mut().insert(PlotOutput::default());
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-right-axis-drag-point-output",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .output(output.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(169.0), Px(81.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(240.5), Px(117.5)),
                buttons: MouseButtons {
                    left: true,
                    ..MouseButtons::default()
                },
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let output_snapshot = output
            .read_ref(&app, |output| output.snapshot)
            .expect("plot output should be readable");
        let drag = output_snapshot
            .drag
            .expect("right-axis draggable point should publish drag output");
        match drag {
            PlotDragOutput::Point {
                id,
                axis,
                point,
                phase,
            } => {
                assert_eq!(id, 70);
                assert_eq!(axis, YAxis::Right);
                assert_eq!(phase, PlotDragPhase::Update);
                assert!(
                    (point.x - 3.0).abs() < 0.03,
                    "dragging point right should map through the X view bounds, got {point:?}"
                );
                assert!(
                    (point.y - 25.0).abs() < 0.3,
                    "dragging point down should map through right-axis bounds, got {point:?}"
                );
            }
            other => panic!("expected right-axis Point drag output, got {other:?}"),
        }

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position: Point::new(Px(240.5), Px(117.5)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: false,
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let output_snapshot = output
            .read_ref(&app, |output| output.snapshot)
            .expect("plot output should be readable");
        let drag = output_snapshot
            .drag
            .expect("right-axis draggable point should publish drag end output");
        match drag {
            PlotDragOutput::Point {
                id,
                axis,
                point,
                phase,
            } => {
                assert_eq!(id, 70);
                assert_eq!(axis, YAxis::Right);
                assert_eq!(phase, PlotDragPhase::End);
                assert!(
                    (point.x - 3.0).abs() < 0.03 && (point.y - 25.0).abs() < 0.3,
                    "drag end should preserve the mapped point, got {point:?}"
                );
            }
            other => panic!("expected right-axis Point drag end output, got {other:?}"),
        }
    }

    #[test]
    fn line_plot_panel_drags_right_axis_rect_output_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let left_series = LineSeries::new(
            "left",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 4.0 }],
                true,
            ),
        );
        let right_series = LineSeries::new(
            "right",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 100.0 }],
                true,
            ),
        )
        .y_axis(YAxis::Right);
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![left_series, right_series]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        plot_state.overlays.drag_rects.push(DragRect::new(
            80,
            DataRect {
                x_min: 1.0,
                x_max: 3.0,
                y_min: 25.0,
                y_max: 75.0,
            },
            YAxis::Right,
        ));
        let state = app.models_mut().insert(plot_state);
        let output = app.models_mut().insert(PlotOutput::default());
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-right-axis-drag-rect-output",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .output(output.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(169.0), Px(81.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(240.5), Px(117.5)),
                buttons: MouseButtons {
                    left: true,
                    ..MouseButtons::default()
                },
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let output_snapshot = output
            .read_ref(&app, |output| output.snapshot)
            .expect("plot output should be readable");
        let drag = output_snapshot
            .drag
            .expect("right-axis draggable rect should publish drag output");
        match drag {
            PlotDragOutput::Rect {
                id,
                axis,
                rect,
                phase,
            } => {
                assert_eq!(id, 80);
                assert_eq!(axis, YAxis::Right);
                assert_eq!(phase, PlotDragPhase::Update);
                assert!(
                    (rect.x_min - 2.0).abs() < 0.03
                        && (rect.x_max - 4.0).abs() < 0.03
                        && (rect.y_min - 0.0).abs() < 0.3
                        && (rect.y_max - 50.0).abs() < 0.3,
                    "dragging inside the rect should move the whole right-axis rect, got {rect:?}"
                );
            }
            other => panic!("expected right-axis Rect drag output, got {other:?}"),
        }

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position: Point::new(Px(240.5), Px(117.5)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: false,
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let output_snapshot = output
            .read_ref(&app, |output| output.snapshot)
            .expect("plot output should be readable");
        let drag = output_snapshot
            .drag
            .expect("right-axis draggable rect should publish drag end output");
        match drag {
            PlotDragOutput::Rect {
                id,
                axis,
                rect,
                phase,
            } => {
                assert_eq!(id, 80);
                assert_eq!(axis, YAxis::Right);
                assert_eq!(phase, PlotDragPhase::End);
                assert!(
                    (rect.x_min - 2.0).abs() < 0.03
                        && (rect.x_max - 4.0).abs() < 0.03
                        && (rect.y_min - 0.0).abs() < 0.3
                        && (rect.y_max - 50.0).abs() < 0.3,
                    "drag end should preserve the mapped rect, got {rect:?}"
                );
            }
            other => panic!("expected right-axis Rect drag end output, got {other:?}"),
        }
    }

    #[test]
    fn line_plot_panel_paints_query_selection_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-query-selection",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let alt = Modifiers {
            alt: true,
            ..Modifiers::default()
        };
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(100.0), Px(50.0)),
                button: MouseButton::Left,
                modifiers: alt,
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(200.0), Px(120.0)),
                buttons: MouseButtons {
                    left: true,
                    ..MouseButtons::default()
                },
                modifiers: alt,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let mut active_scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut active_scene, 1.0);
        let active_rects = line_plot_selection_rects(&active_scene);
        assert_eq!(
            active_rects.len(),
            1,
            "active declarative query drag should paint one selection rectangle"
        );
        assert_line_plot_selection_rect(active_rects[0], 100.0, 50.0, 100.0, 70.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position: Point::new(Px(200.0), Px(120.0)),
                button: MouseButton::Left,
                modifiers: alt,
                is_click: false,
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let mut persisted_scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut persisted_scene, 1.0);
        let persisted_rects = line_plot_selection_rects(&persisted_scene);
        assert_eq!(
            persisted_rects.len(),
            1,
            "persisted declarative query state should paint one selection rectangle"
        );
        assert_line_plot_selection_rect(persisted_rects[0], 100.0, 50.0, 100.0, 70.0);
    }

    #[test]
    fn line_plot_panel_paints_box_zoom_selection_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-box-selection",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(100.0), Px(50.0)),
                button: MouseButton::Right,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(200.0), Px(120.0)),
                buttons: MouseButtons {
                    right: true,
                    ..MouseButtons::default()
                },
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let mut active_scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut active_scene, 1.0);
        let active_rects = line_plot_selection_rects(&active_scene);
        assert_eq!(
            active_rects.len(),
            1,
            "active declarative box zoom should paint one selection rectangle"
        );
        assert_line_plot_selection_rect(active_rects[0], 100.0, 50.0, 100.0, 70.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position: Point::new(Px(200.0), Px(120.0)),
                button: MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: false,
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let mut released_scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut released_scene, 1.0);
        assert!(
            line_plot_selection_rects(&released_scene).is_empty(),
            "box zoom selection rectangle should clear after applying the view change"
        );
    }

    #[test]
    fn line_plot_panel_paints_query_selection_tooltip_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-query-tooltip",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        services.prepared_text.clear();

        let alt = Modifiers {
            alt: true,
            ..Modifiers::default()
        };
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(100.0), Px(50.0)),
                button: MouseButton::Left,
                modifiers: alt,
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(200.0), Px(120.0)),
                buttons: MouseButtons {
                    left: true,
                    ..MouseButtons::default()
                },
                modifiers: alt,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let prepared_text = services.prepared_text.join("\n");
        assert!(
            prepared_text.contains("query\nx=["),
            "declarative query drag should paint a query selection tooltip, got {prepared_text:?}"
        );
        assert!(
            prepared_text.contains("y=["),
            "declarative query selection tooltip should include y-range text, got {prepared_text:?}"
        );
    }

    #[test]
    fn line_plot_panel_paints_box_zoom_selection_tooltip_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-box-tooltip",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        services.prepared_text.clear();

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(100.0), Px(50.0)),
                button: MouseButton::Right,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(200.0), Px(120.0)),
                buttons: MouseButtons {
                    right: true,
                    ..MouseButtons::default()
                },
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let prepared_text = services.prepared_text.join("\n");
        assert!(
            prepared_text.contains("zoom\nx=["),
            "declarative box zoom should paint a zoom selection tooltip, got {prepared_text:?}"
        );
        assert!(
            prepared_text.contains("y=["),
            "declarative box zoom tooltip should include y-range text, got {prepared_text:?}"
        );
    }

    #[test]
    fn line_plot_panel_paints_reference_lines_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        plot_state.overlays.inf_lines_x.push(InfLineX::new(2.0));
        plot_state
            .overlays
            .inf_lines_y
            .push(InfLineY::new(1.0, YAxis::Left));
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-reference-lines",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let reference_lines = line_plot_reference_line_rects(&scene);
        assert!(
            reference_lines.iter().any(|rect| {
                (rect.origin.x.0 - 169.0).abs() < 0.01
                    && (rect.origin.y.0 - 8.0).abs() < 0.01
                    && (rect.size.width.0 - 1.0).abs() < 0.01
                    && (rect.size.height.0 - 146.0).abs() < 0.01
            }),
            "declarative line plot should paint caller-owned X reference line, got {reference_lines:?}"
        );
        assert!(
            reference_lines.iter().any(|rect| {
                (rect.origin.x.0 - 26.0).abs() < 0.01
                    && (rect.origin.y.0 - 117.0).abs() < 0.01
                    && (rect.size.width.0 - 286.0).abs() < 0.01
                    && (rect.size.height.0 - 1.0).abs() < 0.01
            }),
            "declarative line plot should paint caller-owned Y reference line, got {reference_lines:?}"
        );
    }

    #[test]
    fn line_plot_panel_paints_draggable_lines_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        plot_state
            .overlays
            .drag_lines_x
            .push(DragLineX::new(10, 2.0));
        plot_state
            .overlays
            .drag_lines_y
            .push(DragLineY::new(11, 1.0, YAxis::Left));
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-draggable-lines",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let draggable_lines = line_plot_reference_line_rects(&scene);
        assert!(
            draggable_lines.iter().any(|rect| {
                (rect.origin.x.0 - 169.0).abs() < 0.01
                    && (rect.origin.y.0 - 8.0).abs() < 0.01
                    && (rect.size.width.0 - 1.0).abs() < 0.01
                    && (rect.size.height.0 - 146.0).abs() < 0.01
            }),
            "declarative line plot should paint caller-owned draggable X line, got {draggable_lines:?}"
        );
        assert!(
            draggable_lines.iter().any(|rect| {
                (rect.origin.x.0 - 26.0).abs() < 0.01
                    && (rect.origin.y.0 - 117.0).abs() < 0.01
                    && (rect.size.width.0 - 286.0).abs() < 0.01
                    && (rect.size.height.0 - 1.0).abs() < 0.01
            }),
            "declarative line plot should paint caller-owned draggable Y line, got {draggable_lines:?}"
        );
    }

    #[test]
    fn line_plot_panel_paints_draggable_point_and_rect_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        plot_state.overlays.drag_points.push(DragPoint::new(
            20,
            DataPoint { x: 2.0, y: 1.0 },
            YAxis::Left,
        ));
        plot_state.overlays.drag_rects.push(DragRect::new(
            21,
            DataRect {
                x_min: 1.0,
                x_max: 3.0,
                y_min: 1.0,
                y_max: 3.0,
            },
            YAxis::Left,
        ));
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-draggable-point-rect",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let draggable_shapes = line_plot_reference_line_rects(&scene);
        assert!(
            draggable_shapes.iter().any(|rect| {
                (rect.origin.x.0 - 165.0).abs() < 0.01
                    && (rect.origin.y.0 - 114.0).abs() < 0.01
                    && (rect.size.width.0 - 8.0).abs() < 0.01
                    && (rect.size.height.0 - 8.0).abs() < 0.01
            }),
            "declarative line plot should paint caller-owned draggable point, got {draggable_shapes:?}"
        );
        assert!(
            draggable_shapes.iter().any(|rect| {
                (rect.origin.x.0 - 98.0).abs() < 0.01
                    && (rect.origin.y.0 - 45.0).abs() < 0.01
                    && (rect.size.width.0 - 143.0).abs() < 0.01
                    && (rect.size.height.0 - 73.0).abs() < 0.01
            }),
            "declarative line plot should paint caller-owned draggable rect, got {draggable_shapes:?}"
        );
    }

    #[test]
    fn line_plot_panel_paints_right_axis_draggable_shapes_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let left_series = LineSeries::new(
            "left",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 4.0 }],
                true,
            ),
        );
        let right_series = LineSeries::new(
            "right",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 100.0 }],
                true,
            ),
        )
        .y_axis(YAxis::Right);
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![left_series, right_series]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        plot_state
            .overlays
            .drag_lines_y
            .push(DragLineY::new(50, 100.0, YAxis::Right));
        plot_state.overlays.drag_points.push(DragPoint::new(
            51,
            DataPoint { x: 2.0, y: 50.0 },
            YAxis::Right,
        ));
        plot_state.overlays.drag_rects.push(DragRect::new(
            52,
            DataRect {
                x_min: 1.0,
                x_max: 3.0,
                y_min: 25.0,
                y_max: 75.0,
            },
            YAxis::Right,
        ));
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-right-axis-draggable-shapes",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let draggable_shapes = line_plot_reference_line_rects(&scene);
        assert!(
            draggable_shapes.iter().any(|rect| {
                (rect.origin.x.0 - 26.0).abs() < 0.01
                    && (rect.origin.y.0 - 8.0).abs() < 0.01
                    && (rect.size.width.0 - 286.0).abs() < 0.01
                    && (rect.size.height.0 - 1.0).abs() < 0.01
            }),
            "declarative line plot should paint right-axis draggable Y line, got {draggable_shapes:?}"
        );
        assert!(
            draggable_shapes.iter().any(|rect| {
                (rect.origin.x.0 - 165.0).abs() < 0.01
                    && (rect.origin.y.0 - 77.0).abs() < 0.01
                    && (rect.size.width.0 - 8.0).abs() < 0.01
                    && (rect.size.height.0 - 8.0).abs() < 0.01
            }),
            "declarative line plot should paint right-axis draggable point, got {draggable_shapes:?}"
        );
        assert!(
            draggable_shapes.iter().any(|rect| {
                (rect.origin.x.0 - 98.0).abs() < 0.01
                    && (rect.origin.y.0 - 45.0).abs() < 0.01
                    && (rect.size.width.0 - 143.0).abs() < 0.01
                    && (rect.size.height.0 - 73.0).abs() < 0.01
            }),
            "declarative line plot should paint right-axis draggable rect, got {draggable_shapes:?}"
        );
    }

    #[test]
    fn line_plot_panel_paints_plot_text_overlay_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        plot_state.overlays.text.push(
            PlotText::new(2.0, 1.0, YAxis::Left, "threshold note")
                .background(Color::from_srgb_hex_rgb(0x19_33_4c))
                .padding(Px(4.0))
                .offset(Point::new(Px(4.0), Px(-6.0))),
        );
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-plot-text-overlay",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        services.prepared_text.clear();
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let prepared_text = services.prepared_text.join("\n");
        assert!(
            prepared_text.contains("threshold note"),
            "declarative line plot should prepare caller-owned PlotText overlay text, got {prepared_text:?}"
        );

        let text_backgrounds = line_plot_reference_line_rects(&scene);
        assert!(
            text_backgrounds.iter().any(|rect| {
                (rect.origin.x.0 - 173.0).abs() < 0.01
                    && (rect.origin.y.0 - 112.0).abs() < 0.01
                    && (rect.size.width.0 - 8.0).abs() < 0.01
                    && (rect.size.height.0 - 8.0).abs() < 0.01
            }),
            "declarative line plot should paint retained-compatible PlotText background, got {text_backgrounds:?}"
        );
    }

    #[test]
    fn line_plot_panel_paints_tag_x_and_y_overlays_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        plot_state
            .overlays
            .tags_x
            .push(TagX::new(2.0).label("X Gate").show_value(false));
        plot_state.overlays.tags_y.push(
            TagY::new(1.0, YAxis::Left)
                .label("Y Gate")
                .show_value(false),
        );
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-tags",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        services.prepared_text.clear();
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let prepared_text = services.prepared_text.join("\n");
        assert!(
            prepared_text.contains("X Gate"),
            "declarative line plot should prepare caller-owned TagX text, got {prepared_text:?}"
        );
        assert!(
            prepared_text.contains("Y Gate"),
            "declarative line plot should prepare caller-owned TagY text, got {prepared_text:?}"
        );

        let tag_rects = line_plot_reference_line_rects(&scene);
        assert!(
            tag_rects.iter().any(|rect| {
                (rect.origin.x.0 - 168.0).abs() < 0.01
                    && (rect.origin.y.0 - 146.0).abs() < 0.01
                    && (rect.size.width.0 - 2.0).abs() < 0.01
                    && (rect.size.height.0 - 8.0).abs() < 0.01
            }),
            "declarative line plot should paint retained-compatible TagX marker, got {tag_rects:?}"
        );
        assert!(
            tag_rects.iter().any(|rect| {
                (rect.origin.x.0 - 26.0).abs() < 0.01
                    && (rect.origin.y.0 - 117.0).abs() < 0.01
                    && (rect.size.width.0 - 8.0).abs() < 0.01
                    && (rect.size.height.0 - 2.0).abs() < 0.01
            }),
            "declarative line plot should paint retained-compatible left-axis TagY marker, got {tag_rects:?}"
        );
    }

    #[test]
    fn line_plot_panel_paints_draggable_overlay_labels_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        plot_state
            .overlays
            .drag_lines_x
            .push(DragLineX::new(30, 2.0).label("X Drag").show_value(false));
        plot_state.overlays.drag_lines_y.push(
            DragLineY::new(31, 1.0, YAxis::Left)
                .label("Y Drag")
                .show_value(false),
        );
        plot_state.overlays.drag_points.push(
            DragPoint::new(32, DataPoint { x: 2.0, y: 1.0 }, YAxis::Left).label("Point Drag"),
        );
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-drag-labels",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        services.prepared_text.clear();
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let prepared_text = services.prepared_text.join("\n");
        assert!(
            prepared_text.contains("X Drag"),
            "declarative line plot should prepare draggable X-line label text, got {prepared_text:?}"
        );
        assert!(
            prepared_text.contains("Y Drag"),
            "declarative line plot should prepare draggable Y-line label text, got {prepared_text:?}"
        );
        assert!(
            prepared_text.contains("Point Drag"),
            "declarative line plot should prepare draggable point label text, got {prepared_text:?}"
        );

        let label_rects = line_plot_reference_line_rects(&scene);
        assert!(
            label_rects.iter().any(|rect| {
                (rect.origin.x.0 - 168.0).abs() < 0.01
                    && (rect.origin.y.0 - 146.0).abs() < 0.01
                    && (rect.size.width.0 - 2.0).abs() < 0.01
                    && (rect.size.height.0 - 8.0).abs() < 0.01
            }),
            "declarative line plot should paint retained-compatible draggable X-line label marker, got {label_rects:?}"
        );
        assert!(
            label_rects.iter().any(|rect| {
                (rect.origin.x.0 - 26.0).abs() < 0.01
                    && (rect.origin.y.0 - 117.0).abs() < 0.01
                    && (rect.size.width.0 - 8.0).abs() < 0.01
                    && (rect.size.height.0 - 2.0).abs() < 0.01
            }),
            "declarative line plot should paint retained-compatible draggable Y-line label marker, got {label_rects:?}"
        );
    }

    #[test]
    fn line_plot_panel_paints_right_axis_draggable_overlay_labels_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let left_series = LineSeries::new(
            "left",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 4.0 }],
                true,
            ),
        );
        let right_series = LineSeries::new(
            "right",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 100.0 }],
                true,
            ),
        )
        .y_axis(YAxis::Right);
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![left_series, right_series]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        plot_state.overlays.drag_lines_y.push(
            DragLineY::new(40, 100.0, YAxis::Right)
                .label("Right Y Drag")
                .show_value(false),
        );
        plot_state.overlays.drag_points.push(
            DragPoint::new(41, DataPoint { x: 2.0, y: 50.0 }, YAxis::Right)
                .label("Right Point Drag"),
        );
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-right-axis-drag-labels",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        services.prepared_text.clear();
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let prepared_text = services.prepared_text.join("\n");
        assert!(
            prepared_text.contains("Right Y Drag"),
            "declarative line plot should prepare right-axis draggable Y-line label text, got {prepared_text:?}"
        );
        assert!(
            prepared_text.contains("Right Point Drag"),
            "declarative line plot should prepare right-axis draggable point label text, got {prepared_text:?}"
        );

        let label_rects = line_plot_reference_line_rects(&scene);
        assert!(
            label_rects.iter().any(|rect| {
                (rect.origin.x.0 - 304.0).abs() < 0.01
                    && (rect.origin.y.0 - 8.0).abs() < 0.01
                    && (rect.size.width.0 - 8.0).abs() < 0.01
                    && (rect.size.height.0 - 2.0).abs() < 0.01
            }),
            "declarative line plot should paint retained-compatible right-axis draggable Y-line label marker, got {label_rects:?}"
        );
    }

    #[test]
    fn line_plot_panel_paints_plot_image_overlay_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        let uv = UvRect {
            u0: 0.25,
            v0: 0.10,
            u1: 0.75,
            v1: 0.90,
        };
        plot_state.overlays.images.push(
            PlotImage::new(
                ImageId::default(),
                DataRect {
                    x_min: 1.0,
                    x_max: 3.0,
                    y_min: 1.0,
                    y_max: 3.0,
                },
                YAxis::Left,
            )
            .uv(uv)
            .opacity(0.5),
        );
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-image-overlay",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let image_regions = line_plot_image_regions(&scene);
        assert!(
            image_regions.iter().any(|(rect, found_uv, opacity)| {
                (rect.origin.x.0 - 97.5).abs() < 0.01
                    && (rect.origin.y.0 - 44.5).abs() < 0.01
                    && (rect.size.width.0 - 143.0).abs() < 0.01
                    && (rect.size.height.0 - 73.0).abs() < 0.01
                    && *found_uv == uv
                    && (*opacity - 0.5).abs() < 0.01
            }),
            "declarative line plot should paint caller-owned PlotImage overlay, got {image_regions:?}"
        );
    }

    #[test]
    fn line_plot_panel_paints_right_axis_plot_image_overlays_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let left_series = LineSeries::new(
            "left",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 4.0 }],
                true,
            ),
        );
        let right2_series = LineSeries::new(
            "right2",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 200.0 }],
                true,
            ),
        )
        .y_axis(YAxis::Right2);
        let right3_series = LineSeries::new(
            "right3",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.0 },
                    DataPoint { x: 4.0, y: 3000.0 },
                ],
                true,
            ),
        )
        .y_axis(YAxis::Right3);
        let model = app.models_mut().insert(LinePlotModel::from_series(vec![
            left_series,
            right2_series,
            right3_series,
        ]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        plot_state.overlays.images.push(
            PlotImage::new(
                ImageId::default(),
                DataRect {
                    x_min: 1.0,
                    x_max: 3.0,
                    y_min: 0.0,
                    y_max: 200.0,
                },
                YAxis::Right2,
            )
            .opacity(0.42),
        );
        plot_state.overlays.images.push(
            PlotImage::new(
                ImageId::default(),
                DataRect {
                    x_min: 1.0,
                    x_max: 3.0,
                    y_min: 0.0,
                    y_max: 3000.0,
                },
                YAxis::Right3,
            )
            .opacity(0.43),
        );
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-right-axis-image-overlays",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let image_regions = line_plot_image_regions(&scene);
        for expected_opacity in [0.42, 0.43] {
            assert!(
                image_regions.iter().any(|(rect, _uv, opacity)| {
                    (rect.origin.x.0 - 97.5).abs() < 0.01
                        && (rect.origin.y.0 - 8.0).abs() < 0.01
                        && (rect.size.width.0 - 143.0).abs() < 0.01
                        && (rect.size.height.0 - 146.0).abs() < 0.01
                        && (*opacity - expected_opacity).abs() < 0.01
                }),
                "declarative line plot should paint right-axis PlotImage overlay with opacity {expected_opacity}, got {image_regions:?}"
            );
        }
    }

    #[test]
    fn line_plot_panel_paints_right_axis_tag_y_and_plot_text_overlays_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let left_series = LineSeries::new(
            "left",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 4.0 }],
                true,
            ),
        );
        let right_series = LineSeries::new(
            "right",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 100.0 }],
                true,
            ),
        )
        .y_axis(YAxis::Right);
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![left_series, right_series]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        plot_state.overlays.tags_y.push(
            TagY::new(100.0, YAxis::Right)
                .label("threshold")
                .show_value(true),
        );
        plot_state.overlays.text.push(
            PlotText::new(2.0, 50.0, YAxis::Right, "right-axis note")
                .background(Color::from_srgb_hex_rgb(0x0A141E)),
        );
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-right-axis-tagy-text",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let tag_y_quads = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Quad {
                        order: DrawOrder(3),
                        ..
                    }
                )
            })
            .count();
        let tag_y_texts = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Text {
                        order: DrawOrder(3),
                        ..
                    }
                )
            })
            .count();
        assert!(
            tag_y_quads >= 2 && tag_y_texts >= 2,
            "declarative line plot should paint right-axis TagY and PlotText overlays"
        );
    }
    #[test]
    fn line_plot_panel_wheel_zooms_controlled_view_bounds_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-wheel-zoom-view",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Wheel {
                position: Point::new(Px(169.0), Px(81.0)),
                delta: Point::new(Px(0.0), Px(120.0)),
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let updated = state
            .read_ref(&app, |state| (state.view_is_auto, state.view_bounds))
            .expect("plot state should be readable");
        let view = updated
            .1
            .expect("declarative wheel zoom should leave an explicit view bounds");
        assert!(
            !updated.0,
            "declarative wheel zoom should switch/keep plot view in controlled mode"
        );
        assert!(
            view.x_max - view.x_min < 4.0 && view.y_max - view.y_min < 4.0,
            "positive wheel delta should zoom the declarative view in around the pointer, got {view:?}"
        );
        assert!(
            view.x_min > 0.0 && view.x_max < 4.0 && view.y_min > 0.0 && view.y_max < 4.0,
            "center wheel zoom should keep the next view inside the previous bounds, got {view:?}"
        );
    }

    #[test]
    fn line_plot_panel_wheel_zoom_shift_modifier_zooms_x_only_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-wheel-x-only-view",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Wheel {
                position: Point::new(Px(169.0), Px(81.0)),
                delta: Point::new(Px(0.0), Px(120.0)),
                modifiers: Modifiers {
                    shift: true,
                    ..Modifiers::default()
                },
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let view = state
            .read_ref(&app, |state| state.view_bounds)
            .expect("plot state should be readable")
            .expect("declarative wheel zoom should leave an explicit view bounds");
        assert!(
            view.x_max - view.x_min < 4.0,
            "Shift+wheel should zoom the declarative X range, got {view:?}"
        );
        assert!(
            (view.y_min - 0.0).abs() < 0.001 && (view.y_max - 4.0).abs() < 0.001,
            "Shift+wheel should preserve the declarative Y range, got {view:?}"
        );
    }

    #[test]
    fn line_plot_panel_wheel_zoom_ctrl_modifier_zooms_y_only_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-wheel-y-only-view",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Wheel {
                position: Point::new(Px(169.0), Px(81.0)),
                delta: Point::new(Px(0.0), Px(120.0)),
                modifiers: Modifiers {
                    ctrl: true,
                    ..Modifiers::default()
                },
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let view = state
            .read_ref(&app, |state| state.view_bounds)
            .expect("plot state should be readable")
            .expect("declarative wheel zoom should leave an explicit view bounds");
        assert!(
            (view.x_min - 0.0).abs() < 0.001 && (view.x_max - 4.0).abs() < 0.001,
            "Ctrl+wheel should preserve the declarative X range, got {view:?}"
        );
        assert!(
            view.y_max - view.y_min < 4.0,
            "Ctrl+wheel should zoom the declarative Y range, got {view:?}"
        );
    }

    #[test]
    fn line_plot_panel_wheel_zoom_on_x_axis_zooms_x_only_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-wheel-x-axis-view",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Wheel {
                position: Point::new(Px(169.0), Px(163.0)),
                delta: Point::new(Px(0.0), Px(120.0)),
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let view = state
            .read_ref(&app, |state| state.view_bounds)
            .expect("plot state should be readable")
            .expect("declarative x-axis wheel zoom should leave an explicit view bounds");
        assert!(
            view.x_max - view.x_min < 4.0,
            "wheel over the declarative X axis should zoom the X range, got {view:?}"
        );
        assert!(
            (view.y_min - 0.0).abs() < 0.001 && (view.y_max - 4.0).abs() < 0.001,
            "wheel over the declarative X axis should preserve the Y range, got {view:?}"
        );
    }

    #[test]
    fn line_plot_panel_wheel_zoom_on_y_axis_zooms_y_only_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-wheel-y-axis-view",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Wheel {
                position: Point::new(Px(17.0), Px(81.0)),
                delta: Point::new(Px(0.0), Px(120.0)),
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let view = state
            .read_ref(&app, |state| state.view_bounds)
            .expect("plot state should be readable")
            .expect("declarative y-axis wheel zoom should leave an explicit view bounds");
        assert!(
            (view.x_min - 0.0).abs() < 0.001 && (view.x_max - 4.0).abs() < 0.001,
            "wheel over the declarative Y axis should preserve the X range, got {view:?}"
        );
        assert!(
            view.y_max - view.y_min < 4.0,
            "wheel over the declarative Y axis should zoom the Y range, got {view:?}"
        );
    }

    #[test]
    fn line_plot_panel_wheel_zoom_respects_x_zoom_lock_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        plot_state.axis_locks.x.zoom = true;
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-wheel-x-lock-view",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Wheel {
                position: Point::new(Px(169.0), Px(81.0)),
                delta: Point::new(Px(0.0), Px(120.0)),
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let view = state
            .read_ref(&app, |state| state.view_bounds)
            .expect("plot state should be readable")
            .expect("declarative locked wheel zoom should leave an explicit view bounds");
        assert!(
            (view.x_min - 0.0).abs() < 0.001 && (view.x_max - 4.0).abs() < 0.001,
            "X zoom lock should preserve the declarative X range, got {view:?}"
        );
        assert!(
            view.y_max - view.y_min < 4.0,
            "X zoom lock should still allow declarative Y zoom, got {view:?}"
        );
    }

    #[test]
    fn line_plot_panel_wheel_zoom_respects_y_zoom_lock_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        plot_state.axis_locks.y.zoom = true;
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-wheel-y-lock-view",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Wheel {
                position: Point::new(Px(169.0), Px(81.0)),
                delta: Point::new(Px(0.0), Px(120.0)),
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let view = state
            .read_ref(&app, |state| state.view_bounds)
            .expect("plot state should be readable")
            .expect("declarative locked wheel zoom should leave an explicit view bounds");
        assert!(
            view.x_max - view.x_min < 4.0,
            "Y zoom lock should still allow declarative X zoom, got {view:?}"
        );
        assert!(
            (view.y_min - 0.0).abs() < 0.001 && (view.y_max - 4.0).abs() < 0.001,
            "Y zoom lock should preserve the declarative Y range, got {view:?}"
        );
    }

    #[test]
    fn line_plot_panel_wheel_zoom_noops_when_both_axes_locked_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.view_is_auto = false;
        plot_state.view_bounds = Some(DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        });
        plot_state.axis_locks.x.zoom = true;
        plot_state.axis_locks.y.zoom = true;
        let state = app.models_mut().insert(plot_state);
        let style = LinePlotStyle {
            clamp_to_data_bounds: false,
            ..LinePlotStyle::default()
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-wheel-both-lock-view",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone())
                        .state(state.clone())
                        .style(style),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Wheel {
                position: Point::new(Px(169.0), Px(81.0)),
                delta: Point::new(Px(0.0), Px(120.0)),
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let view = state
            .read_ref(&app, |state| state.view_bounds)
            .expect("plot state should be readable")
            .expect("declarative locked wheel zoom should preserve explicit view bounds");
        assert_eq!(
            view,
            DataRect {
                x_min: 0.0,
                x_max: 4.0,
                y_min: 0.0,
                y_max: 4.0,
            },
            "wheel zoom should not change declarative view bounds when both axes are zoom-locked"
        );
    }

    #[test]
    fn line_plot_panel_updates_output_cursor_on_pointer_move() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let output = app.models_mut().insert(PlotOutput::default());

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-pointer-output",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone()).output(output.clone()),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(169.0), Px(81.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let published = output
            .read_ref(&app, |output| *output)
            .expect("plot output model should be readable");
        assert_eq!(published.revision, 1);
        let cursor = published
            .snapshot
            .cursor
            .expect("pointer inside the plot region should publish cursor data");
        assert!(
            (cursor.x - 1.0).abs() < 0.02,
            "expected pointer x to map to the middle of the data domain, got {:?}",
            cursor
        );
        assert!(
            (cursor.y - 0.5).abs() < 0.04,
            "expected pointer y to map to the middle of the data domain, got {:?}",
            cursor
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(4.0), Px(4.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        let published = output
            .read_ref(&app, |output| *output)
            .expect("plot output model should be readable");
        assert_eq!(published.revision, 2);
        assert_eq!(published.snapshot.cursor, None);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        assert!(
            scene.ops().iter().any(|op| matches!(
                op,
                fret_core::SceneOp::Path {
                    order: DrawOrder(20),
                    ..
                }
            )),
            "managed-surface pointer handling must preserve declarative line painting"
        );
    }

    #[test]
    fn line_plot_panel_paints_cursor_readout_without_output_model_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-cursor-readout",
            |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(169.0), Px(81.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let cursor_guides = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Quad {
                        order: DrawOrder(3),
                        ..
                    }
                )
            })
            .count();
        assert!(
            cursor_guides >= 2,
            "declarative line plot should paint cursor crosshair guides"
        );

        let readout_backgrounds = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Quad {
                        order: DrawOrder(12),
                        ..
                    }
                )
            })
            .count();
        assert!(
            readout_backgrounds >= 1,
            "declarative line plot should paint mouse readout overlay chrome"
        );

        let readout_text = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Text {
                        order: DrawOrder(13),
                        ..
                    }
                )
            })
            .count();
        assert!(
            readout_text >= 1,
            "declarative line plot should paint mouse readout text"
        );

        assert!(
            scene.ops().iter().any(|op| matches!(
                op,
                fret_core::SceneOp::Path {
                    order: DrawOrder(20),
                    ..
                }
            )),
            "cursor readout painting must preserve declarative line painting"
        );
    }

    #[test]
    fn line_plot_panel_paints_series_readout_rows_on_declarative_cursor_overlay() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Alpha",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-series-readout",
            |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(169.0), Px(81.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let mut prepared_text = services.prepared_text.join("\n");
        prepared_text.make_ascii_lowercase();
        assert!(
            prepared_text.contains("alpha: y="),
            "declarative cursor readout should include per-series readout rows, got {prepared_text:?}"
        );
    }

    #[test]
    fn line_plot_panel_paints_right_axis_series_readout_with_right_axis_formatter_on_declarative_path()
     {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app.models_mut().insert(LinePlotModel::from_series(vec![
            LineSeries::new(
                "RightAxis",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )
            .y_axis(YAxis::Right),
        ]));

        let right_axis_labels =
            AxisLabelFormatter::custom(0x5279_6768_7441, |v, _span| format!("R{v:.1}"));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-right-axis-series-readout",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone()).y2_axis_labels(right_axis_labels),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(169.0), Px(81.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let mut prepared_text = services.prepared_text.join("\n");
        prepared_text.make_ascii_lowercase();
        assert!(
            prepared_text.contains("rightaxis: y2=r1.0"),
            "right-axis cursor readout should use the right-axis formatter, got {prepared_text:?}"
        );
    }

    #[test]
    fn line_plot_panel_paints_linked_cursor_readout_from_state_on_declarative_path() {
        let mut app = TestHost::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices::default();
        let model = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "Series",
                Series::from_points_sorted(
                    vec![
                        DataPoint { x: 0.0, y: 0.0 },
                        DataPoint { x: 1.0, y: 1.0 },
                        DataPoint { x: 2.0, y: 0.0 },
                    ],
                    true,
                ),
            )]));
        let mut plot_state = PlotState::default();
        plot_state.linked_cursor_x = Some(1.0);
        let state = app.models_mut().insert(plot_state);

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot-declarative-linked-cursor-readout",
            |cx| {
                vec![line_plot_panel(
                    cx,
                    LinePlotPanelProps::new(model.clone()).state(state.clone()),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let linked_cursor_guides = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Quad {
                        order: DrawOrder(3),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(
            linked_cursor_guides, 1,
            "linked cursor should paint one vertical guide when no local cursor is active"
        );

        assert!(
            scene.ops().iter().any(|op| matches!(
                op,
                fret_core::SceneOp::Quad {
                    order: DrawOrder(12),
                    ..
                }
            )),
            "linked cursor should paint readout overlay chrome"
        );
        assert!(
            scene.ops().iter().any(|op| matches!(
                op,
                fret_core::SceneOp::Text {
                    order: DrawOrder(13),
                    ..
                }
            )),
            "linked cursor should paint readout text"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(169.0), Px(81.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        scene.clear();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        let local_cursor_guides = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    fret_core::SceneOp::Quad {
                        order: DrawOrder(3),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(
            local_cursor_guides, 2,
            "local cursor crosshair should take precedence over linked cursor"
        );
    }
}
