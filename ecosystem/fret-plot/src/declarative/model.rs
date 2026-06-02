//! Declarative plot model projection owner.

use fret_core::{Color, Px};

use crate::cartesian::{DataPoint, DataRect};
use crate::models::{
    AreaPlotModel, BarsPlotModel, CandlestickPlotModel, ErrorBarsPlotModel, HeatmapPlotModel,
    Histogram2DPlotModel, HistogramPlotModel, LinePlotModel, ShadedPlotModel, StemsPlotModel,
    YAxis,
};
use crate::plot::histogram::histogram_bins;
use crate::series::{Series, SeriesId};
#[derive(Debug, Clone)]
pub(super) struct PlotPanelModel {
    pub(super) data_bounds: DataRect,
    pub(super) data_bounds_y2: Option<DataRect>,
    pub(super) data_bounds_y3: Option<DataRect>,
    pub(super) data_bounds_y4: Option<DataRect>,
    pub(super) heatmap: Option<PlotPanelHeatmap>,
    pub(super) series: Vec<PlotPanelSeries>,
}

#[derive(Debug, Clone)]
pub(super) struct PlotPanelHeatmap {
    pub(super) data_bounds: DataRect,
    pub(super) cols: usize,
    pub(super) rows: usize,
    pub(super) values: std::sync::Arc<[f32]>,
    pub(super) value_min: f32,
    pub(super) value_max: f32,
}

#[derive(Debug, Clone)]
pub(super) struct PlotPanelSeries {
    pub(super) id: SeriesId,
    pub(super) label: std::sync::Arc<str>,
    pub(super) data: Series,
    pub(super) lower_data: Option<Series>,
    pub(super) error_bars: Option<PlotPanelErrorBars>,
    pub(super) histogram: Option<PlotPanelHistogram>,
    pub(super) bars: Option<PlotPanelBars>,
    pub(super) candlestick: Option<PlotPanelCandlestick>,
    pub(super) y_axis: YAxis,
    pub(super) stroke_color: Option<Color>,
    pub(super) stroke_width: Option<Px>,
    pub(super) fill: Option<PlotPanelFill>,
    pub(super) stem_baseline: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PlotPanelFill {
    pub(super) color: Option<Color>,
    pub(super) alpha: f32,
    pub(super) baseline: f32,
}

#[derive(Debug, Clone)]
pub(super) struct PlotPanelErrorBars {
    pub(super) x_errors: Option<std::sync::Arc<[crate::models::ErrorBar]>>,
    pub(super) y_errors: Option<std::sync::Arc<[crate::models::ErrorBar]>>,
    pub(super) cap_size: Px,
    pub(super) show_caps: bool,
    pub(super) marker_radius: Px,
    pub(super) show_markers: bool,
    pub(super) marker_shape: crate::models::MarkerShape,
}

#[derive(Debug, Clone)]
pub(super) struct PlotPanelHistogram {
    pub(super) bin_width: f64,
    pub(super) bar_gap_fraction: f32,
}

#[derive(Debug, Clone)]
pub(super) struct PlotPanelBars {
    pub(super) bar_width: f64,
    pub(super) baseline: f64,
    pub(super) baselines: Option<std::sync::Arc<[f64]>>,
}

#[derive(Debug, Clone)]
pub(super) struct PlotPanelCandlestick {
    pub(super) points: std::sync::Arc<[crate::models::OhlcPoint]>,
    pub(super) candle_width: f64,
    pub(super) up_fill: Option<Color>,
    pub(super) down_fill: Option<Color>,
    pub(super) wick_color: Option<Color>,
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
