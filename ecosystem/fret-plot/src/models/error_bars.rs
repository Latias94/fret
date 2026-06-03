use std::sync::Arc;

use crate::cartesian::{DataPoint, DataRect};
use crate::plot::view::sanitize_data_rect;
use crate::series::{Series, SeriesId};
use fret_core::geometry::Px;
use fret_core::scene::Color;

use super::{MarkerShape, YAxis};

#[derive(Debug, Clone, Copy)]
pub struct ErrorBar {
    pub neg: f64,
    pub pos: f64,
}

impl ErrorBar {
    pub fn symmetric(v: f64) -> Self {
        let v = v.abs();
        Self { neg: v, pos: v }
    }

    pub fn new(neg: f64, pos: f64) -> Self {
        Self {
            neg: neg.abs(),
            pos: pos.abs(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ErrorBarsSeries {
    pub id: SeriesId,
    pub label: Arc<str>,
    /// Center points (X,Y).
    pub data: Series,
    pub y_axis: YAxis,
    pub stroke_color: Option<Color>,
    pub stroke_width: Option<Px>,
    /// Optional per-point error bars in X.
    ///
    /// The slice is indexed by point index and is expected to match `data.len()`.
    pub x_errors: Option<Arc<[ErrorBar]>>,
    /// Optional per-point error bars in Y.
    ///
    /// The slice is indexed by point index and is expected to match `data.len()`.
    pub y_errors: Option<Arc<[ErrorBar]>>,
    /// Error bar cap half-length in plot-local logical pixels.
    pub cap_size: Px,
    pub show_caps: bool,
    /// Cross marker radius in plot-local logical pixels.
    pub marker_radius: Px,
    pub show_markers: bool,
    pub marker_shape: MarkerShape,
}

impl ErrorBarsSeries {
    pub fn new(label: impl Into<Arc<str>>, data: Series) -> Self {
        let label = label.into();
        Self {
            id: SeriesId::from_label(&label),
            label,
            data,
            y_axis: YAxis::Left,
            stroke_color: None,
            stroke_width: None,
            x_errors: None,
            y_errors: None,
            cap_size: Px(6.0),
            show_caps: true,
            marker_radius: Px(4.0),
            show_markers: true,
            marker_shape: MarkerShape::default(),
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.stroke_color = Some(color);
        self
    }

    pub fn stroke_width(mut self, width: Px) -> Self {
        self.stroke_width = Some(width);
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

    pub fn x_errors(mut self, errors: Arc<[ErrorBar]>) -> Self {
        self.x_errors = Some(errors);
        self
    }

    pub fn y_errors(mut self, errors: Arc<[ErrorBar]>) -> Self {
        self.y_errors = Some(errors);
        self
    }

    pub fn cap_size(mut self, cap: Px) -> Self {
        self.cap_size = cap;
        self
    }

    pub fn show_caps(mut self, show: bool) -> Self {
        self.show_caps = show;
        self
    }

    pub fn marker_radius(mut self, radius: Px) -> Self {
        self.marker_radius = radius;
        self
    }

    pub fn show_markers(mut self, show: bool) -> Self {
        self.show_markers = show;
        self
    }

    pub fn marker_shape(mut self, shape: MarkerShape) -> Self {
        self.marker_shape = shape;
        self
    }
}

#[derive(Debug, Clone)]
pub struct ErrorBarsPlotModel {
    pub data_bounds: DataRect,
    pub data_bounds_y2: Option<DataRect>,
    pub data_bounds_y3: Option<DataRect>,
    pub data_bounds_y4: Option<DataRect>,
    pub series: Vec<ErrorBarsSeries>,
}

impl ErrorBarsPlotModel {
    pub fn from_series(series: Vec<ErrorBarsSeries>) -> Self {
        let bounds_all = compute_data_bounds_from_error_bars_series(&series, |_| true);
        let bounds_left =
            compute_data_bounds_from_error_bars_series(&series, |s| s.y_axis == YAxis::Left);
        let bounds_right =
            compute_data_bounds_from_error_bars_series(&series, |s| s.y_axis == YAxis::Right);
        let bounds_right2 =
            compute_data_bounds_from_error_bars_series(&series, |s| s.y_axis == YAxis::Right2);
        let bounds_right3 =
            compute_data_bounds_from_error_bars_series(&series, |s| s.y_axis == YAxis::Right3);

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

fn compute_data_bounds_from_error_bars_series(
    series: &[ErrorBarsSeries],
    include: impl Fn(&ErrorBarsSeries) -> bool,
) -> Option<DataRect> {
    let mut out: Option<DataRect> = None;

    for s in series {
        if !include(s) {
            continue;
        }

        let x_errors = s.x_errors.as_deref();
        let y_errors = s.y_errors.as_deref();
        let data = &s.data;

        let mut bounds: Option<DataRect> = None;

        let mut consider = |idx: usize, p: DataPoint| {
            if !p.x.is_finite() || !p.y.is_finite() {
                return;
            }

            let mut x_min = p.x;
            let mut x_max = p.x;
            let mut y_min = p.y;
            let mut y_max = p.y;

            if let Some(e) = x_errors.and_then(|e| e.get(idx)) {
                let neg = e.neg.abs();
                let pos = e.pos.abs();
                if neg.is_finite() && pos.is_finite() {
                    x_min = x_min.min(p.x - neg);
                    x_max = x_max.max(p.x + pos);
                }
            }
            if let Some(e) = y_errors.and_then(|e| e.get(idx)) {
                let neg = e.neg.abs();
                let pos = e.pos.abs();
                if neg.is_finite() && pos.is_finite() {
                    y_min = y_min.min(p.y - neg);
                    y_max = y_max.max(p.y + pos);
                }
            }

            let rect = DataRect {
                x_min,
                x_max,
                y_min,
                y_max,
            };
            bounds = Some(bounds.map_or(rect, |acc| acc.union(rect)));
        };

        if let Some(slice) = data.as_slice() {
            for (idx, p) in slice.iter().copied().enumerate() {
                consider(idx, p);
            }
        } else {
            for idx in 0..data.len() {
                let Some(p) = data.get(idx) else {
                    continue;
                };
                consider(idx, p);
            }
        }

        let Some(bounds) = bounds else {
            continue;
        };
        out = Some(out.map_or(bounds, |acc| acc.union(bounds)));
    }

    out
}
