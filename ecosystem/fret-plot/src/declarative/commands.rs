//! Declarative line-plot path-command projection owner.

use fret_core::{PathCommand, Point, Px};

use crate::cartesian::{DataPoint, PlotTransform};
use crate::models::StepMode;

mod bar_histogram;
mod candlestick;
mod error_bars;
mod shaded;

pub(super) use bar_histogram::{bars_commands_from_series, histogram_commands_from_series};
pub(super) use candlestick::{
    candlestick_commands_from_series, line_plot_candlestick_down_path_key,
};
pub(super) use error_bars::error_bars_commands_from_series;
pub(super) use shaded::{line_plot_shaded_lower_path_key, shaded_band_commands_from_series};

pub(super) fn line_plot_series_path_key(series_id: u64) -> u64 {
    0x706c_6f74_6c69_6e65_u64 ^ series_id
}

pub(super) fn line_plot_area_fill_path_key(series_id: u64) -> u64 {
    0x706c_6f74_6172_6561_u64 ^ series_id
}

pub(super) fn area_fill_commands_from_polyline(
    polyline: &[PathCommand],
    baseline_y: Px,
) -> Vec<PathCommand> {
    if polyline.is_empty() || !baseline_y.0.is_finite() {
        return Vec::new();
    }

    let mut out: Vec<PathCommand> = Vec::new();
    let mut segment: Vec<Point> = Vec::new();

    let mut flush_segment = |segment: &mut Vec<Point>| {
        if segment.len() < 2 {
            segment.clear();
            return;
        }

        let first = segment[0];
        let last = *segment.last().expect("len>=2");
        out.push(PathCommand::MoveTo(Point::new(first.x, baseline_y)));
        out.push(PathCommand::LineTo(first));
        for point in segment.iter().copied().skip(1) {
            out.push(PathCommand::LineTo(point));
        }
        out.push(PathCommand::LineTo(Point::new(last.x, baseline_y)));
        out.push(PathCommand::Close);
        segment.clear();
    };

    for command in polyline {
        match *command {
            PathCommand::MoveTo(point) => {
                flush_segment(&mut segment);
                segment.push(point);
            }
            PathCommand::LineTo(point) => {
                segment.push(point);
            }
            _ => {}
        }
    }

    flush_segment(&mut segment);
    out
}

pub(super) fn stems_commands_from_points(
    transform: PlotTransform,
    points: &[DataPoint],
    baseline: f32,
) -> Vec<PathCommand> {
    let Some(baseline_y) = transform.data_y_to_px(f64::from(baseline)) else {
        return Vec::new();
    };
    if !baseline_y.0.is_finite() {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(points.len().saturating_mul(2));
    for point in points {
        if !point.x.is_finite() || !point.y.is_finite() {
            continue;
        }
        let px = transform.data_to_px(*point);
        if !px.x.0.is_finite() || !px.y.0.is_finite() {
            continue;
        }
        out.push(PathCommand::MoveTo(Point::new(px.x, baseline_y)));
        out.push(PathCommand::LineTo(px));
    }
    out
}

pub(super) fn step_commands_from_polyline(
    polyline: &[PathCommand],
    step_mode: StepMode,
) -> Vec<PathCommand> {
    if polyline.is_empty() {
        return Vec::new();
    }

    let mut out: Vec<PathCommand> = Vec::with_capacity(polyline.len().saturating_mul(2));
    let mut last: Option<Point> = None;

    for cmd in polyline {
        match *cmd {
            PathCommand::MoveTo(p) => {
                out.push(PathCommand::MoveTo(p));
                last = Some(p);
            }
            PathCommand::LineTo(p) => {
                let Some(prev) = last else {
                    out.push(PathCommand::MoveTo(p));
                    last = Some(p);
                    continue;
                };

                let mid = match step_mode {
                    StepMode::Pre => Point::new(prev.x, p.y),
                    StepMode::Post => Point::new(p.x, prev.y),
                };

                if mid != prev {
                    out.push(PathCommand::LineTo(mid));
                }
                if p != mid {
                    out.push(PathCommand::LineTo(p));
                }
                last = Some(p);
            }
            _ => {}
        }
    }

    out
}
