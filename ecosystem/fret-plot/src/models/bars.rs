use std::sync::Arc;

use crate::cartesian::{DataPoint, DataRect};
use crate::plot::view::sanitize_data_rect;
use crate::series::{Series, SeriesId};
use fret_core::scene::Color;

use super::YAxis;

#[derive(Debug, Clone)]
pub struct BarSeries {
    pub id: SeriesId,
    pub label: Arc<str>,
    pub data: Series,
    pub y_axis: YAxis,
    /// Bar width in data-space units (X axis).
    pub bar_width: f32,
    pub fill_color: Option<Color>,
    pub baseline: f32,
    /// Optional baseline for each point in `data` (indexed by the original point index).
    ///
    /// When set, this enables stacked bars and other per-category baseline strategies.
    pub baseline_by_index: Option<Arc<[f64]>>,
}

impl BarSeries {
    pub fn new(label: impl Into<Arc<str>>, data: Series) -> Self {
        let label = label.into();
        Self {
            id: SeriesId::from_label(&label),
            label,
            data,
            y_axis: YAxis::Left,
            bar_width: 0.8,
            fill_color: None,
            baseline: 0.0,
            baseline_by_index: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.bar_width = width;
        self
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill_color = Some(color);
        self
    }

    pub fn baseline(mut self, y: f32) -> Self {
        self.baseline = y;
        self
    }

    pub fn baseline_by_index(mut self, values: Arc<[f64]>) -> Self {
        self.baseline_by_index = Some(values);
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
pub struct CategoryBarSeries {
    pub id: SeriesId,
    pub label: Arc<str>,
    pub values: Arc<[f64]>,
    pub y_axis: YAxis,
    pub fill_color: Option<Color>,
}

impl CategoryBarSeries {
    pub fn new(label: impl Into<Arc<str>>, values: Arc<[f64]>) -> Self {
        let label = label.into();
        Self {
            id: SeriesId::from_label(&label),
            label,
            values,
            y_axis: YAxis::Left,
            fill_color: None,
        }
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
pub struct BarsPlotModel {
    pub data_bounds: DataRect,
    pub data_bounds_y2: Option<DataRect>,
    pub data_bounds_y3: Option<DataRect>,
    pub data_bounds_y4: Option<DataRect>,
    pub series: Vec<BarSeries>,
}

impl BarsPlotModel {
    pub fn from_series(series: Vec<BarSeries>) -> Self {
        let bounds_all = compute_data_bounds_from_bar_series(&series);
        let bounds_left = compute_data_bounds_from_bar_series_by_axis(&series, YAxis::Left);
        let bounds_right = compute_data_bounds_from_bar_series_by_axis(&series, YAxis::Right);
        let bounds_right2 = compute_data_bounds_from_bar_series_by_axis(&series, YAxis::Right2);
        let bounds_right3 = compute_data_bounds_from_bar_series_by_axis(&series, YAxis::Right3);

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

    pub fn from_series_with_bounds(series: Vec<BarSeries>, data_bounds: DataRect) -> Self {
        let primary = sanitize_data_rect(data_bounds);
        let bounds_right = compute_data_bounds_from_bar_series_by_axis(&series, YAxis::Right);
        let bounds_right2 = compute_data_bounds_from_bar_series_by_axis(&series, YAxis::Right2);
        let bounds_right3 = compute_data_bounds_from_bar_series_by_axis(&series, YAxis::Right3);
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

    pub fn grouped_categories(
        categories_x: Arc<[f64]>,
        series: Vec<CategoryBarSeries>,
        bar_width: f32,
        bar_gap_fraction: f32,
        baseline: f32,
    ) -> Self {
        let group_count = series.len();
        if group_count == 0 || categories_x.is_empty() {
            return Self::from_series(Vec::new());
        }

        let bar_width = bar_width.abs().max(0.0);
        let gap = bar_gap_fraction.clamp(0.0, 0.95);
        let gap_width = f64::from(bar_width) * f64::from(gap);
        let bw = f64::from(bar_width);
        let group_total =
            bw * group_count as f64 + gap_width * (group_count.saturating_sub(1) as f64);
        let group_start = -group_total * 0.5 + bw * 0.5;

        let mut out: Vec<BarSeries> = Vec::with_capacity(group_count);
        for (i, s) in series.into_iter().enumerate() {
            let offset = group_start + (bw + gap_width) * i as f64;
            let n = categories_x.len().min(s.values.len());

            let mut points: Vec<DataPoint> = Vec::with_capacity(n);
            for idx in 0..n {
                let x0 = categories_x[idx];
                let y = s.values[idx];
                points.push(DataPoint { x: x0 + offset, y });
            }

            let data = Series::from_points_sorted(points, true);
            let mut bar = BarSeries::new(s.label.clone(), data)
                .id(s.id)
                .y_axis(s.y_axis)
                .width(bar_width)
                .baseline(baseline);
            if let Some(color) = s.fill_color {
                bar = bar.fill(color);
            }
            out.push(bar);
        }

        Self::from_series(out)
    }

    pub fn stacked_categories(
        categories_x: Arc<[f64]>,
        series: Vec<CategoryBarSeries>,
        bar_width: f32,
    ) -> Self {
        if series.is_empty() || categories_x.is_empty() {
            return Self::from_series(Vec::new());
        }

        let n = categories_x.len();
        let mut pos_acc: Vec<f64> = vec![0.0; n];
        let mut neg_acc: Vec<f64> = vec![0.0; n];

        let bar_width = bar_width.abs().max(0.0);
        let mut out: Vec<BarSeries> = Vec::with_capacity(series.len());
        for s in series {
            let mut baselines: Vec<f64> = vec![f64::NAN; n];
            let mut points: Vec<DataPoint> = Vec::with_capacity(n);

            for idx in 0..n {
                let x = categories_x[idx];
                let v = s.values.get(idx).copied().unwrap_or(0.0);
                if !x.is_finite() || !v.is_finite() {
                    points.push(DataPoint { x, y: f64::NAN });
                    continue;
                }

                let (base, end) = if v >= 0.0 {
                    let base = pos_acc[idx];
                    let end = base + v;
                    pos_acc[idx] = end;
                    (base, end)
                } else {
                    let base = neg_acc[idx];
                    let end = base + v;
                    neg_acc[idx] = end;
                    (base, end)
                };

                baselines[idx] = base;
                points.push(DataPoint { x, y: end });
            }

            let data = Series::from_points_sorted(points, true);
            let mut bar = BarSeries::new(s.label.clone(), data)
                .id(s.id)
                .y_axis(s.y_axis)
                .width(bar_width)
                .baseline(0.0)
                .baseline_by_index(baselines.into());
            if let Some(color) = s.fill_color {
                bar = bar.fill(color);
            }
            out.push(bar);
        }

        Self::from_series(out)
    }
}

fn compute_data_bounds_from_bar_series(series: &[BarSeries]) -> Option<DataRect> {
    let mut out: Option<DataRect> = None;

    for s in series {
        let half_w = f64::from((s.bar_width * 0.5).abs());
        let baseline = f64::from(s.baseline);
        let (baseline_min, baseline_max) = if let Some(values) = s.baseline_by_index.as_deref() {
            let mut min = baseline;
            let mut max = baseline;
            for v in values.iter().copied() {
                if !v.is_finite() {
                    continue;
                }
                min = min.min(v);
                max = max.max(v);
            }
            (min, max)
        } else {
            (baseline, baseline)
        };
        let data = &s.data;

        let bounds = if let Some(hint) = data.bounds_hint() {
            Some(DataRect {
                x_min: hint.x_min - half_w,
                x_max: hint.x_max + half_w,
                y_min: hint.y_min.min(baseline_min),
                y_max: hint.y_max.max(baseline_max),
            })
        } else if let Some(slice) = data.as_slice() {
            let mut x_min: Option<f64> = None;
            let mut x_max: Option<f64> = None;
            let mut y_min: Option<f64> = Some(baseline);
            let mut y_max: Option<f64> = Some(baseline);

            for (idx, p) in slice.iter().copied().enumerate() {
                if !p.x.is_finite() || !p.y.is_finite() {
                    continue;
                }
                let baseline = s
                    .baseline_by_index
                    .as_deref()
                    .and_then(|b| b.get(idx).copied())
                    .unwrap_or(baseline);
                if !baseline.is_finite() {
                    continue;
                }
                x_min = Some(x_min.map_or(p.x - half_w, |v| v.min(p.x - half_w)));
                x_max = Some(x_max.map_or(p.x + half_w, |v| v.max(p.x + half_w)));
                y_min = Some(y_min.map_or(p.y.min(baseline), |v| v.min(p.y.min(baseline))));
                y_max = Some(y_max.map_or(p.y.max(baseline), |v| v.max(p.y.max(baseline))));
            }

            Some(DataRect {
                x_min: x_min?,
                x_max: x_max?,
                y_min: y_min?,
                y_max: y_max?,
            })
        } else {
            let mut x_min: Option<f64> = None;
            let mut x_max: Option<f64> = None;
            let mut y_min: Option<f64> = Some(baseline);
            let mut y_max: Option<f64> = Some(baseline);

            for i in 0..data.len() {
                let Some(p) = data.get(i) else {
                    continue;
                };
                if !p.x.is_finite() || !p.y.is_finite() {
                    continue;
                }
                let baseline = s
                    .baseline_by_index
                    .as_deref()
                    .and_then(|b| b.get(i).copied())
                    .unwrap_or(baseline);
                if !baseline.is_finite() {
                    continue;
                }
                x_min = Some(x_min.map_or(p.x - half_w, |v| v.min(p.x - half_w)));
                x_max = Some(x_max.map_or(p.x + half_w, |v| v.max(p.x + half_w)));
                y_min = Some(y_min.map_or(p.y.min(baseline), |v| v.min(p.y.min(baseline))));
                y_max = Some(y_max.map_or(p.y.max(baseline), |v| v.max(p.y.max(baseline))));
            }

            Some(DataRect {
                x_min: x_min?,
                x_max: x_max?,
                y_min: y_min?,
                y_max: y_max?,
            })
        };

        let Some(bounds) = bounds else {
            continue;
        };

        out = Some(out.map_or(bounds, |acc| acc.union(bounds)));
    }

    out
}

fn compute_data_bounds_from_bar_series_by_axis(
    series: &[BarSeries],
    axis: YAxis,
) -> Option<DataRect> {
    let mut out: Option<DataRect> = None;

    for s in series {
        if s.y_axis != axis {
            continue;
        }

        let half_w = f64::from((s.bar_width * 0.5).abs());
        let baseline = f64::from(s.baseline);
        let (baseline_min, baseline_max) = if let Some(values) = s.baseline_by_index.as_deref() {
            let mut min = baseline;
            let mut max = baseline;
            for v in values.iter().copied() {
                if !v.is_finite() {
                    continue;
                }
                min = min.min(v);
                max = max.max(v);
            }
            (min, max)
        } else {
            (baseline, baseline)
        };
        let data = &s.data;

        let bounds = if let Some(hint) = data.bounds_hint() {
            Some(DataRect {
                x_min: hint.x_min - half_w,
                x_max: hint.x_max + half_w,
                y_min: hint.y_min.min(baseline_min),
                y_max: hint.y_max.max(baseline_max),
            })
        } else if let Some(slice) = data.as_slice() {
            let mut x_min: Option<f64> = None;
            let mut x_max: Option<f64> = None;
            let mut y_min: Option<f64> = Some(baseline);
            let mut y_max: Option<f64> = Some(baseline);

            for (idx, p) in slice.iter().copied().enumerate() {
                if !p.x.is_finite() || !p.y.is_finite() {
                    continue;
                }
                let baseline = s
                    .baseline_by_index
                    .as_deref()
                    .and_then(|b| b.get(idx).copied())
                    .unwrap_or(baseline);
                if !baseline.is_finite() {
                    continue;
                }
                x_min = Some(x_min.map_or(p.x - half_w, |v| v.min(p.x - half_w)));
                x_max = Some(x_max.map_or(p.x + half_w, |v| v.max(p.x + half_w)));
                y_min = Some(y_min.map_or(p.y.min(baseline), |v| v.min(p.y.min(baseline))));
                y_max = Some(y_max.map_or(p.y.max(baseline), |v| v.max(p.y.max(baseline))));
            }

            Some(DataRect {
                x_min: x_min?,
                x_max: x_max?,
                y_min: y_min?,
                y_max: y_max?,
            })
        } else {
            let mut x_min: Option<f64> = None;
            let mut x_max: Option<f64> = None;
            let mut y_min: Option<f64> = Some(baseline);
            let mut y_max: Option<f64> = Some(baseline);

            for i in 0..data.len() {
                let Some(p) = data.get(i) else {
                    continue;
                };
                if !p.x.is_finite() || !p.y.is_finite() {
                    continue;
                }
                let baseline = s
                    .baseline_by_index
                    .as_deref()
                    .and_then(|b| b.get(i).copied())
                    .unwrap_or(baseline);
                if !baseline.is_finite() {
                    continue;
                }
                x_min = Some(x_min.map_or(p.x - half_w, |v| v.min(p.x - half_w)));
                x_max = Some(x_max.map_or(p.x + half_w, |v| v.max(p.x + half_w)));
                y_min = Some(y_min.map_or(p.y.min(baseline), |v| v.min(p.y.min(baseline))));
                y_max = Some(y_max.map_or(p.y.max(baseline), |v| v.max(p.y.max(baseline))));
            }

            Some(DataRect {
                x_min: x_min?,
                x_max: x_max?,
                y_min: y_min?,
                y_max: y_max?,
            })
        };

        let Some(bounds) = bounds else {
            continue;
        };

        out = Some(out.map_or(bounds, |acc| acc.union(bounds)));
    }

    out
}
