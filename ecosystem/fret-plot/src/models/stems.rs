use std::sync::Arc;

use crate::cartesian::DataRect;
use crate::plot::view::sanitize_data_rect;
use crate::series::{Series, SeriesId};
use fret_core::geometry::Px;
use fret_core::scene::Color;

use super::{
    YAxis, compute_data_bounds_from_series_data, compute_data_bounds_from_series_data_by_axis,
};

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
