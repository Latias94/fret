//! Declarative bar and histogram path-command projection owner.

use fret_core::{PathCommand, Point, Px};

use crate::cartesian::{DataPoint, PlotTransform};

use super::super::model::{PlotPanelBars, PlotPanelHistogram};

pub(in crate::declarative) fn histogram_commands_from_series(
    transform: PlotTransform,
    data: &dyn crate::series::SeriesData,
    histogram: &PlotPanelHistogram,
) -> Vec<PathCommand> {
    let bin_width = histogram.bin_width;
    if !bin_width.is_finite() || bin_width <= 0.0 {
        return Vec::new();
    }

    let gap = histogram.bar_gap_fraction.clamp(0.0, 0.95);
    let bar_width = bin_width * f64::from(1.0 - gap);
    if !bar_width.is_finite() || bar_width <= 0.0 {
        return Vec::new();
    }

    let mut out = Vec::new();
    for index in 0..data.len() {
        let Some(point) = data.get(index) else {
            continue;
        };
        if !point.x.is_finite() || !point.y.is_finite() || point.y <= 0.0 {
            continue;
        }

        let x0 = point.x - bar_width * 0.5;
        let x1 = point.x + bar_width * 0.5;
        let p00 = transform.data_to_px(DataPoint { x: x0, y: 0.0 });
        let p10 = transform.data_to_px(DataPoint { x: x1, y: 0.0 });
        let p01 = transform.data_to_px(DataPoint { x: x0, y: point.y });
        let p11 = transform.data_to_px(DataPoint { x: x1, y: point.y });

        if !p00.x.0.is_finite()
            || !p00.y.0.is_finite()
            || !p10.x.0.is_finite()
            || !p10.y.0.is_finite()
            || !p01.x.0.is_finite()
            || !p01.y.0.is_finite()
            || !p11.x.0.is_finite()
            || !p11.y.0.is_finite()
        {
            continue;
        }

        let left = p00.x.0.min(p10.x.0).min(p01.x.0).min(p11.x.0);
        let right = p00.x.0.max(p10.x.0).max(p01.x.0).max(p11.x.0);
        let top = p00.y.0.min(p10.y.0).min(p01.y.0).min(p11.y.0);
        let bottom = p00.y.0.max(p10.y.0).max(p01.y.0).max(p11.y.0);

        if !left.is_finite()
            || !right.is_finite()
            || !top.is_finite()
            || !bottom.is_finite()
            || right <= left
            || bottom <= top
        {
            continue;
        }

        let a = Point::new(Px(left), Px(top));
        let b = Point::new(Px(right), Px(top));
        let c = Point::new(Px(right), Px(bottom));
        let d = Point::new(Px(left), Px(bottom));

        out.push(PathCommand::MoveTo(a));
        out.push(PathCommand::LineTo(b));
        out.push(PathCommand::LineTo(c));
        out.push(PathCommand::LineTo(d));
        out.push(PathCommand::Close);
    }

    out
}

pub(in crate::declarative) fn bars_commands_from_series(
    transform: PlotTransform,
    data: &dyn crate::series::SeriesData,
    bars: &PlotPanelBars,
) -> Vec<PathCommand> {
    let bar_width = bars.bar_width.abs();
    if !bar_width.is_finite() || bar_width <= 0.0 {
        return Vec::new();
    }

    let mut out = Vec::new();
    for index in 0..data.len() {
        let Some(point) = data.get(index) else {
            continue;
        };
        let baseline = bars
            .baselines
            .as_deref()
            .and_then(|baselines| baselines.get(index).copied())
            .unwrap_or(bars.baseline);
        if !point.x.is_finite()
            || !point.y.is_finite()
            || !baseline.is_finite()
            || point.y == baseline
        {
            continue;
        }

        let x0 = point.x - bar_width * 0.5;
        let x1 = point.x + bar_width * 0.5;
        let p00 = transform.data_to_px(DataPoint { x: x0, y: baseline });
        let p10 = transform.data_to_px(DataPoint { x: x1, y: baseline });
        let p01 = transform.data_to_px(DataPoint { x: x0, y: point.y });
        let p11 = transform.data_to_px(DataPoint { x: x1, y: point.y });

        if !p00.x.0.is_finite()
            || !p00.y.0.is_finite()
            || !p10.x.0.is_finite()
            || !p10.y.0.is_finite()
            || !p01.x.0.is_finite()
            || !p01.y.0.is_finite()
            || !p11.x.0.is_finite()
            || !p11.y.0.is_finite()
        {
            continue;
        }

        let left = p00.x.0.min(p10.x.0).min(p01.x.0).min(p11.x.0);
        let right = p00.x.0.max(p10.x.0).max(p01.x.0).max(p11.x.0);
        let top = p00.y.0.min(p10.y.0).min(p01.y.0).min(p11.y.0);
        let bottom = p00.y.0.max(p10.y.0).max(p01.y.0).max(p11.y.0);

        if !left.is_finite()
            || !right.is_finite()
            || !top.is_finite()
            || !bottom.is_finite()
            || right <= left
            || bottom <= top
        {
            continue;
        }

        let a = Point::new(Px(left), Px(top));
        let b = Point::new(Px(right), Px(top));
        let c = Point::new(Px(right), Px(bottom));
        let d = Point::new(Px(left), Px(bottom));

        out.push(PathCommand::MoveTo(a));
        out.push(PathCommand::LineTo(b));
        out.push(PathCommand::LineTo(c));
        out.push(PathCommand::LineTo(d));
        out.push(PathCommand::Close);
    }

    out
}
