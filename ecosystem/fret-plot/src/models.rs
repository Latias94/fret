//! Plot series and data models.
//!
//! This module is kept data-focused: it defines plot model types (`*PlotModel`) and series item
//! types (`*Series`) that are consumed by retained plot canvases.

use crate::cartesian::DataRect;
use crate::plot::histogram::histogram_bins;
use crate::plot::view::sanitize_data_rect;
use crate::series::{Series, SeriesId};
use fret_core::scene::Color;
use std::sync::Arc;

mod area;
mod bars;
mod candlestick;
mod error_bars;
mod line;
mod scatter;
mod shaded;
mod stems;

pub use area::{AreaPlotModel, AreaSeries};
pub use bars::{BarSeries, BarsPlotModel, CategoryBarSeries};
pub use candlestick::{CandlestickPlotModel, CandlestickSeries, OhlcPoint};
pub use error_bars::{ErrorBar, ErrorBarsPlotModel, ErrorBarsSeries};
pub use line::{LinePlotModel, LineSeries};
pub use scatter::{ScatterPlotModel, ScatterSeries};
pub use shaded::{ShadedPlotModel, ShadedSeries};
pub use stems::{StemsPlotModel, StemsSeries};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum YAxis {
    /// Primary (left) Y axis.
    Left,
    /// First right-side Y axis (ImPlot's Y2).
    Right,
    /// Second right-side Y axis (ImPlot's Y3).
    Right2,
    /// Third right-side Y axis (ImPlot's Y4).
    Right3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MarkerShape {
    #[default]
    Plus,
    X,
    Square,
    Diamond,
    TriangleUp,
    TriangleDown,
    Circle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StepMode {
    Pre,
    #[default]
    Post,
}

#[derive(Debug, Clone)]
pub struct HistogramSeries {
    pub id: SeriesId,
    pub label: Arc<str>,
    /// Raw samples in the histogram domain (X).
    pub values: Arc<[f64]>,
    pub y_axis: YAxis,
    pub bin_count: usize,
    pub range: Option<(f64, f64)>,
    /// Fraction of each bin reserved as empty space (0 = touching bars).
    pub bar_gap_fraction: f32,
    pub fill_color: Option<Color>,
}

impl HistogramSeries {
    pub fn new(label: impl Into<Arc<str>>, values: Arc<[f64]>) -> Self {
        let label = label.into();
        Self {
            id: SeriesId::from_label(&label),
            label,
            values,
            y_axis: YAxis::Left,
            bin_count: 50,
            range: None,
            bar_gap_fraction: 0.10,
            fill_color: None,
        }
    }

    pub fn bins(mut self, count: usize) -> Self {
        self.bin_count = count;
        self
    }

    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.range = Some((min, max));
        self
    }

    pub fn bar_gap_fraction(mut self, fraction: f32) -> Self {
        self.bar_gap_fraction = fraction;
        self
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill_color = Some(color);
        self
    }

    pub fn id(mut self, id: SeriesId) -> Self {
        self.id = id;
        self
    }

    pub fn y_axis(mut self, axis: YAxis) -> Self {
        self.y_axis = axis;
        self
    }
}

#[derive(Debug, Clone)]
pub struct HistogramPlotModel {
    pub data_bounds: DataRect,
    pub data_bounds_y2: Option<DataRect>,
    pub data_bounds_y3: Option<DataRect>,
    pub data_bounds_y4: Option<DataRect>,
    pub series: Vec<HistogramSeries>,
}

impl HistogramPlotModel {
    pub fn from_series(series: Vec<HistogramSeries>) -> Self {
        let bounds_all = compute_data_bounds_from_histogram_series(&series);
        let bounds_left = compute_data_bounds_from_histogram_series_by_axis(&series, YAxis::Left);
        let bounds_right = compute_data_bounds_from_histogram_series_by_axis(&series, YAxis::Right);
        let bounds_right2 =
            compute_data_bounds_from_histogram_series_by_axis(&series, YAxis::Right2);
        let bounds_right3 =
            compute_data_bounds_from_histogram_series_by_axis(&series, YAxis::Right3);

        let fallback = DataRect {
            x_min: 0.0,
            x_max: 1.0,
            y_min: 0.0,
            y_max: 1.0,
        };

        let x_source = bounds_all
            .or(bounds_left)
            .or(bounds_right)
            .or(bounds_right2)
            .or(bounds_right3)
            .unwrap_or(fallback);
        let y_source = bounds_left
            .or(bounds_right)
            .or(bounds_right2)
            .or(bounds_right3)
            .unwrap_or(x_source);

        let primary = sanitize_data_rect(DataRect {
            x_min: x_source.x_min,
            x_max: x_source.x_max,
            y_min: y_source.y_min,
            y_max: y_source.y_max,
        });

        let y2 = bounds_right.map(|b| {
            sanitize_data_rect(DataRect {
                x_min: primary.x_min,
                x_max: primary.x_max,
                y_min: b.y_min,
                y_max: b.y_max,
            })
        });
        let y3 = bounds_right2.map(|b| {
            sanitize_data_rect(DataRect {
                x_min: primary.x_min,
                x_max: primary.x_max,
                y_min: b.y_min,
                y_max: b.y_max,
            })
        });
        let y4 = bounds_right3.map(|b| {
            sanitize_data_rect(DataRect {
                x_min: primary.x_min,
                x_max: primary.x_max,
                y_min: b.y_min,
                y_max: b.y_max,
            })
        });

        Self {
            data_bounds: primary,
            data_bounds_y2: y2,
            data_bounds_y3: y3,
            data_bounds_y4: y4,
            series,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HeatmapPlotModel {
    /// Grid domain in data space.
    pub data_bounds: DataRect,
    pub cols: usize,
    pub rows: usize,
    /// Row-major values, length == cols * rows.
    pub values: Arc<[f32]>,
    pub value_min: f32,
    pub value_max: f32,
}

impl HeatmapPlotModel {
    pub fn new(
        data_bounds: DataRect,
        cols: usize,
        rows: usize,
        values: impl Into<Arc<[f32]>>,
    ) -> Self {
        let values: Arc<[f32]> = values.into();
        let expected = cols.saturating_mul(rows);
        debug_assert_eq!(values.len(), expected, "values.len != cols*rows");

        let mut min_v: Option<f32> = None;
        let mut max_v: Option<f32> = None;
        for v in values.iter().copied() {
            if !v.is_finite() {
                continue;
            }
            min_v = Some(min_v.map_or(v, |m| m.min(v)));
            max_v = Some(max_v.map_or(v, |m| m.max(v)));
        }

        let (value_min, value_max) = match min_v.zip(max_v) {
            Some((min_v, max_v)) if min_v.is_finite() && max_v.is_finite() && max_v >= min_v => {
                (min_v, max_v)
            }
            _ => (0.0, 1.0),
        };

        Self {
            data_bounds: sanitize_data_rect(data_bounds),
            cols,
            rows,
            values,
            value_min,
            value_max,
        }
    }

    pub fn value_at(&self, col: usize, row: usize) -> Option<f32> {
        if col >= self.cols || row >= self.rows {
            return None;
        }
        let idx = row.saturating_mul(self.cols).saturating_add(col);
        self.values.get(idx).copied()
    }
}

#[derive(Debug, Clone)]
pub struct Histogram2DPlotModel {
    /// Grid domain in data space.
    pub data_bounds: DataRect,
    pub cols: usize,
    pub rows: usize,
    /// Row-major bin values, length == cols * rows.
    pub values: Arc<[f32]>,
    pub value_min: f32,
    pub value_max: f32,
}

impl Histogram2DPlotModel {
    pub fn new(
        data_bounds: DataRect,
        cols: usize,
        rows: usize,
        values: impl Into<Arc<[f32]>>,
    ) -> Self {
        let values: Arc<[f32]> = values.into();
        let expected = cols.saturating_mul(rows);
        debug_assert_eq!(values.len(), expected, "values.len != cols*rows");

        let mut min_v: Option<f32> = None;
        let mut max_v: Option<f32> = None;
        for v in values.iter().copied() {
            if !v.is_finite() {
                continue;
            }
            min_v = Some(min_v.map_or(v, |m| m.min(v)));
            max_v = Some(max_v.map_or(v, |m| m.max(v)));
        }

        let (value_min, value_max) = match min_v.zip(max_v) {
            Some((min_v, max_v)) if min_v.is_finite() && max_v.is_finite() && max_v >= min_v => {
                (min_v, max_v)
            }
            _ => (0.0, 1.0),
        };

        Self {
            data_bounds: sanitize_data_rect(data_bounds),
            cols,
            rows,
            values,
            value_min,
            value_max,
        }
    }

    pub fn value_at(&self, col: usize, row: usize) -> Option<f32> {
        if col >= self.cols || row >= self.rows {
            return None;
        }
        let idx = row.saturating_mul(self.cols).saturating_add(col);
        self.values.get(idx).copied()
    }
}

fn compute_data_bounds_from_series_data<T>(
    series: &[T],
    data: impl Fn(&T) -> &Series,
) -> Option<DataRect> {
    let mut out: Option<DataRect> = None;

    for s in series {
        let data = data(s);
        let bounds = if let Some(hint) = data.bounds_hint() {
            Some(hint)
        } else if let Some(slice) = data.as_slice() {
            DataRect::from_points(slice.iter().copied())
        } else {
            DataRect::from_points((0..data.len()).filter_map(|i| data.get(i)))
        };

        let Some(bounds) = bounds else {
            continue;
        };

        out = Some(out.map_or(bounds, |acc| acc.union(bounds)));
    }

    out
}

fn compute_data_bounds_from_series_data_by_axis<T>(
    series: &[T],
    axis: YAxis,
    series_axis: impl Fn(&T) -> YAxis,
    data: impl Fn(&T) -> &Series,
) -> Option<DataRect> {
    let mut out: Option<DataRect> = None;

    for s in series {
        if series_axis(s) != axis {
            continue;
        }

        let data = data(s);
        let bounds = if let Some(hint) = data.bounds_hint() {
            Some(hint)
        } else if let Some(slice) = data.as_slice() {
            DataRect::from_points(slice.iter().copied())
        } else {
            DataRect::from_points((0..data.len()).filter_map(|i| data.get(i)))
        };

        let Some(bounds) = bounds else {
            continue;
        };

        out = Some(out.map_or(bounds, |acc| acc.union(bounds)));
    }

    out
}

fn compute_data_bounds_from_histogram_series(series: &[HistogramSeries]) -> Option<DataRect> {
    let mut out: Option<DataRect> = None;

    for s in series {
        let Some(bins) = histogram_bins(&s.values, s.bin_count, s.range) else {
            continue;
        };
        if bins.is_empty() {
            continue;
        }

        let rect = DataRect {
            x_min: bins.x_min,
            x_max: bins.x_max,
            y_min: 0.0,
            y_max: bins.max_count(),
        };
        out = Some(out.map_or(rect, |acc| acc.union(rect)));
    }

    out
}

fn compute_data_bounds_from_histogram_series_by_axis(
    series: &[HistogramSeries],
    axis: YAxis,
) -> Option<DataRect> {
    let mut out: Option<DataRect> = None;

    for s in series {
        if s.y_axis != axis {
            continue;
        }

        let Some(bins) = histogram_bins(&s.values, s.bin_count, s.range) else {
            continue;
        };
        if bins.is_empty() {
            continue;
        }

        let rect = DataRect {
            x_min: bins.x_min,
            x_max: bins.x_max,
            y_min: 0.0,
            y_max: bins.max_count(),
        };
        out = Some(out.map_or(rect, |acc| acc.union(rect)));
    }

    out
}
