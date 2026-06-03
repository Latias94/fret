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
