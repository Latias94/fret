//! Plot series and data models.
//!
//! This module is kept data-focused: it defines plot model types (`*PlotModel`) and series item
//! types (`*Series`) that are consumed by retained plot canvases.

use crate::cartesian::DataRect;
use crate::plot::view::sanitize_data_rect;
use crate::series::Series;
use std::sync::Arc;

mod area;
mod bars;
mod candlestick;
mod error_bars;
mod histogram;
mod line;
mod scatter;
mod shaded;
mod stems;

pub use area::{AreaPlotModel, AreaSeries};
pub use bars::{BarSeries, BarsPlotModel, CategoryBarSeries};
pub use candlestick::{CandlestickPlotModel, CandlestickSeries, OhlcPoint};
pub use error_bars::{ErrorBar, ErrorBarsPlotModel, ErrorBarsSeries};
pub use histogram::{HistogramPlotModel, HistogramSeries};
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
