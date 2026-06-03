//! Plot series and data models.
//!
//! This module is kept data-focused: it defines plot model types (`*PlotModel`) and series item
//! types (`*Series`) that are consumed by retained plot canvases.

use crate::cartesian::DataRect;
use crate::series::Series;

mod area;
mod bars;
mod candlestick;
mod error_bars;
mod heatmap;
mod histogram;
mod line;
mod scatter;
mod shaded;
mod stems;

pub use area::{AreaPlotModel, AreaSeries};
pub use bars::{BarSeries, BarsPlotModel, CategoryBarSeries};
pub use candlestick::{CandlestickPlotModel, CandlestickSeries, OhlcPoint};
pub use error_bars::{ErrorBar, ErrorBarsPlotModel, ErrorBarsSeries};
pub use heatmap::{HeatmapPlotModel, Histogram2DPlotModel};
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
