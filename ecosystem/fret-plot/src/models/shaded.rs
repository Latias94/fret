use std::sync::Arc;

use crate::cartesian::DataRect;
use crate::plot::view::sanitize_data_rect;
use crate::series::{Series, SeriesId};
use fret_core::geometry::Px;
use fret_core::scene::Color;

use super::YAxis;

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
