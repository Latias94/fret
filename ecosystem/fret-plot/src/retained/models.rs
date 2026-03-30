//! Plot series and data models.
//!
//! This module is kept data-focused: it defines plot model types (`*PlotModel`) and series item
//! types (`*Series`) that are consumed by retained plot canvases.

use crate::cartesian::{DataPoint, DataRect};
use crate::plot::histogram::histogram_bins;
use crate::plot::view::sanitize_data_rect;
use crate::series::{Series, SeriesData, SeriesId};
use fret_core::geometry::Px;
use fret_core::scene::Color;
use std::sync::Arc;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Default)]
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


#[derive(Debug, Clone)]
pub struct LineSeries {
    pub id: SeriesId,
    pub label: Arc<str>,
    pub data: Series,
    pub y_axis: YAxis,
    pub stroke_color: Option<Color>,
    pub stroke_width: Option<Px>,
}

impl LineSeries {
    pub fn new(label: impl Into<Arc<str>>, data: Series) -> Self {
        let label = label.into();
        Self {
            id: SeriesId::from_label(&label),
            label,
            data,
            y_axis: YAxis::Left,
            stroke_color: None,
            stroke_width: None,
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
}

#[derive(Debug, Clone)]
pub struct LinePlotModel {
    pub data_bounds: DataRect,
    pub data_bounds_y2: Option<DataRect>,
    pub data_bounds_y3: Option<DataRect>,
    pub data_bounds_y4: Option<DataRect>,
    pub series: Vec<LineSeries>,
}

impl LinePlotModel {
    pub fn from_series(series: Vec<LineSeries>) -> Self {
        let bounds_all = compute_data_bounds_from_series_data(&series, |s| &s.data);
        let bounds_left = compute_data_bounds_from_series_data_by_axis(
            &series,
            YAxis::Left,
            |s| s.y_axis,
            |s| &s.data,
        );
        let bounds_right = compute_data_bounds_from_series_data_by_axis(
            &series,
            YAxis::Right,
            |s| s.y_axis,
            |s| &s.data,
        );
        let bounds_right2 = compute_data_bounds_from_series_data_by_axis(
            &series,
            YAxis::Right2,
            |s| s.y_axis,
            |s| &s.data,
        );
        let bounds_right3 = compute_data_bounds_from_series_data_by_axis(
            &series,
            YAxis::Right3,
            |s| s.y_axis,
            |s| &s.data,
        );

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

    pub fn from_series_with_bounds(series: Vec<LineSeries>, data_bounds: DataRect) -> Self {
        let primary = sanitize_data_rect(data_bounds);
        let bounds_right = compute_data_bounds_from_series_data_by_axis(
            &series,
            YAxis::Right,
            |s| s.y_axis,
            |s| &s.data,
        );
        let bounds_right2 = compute_data_bounds_from_series_data_by_axis(
            &series,
            YAxis::Right2,
            |s| s.y_axis,
            |s| &s.data,
        );
        let bounds_right3 = compute_data_bounds_from_series_data_by_axis(
            &series,
            YAxis::Right3,
            |s| s.y_axis,
            |s| &s.data,
        );
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
pub struct StemsSeries {
    pub id: SeriesId,
    pub label: Arc<str>,
    pub data: Series,
    pub y_axis: YAxis,
    pub stroke_color: Option<Color>,
    pub stroke_width: Option<Px>,
    pub baseline: f32,
}

impl StemsSeries {
    pub fn new(label: impl Into<Arc<str>>, data: Series) -> Self {
        let label = label.into();
        Self {
            id: SeriesId::from_label(&label),
            label,
            data,
            y_axis: YAxis::Left,
            stroke_color: None,
            stroke_width: None,
            baseline: 0.0,
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

    pub fn baseline(mut self, y: f32) -> Self {
        self.baseline = y;
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
pub struct ScatterSeries {
    pub id: SeriesId,
    pub label: Arc<str>,
    pub data: Series,
    pub y_axis: YAxis,
    pub stroke_color: Option<Color>,
    pub stroke_width: Option<Px>,
    pub marker_radius: Option<Px>,
    pub marker_shape: Option<MarkerShape>,
}

impl ScatterSeries {
    pub fn new(label: impl Into<Arc<str>>, data: Series) -> Self {
        let label = label.into();
        Self {
            id: SeriesId::from_label(&label),
            label,
            data,
            y_axis: YAxis::Left,
            stroke_color: None,
            stroke_width: None,
            marker_radius: None,
            marker_shape: None,
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

    pub fn marker_radius(mut self, radius: Px) -> Self {
        self.marker_radius = Some(radius);
        self
    }

    pub fn marker_shape(mut self, shape: MarkerShape) -> Self {
        self.marker_shape = Some(shape);
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
pub struct StemsPlotModel {
    pub data_bounds: DataRect,
    pub data_bounds_y2: Option<DataRect>,
    pub data_bounds_y3: Option<DataRect>,
    pub data_bounds_y4: Option<DataRect>,
    pub series: Vec<StemsSeries>,
}

impl StemsPlotModel {
    pub fn from_series(series: Vec<StemsSeries>) -> Self {
        let bounds_all = compute_data_bounds_from_series_data(&series, |s| &s.data);
        let bounds_left = compute_data_bounds_from_series_data_by_axis(
            &series,
            YAxis::Left,
            |s| s.y_axis,
            |s| &s.data,
        );
        let bounds_right = compute_data_bounds_from_series_data_by_axis(
            &series,
            YAxis::Right,
            |s| s.y_axis,
            |s| &s.data,
        );
        let bounds_right2 = compute_data_bounds_from_series_data_by_axis(
            &series,
            YAxis::Right2,
            |s| s.y_axis,
            |s| &s.data,
        );
        let bounds_right3 = compute_data_bounds_from_series_data_by_axis(
            &series,
            YAxis::Right3,
            |s| s.y_axis,
            |s| &s.data,
        );

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

        let baseline_for_axis = |axis: YAxis| {
            let mut min: Option<f64> = None;
            let mut max: Option<f64> = None;
            for s in &series {
                if s.y_axis != axis {
                    continue;
                }
                let v = f64::from(s.baseline);
                if !v.is_finite() {
                    continue;
                }
                min = Some(min.map_or(v, |a| a.min(v)));
                max = Some(max.map_or(v, |a| a.max(v)));
            }
            min.zip(max)
        };

        let (bmin_y, bmax_y) = baseline_for_axis(YAxis::Left).unwrap_or((0.0, 0.0));
        let primary = sanitize_data_rect(DataRect {
            x_min: x_source.x_min,
            x_max: x_source.x_max,
            y_min: y_source.y_min.min(bmin_y),
            y_max: y_source.y_max.max(bmax_y),
        });

        let y2 = bounds_right.map(|b| {
            let (bmin, bmax) = baseline_for_axis(YAxis::Right).unwrap_or((0.0, 0.0));
            sanitize_data_rect(DataRect {
                x_min: primary.x_min,
                x_max: primary.x_max,
                y_min: b.y_min.min(bmin),
                y_max: b.y_max.max(bmax),
            })
        });
        let y3 = bounds_right2.map(|b| {
            let (bmin, bmax) = baseline_for_axis(YAxis::Right2).unwrap_or((0.0, 0.0));
            sanitize_data_rect(DataRect {
                x_min: primary.x_min,
                x_max: primary.x_max,
                y_min: b.y_min.min(bmin),
                y_max: b.y_max.max(bmax),
            })
        });
        let y4 = bounds_right3.map(|b| {
            let (bmin, bmax) = baseline_for_axis(YAxis::Right3).unwrap_or((0.0, 0.0));
            sanitize_data_rect(DataRect {
                x_min: primary.x_min,
                x_max: primary.x_max,
                y_min: b.y_min.min(bmin),
                y_max: b.y_max.max(bmax),
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
pub struct ScatterPlotModel {
    pub data_bounds: DataRect,
    pub data_bounds_y2: Option<DataRect>,
    pub data_bounds_y3: Option<DataRect>,
    pub data_bounds_y4: Option<DataRect>,
    pub series: Vec<ScatterSeries>,
}

impl ScatterPlotModel {
    pub fn from_series(series: Vec<ScatterSeries>) -> Self {
        let bounds_all = compute_data_bounds_from_series_data(&series, |s| &s.data);
        let bounds_left = compute_data_bounds_from_series_data_by_axis(
            &series,
            YAxis::Left,
            |s| s.y_axis,
            |s| &s.data,
        );
        let bounds_right = compute_data_bounds_from_series_data_by_axis(
            &series,
            YAxis::Right,
            |s| s.y_axis,
            |s| &s.data,
        );
        let bounds_right2 = compute_data_bounds_from_series_data_by_axis(
            &series,
            YAxis::Right2,
            |s| s.y_axis,
            |s| &s.data,
        );
        let bounds_right3 = compute_data_bounds_from_series_data_by_axis(
            &series,
            YAxis::Right3,
            |s| s.y_axis,
            |s| &s.data,
        );

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

    pub fn from_series_with_bounds(series: Vec<ScatterSeries>, data_bounds: DataRect) -> Self {
        let primary = sanitize_data_rect(data_bounds);
        let bounds_right = compute_data_bounds_from_series_data_by_axis(
            &series,
            YAxis::Right,
            |s| s.y_axis,
            |s| &s.data,
        );
        let bounds_right2 = compute_data_bounds_from_series_data_by_axis(
            &series,
            YAxis::Right2,
            |s| s.y_axis,
            |s| &s.data,
        );
        let bounds_right3 = compute_data_bounds_from_series_data_by_axis(
            &series,
            YAxis::Right3,
            |s| s.y_axis,
            |s| &s.data,
        );
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

#[derive(Debug, Clone, Copy)]
pub struct OhlcPoint {
    pub x: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

impl OhlcPoint {
    pub fn is_finite(self) -> bool {
        self.x.is_finite()
            && self.open.is_finite()
            && self.high.is_finite()
            && self.low.is_finite()
            && self.close.is_finite()
    }
}

#[derive(Debug, Clone)]
struct OhlcCloseSeriesData {
    points: Arc<[OhlcPoint]>,
    sorted_by_x: bool,
    bounds: Option<DataRect>,
}

impl OhlcCloseSeriesData {
    fn new(points: Arc<[OhlcPoint]>, sorted_by_x: bool) -> Self {
        let mut bounds: Option<DataRect> = None;
        for p in points.iter().copied() {
            if !p.is_finite() {
                continue;
            }
            let rect = DataRect {
                x_min: p.x,
                x_max: p.x,
                y_min: p.low.min(p.high).min(p.open).min(p.close),
                y_max: p.low.max(p.high).max(p.open).max(p.close),
            };
            bounds = Some(bounds.map_or(rect, |acc| acc.union(rect)));
        }
        Self {
            points,
            sorted_by_x,
            bounds,
        }
    }
}

impl SeriesData for OhlcCloseSeriesData {
    fn len(&self) -> usize {
        self.points.len()
    }

    fn get(&self, index: usize) -> Option<DataPoint> {
        let p = *self.points.get(index)?;
        if !p.is_finite() {
            return None;
        }
        Some(DataPoint { x: p.x, y: p.close })
    }

    fn bounds_hint(&self) -> Option<DataRect> {
        self.bounds
    }

    fn is_sorted_by_x(&self) -> bool {
        self.sorted_by_x
    }

    fn as_slice(&self) -> Option<&[DataPoint]> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct CandlestickSeries {
    pub id: SeriesId,
    pub label: Arc<str>,
    pub points: Arc<[OhlcPoint]>,
    pub(crate) close_series: Series,
    pub y_axis: YAxis,
    /// Candle body width in data-space units (X axis).
    pub candle_width: f32,
    pub up_fill: Option<Color>,
    pub down_fill: Option<Color>,
    pub wick_color: Option<Color>,
    pub stroke_width: Option<Px>,
}

impl CandlestickSeries {
    pub fn new(label: impl Into<Arc<str>>, points: Arc<[OhlcPoint]>) -> Self {
        Self::new_sorted(label, points, false)
    }

    pub fn new_sorted(
        label: impl Into<Arc<str>>,
        points: Arc<[OhlcPoint]>,
        sorted_by_x: bool,
    ) -> Self {
        let label = label.into();
        let close_series = Series::new(OhlcCloseSeriesData::new(points.clone(), sorted_by_x));
        Self {
            id: SeriesId::from_label(&label),
            label,
            points,
            close_series,
            y_axis: YAxis::Left,
            candle_width: 0.8,
            up_fill: None,
            down_fill: None,
            wick_color: None,
            stroke_width: None,
        }
    }

    pub fn id(mut self, id: SeriesId) -> Self {
        self.id = id;
        self
    }

    pub fn y_axis(mut self, axis: YAxis) -> Self {
        self.y_axis = axis;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.candle_width = width;
        self
    }

    pub fn up_fill(mut self, color: Color) -> Self {
        self.up_fill = Some(color);
        self
    }

    pub fn down_fill(mut self, color: Color) -> Self {
        self.down_fill = Some(color);
        self
    }

    pub fn wick_color(mut self, color: Color) -> Self {
        self.wick_color = Some(color);
        self
    }

    pub fn stroke_width(mut self, width: Px) -> Self {
        self.stroke_width = Some(width);
        self
    }
}

#[derive(Debug, Clone)]
pub struct CandlestickPlotModel {
    pub data_bounds: DataRect,
    pub data_bounds_y2: Option<DataRect>,
    pub data_bounds_y3: Option<DataRect>,
    pub data_bounds_y4: Option<DataRect>,
    pub series: Vec<CandlestickSeries>,
}

impl CandlestickPlotModel {
    pub fn from_series(series: Vec<CandlestickSeries>) -> Self {
        let bounds_all = compute_data_bounds_from_candlestick_series(&series, |_| true);
        let bounds_left =
            compute_data_bounds_from_candlestick_series(&series, |s| s.y_axis == YAxis::Left);
        let bounds_right =
            compute_data_bounds_from_candlestick_series(&series, |s| s.y_axis == YAxis::Right);
        let bounds_right2 =
            compute_data_bounds_from_candlestick_series(&series, |s| s.y_axis == YAxis::Right2);
        let bounds_right3 =
            compute_data_bounds_from_candlestick_series(&series, |s| s.y_axis == YAxis::Right3);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum StepMode {
    Pre,
    #[default]
    Post,
}


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

#[derive(Debug, Clone)]
pub struct AreaSeries {
    pub id: SeriesId,
    pub label: Arc<str>,
    pub data: Series,
    pub y_axis: YAxis,
    pub fill_color: Option<Color>,
    pub fill_alpha: f32,
    pub stroke_color: Option<Color>,
    pub stroke_width: Option<Px>,
    pub baseline: f32,
}

impl AreaSeries {
    pub fn new(label: impl Into<Arc<str>>, data: Series) -> Self {
        let label = label.into();
        Self {
            id: SeriesId::from_label(&label),
            label,
            data,
            y_axis: YAxis::Left,
            fill_color: None,
            fill_alpha: 0.22,
            stroke_color: None,
            stroke_width: None,
            baseline: 0.0,
        }
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill_color = Some(color);
        self
    }

    pub fn fill_alpha(mut self, alpha: f32) -> Self {
        self.fill_alpha = alpha;
        self
    }

    pub fn stroke(mut self, color: Color) -> Self {
        self.stroke_color = Some(color);
        self
    }

    pub fn stroke_width(mut self, width: Px) -> Self {
        self.stroke_width = Some(width);
        self
    }

    pub fn baseline(mut self, y: f32) -> Self {
        self.baseline = y;
        self
    }

    pub fn y_axis(mut self, axis: YAxis) -> Self {
        self.y_axis = axis;
        self
    }

    pub fn id(mut self, id: SeriesId) -> Self {
        self.id = id;
        self
    }
}

#[derive(Debug, Clone)]
pub struct AreaPlotModel {
    pub data_bounds: DataRect,
    pub data_bounds_y2: Option<DataRect>,
    pub data_bounds_y3: Option<DataRect>,
    pub data_bounds_y4: Option<DataRect>,
    pub series: Vec<AreaSeries>,
}

impl AreaPlotModel {
    pub fn from_series(series: Vec<AreaSeries>) -> Self {
        let bounds_all = compute_data_bounds_from_area_series(&series);
        let bounds_left = compute_data_bounds_from_area_series_by_axis(&series, YAxis::Left);
        let bounds_right = compute_data_bounds_from_area_series_by_axis(&series, YAxis::Right);
        let bounds_right2 = compute_data_bounds_from_area_series_by_axis(&series, YAxis::Right2);
        let bounds_right3 = compute_data_bounds_from_area_series_by_axis(&series, YAxis::Right3);

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

    pub fn from_series_with_bounds(series: Vec<AreaSeries>, data_bounds: DataRect) -> Self {
        let primary = sanitize_data_rect(data_bounds);
        let bounds_right = compute_data_bounds_from_area_series_by_axis(&series, YAxis::Right);
        let bounds_right2 = compute_data_bounds_from_area_series_by_axis(&series, YAxis::Right2);
        let bounds_right3 = compute_data_bounds_from_area_series_by_axis(&series, YAxis::Right3);
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
pub struct ShadedSeries {
    pub id: SeriesId,
    pub label: Arc<str>,
    pub upper: Series,
    pub lower: Series,
    pub y_axis: YAxis,
    pub fill_color: Option<Color>,
    pub fill_alpha: f32,
    pub stroke_color: Option<Color>,
    pub stroke_width: Option<Px>,
}

impl ShadedSeries {
    pub fn new(label: impl Into<Arc<str>>, upper: Series, lower: Series) -> Self {
        let label = label.into();
        Self {
            id: SeriesId::from_label(&label),
            label,
            upper,
            lower,
            y_axis: YAxis::Left,
            fill_color: None,
            fill_alpha: 0.18,
            stroke_color: None,
            stroke_width: None,
        }
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill_color = Some(color);
        self
    }

    pub fn fill_alpha(mut self, alpha: f32) -> Self {
        self.fill_alpha = alpha;
        self
    }

    pub fn stroke(mut self, color: Color) -> Self {
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
}

#[derive(Debug, Clone)]
pub struct ShadedPlotModel {
    pub data_bounds: DataRect,
    pub data_bounds_y2: Option<DataRect>,
    pub data_bounds_y3: Option<DataRect>,
    pub data_bounds_y4: Option<DataRect>,
    pub series: Vec<ShadedSeries>,
}

impl ShadedPlotModel {
    pub fn from_series(series: Vec<ShadedSeries>) -> Self {
        let bounds_all = compute_data_bounds_from_shaded_series(&series);
        let bounds_left = compute_data_bounds_from_shaded_series_by_axis(&series, YAxis::Left);
        let bounds_right = compute_data_bounds_from_shaded_series_by_axis(&series, YAxis::Right);
        let bounds_right2 = compute_data_bounds_from_shaded_series_by_axis(&series, YAxis::Right2);
        let bounds_right3 = compute_data_bounds_from_shaded_series_by_axis(&series, YAxis::Right3);

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

    pub fn from_series_with_bounds(series: Vec<ShadedSeries>, data_bounds: DataRect) -> Self {
        let primary = sanitize_data_rect(data_bounds);
        let bounds_right = compute_data_bounds_from_shaded_series_by_axis(&series, YAxis::Right);
        let bounds_right2 = compute_data_bounds_from_shaded_series_by_axis(&series, YAxis::Right2);
        let bounds_right3 = compute_data_bounds_from_shaded_series_by_axis(&series, YAxis::Right3);
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

fn compute_data_bounds_from_candlestick_series(
    series: &[CandlestickSeries],
    include: impl Fn(&CandlestickSeries) -> bool,
) -> Option<DataRect> {
    let mut out: Option<DataRect> = None;

    for s in series {
        if !include(s) {
            continue;
        }

        let half_w = f64::from((s.candle_width * 0.5).abs());

        let mut bounds: Option<DataRect> = None;
        for p in s.points.iter().copied() {
            if !p.is_finite() {
                continue;
            }

            let y_min = p.low.min(p.high).min(p.open).min(p.close);
            let y_max = p.low.max(p.high).max(p.open).max(p.close);
            if !y_min.is_finite() || !y_max.is_finite() {
                continue;
            }

            let rect = DataRect {
                x_min: p.x - half_w,
                x_max: p.x + half_w,
                y_min,
                y_max,
            };
            bounds = Some(bounds.map_or(rect, |acc| acc.union(rect)));
        }

        let Some(bounds) = bounds else {
            continue;
        };
        out = Some(out.map_or(bounds, |acc| acc.union(bounds)));
    }

    out
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

fn compute_data_bounds_from_area_series(series: &[AreaSeries]) -> Option<DataRect> {
    let mut out: Option<DataRect> = None;

    for s in series {
        let data = &s.data;
        let baseline = f64::from(s.baseline);
        let bounds = if let Some(hint) = data.bounds_hint() {
            Some(DataRect {
                y_min: hint.y_min.min(baseline),
                y_max: hint.y_max.max(baseline),
                ..hint
            })
        } else if let Some(slice) = data.as_slice() {
            DataRect::from_points(slice.iter().copied()).map(|b| DataRect {
                y_min: b.y_min.min(baseline),
                y_max: b.y_max.max(baseline),
                ..b
            })
        } else {
            DataRect::from_points((0..data.len()).filter_map(|i| data.get(i))).map(|b| DataRect {
                y_min: b.y_min.min(baseline),
                y_max: b.y_max.max(baseline),
                ..b
            })
        };

        let Some(bounds) = bounds else {
            continue;
        };

        out = Some(out.map_or(bounds, |acc| acc.union(bounds)));
    }

    out
}

fn compute_data_bounds_from_area_series_by_axis(
    series: &[AreaSeries],
    axis: YAxis,
) -> Option<DataRect> {
    let mut out: Option<DataRect> = None;

    for s in series {
        if s.y_axis != axis {
            continue;
        }

        let data = &s.data;
        let baseline = f64::from(s.baseline);
        let bounds = if let Some(hint) = data.bounds_hint() {
            Some(DataRect {
                y_min: hint.y_min.min(baseline),
                y_max: hint.y_max.max(baseline),
                ..hint
            })
        } else if let Some(slice) = data.as_slice() {
            DataRect::from_points(slice.iter().copied()).map(|b| DataRect {
                y_min: b.y_min.min(baseline),
                y_max: b.y_max.max(baseline),
                ..b
            })
        } else {
            DataRect::from_points((0..data.len()).filter_map(|i| data.get(i))).map(|b| DataRect {
                y_min: b.y_min.min(baseline),
                y_max: b.y_max.max(baseline),
                ..b
            })
        };

        let Some(bounds) = bounds else {
            continue;
        };

        out = Some(out.map_or(bounds, |acc| acc.union(bounds)));
    }

    out
}

fn compute_data_bounds_from_shaded_series(series: &[ShadedSeries]) -> Option<DataRect> {
    let mut out: Option<DataRect> = None;

    let series_bounds = |data: &Series| -> Option<DataRect> {
        if let Some(hint) = data.bounds_hint() {
            Some(hint)
        } else if let Some(slice) = data.as_slice() {
            DataRect::from_points(slice.iter().copied())
        } else {
            DataRect::from_points((0..data.len()).filter_map(|i| data.get(i)))
        }
    };

    for s in series {
        let bounds = match (series_bounds(&s.upper), series_bounds(&s.lower)) {
            (Some(a), Some(b)) => Some(a.union(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };

        let Some(bounds) = bounds else {
            continue;
        };

        out = Some(out.map_or(bounds, |acc| acc.union(bounds)));
    }

    out
}

fn compute_data_bounds_from_shaded_series_by_axis(
    series: &[ShadedSeries],
    axis: YAxis,
) -> Option<DataRect> {
    let mut out: Option<DataRect> = None;

    let series_bounds = |data: &Series| -> Option<DataRect> {
        if let Some(hint) = data.bounds_hint() {
            Some(hint)
        } else if let Some(slice) = data.as_slice() {
            DataRect::from_points(slice.iter().copied())
        } else {
            DataRect::from_points((0..data.len()).filter_map(|i| data.get(i)))
        }
    };

    for s in series {
        if s.y_axis != axis {
            continue;
        }

        let bounds = match (series_bounds(&s.upper), series_bounds(&s.lower)) {
            (Some(a), Some(b)) => Some(a.union(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };

        let Some(bounds) = bounds else {
            continue;
        };

        out = Some(out.map_or(bounds, |acc| acc.union(bounds)));
    }

    out
}

#[cfg(test)]
mod candlestick_bounds_tests {
    use super::*;

    #[test]
    fn candlestick_bounds_include_wicks_and_width() {
        let points: Arc<[OhlcPoint]> = Arc::from(vec![OhlcPoint {
            x: 10.0,
            open: 2.0,
            high: 5.0,
            low: -1.0,
            close: 3.0,
        }]);
        let series = CandlestickSeries::new_sorted("c", points, true).width(2.0);
        let model = CandlestickPlotModel::from_series(vec![series]);

        assert!((model.data_bounds.x_min - 9.0).abs() < 1.0e-9);
        assert!((model.data_bounds.x_max - 11.0).abs() < 1.0e-9);
        assert!((model.data_bounds.y_min - -1.0).abs() < 1.0e-9);
        assert!((model.data_bounds.y_max - 5.0).abs() < 1.0e-9);
    }
}
