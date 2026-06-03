use std::sync::Arc;

use crate::cartesian::{DataPoint, DataRect};
use crate::plot::view::sanitize_data_rect;
use crate::series::{Series, SeriesData, SeriesId};
use fret_core::geometry::Px;
use fret_core::scene::Color;

use super::YAxis;

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
    #[cfg_attr(not(feature = "compat-retained-canvas"), allow(dead_code))]
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

#[cfg(test)]
mod tests {
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
