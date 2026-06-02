//! Declarative candlestick path-command projection owner.

use fret_core::{PathCommand, Point, Px};

use crate::cartesian::PlotTransform;
use crate::models::OhlcPoint;

use super::super::model::PlotPanelCandlestick;

pub(in crate::declarative) fn line_plot_candlestick_down_path_key(series_id: u64) -> u64 {
    0x706c_6f74_6364_6f77_u64 ^ series_id
}

pub(in crate::declarative) fn candlestick_commands_from_series(
    transform: PlotTransform,
    candlestick: &PlotPanelCandlestick,
    stroke_width: Px,
    scale_factor: f32,
) -> (Vec<PathCommand>, Vec<PathCommand>, Vec<PathCommand>) {
    let points = &candlestick.points;
    if points.is_empty() {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    let candle_width = candlestick.candle_width.abs();
    if !candle_width.is_finite() {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    let view_x_min = transform.data.x_min.min(transform.data.x_max);
    let view_x_max = transform.data.x_min.max(transform.data.x_max);
    let half_w = candle_width * 0.5;
    let max_count = line_plot_device_point_budget(transform, scale_factor).max(8);

    let mut wick = Vec::new();
    let mut body_up = Vec::new();
    let mut body_down = Vec::new();

    let push_rect = |out: &mut Vec<PathCommand>, x0: Px, x1: Px, y0: Px, y1: Px| {
        let left = x0.0.min(x1.0);
        let right = x0.0.max(x1.0);
        let top = y0.0.min(y1.0);
        let bottom = y0.0.max(y1.0);
        if !left.is_finite() || !right.is_finite() || !top.is_finite() || !bottom.is_finite() {
            return;
        }

        let width = (right - left).max(stroke_width.0.max(1.0));
        let height = (bottom - top).max(stroke_width.0.max(1.0));
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            return;
        }

        let p0 = Point::new(Px(left), Px(top));
        let p1 = Point::new(Px(left + width), Px(top));
        let p2 = Point::new(Px(left + width), Px(top + height));
        let p3 = Point::new(Px(left), Px(top + height));

        out.push(PathCommand::MoveTo(p0));
        out.push(PathCommand::LineTo(p1));
        out.push(PathCommand::LineTo(p2));
        out.push(PathCommand::LineTo(p3));
        out.push(PathCommand::Close);
    };

    let mut push_point = |point: OhlcPoint| {
        if !point.is_finite() || point.x < view_x_min || point.x > view_x_max {
            return;
        }

        let Some(x_px) = transform.data_x_to_px(point.x) else {
            return;
        };
        let Some(high_px) = transform.data_y_to_px(point.high) else {
            return;
        };
        let Some(low_px) = transform.data_y_to_px(point.low) else {
            return;
        };
        wick.push(PathCommand::MoveTo(Point::new(x_px, high_px)));
        wick.push(PathCommand::LineTo(Point::new(x_px, low_px)));

        let Some(x0_px) = transform.data_x_to_px(point.x - half_w) else {
            return;
        };
        let Some(x1_px) = transform.data_x_to_px(point.x + half_w) else {
            return;
        };
        let Some(open_px) = transform.data_y_to_px(point.open) else {
            return;
        };
        let Some(close_px) = transform.data_y_to_px(point.close) else {
            return;
        };

        if point.close >= point.open {
            push_rect(&mut body_up, x0_px, x1_px, open_px, close_px);
        } else {
            push_rect(&mut body_down, x0_px, x1_px, open_px, close_px);
        }
    };

    if points.len() <= max_count {
        for point in points.iter().copied() {
            push_point(point);
        }
    } else {
        let stride = points.len().div_ceil(max_count).max(1);
        for point in points.iter().copied().step_by(stride) {
            push_point(point);
        }
    }

    (wick, body_up, body_down)
}

fn line_plot_device_point_budget(transform: PlotTransform, scale_factor: f32) -> usize {
    let width = transform.viewport.size.width.0.max(0.0);
    let device_width = (width * scale_factor.max(1.0)).max(1.0);
    device_width as usize * 2
}
