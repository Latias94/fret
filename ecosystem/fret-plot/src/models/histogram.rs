use std::sync::Arc;

use crate::cartesian::DataRect;
use crate::plot::histogram::histogram_bins;
use crate::plot::view::sanitize_data_rect;
use crate::series::SeriesId;
use fret_core::scene::Color;

use super::YAxis;

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
