//! Retained plot canvas implementation.

use fret_core::geometry::{Point, Px, Rect, Size};
use fret_core::scene::{Color, DrawOrder, SceneOp};
use fret_core::{
    Event, FontId, FontWeight, KeyCode, MouseButton, PathConstraints, PathId, PathStyle,
    PointerEvent, SemanticsRole, TextBlobId, TextConstraints, TextMetrics, TextOverflow, TextStyle,
    TextWrap, UiServices,
};
use fret_runtime::{Model, TextFontStackKey};
use fret_ui::UiHost;
use fret_ui::retained_bridge::{
    Invalidation, LayoutCx, PaintCx, SemanticsCx, UiTreeRetainedExt, Widget,
};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use super::layout::{PlotLayout, PlotRegion};
use super::state::{PlotHoverOutput, PlotOutput, PlotOutputSnapshot, PlotState};

use crate::cartesian::{AxisScale, DataPoint, DataRect, PlotTransform};
use crate::input_map::{ModifierKey, ModifiersMask, PlotInputMap};
use crate::plot::axis::{AxisLabelFormat, AxisLabelFormatter, AxisTicks, axis_ticks_scaled};
use crate::plot::decimate::{
    SamplePoint, decimate_points, decimate_polyline, decimate_samples, decimate_shaded_band,
    device_point_budget, view_x_range, visible_sorted_slice,
};
use crate::plot::view::{
    clamp_view_to_data_scaled, clamp_zoom_factors, data_rect_from_plot_points_scaled,
    data_rect_key_scaled, expand_data_bounds_scaled, local_from_absolute, pan_view_by_px_scaled,
    sanitize_data_rect, sanitize_data_rect_scaled, zoom_view_at_px_scaled,
};
#[cfg(test)]
use crate::series::Series;
use crate::series::{SeriesData, SeriesId};

pub use super::models::*;
pub use super::style::*;

struct SeriesStyle {
    stroke_color: Color,
}

const SERIES_PALETTE: [Color; 10] = [
    Color {
        r: 0.35,
        g: 0.65,
        b: 0.95,
        a: 1.0,
    },
    Color {
        r: 0.95,
        g: 0.45,
        b: 0.55,
        a: 1.0,
    },
    Color {
        r: 0.45,
        g: 0.85,
        b: 0.55,
        a: 1.0,
    },
    Color {
        r: 0.95,
        g: 0.75,
        b: 0.35,
        a: 1.0,
    },
    Color {
        r: 0.75,
        g: 0.55,
        b: 0.95,
        a: 1.0,
    },
    Color {
        r: 0.35,
        g: 0.85,
        b: 0.85,
        a: 1.0,
    },
    Color {
        r: 0.95,
        g: 0.35,
        b: 0.85,
        a: 1.0,
    },
    Color {
        r: 0.65,
        g: 0.65,
        b: 0.65,
        a: 1.0,
    },
    Color {
        r: 0.55,
        g: 0.75,
        b: 0.35,
        a: 1.0,
    },
    Color {
        r: 0.35,
        g: 0.55,
        b: 0.95,
        a: 1.0,
    },
];

fn resolve_series_color(
    series_index: usize,
    plot_style: LinePlotStyle,
    series_count: usize,
    override_color: Option<Color>,
) -> Color {
    if series_count <= 1 {
        return override_color.unwrap_or(plot_style.stroke_color);
    }
    override_color.unwrap_or(SERIES_PALETTE[series_index % SERIES_PALETTE.len()])
}

fn series_style(
    series: &LineSeries,
    series_index: usize,
    plot_style: LinePlotStyle,
    series_count: usize,
) -> SeriesStyle {
    SeriesStyle {
        stroke_color: resolve_series_color(
            series_index,
            plot_style,
            series_count,
            series.stroke_color,
        ),
    }
}

pub(super) fn contains_point(rect: Rect, p: Point) -> bool {
    p.x.0 >= rect.origin.x.0
        && p.y.0 >= rect.origin.y.0
        && p.x.0 <= rect.origin.x.0 + rect.size.width.0
        && p.y.0 <= rect.origin.y.0 + rect.size.height.0
}

fn offset_rect(rect: Rect, origin: Point) -> Rect {
    Rect::new(
        Point::new(
            Px(origin.x.0 + rect.origin.x.0),
            Px(origin.y.0 + rect.origin.y.0),
        ),
        rect.size,
    )
}

fn overlay_rect_in_plot(plot: Rect, size: Size, anchor: OverlayAnchor, margin: Px) -> Option<Rect> {
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return None;
    }
    if size.width.0 <= 0.0 || size.height.0 <= 0.0 {
        return None;
    }

    let w = size.width.0;
    let h = size.height.0;
    let m = margin.0.max(0.0);

    let x = match anchor {
        OverlayAnchor::TopLeft | OverlayAnchor::BottomLeft => plot.origin.x.0 + m,
        OverlayAnchor::TopRight | OverlayAnchor::BottomRight => {
            plot.origin.x.0 + plot.size.width.0 - m - w
        }
    };
    let y = match anchor {
        OverlayAnchor::TopLeft | OverlayAnchor::TopRight => plot.origin.y.0 + m,
        OverlayAnchor::BottomLeft | OverlayAnchor::BottomRight => {
            plot.origin.y.0 + plot.size.height.0 - m - h
        }
    };

    let max_x = plot.origin.x.0 + plot.size.width.0 - w;
    let max_y = plot.origin.y.0 + plot.size.height.0 - h;

    let x = x.clamp(plot.origin.x.0, max_x);
    let y = y.clamp(plot.origin.y.0, max_y);

    Some(Rect::new(Point::new(Px(x), Px(y)), size))
}

fn apply_readout_policy(
    rows: &mut Vec<PlotCursorReadoutRow>,
    pinned: Option<SeriesId>,
    legend_hover: Option<SeriesId>,
    policy: ReadoutSeriesPolicy,
) {
    match policy {
        ReadoutSeriesPolicy::PinnedOrAll => {
            if let Some(pinned) = pinned {
                rows.retain(|r| r.series_id == pinned);
            }
        }
        ReadoutSeriesPolicy::PinnedOnly => {
            if let Some(pinned) = pinned {
                rows.retain(|r| r.series_id == pinned);
            } else {
                rows.clear();
            }
        }
        ReadoutSeriesPolicy::PinnedOrLegendHoverOrAll => {
            if let Some(pinned) = pinned {
                rows.retain(|r| r.series_id == pinned);
            } else if let Some(hovered) = legend_hover {
                rows.retain(|r| r.series_id == hovered);
            }
        }
    }
}

fn dim_color(color: Color, factor: f32) -> Color {
    let factor = factor.clamp(0.0, 1.0);
    Color {
        a: (color.a * factor).clamp(0.0, 1.0),
        ..color
    }
}

fn axis_to_units(scale: AxisScale, v: f64) -> Option<f64> {
    if !v.is_finite() {
        return None;
    }
    match scale {
        AxisScale::Linear => Some(v),
        AxisScale::Log10 => (v > 0.0).then(|| v.log10()).filter(|u| u.is_finite()),
    }
}

fn axis_from_units(scale: AxisScale, u: f64) -> Option<f64> {
    if !u.is_finite() {
        return None;
    }
    match scale {
        AxisScale::Linear => Some(u),
        AxisScale::Log10 => Some(10.0_f64.powf(u)).filter(|v| v.is_finite() && *v > 0.0),
    }
}

fn constrain_axis_range_scaled(
    scale: AxisScale,
    min: f64,
    max: f64,
    constraints: AxisConstraints,
) -> Option<(f64, f64)> {
    if !min.is_finite() || !max.is_finite() || max <= min {
        return None;
    }

    let mut u_min = axis_to_units(scale, min)?;
    let mut u_max = axis_to_units(scale, max)?;
    if u_max <= u_min {
        return None;
    }

    let mut u_allowed_min = constraints.min.and_then(|v| axis_to_units(scale, v));
    let mut u_allowed_max = constraints.max.and_then(|v| axis_to_units(scale, v));

    if let (Some(a), Some(b)) = (u_allowed_min, u_allowed_max) {
        if !(a.is_finite() && b.is_finite()) || b <= a {
            u_allowed_min = None;
            u_allowed_max = None;
        }
    }

    if let Some(max_span) = constraints.max_span.filter(|s| s.is_finite() && *s > 0.0) {
        let span = u_max - u_min;
        if span.is_finite() && span > max_span {
            let center = (u_min + u_max) * 0.5;
            u_min = center - max_span * 0.5;
            u_max = center + max_span * 0.5;
        }
    }

    if let Some(min_span) = constraints.min_span.filter(|s| s.is_finite() && *s > 0.0) {
        let span = u_max - u_min;
        if span.is_finite() && span < min_span {
            let center = (u_min + u_max) * 0.5;
            u_min = center - min_span * 0.5;
            u_max = center + min_span * 0.5;
        }
    }

    let span = u_max - u_min;
    if !span.is_finite() || span <= 0.0 {
        return None;
    }

    match (u_allowed_min, u_allowed_max) {
        (Some(a), Some(b)) => {
            let allowed_span = b - a;
            if !allowed_span.is_finite() || allowed_span <= 0.0 {
                return None;
            }

            if span >= allowed_span {
                u_min = a;
                u_max = b;
            } else {
                if u_min < a {
                    let d = a - u_min;
                    u_min += d;
                    u_max += d;
                }
                if u_max > b {
                    let d = u_max - b;
                    u_min -= d;
                    u_max -= d;
                }

                u_min = u_min.max(a);
                u_max = u_max.min(b);
            }
        }
        (Some(a), None) => {
            if u_min < a {
                let d = a - u_min;
                u_min += d;
                u_max += d;
            }
        }
        (None, Some(b)) => {
            if u_max > b {
                let d = u_max - b;
                u_min -= d;
                u_max -= d;
            }
        }
        (None, None) => {}
    }

    if u_max <= u_min || !u_min.is_finite() || !u_max.is_finite() {
        return None;
    }

    let out_min = axis_from_units(scale, u_min)?;
    let out_max = axis_from_units(scale, u_max)?;
    (out_max > out_min).then_some((out_min, out_max))
}

fn constrain_view_bounds_scaled(
    view: DataRect,
    x_scale: AxisScale,
    y_scale: AxisScale,
    x_constraints: AxisConstraints,
    y_constraints: AxisConstraints,
) -> DataRect {
    let mut out = view;

    if let Some((x0, x1)) =
        constrain_axis_range_scaled(x_scale, out.x_min, out.x_max, x_constraints)
    {
        out.x_min = x0;
        out.x_max = x1;
    }
    if let Some((y0, y1)) =
        constrain_axis_range_scaled(y_scale, out.y_min, out.y_max, y_constraints)
    {
        out.y_min = y0;
        out.y_max = y1;
    }

    sanitize_data_rect_scaled(out, x_scale, y_scale)
}

#[cfg(test)]
mod axis_constraints_tests {
    use super::*;

    #[test]
    fn linear_limits_clamp_span_to_allowed_range() {
        let c = AxisConstraints::limits(2.0, 8.0);
        let out = constrain_axis_range_scaled(AxisScale::Linear, 0.0, 10.0, c).unwrap();
        assert_eq!(out, (2.0, 8.0));
    }

    #[test]
    fn linear_min_shifts_range_without_changing_span() {
        let c = AxisConstraints {
            min: Some(2.0),
            max: None,
            ..Default::default()
        };
        let out = constrain_axis_range_scaled(AxisScale::Linear, 0.0, 10.0, c).unwrap();
        assert_eq!(out, (2.0, 12.0));
    }

    #[test]
    fn log10_min_span_is_expressed_in_decades() {
        let c = AxisConstraints::default().min_span(4.0);
        let (a, b) = constrain_axis_range_scaled(AxisScale::Log10, 1.0, 1000.0, c).unwrap();
        let span_decades = b.log10() - a.log10();
        assert!((span_decades - 4.0).abs() < 1.0e-9);
    }
}

fn interpolate_y_at_x(series: &dyn SeriesData, x: f64) -> Option<f64> {
    if !x.is_finite() || !series.is_sorted_by_x() {
        return None;
    }

    let len = series.len();
    if len == 0 {
        return None;
    }

    let lower = lower_bound_valid_by_x(series, x)?;
    let right = if lower < len {
        find_valid_at_or_after(series, lower)
    } else {
        None
    };
    let left = if lower > 0 {
        find_valid_at_or_before(series, lower - 1)
    } else {
        None
    };

    match (left, right) {
        (Some((_li, a)), Some((_ri, b))) => {
            let x0 = a.x;
            let x1 = b.x;
            let y0 = a.y;
            let y1 = b.y;

            if !x0.is_finite() || !x1.is_finite() || !y0.is_finite() || !y1.is_finite() {
                return None;
            }
            if x0 == x1 {
                return Some(y0);
            }

            let t = (x - x0) / (x1 - x0);
            if !t.is_finite() {
                return None;
            }
            Some(y0 + (y1 - y0) * t)
        }
        (Some((_i, p)), None) | (None, Some((_i, p))) => p.y.is_finite().then_some(p.y),
        (None, None) => None,
    }
}

/// Estimates a series' Y value at the given X coordinate for cursor readouts.
///
/// Strategy:
/// - If the series reports `sorted_by_x`, we do an O(log N) lookup + linear interpolation.
/// - Otherwise, we first try `SeriesData::sample_range` (view-dependent, bounded by `budget`) and
///   interpolate within the sampled polyline.
/// - As a last resort we do a budgeted scan to find the nearest-X point (O(budget)).
fn cursor_readout_y_at_x(
    series: &dyn SeriesData,
    x: f64,
    view_x_range: Option<std::ops::RangeInclusive<f64>>,
    budget: usize,
) -> Option<f64> {
    if !x.is_finite() {
        return None;
    }

    if series.is_sorted_by_x() {
        return interpolate_y_at_x(series, x);
    }

    if let Some(view_x_range) = view_x_range
        && view_x_range.start().is_finite()
        && view_x_range.end().is_finite()
        && let Some(sampled) = series.sample_range(view_x_range, budget.max(2))
    {
        return interpolate_sampled_y_at_x(sampled, x);
    }

    nearest_point_y_by_x_budgeted(series, x, budget)
}

fn interpolate_sampled_y_at_x(mut points: Vec<DataPoint>, x: f64) -> Option<f64> {
    points.retain(|p| p.x.is_finite() && p.y.is_finite());
    if points.is_empty() {
        return None;
    }

    points.sort_by(|a, b| a.x.total_cmp(&b.x));

    let right = points.partition_point(|p| p.x < x);
    if right == 0 {
        return Some(points[0].y);
    }
    if right >= points.len() {
        return Some(points[points.len().saturating_sub(1)].y);
    }

    let a = points[right - 1];
    let b = points[right];
    if a.x == b.x {
        return Some(a.y);
    }

    let t = (x - a.x) / (b.x - a.x);
    if !t.is_finite() {
        return None;
    }
    Some(a.y + (b.y - a.y) * t)
}

fn nearest_point_y_by_x_budgeted(series: &dyn SeriesData, x: f64, budget: usize) -> Option<f64> {
    let len = series.len();
    if len == 0 || !x.is_finite() {
        return None;
    }

    let budget = budget.max(1).min(len);
    let stride = ((len + budget - 1) / budget).max(1);

    let mut best_dx = f64::INFINITY;
    let mut best_y: Option<f64> = None;

    for idx in (0..len).step_by(stride) {
        let Some(p) = series.get(idx) else {
            continue;
        };
        if !p.x.is_finite() || !p.y.is_finite() {
            continue;
        }

        let dx = (p.x - x).abs();
        if dx < best_dx {
            best_dx = dx;
            best_y = Some(p.y);
        }
    }

    best_y
}

#[cfg(test)]
mod cursor_readout_value_tests {
    use super::*;

    #[test]
    fn unsorted_series_returns_nearest_x_point() {
        let series = Series::from_points(vec![
            DataPoint { x: 10.0, y: 10.0 },
            DataPoint { x: 0.0, y: 0.0 },
            DataPoint { x: 5.0, y: 5.0 },
        ]);

        let y = cursor_readout_y_at_x(&*series, 5.1, Some(0.0..=10.0), 64).unwrap();
        assert!((y - 5.0).abs() < 1.0e-9);
    }

    #[test]
    fn unsorted_sample_range_is_sorted_before_interpolation() {
        struct UnsortedSampleRange;

        impl SeriesData for UnsortedSampleRange {
            fn len(&self) -> usize {
                0
            }

            fn get(&self, _index: usize) -> Option<DataPoint> {
                None
            }

            fn sample_range(
                &self,
                _x_range: std::ops::RangeInclusive<f64>,
                _budget: usize,
            ) -> Option<Vec<DataPoint>> {
                Some(vec![
                    DataPoint { x: 2.0, y: 20.0 },
                    DataPoint { x: 0.0, y: 0.0 },
                    DataPoint { x: 1.0, y: 10.0 },
                ])
            }
        }

        let y = cursor_readout_y_at_x(&UnsortedSampleRange, 1.5, Some(0.0..=2.0), 16).unwrap();
        assert!((y - 15.0).abs() < 1.0e-9);
    }
}

fn lower_bound_valid_by_x(series: &dyn SeriesData, x: f64) -> Option<usize> {
    let len = series.len();
    if len == 0 {
        return None;
    }

    let mut lo = 0usize;
    let mut hi = len;

    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let (idx, p) = nearest_valid_in_range(series, mid, lo, hi, 8)?;
        if p.x < x {
            lo = idx.saturating_add(1);
        } else {
            hi = idx;
        }
    }

    Some(lo)
}

fn step_commands_from_polyline(
    polyline: &[fret_core::PathCommand],
    step_mode: StepMode,
) -> Vec<fret_core::PathCommand> {
    if polyline.is_empty() {
        return Vec::new();
    }

    let mut out: Vec<fret_core::PathCommand> = Vec::with_capacity(polyline.len().saturating_mul(2));
    let mut last: Option<Point> = None;

    for cmd in polyline {
        match *cmd {
            fret_core::PathCommand::MoveTo(p) => {
                out.push(fret_core::PathCommand::MoveTo(p));
                last = Some(p);
            }
            fret_core::PathCommand::LineTo(p) => {
                let Some(prev) = last else {
                    out.push(fret_core::PathCommand::MoveTo(p));
                    last = Some(p);
                    continue;
                };

                let mid = match step_mode {
                    StepMode::Pre => Point::new(prev.x, p.y),
                    StepMode::Post => Point::new(p.x, prev.y),
                };

                if mid != prev {
                    out.push(fret_core::PathCommand::LineTo(mid));
                }
                if p != mid {
                    out.push(fret_core::PathCommand::LineTo(p));
                }
                last = Some(p);
            }
            _ => {}
        }
    }

    out
}

fn nearest_valid_in_range(
    series: &dyn SeriesData,
    center: usize,
    lo: usize,
    hi: usize,
    max_steps: usize,
) -> Option<(usize, DataPoint)> {
    if lo >= hi {
        return None;
    }
    let center = center.clamp(lo, hi - 1);

    for step in 0..=max_steps {
        let left = center.saturating_sub(step);
        if left >= lo {
            if let Some(p) = series.get(left)
                && p.x.is_finite()
                && p.y.is_finite()
            {
                return Some((left, p));
            }
        }

        let right = center.saturating_add(step);
        if step > 0 && right < hi {
            if let Some(p) = series.get(right)
                && p.x.is_finite()
                && p.y.is_finite()
            {
                return Some((right, p));
            }
        }
    }

    None
}

fn find_valid_at_or_before(series: &dyn SeriesData, mut idx: usize) -> Option<(usize, DataPoint)> {
    let max_steps = 64usize;
    let mut steps = 0usize;
    loop {
        if let Some(p) = series.get(idx)
            && p.x.is_finite()
            && p.y.is_finite()
        {
            return Some((idx, p));
        }
        if idx == 0 {
            return None;
        }
        idx -= 1;
        steps += 1;
        if steps >= max_steps {
            return None;
        }
    }
}

fn find_valid_at_or_after(series: &dyn SeriesData, mut idx: usize) -> Option<(usize, DataPoint)> {
    let len = series.len();
    if idx >= len {
        return None;
    }
    let max_steps = 64usize;
    let mut steps = 0usize;
    loop {
        if let Some(p) = series.get(idx)
            && p.x.is_finite()
            && p.y.is_finite()
        {
            return Some((idx, p));
        }
        idx = idx.saturating_add(1);
        steps += 1;
        if idx >= len || steps >= max_steps {
            return None;
        }
    }
}

fn query_rect_from_plot_points_raw(
    view_bounds: DataRect,
    viewport: Size,
    a: Point,
    b: Point,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Option<DataRect> {
    let viewport_w = viewport.width.0;
    let viewport_h = viewport.height.0;
    if !viewport_w.is_finite() || !viewport_h.is_finite() || viewport_w <= 0.0 || viewport_h <= 0.0
    {
        return None;
    }

    let x0 = a.x.0.min(b.x.0).clamp(0.0, viewport_w);
    let x1 = a.x.0.max(b.x.0).clamp(0.0, viewport_w);
    let y0 = a.y.0.min(b.y.0).clamp(0.0, viewport_h);
    let y1 = a.y.0.max(b.y.0).clamp(0.0, viewport_h);

    let transform = PlotTransform {
        viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), viewport),
        data: view_bounds,
        x_scale,
        y_scale,
    };

    let da = transform.px_to_data(Point::new(Px(x0), Px(y0)));
    let db = transform.px_to_data(Point::new(Px(x1), Px(y1)));

    if !da.x.is_finite() || !da.y.is_finite() || !db.x.is_finite() || !db.y.is_finite() {
        return None;
    }

    Some(DataRect {
        x_min: da.x.min(db.x),
        x_max: da.x.max(db.x),
        y_min: da.y.min(db.y),
        y_max: da.y.max(db.y),
    })
}

fn apply_axis_locks(
    view_before: DataRect,
    mut next: DataRect,
    lock_x: bool,
    lock_y: bool,
) -> DataRect {
    if lock_x {
        next.x_min = view_before.x_min;
        next.x_max = view_before.x_max;
    }
    if lock_y {
        next.y_min = view_before.y_min;
        next.y_max = view_before.y_max;
    }
    next
}

fn log10_decade_exponent(v: f64) -> Option<i32> {
    if !v.is_finite() || v <= 0.0 {
        return None;
    }
    let e = v.log10();
    if !e.is_finite() {
        return None;
    }

    let rounded = e.round();
    let eps = 1.0e-10_f64;
    ((e - rounded).abs() <= eps).then_some(rounded as i32)
}

fn log10_tick_label_or_empty(v: f64) -> String {
    let Some(exp) = log10_decade_exponent(v) else {
        return String::new();
    };
    format!("10^{exp}")
}

fn scatter_marker_commands(samples: &[SamplePoint], radius: Px) -> Vec<fret_core::PathCommand> {
    if samples.is_empty() {
        return Vec::new();
    }

    let r = radius.0.max(0.0);
    let mut out: Vec<fret_core::PathCommand> = Vec::with_capacity(samples.len().saturating_mul(4));

    for s in samples {
        let x = s.plot_px.x.0;
        let y = s.plot_px.y.0;
        if !x.is_finite() || !y.is_finite() {
            continue;
        }

        // Cross marker: horizontal then vertical.
        out.push(fret_core::PathCommand::MoveTo(Point::new(Px(x - r), Px(y))));
        out.push(fret_core::PathCommand::LineTo(Point::new(Px(x + r), Px(y))));
        out.push(fret_core::PathCommand::MoveTo(Point::new(Px(x), Px(y - r))));
        out.push(fret_core::PathCommand::LineTo(Point::new(Px(x), Px(y + r))));
    }

    out
}

fn error_bars_commands(
    series: &ErrorBarsSeries,
    transform: PlotTransform,
    samples: &[SamplePoint],
) -> Vec<fret_core::PathCommand> {
    if samples.is_empty() {
        return Vec::new();
    }

    let cap = series.cap_size.0.max(0.0);
    let marker = series.marker_radius.0.max(0.0);

    let mut out: Vec<fret_core::PathCommand> = Vec::new();

    let mut push_point = |idx: usize, p: DataPoint| {
        if !p.x.is_finite() || !p.y.is_finite() {
            return;
        }

        let Some(x_px) = transform.data_x_to_px(p.x) else {
            return;
        };

        if let Some(y_err) = series.y_errors.as_ref().and_then(|e| e.get(idx).copied()) {
            let y0 = p.y - y_err.neg.abs();
            let y1 = p.y + y_err.pos.abs();

            if let (Some(y0_px), Some(y1_px)) =
                (transform.data_y_to_px(y0), transform.data_y_to_px(y1))
            {
                out.push(fret_core::PathCommand::MoveTo(Point::new(x_px, y0_px)));
                out.push(fret_core::PathCommand::LineTo(Point::new(x_px, y1_px)));

                if series.show_caps && cap > 0.0 {
                    let x0 = Px(x_px.0 - cap);
                    let x1 = Px(x_px.0 + cap);
                    out.push(fret_core::PathCommand::MoveTo(Point::new(x0, y0_px)));
                    out.push(fret_core::PathCommand::LineTo(Point::new(x1, y0_px)));
                    out.push(fret_core::PathCommand::MoveTo(Point::new(x0, y1_px)));
                    out.push(fret_core::PathCommand::LineTo(Point::new(x1, y1_px)));
                }
            }
        }

        if let Some(x_err) = series.x_errors.as_ref().and_then(|e| e.get(idx).copied()) {
            if let Some(y_px) = transform.data_y_to_px(p.y) {
                let x0 = p.x - x_err.neg.abs();
                let x1 = p.x + x_err.pos.abs();

                if let (Some(x0_px), Some(x1_px)) =
                    (transform.data_x_to_px(x0), transform.data_x_to_px(x1))
                {
                    out.push(fret_core::PathCommand::MoveTo(Point::new(x0_px, y_px)));
                    out.push(fret_core::PathCommand::LineTo(Point::new(x1_px, y_px)));

                    if series.show_caps && cap > 0.0 {
                        let y0 = Px(y_px.0 - cap);
                        let y1 = Px(y_px.0 + cap);
                        out.push(fret_core::PathCommand::MoveTo(Point::new(x0_px, y0)));
                        out.push(fret_core::PathCommand::LineTo(Point::new(x0_px, y1)));
                        out.push(fret_core::PathCommand::MoveTo(Point::new(x1_px, y0)));
                        out.push(fret_core::PathCommand::LineTo(Point::new(x1_px, y1)));
                    }
                }
            }
        }

        if series.show_markers && marker > 0.0 {
            let Some(y_px) = transform.data_y_to_px(p.y) else {
                return;
            };

            out.push(fret_core::PathCommand::MoveTo(Point::new(
                Px(x_px.0 - marker),
                y_px,
            )));
            out.push(fret_core::PathCommand::LineTo(Point::new(
                Px(x_px.0 + marker),
                y_px,
            )));
            out.push(fret_core::PathCommand::MoveTo(Point::new(
                x_px,
                Px(y_px.0 - marker),
            )));
            out.push(fret_core::PathCommand::LineTo(Point::new(
                x_px,
                Px(y_px.0 + marker),
            )));
        }
    };

    for s in samples {
        push_point(s.index, s.data);
    }

    out
}

fn candlestick_paths(
    series: &CandlestickSeries,
    transform: PlotTransform,
    view_bounds: DataRect,
    scale_factor: f32,
    stroke_width: Px,
) -> (
    Vec<fret_core::PathCommand>,
    Vec<fret_core::PathCommand>,
    Vec<fret_core::PathCommand>,
) {
    let points = &series.points;
    if points.is_empty() {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    let view_x_min = view_bounds.x_min.min(view_bounds.x_max);
    let view_x_max = view_bounds.x_min.max(view_bounds.x_max);

    let half_w = (series.candle_width.abs() * 0.5) as f64;
    let max_count = device_point_budget(transform, scale_factor).max(8);

    let mut wick: Vec<fret_core::PathCommand> = Vec::new();
    let mut body_up: Vec<fret_core::PathCommand> = Vec::new();
    let mut body_down: Vec<fret_core::PathCommand> = Vec::new();

    let push_rect = |out: &mut Vec<fret_core::PathCommand>, x0: Px, x1: Px, y0: Px, y1: Px| {
        let left = x0.0.min(x1.0);
        let right = x0.0.max(x1.0);
        let top = y0.0.min(y1.0);
        let bottom = y0.0.max(y1.0);

        let w = (right - left).max(stroke_width.0.max(1.0));
        let h = (bottom - top).max(stroke_width.0.max(1.0));

        let p0 = Point::new(Px(left), Px(top));
        let p1 = Point::new(Px(left + w), Px(top));
        let p2 = Point::new(Px(left + w), Px(top + h));
        let p3 = Point::new(Px(left), Px(top + h));

        out.push(fret_core::PathCommand::MoveTo(p0));
        out.push(fret_core::PathCommand::LineTo(p1));
        out.push(fret_core::PathCommand::LineTo(p2));
        out.push(fret_core::PathCommand::LineTo(p3));
        out.push(fret_core::PathCommand::Close);
    };

    let mut push = |p: OhlcPoint| {
        if !p.is_finite() {
            return;
        }
        if p.x < view_x_min || p.x > view_x_max {
            return;
        }

        let Some(x_px) = transform.data_x_to_px(p.x) else {
            return;
        };
        let Some(y_hi) = transform.data_y_to_px(p.high) else {
            return;
        };
        let Some(y_lo) = transform.data_y_to_px(p.low) else {
            return;
        };

        wick.push(fret_core::PathCommand::MoveTo(Point::new(x_px, y_hi)));
        wick.push(fret_core::PathCommand::LineTo(Point::new(x_px, y_lo)));

        let x0 = p.x - half_w;
        let x1 = p.x + half_w;
        let Some(x0_px) = transform.data_x_to_px(x0) else {
            return;
        };
        let Some(x1_px) = transform.data_x_to_px(x1) else {
            return;
        };
        let Some(y_open) = transform.data_y_to_px(p.open) else {
            return;
        };
        let Some(y_close) = transform.data_y_to_px(p.close) else {
            return;
        };

        if p.close >= p.open {
            push_rect(&mut body_up, x0_px, x1_px, y_open, y_close);
        } else {
            push_rect(&mut body_down, x0_px, x1_px, y_open, y_close);
        }
    };

    if points.len() <= max_count {
        for p in points.iter().copied() {
            push(p);
        }
    } else {
        let stride = ((points.len() + max_count - 1) / max_count).max(1);
        for p in points.iter().copied().step_by(stride) {
            push(p);
        }
    }

    (wick, body_up, body_down)
}

#[derive(Debug)]
struct CachedPath {
    id: Option<PathId>,
    series_id: SeriesId,
    model_revision: u64,
    scale_factor_bits: u32,
    viewport_w_bits: u32,
    viewport_h_bits: u32,
    stroke_width: Px,
    view_key: u64,
    samples: Vec<SamplePoint>,
}

#[derive(Debug)]
struct CachedCandlestickPath {
    wick_id: Option<PathId>,
    up_id: Option<PathId>,
    down_id: Option<PathId>,
    series_id: SeriesId,
    model_revision: u64,
    scale_factor_bits: u32,
    viewport_w_bits: u32,
    viewport_h_bits: u32,
    stroke_width: Px,
    view_key: u64,
    samples: Vec<SamplePoint>,
}

#[derive(Debug)]
struct CachedAreaPath {
    fill_id: Option<PathId>,
    stroke_id: Option<PathId>,
    series_id: SeriesId,
    model_revision: u64,
    scale_factor_bits: u32,
    viewport_w_bits: u32,
    viewport_h_bits: u32,
    stroke_width: Px,
    view_key: u64,
    baseline_bits: u32,
    fill_alpha_bits: u32,
    samples: Vec<SamplePoint>,
}

#[derive(Debug)]
struct CachedShadedPath {
    fill_id: Option<PathId>,
    upper_stroke_id: Option<PathId>,
    lower_stroke_id: Option<PathId>,
    series_id: SeriesId,
    model_revision: u64,
    scale_factor_bits: u32,
    viewport_w_bits: u32,
    viewport_h_bits: u32,
    stroke_width: Px,
    view_key: u64,
    fill_alpha_bits: u32,
    samples: Vec<SamplePoint>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotHover {
    pub series_id: SeriesId,
    pub index: usize,
    pub data: DataPoint,
    pub plot_px: Point,
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
struct PreparedText {
    blob: TextBlobId,
    metrics: TextMetrics,
    key: u64,
}

#[derive(Debug, Clone)]
struct LegendEntry {
    id: SeriesId,
    text: PreparedText,
}

#[derive(Debug)]
pub struct SeriesMeta {
    pub id: SeriesId,
    pub label: Arc<str>,
    pub y_axis: YAxis,
    pub stroke_color: Option<Color>,
}

#[derive(Debug, Clone)]
pub struct PlotCursorReadoutRow {
    pub series_id: SeriesId,
    pub label: Arc<str>,
    pub y_axis: YAxis,
    pub y: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
pub struct PlotCursorReadoutArgs<'a> {
    pub x: f64,
    pub plot_size: Size,
    pub view_bounds: DataRect,
    pub x_scale: AxisScale,
    pub y_scale: AxisScale,
    pub scale_factor: f32,
    pub hidden: &'a HashSet<SeriesId>,
}

pub trait PlotLayer {
    type Model: Clone + 'static;

    fn data_bounds(model: &Self::Model) -> DataRect;
    fn data_bounds_y2(_model: &Self::Model) -> Option<DataRect> {
        None
    }
    fn data_bounds_y3(_model: &Self::Model) -> Option<DataRect> {
        None
    }
    fn data_bounds_y4(_model: &Self::Model) -> Option<DataRect> {
        None
    }
    fn series_meta(model: &Self::Model) -> Vec<SeriesMeta>;
    fn series_label(model: &Self::Model, series_id: SeriesId) -> Option<String>;
    fn series_y_axis(_model: &Self::Model, _series_id: SeriesId) -> YAxis {
        YAxis::Left
    }

    /// Optional per-series readout at the given X coordinate in data space.
    ///
    /// This powers the common "cursor readout" UX (ImPlot-style), where the plot can show each
    /// series' Y value at the cursor's X position even when the cursor is not close enough to
    /// trigger nearest-point hover.
    ///
    /// The default implementation returns no rows.
    fn cursor_readout(
        _model: &Self::Model,
        _args: PlotCursorReadoutArgs<'_>,
    ) -> Vec<PlotCursorReadoutRow> {
        Vec::new()
    }

    fn paint_paths<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        model: &Self::Model,
        args: PlotPaintArgs<'_>,
    ) -> Vec<(SeriesId, PathId, Color)>;

    fn paint_quads<H: UiHost>(
        &mut self,
        _cx: &mut PaintCx<'_, H>,
        _model: &Self::Model,
        _args: PlotPaintArgs<'_>,
    ) -> Vec<PlotQuad> {
        Vec::new()
    }

    fn hit_test(&mut self, model: &Self::Model, args: PlotHitTestArgs<'_>) -> Option<PlotHover>;

    fn cleanup_resources(&mut self, services: &mut dyn UiServices);
}

#[derive(Debug, Clone, Copy)]
pub struct PlotPaintArgs<'a> {
    pub model_revision: u64,
    pub plot: Rect,
    pub view_bounds: DataRect,
    pub view_bounds_y2: Option<DataRect>,
    pub view_bounds_y3: Option<DataRect>,
    pub view_bounds_y4: Option<DataRect>,
    pub x_scale: AxisScale,
    pub y_scale: AxisScale,
    pub y2_scale: AxisScale,
    pub y3_scale: AxisScale,
    pub y4_scale: AxisScale,
    pub style: LinePlotStyle,
    pub hidden: &'a HashSet<SeriesId>,
}

#[derive(Debug, Clone, Copy)]
pub struct PlotQuad {
    /// Plot-local rect (origin at plot top-left, before `layout.plot.origin` is applied).
    pub rect_local: Rect,
    pub background: Color,
    pub order: DrawOrder,
}

#[derive(Debug, Clone, Copy)]
pub struct PlotHitTestArgs<'a> {
    pub model_revision: u64,
    pub plot_size: Size,
    pub view_bounds: DataRect,
    pub view_bounds_y2: Option<DataRect>,
    pub view_bounds_y3: Option<DataRect>,
    pub view_bounds_y4: Option<DataRect>,
    pub x_scale: AxisScale,
    pub y_scale: AxisScale,
    pub y2_scale: AxisScale,
    pub y3_scale: AxisScale,
    pub y4_scale: AxisScale,
    pub scale_factor: f32,
    pub local: Point,
    pub style: LinePlotStyle,
    pub hover_threshold: Px,
    pub hidden: &'a HashSet<SeriesId>,
    pub pinned: Option<SeriesId>,
}

#[derive(Debug, Default)]
pub struct LinePlotLayer {
    cached_paths: Vec<CachedPath>,
}

pub type LinePlotCanvas = PlotCanvas<LinePlotLayer>;

#[derive(Debug, Default)]
pub struct ScatterPlotLayer {
    cached_paths: Vec<CachedPath>,
}

pub type ScatterPlotCanvas = PlotCanvas<ScatterPlotLayer>;

#[derive(Debug, Default)]
pub struct ErrorBarsPlotLayer {
    cached_paths: Vec<CachedPath>,
}

pub type ErrorBarsPlotCanvas = PlotCanvas<ErrorBarsPlotLayer>;

#[derive(Debug, Default)]
pub struct CandlestickPlotLayer {
    cached_paths: Vec<CachedCandlestickPath>,
}

pub type CandlestickPlotCanvas = PlotCanvas<CandlestickPlotLayer>;

#[derive(Debug, Default)]
pub struct StairsPlotLayer {
    cached_paths: Vec<CachedPath>,
    step_mode: StepMode,
}

pub type StairsPlotCanvas = PlotCanvas<StairsPlotLayer>;

impl PlotCanvas<StairsPlotLayer> {
    pub fn new(model: Model<LinePlotModel>) -> Self {
        Self::with_layer(
            model,
            StairsPlotLayer {
                cached_paths: Vec::new(),
                step_mode: StepMode::default(),
            },
        )
    }

    pub fn step_mode(mut self, mode: StepMode) -> Self {
        self.layer.step_mode = mode;
        self
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct AxisLock {
    pan: bool,
    zoom: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AxisConstraints {
    pub min: Option<f64>,
    pub max: Option<f64>,
    /// Minimum axis span in axis units (linear: data units, log: decades).
    pub min_span: Option<f64>,
    /// Maximum axis span in axis units (linear: data units, log: decades).
    pub max_span: Option<f64>,
}

impl AxisConstraints {
    pub fn limits(min: f64, max: f64) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
            ..Default::default()
        }
    }

    pub fn min_span(mut self, span: f64) -> Self {
        self.min_span = Some(span);
        self
    }

    pub fn max_span(mut self, span: f64) -> Self {
        self.max_span = Some(span);
        self
    }
}

#[derive(Debug)]
pub struct PlotCanvas<L: PlotLayer + 'static> {
    model: Model<L::Model>,
    style: LinePlotStyle,
    input_map: PlotInputMap,
    x_axis_ticks: AxisTicks,
    y_axis_ticks: AxisTicks,
    y2_axis_ticks: AxisTicks,
    y3_axis_ticks: AxisTicks,
    y4_axis_ticks: AxisTicks,
    x_scale: AxisScale,
    y_scale: AxisScale,
    y2_scale: AxisScale,
    y3_scale: AxisScale,
    y4_scale: AxisScale,
    x_axis_labels: AxisLabelFormatter,
    y_axis_labels: AxisLabelFormatter,
    y2_axis_labels: AxisLabelFormatter,
    y3_axis_labels: AxisLabelFormatter,
    y4_axis_labels: AxisLabelFormatter,
    tooltip_x_labels: AxisLabelFormatter,
    tooltip_y_labels: AxisLabelFormatter,
    layer: L,
    hover: Option<PlotHover>,
    plot_state: PlotState,
    plot_state_model: Option<Model<PlotState>>,
    plot_output: PlotOutput,
    plot_output_model: Option<Model<PlotOutput>>,
    legend_hover: Option<SeriesId>,
    cursor_px: Option<Point>,
    last_pointer_pos: Option<Point>,
    last_scale_factor: f32,
    x_axis_thickness: Px,
    y_axis_thickness: Px,
    y_axis_right_thickness: Px,
    y_axis_right2_thickness: Px,
    y_axis_right3_thickness: Px,
    show_y2_axis: bool,
    show_y3_axis: bool,
    show_y4_axis: bool,
    lock_x: AxisLock,
    lock_y: AxisLock,
    lock_y2: AxisLock,
    lock_y3: AxisLock,
    lock_y4: AxisLock,
    x_constraints: AxisConstraints,
    y_constraints: AxisConstraints,
    y2_constraints: AxisConstraints,
    y3_constraints: AxisConstraints,
    y4_constraints: AxisConstraints,
    pan_button: Option<MouseButton>,
    pan_target: Option<PlotRegion>,
    pan_start_pos: Option<Point>,
    pan_last_pos: Option<Point>,
    box_zoom_start: Option<Point>,
    box_zoom_current: Option<Point>,
    box_zoom_button: Option<MouseButton>,
    box_zoom_required_mods: Option<ModifiersMask>,
    query_drag_button: Option<MouseButton>,
    query_drag_start: Option<Point>,
    query_drag_current: Option<Point>,
    axis_label_key: Option<u64>,
    axis_ticks_x: Vec<f64>,
    axis_ticks_y: Vec<f64>,
    axis_ticks_y2: Vec<f64>,
    axis_ticks_y3: Vec<f64>,
    axis_ticks_y4: Vec<f64>,
    axis_labels_x: Vec<PreparedText>,
    axis_labels_y: Vec<PreparedText>,
    axis_labels_y2: Vec<PreparedText>,
    axis_labels_y3: Vec<PreparedText>,
    axis_labels_y4: Vec<PreparedText>,
    legend_key: Option<u64>,
    legend_entries: Vec<LegendEntry>,
    tooltip_text: Option<PreparedText>,
    mouse_readout_text: Option<PreparedText>,
    linked_cursor_readout_text: Option<PreparedText>,
}

impl PlotCanvas<LinePlotLayer> {
    pub fn new(model: Model<LinePlotModel>) -> Self {
        Self::with_layer(model, LinePlotLayer::default())
    }
}

impl PlotCanvas<ScatterPlotLayer> {
    pub fn new(model: Model<ScatterPlotModel>) -> Self {
        Self::with_layer(model, ScatterPlotLayer::default())
    }
}

impl PlotCanvas<ErrorBarsPlotLayer> {
    pub fn new(model: Model<ErrorBarsPlotModel>) -> Self {
        Self::with_layer(model, ErrorBarsPlotLayer::default())
    }
}

impl PlotCanvas<CandlestickPlotLayer> {
    pub fn new(model: Model<CandlestickPlotModel>) -> Self {
        Self::with_layer(model, CandlestickPlotLayer::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HeatmapCacheKey {
    model_revision: u64,
    view_key: u64,
    cols: usize,
    rows: usize,
    viewport_w_bits: u32,
    viewport_h_bits: u32,
    value_min_bits: u32,
    value_max_bits: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HeatmapMipKey {
    model_revision: u64,
    cols: usize,
    rows: usize,
    values_ptr: usize,
}

#[derive(Debug, Clone)]
struct HeatmapMipLevel {
    cols: usize,
    rows: usize,
    values: Vec<f32>,
}

#[derive(Debug, Default)]
pub struct HeatmapPlotLayer {
    cache_key: Option<HeatmapCacheKey>,
    cached_quads: Vec<PlotQuad>,
    mip_key: Option<HeatmapMipKey>,
    mips: Vec<HeatmapMipLevel>,
}

fn ceil_div_usize(a: usize, b: usize) -> usize {
    if b == 0 {
        return a;
    }
    a / b + usize::from(a % b != 0)
}

fn select_heatmap_mip_level(
    visible_cols: usize,
    visible_rows: usize,
    viewport_w: f32,
    viewport_h: f32,
    max_quads: usize,
    max_level: usize,
) -> usize {
    let target_cols = (viewport_w.ceil().max(1.0) as usize).max(1);
    let target_rows = (viewport_h.ceil().max(1.0) as usize).max(1);

    let mut level = 0usize;
    loop {
        if level >= max_level {
            return max_level;
        }

        let scale = 1usize << level.min(usize::BITS as usize - 1);
        let vc = ceil_div_usize(visible_cols, scale).max(1);
        let vr = ceil_div_usize(visible_rows, scale).max(1);
        let quad_count = (vc as u64).saturating_mul(vr as u64);

        if vc <= target_cols && vr <= target_rows && quad_count <= max_quads as u64 {
            return level;
        }
        level = level.saturating_add(1);
    }
}

impl HeatmapPlotLayer {
    fn rebuild_mips_if_needed(&mut self, model_revision: u64, model: &HeatmapPlotModel) {
        let mip_key = HeatmapMipKey {
            model_revision,
            cols: model.cols,
            rows: model.rows,
            values_ptr: model.values.as_ptr() as usize,
        };

        if self.mip_key == Some(mip_key) {
            return;
        }

        self.mip_key = Some(mip_key);
        self.mips.clear();

        let mut prev_cols = model.cols;
        let mut prev_rows = model.rows;
        let mut prev: &[f32] = &model.values;

        while prev_cols > 1 || prev_rows > 1 {
            let next_cols = ceil_div_usize(prev_cols, 2).max(1);
            let next_rows = ceil_div_usize(prev_rows, 2).max(1);
            let mut next: Vec<f32> = vec![f32::NAN; next_cols.saturating_mul(next_rows)];

            for r in 0..next_rows {
                for c in 0..next_cols {
                    let mut sum = 0.0f64;
                    let mut count = 0u32;
                    for dr in 0..2 {
                        for dc in 0..2 {
                            let rr = r * 2 + dr;
                            let cc = c * 2 + dc;
                            if rr >= prev_rows || cc >= prev_cols {
                                continue;
                            }
                            let idx = rr * prev_cols + cc;
                            let Some(v) = prev.get(idx).copied() else {
                                continue;
                            };
                            if !v.is_finite() {
                                continue;
                            }
                            sum += v as f64;
                            count += 1;
                        }
                    }

                    if count > 0 {
                        let avg = (sum / f64::from(count)) as f32;
                        let idx = r * next_cols + c;
                        if let Some(slot) = next.get_mut(idx) {
                            *slot = avg;
                        }
                    }
                }
            }

            self.mips.push(HeatmapMipLevel {
                cols: next_cols,
                rows: next_rows,
                values: next,
            });

            let last = self.mips.last().expect("just pushed");
            prev_cols = last.cols;
            prev_rows = last.rows;
            prev = &last.values;
        }
    }

    fn mip_level_values<'a>(
        &'a self,
        level: usize,
        model: &'a HeatmapPlotModel,
    ) -> (usize, usize, &'a [f32]) {
        if level == 0 {
            return (model.cols, model.rows, &model.values);
        }

        let mip = self
            .mips
            .get(level.saturating_sub(1))
            .expect("level > 0 implies mip exists");
        (mip.cols, mip.rows, &mip.values)
    }
}

#[cfg(test)]
mod heatmap_lod_tests {
    use super::*;

    #[test]
    fn mip_generation_halves_with_round_up() {
        let cols = 3usize;
        let rows = 3usize;
        let values: Vec<f32> = (0..cols * rows).map(|v| v as f32).collect();
        let model = HeatmapPlotModel::new(
            DataRect {
                x_min: 0.0,
                x_max: 3.0,
                y_min: 0.0,
                y_max: 3.0,
            },
            cols,
            rows,
            values,
        );

        let mut layer = HeatmapPlotLayer::default();
        layer.rebuild_mips_if_needed(1, &model);

        assert_eq!(layer.mips.len(), 2);
        assert_eq!((layer.mips[0].cols, layer.mips[0].rows), (2, 2));
        assert_eq!((layer.mips[1].cols, layer.mips[1].rows), (1, 1));
    }

    #[test]
    fn select_mip_level_respects_max_quads() {
        let visible_cols = 4096usize;
        let visible_rows = 4096usize;
        let max_quads = 50_000usize;
        let max_level = 16usize;

        let level = select_heatmap_mip_level(
            visible_cols,
            visible_rows,
            800.0,
            600.0,
            max_quads,
            max_level,
        );
        assert!(level <= max_level);

        let scale = 1usize << level.min(usize::BITS as usize - 1);
        let vc = ceil_div_usize(visible_cols, scale).max(1);
        let vr = ceil_div_usize(visible_rows, scale).max(1);
        let quad_count = (vc as u64).saturating_mul(vr as u64);
        assert!(quad_count <= max_quads as u64);
    }

    #[test]
    fn select_mip_level_uses_level0_for_small_grids() {
        let level = select_heatmap_mip_level(64, 64, 1024.0, 768.0, 50_000, 8);
        assert_eq!(level, 0);
    }
}

#[cfg(test)]
mod hover_segment_tests {
    use super::*;

    use crate::series::OwnedSeriesData;

    #[test]
    fn polyline_hover_hits_segments_between_points() {
        let data = OwnedSeriesData::new(vec![
            DataPoint { x: 0.0, y: 5.0 },
            DataPoint { x: 10.0, y: 5.0 },
        ]);

        let series = [(SeriesId(1), YAxis::Left, &data as &dyn SeriesData)];
        let hidden: HashSet<SeriesId> = HashSet::new();

        let args = PlotHitTestArgs {
            model_revision: 0,
            plot_size: Size::new(Px(10.0), Px(10.0)),
            view_bounds: DataRect {
                x_min: 0.0,
                x_max: 10.0,
                y_min: 0.0,
                y_max: 10.0,
            },
            view_bounds_y2: None,
            view_bounds_y3: None,
            view_bounds_y4: None,
            x_scale: AxisScale::Linear,
            y_scale: AxisScale::Linear,
            y2_scale: AxisScale::Linear,
            y3_scale: AxisScale::Linear,
            y4_scale: AxisScale::Linear,
            scale_factor: 1.0,
            local: Point::new(Px(5.0), Px(6.0)),
            style: LinePlotStyle::default(),
            hover_threshold: Px(1.5),
            hidden: &hidden,
            pinned: None,
        };

        let hover = hit_test_polyline_series_data(&[], &series, args).expect("expected hover hit");
        assert!((hover.plot_px.x.0 - 5.0).abs() < 1.0e-6);
        assert!((hover.plot_px.y.0 - 5.0).abs() < 1.0e-6);
        assert!((hover.data.x - 5.0).abs() < 1.0e-6);
        assert!((hover.data.y - 5.0).abs() < 1.0e-6);
    }

    #[test]
    fn polyline_hover_does_not_connect_across_nan_breaks() {
        let data = OwnedSeriesData::new(vec![
            DataPoint { x: 0.0, y: 5.0 },
            DataPoint {
                x: 5.0,
                y: f64::NAN,
            },
            DataPoint { x: 10.0, y: 5.0 },
        ]);

        let series = [(SeriesId(1), YAxis::Left, &data as &dyn SeriesData)];
        let hidden: HashSet<SeriesId> = HashSet::new();

        let args = PlotHitTestArgs {
            model_revision: 0,
            plot_size: Size::new(Px(10.0), Px(10.0)),
            view_bounds: DataRect {
                x_min: 0.0,
                x_max: 10.0,
                y_min: 0.0,
                y_max: 10.0,
            },
            view_bounds_y2: None,
            view_bounds_y3: None,
            view_bounds_y4: None,
            x_scale: AxisScale::Linear,
            y_scale: AxisScale::Linear,
            y2_scale: AxisScale::Linear,
            y3_scale: AxisScale::Linear,
            y4_scale: AxisScale::Linear,
            scale_factor: 1.0,
            local: Point::new(Px(5.0), Px(6.0)),
            style: LinePlotStyle::default(),
            hover_threshold: Px(1.5),
            hidden: &hidden,
            pinned: None,
        };

        assert!(hit_test_polyline_series_data(&[], &series, args).is_none());
    }
}

#[cfg(test)]
mod box_select_modifier_tests {
    use super::*;

    #[test]
    fn box_select_modifiers_expand_to_edges() {
        let plot_size = Size::new(Px(100.0), Px(50.0));
        let start = Point::new(Px(10.0), Px(10.0));
        let end = Point::new(Px(20.0), Px(20.0));
        let expand_x = Some(ModifierKey::Alt);
        let expand_y = Some(ModifierKey::Shift);

        let mods_x = fret_core::Modifiers {
            alt: true,
            ..fret_core::Modifiers::default()
        };
        let (sx, ex) = PlotCanvas::<LinePlotLayer>::apply_box_select_modifiers(
            plot_size,
            start,
            end,
            mods_x,
            expand_x,
            expand_y,
            ModifiersMask::NONE,
        );
        assert_eq!(sx.x.0, 0.0);
        assert_eq!(ex.x.0, 100.0);
        assert_eq!(sx.y.0, 10.0);
        assert_eq!(ex.y.0, 20.0);

        let mods_y = fret_core::Modifiers {
            shift: true,
            ..fret_core::Modifiers::default()
        };
        let (sy, ey) = PlotCanvas::<LinePlotLayer>::apply_box_select_modifiers(
            plot_size,
            start,
            end,
            mods_y,
            expand_x,
            expand_y,
            ModifiersMask::NONE,
        );
        assert_eq!(sy.y.0, 0.0);
        assert_eq!(ey.y.0, 50.0);
        assert_eq!(sy.x.0, 10.0);
        assert_eq!(ey.x.0, 20.0);

        let mods_xy = fret_core::Modifiers {
            alt: true,
            shift: true,
            ..fret_core::Modifiers::default()
        };
        let (sxy, exy) = PlotCanvas::<LinePlotLayer>::apply_box_select_modifiers(
            plot_size,
            start,
            end,
            mods_xy,
            expand_x,
            expand_y,
            ModifiersMask::NONE,
        );
        assert_eq!((sxy.x.0, sxy.y.0), (0.0, 0.0));
        assert_eq!((exy.x.0, exy.y.0), (100.0, 50.0));

        let required_shift = ModifiersMask {
            shift: true,
            ..ModifiersMask::NONE
        };
        let (s_req, e_req) = PlotCanvas::<LinePlotLayer>::apply_box_select_modifiers(
            plot_size,
            start,
            end,
            mods_y,
            expand_x,
            expand_y,
            required_shift,
        );
        assert_eq!((s_req.x.0, s_req.y.0), (10.0, 10.0));
        assert_eq!((e_req.x.0, e_req.y.0), (20.0, 20.0));
    }
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

pub type HeatmapPlotCanvas = PlotCanvas<HeatmapPlotLayer>;

impl PlotCanvas<HeatmapPlotLayer> {
    pub fn new(model: Model<HeatmapPlotModel>) -> Self {
        Self::with_layer(model, HeatmapPlotLayer::default())
    }
}

#[derive(Debug, Default)]
pub struct BarsPlotLayer {
    cached_paths: Vec<CachedPath>,
}

pub type BarsPlotCanvas = PlotCanvas<BarsPlotLayer>;

impl PlotCanvas<BarsPlotLayer> {
    pub fn new(model: Model<BarsPlotModel>) -> Self {
        Self::with_layer(model, BarsPlotLayer::default())
    }
}

#[derive(Debug, Default)]
pub struct AreaPlotLayer {
    cached_paths: Vec<CachedAreaPath>,
}

pub type AreaPlotCanvas = PlotCanvas<AreaPlotLayer>;

impl PlotCanvas<AreaPlotLayer> {
    pub fn new(model: Model<AreaPlotModel>) -> Self {
        Self::with_layer(model, AreaPlotLayer::default())
    }
}

#[derive(Debug, Default)]
pub struct ShadedPlotLayer {
    cached_paths: Vec<CachedShadedPath>,
}

pub type ShadedPlotCanvas = PlotCanvas<ShadedPlotLayer>;

impl PlotCanvas<ShadedPlotLayer> {
    pub fn new(model: Model<ShadedPlotModel>) -> Self {
        Self::with_layer(model, ShadedPlotLayer::default())
    }
}

impl<L: PlotLayer + 'static> PlotCanvas<L> {
    fn apply_box_select_modifiers(
        plot_size: Size,
        start: Point,
        end: Point,
        modifiers: fret_core::Modifiers,
        expand_x: Option<ModifierKey>,
        expand_y: Option<ModifierKey>,
        required: ModifiersMask,
    ) -> (Point, Point) {
        let mut start = start;
        let mut end = end;

        // Matches ImPlot's default selection modifiers:
        // - Alt: expand selection horizontally to plot edge.
        // - Shift: expand selection vertically to plot edge.
        //
        // Note: when a modifier is required to start the drag gesture (e.g. Shift+LMB alternative),
        // treat it as part of the gesture chord and do not implicitly apply edge expansion.
        if expand_x.is_some_and(|k| k.is_pressed(modifiers) && !k.is_required_by(required)) {
            start.x = Px(0.0);
            end.x = plot_size.width;
        }
        if expand_y.is_some_and(|k| k.is_pressed(modifiers) && !k.is_required_by(required)) {
            start.y = Px(0.0);
            end.y = plot_size.height;
        }

        (start, end)
    }

    fn ensure_required_axes_enabled<H: UiHost>(&mut self, app: &mut H) {
        if !self.show_y2_axis {
            let has_y2 = self
                .model
                .read(app, |_app, m| L::data_bounds_y2(m).is_some())
                .unwrap_or(false);
            if has_y2 {
                self.show_y2_axis = true;
            }
        }
        if !self.show_y3_axis {
            let has_y3 = self
                .model
                .read(app, |_app, m| L::data_bounds_y3(m).is_some())
                .unwrap_or(false);
            if has_y3 {
                self.show_y3_axis = true;
            }
        }
        if !self.show_y4_axis {
            let has_y4 = self
                .model
                .read(app, |_app, m| L::data_bounds_y4(m).is_some())
                .unwrap_or(false);
            if has_y4 {
                self.show_y4_axis = true;
            }
        }
    }

    fn fit_view_to_data_now<H: UiHost>(
        &self,
        app: &mut H,
    ) -> (
        DataRect,
        Option<DataRect>,
        Option<DataRect>,
        Option<DataRect>,
    ) {
        let data_bounds = self.read_data_bounds(app);
        let view = if self.style.clamp_to_data_bounds {
            expand_data_bounds_scaled(
                data_bounds,
                self.style.overscroll_fraction,
                self.x_scale,
                self.y_scale,
            )
        } else {
            sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y_scale)
        };
        let view = constrain_view_bounds_scaled(
            view,
            self.x_scale,
            self.y_scale,
            self.x_constraints,
            self.y_constraints,
        );

        let view_y2 = if self.show_y2_axis {
            self.read_data_bounds_y2(app).map(|data_bounds| {
                let y2_bounds = if self.style.clamp_to_data_bounds {
                    expand_data_bounds_scaled(
                        data_bounds,
                        self.style.overscroll_fraction,
                        self.x_scale,
                        self.y2_scale,
                    )
                } else {
                    sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y2_scale)
                };

                let view_y2 = sanitize_data_rect_scaled(
                    DataRect {
                        x_min: view.x_min,
                        x_max: view.x_max,
                        y_min: y2_bounds.y_min,
                        y_max: y2_bounds.y_max,
                    },
                    self.x_scale,
                    self.y2_scale,
                );

                constrain_view_bounds_scaled(
                    view_y2,
                    self.x_scale,
                    self.y2_scale,
                    self.x_constraints,
                    self.y2_constraints,
                )
            })
        } else {
            None
        };

        let view_y3 = if self.show_y3_axis {
            self.read_data_bounds_y3(app).map(|data_bounds| {
                let y3_bounds = if self.style.clamp_to_data_bounds {
                    expand_data_bounds_scaled(
                        data_bounds,
                        self.style.overscroll_fraction,
                        self.x_scale,
                        self.y3_scale,
                    )
                } else {
                    sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y3_scale)
                };

                let view_y3 = sanitize_data_rect_scaled(
                    DataRect {
                        x_min: view.x_min,
                        x_max: view.x_max,
                        y_min: y3_bounds.y_min,
                        y_max: y3_bounds.y_max,
                    },
                    self.x_scale,
                    self.y3_scale,
                );

                constrain_view_bounds_scaled(
                    view_y3,
                    self.x_scale,
                    self.y3_scale,
                    self.x_constraints,
                    self.y3_constraints,
                )
            })
        } else {
            None
        };

        let view_y4 = if self.show_y4_axis {
            self.read_data_bounds_y4(app).map(|data_bounds| {
                let y4_bounds = if self.style.clamp_to_data_bounds {
                    expand_data_bounds_scaled(
                        data_bounds,
                        self.style.overscroll_fraction,
                        self.x_scale,
                        self.y4_scale,
                    )
                } else {
                    sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y4_scale)
                };

                let view_y4 = sanitize_data_rect_scaled(
                    DataRect {
                        x_min: view.x_min,
                        x_max: view.x_max,
                        y_min: y4_bounds.y_min,
                        y_max: y4_bounds.y_max,
                    },
                    self.x_scale,
                    self.y4_scale,
                );

                constrain_view_bounds_scaled(
                    view_y4,
                    self.x_scale,
                    self.y4_scale,
                    self.x_constraints,
                    self.y4_constraints,
                )
            })
        } else {
            None
        };

        (view, view_y2, view_y3, view_y4)
    }

    pub fn with_layer(model: Model<L::Model>, layer: L) -> Self {
        let axis_gap = LinePlotStyle::default().axis_gap;
        Self {
            model,
            style: LinePlotStyle::default(),
            input_map: PlotInputMap::default(),
            x_axis_ticks: AxisTicks::default(),
            y_axis_ticks: AxisTicks::default(),
            y2_axis_ticks: AxisTicks::default(),
            y3_axis_ticks: AxisTicks::default(),
            y4_axis_ticks: AxisTicks::default(),
            x_scale: AxisScale::Linear,
            y_scale: AxisScale::Linear,
            y2_scale: AxisScale::Linear,
            y3_scale: AxisScale::Linear,
            y4_scale: AxisScale::Linear,
            x_axis_labels: AxisLabelFormatter::default(),
            y_axis_labels: AxisLabelFormatter::default(),
            y2_axis_labels: AxisLabelFormatter::default(),
            y3_axis_labels: AxisLabelFormatter::default(),
            y4_axis_labels: AxisLabelFormatter::default(),
            tooltip_x_labels: AxisLabelFormatter::default(),
            tooltip_y_labels: AxisLabelFormatter::default(),
            layer,
            hover: None,
            plot_state: PlotState::default(),
            plot_state_model: None,
            plot_output: PlotOutput::default(),
            plot_output_model: None,
            legend_hover: None,
            cursor_px: None,
            last_pointer_pos: None,
            last_scale_factor: 1.0,
            x_axis_thickness: axis_gap,
            y_axis_thickness: axis_gap,
            y_axis_right_thickness: Px(0.0),
            y_axis_right2_thickness: Px(0.0),
            y_axis_right3_thickness: Px(0.0),
            show_y2_axis: false,
            show_y3_axis: false,
            show_y4_axis: false,
            lock_x: AxisLock::default(),
            lock_y: AxisLock::default(),
            lock_y2: AxisLock::default(),
            lock_y3: AxisLock::default(),
            lock_y4: AxisLock::default(),
            x_constraints: AxisConstraints::default(),
            y_constraints: AxisConstraints::default(),
            y2_constraints: AxisConstraints::default(),
            y3_constraints: AxisConstraints::default(),
            y4_constraints: AxisConstraints::default(),
            pan_button: None,
            pan_target: None,
            pan_start_pos: None,
            pan_last_pos: None,
            box_zoom_start: None,
            box_zoom_current: None,
            box_zoom_button: None,
            box_zoom_required_mods: None,
            query_drag_button: None,
            query_drag_start: None,
            query_drag_current: None,
            axis_label_key: None,
            axis_ticks_x: Vec::new(),
            axis_ticks_y: Vec::new(),
            axis_ticks_y2: Vec::new(),
            axis_ticks_y3: Vec::new(),
            axis_ticks_y4: Vec::new(),
            axis_labels_x: Vec::new(),
            axis_labels_y: Vec::new(),
            axis_labels_y2: Vec::new(),
            axis_labels_y3: Vec::new(),
            axis_labels_y4: Vec::new(),
            legend_key: None,
            legend_entries: Vec::new(),
            tooltip_text: None,
            mouse_readout_text: None,
            linked_cursor_readout_text: None,
        }
    }

    pub fn style(mut self, style: LinePlotStyle) -> Self {
        self.style = style;
        self
    }

    pub fn input_map(mut self, input_map: PlotInputMap) -> Self {
        self.input_map = input_map;
        self
    }

    pub fn x_axis_format(mut self, format: AxisLabelFormat) -> Self {
        self.x_axis_ticks = format.ticks();
        let labels = format.labels();
        self.x_axis_labels = labels.clone();
        self.tooltip_x_labels = labels;
        self
    }

    pub fn y_axis_format(mut self, format: AxisLabelFormat) -> Self {
        self.y_axis_ticks = format.ticks();
        let labels = format.labels();
        self.y_axis_labels = labels.clone();
        self.tooltip_y_labels = labels;
        self
    }

    pub fn y2_axis_format(mut self, format: AxisLabelFormat) -> Self {
        self.show_y2_axis = true;
        self.y2_axis_ticks = format.ticks();
        self.y2_axis_labels = format.labels();
        self
    }

    pub fn y3_axis_format(mut self, format: AxisLabelFormat) -> Self {
        self.show_y3_axis = true;
        self.y3_axis_ticks = format.ticks();
        self.y3_axis_labels = format.labels();
        self
    }

    pub fn y4_axis_format(mut self, format: AxisLabelFormat) -> Self {
        self.show_y4_axis = true;
        self.y4_axis_ticks = format.ticks();
        self.y4_axis_labels = format.labels();
        self
    }

    pub fn x_axis_locked(mut self, locked: bool) -> Self {
        self.lock_x = AxisLock {
            pan: locked,
            zoom: locked,
        };
        self
    }

    pub fn y_axis_locked(mut self, locked: bool) -> Self {
        self.lock_y = AxisLock {
            pan: locked,
            zoom: locked,
        };
        self
    }

    pub fn y2_axis_locked(mut self, locked: bool) -> Self {
        self.lock_y2 = AxisLock {
            pan: locked,
            zoom: locked,
        };
        self
    }

    pub fn y3_axis_locked(mut self, locked: bool) -> Self {
        self.lock_y3 = AxisLock {
            pan: locked,
            zoom: locked,
        };
        self
    }

    pub fn y4_axis_locked(mut self, locked: bool) -> Self {
        self.lock_y4 = AxisLock {
            pan: locked,
            zoom: locked,
        };
        self
    }

    pub fn x_axis_pan_locked(mut self, locked: bool) -> Self {
        self.lock_x.pan = locked;
        self
    }

    pub fn x_axis_zoom_locked(mut self, locked: bool) -> Self {
        self.lock_x.zoom = locked;
        self
    }

    pub fn y_axis_pan_locked(mut self, locked: bool) -> Self {
        self.lock_y.pan = locked;
        self
    }

    pub fn y_axis_zoom_locked(mut self, locked: bool) -> Self {
        self.lock_y.zoom = locked;
        self
    }

    pub fn y2_axis_pan_locked(mut self, locked: bool) -> Self {
        self.lock_y2.pan = locked;
        self
    }

    pub fn y2_axis_zoom_locked(mut self, locked: bool) -> Self {
        self.lock_y2.zoom = locked;
        self
    }

    pub fn y3_axis_pan_locked(mut self, locked: bool) -> Self {
        self.lock_y3.pan = locked;
        self
    }

    pub fn y3_axis_zoom_locked(mut self, locked: bool) -> Self {
        self.lock_y3.zoom = locked;
        self
    }

    pub fn y4_axis_pan_locked(mut self, locked: bool) -> Self {
        self.lock_y4.pan = locked;
        self
    }

    pub fn y4_axis_zoom_locked(mut self, locked: bool) -> Self {
        self.lock_y4.zoom = locked;
        self
    }

    pub fn x_axis_constraints(mut self, constraints: AxisConstraints) -> Self {
        self.x_constraints = constraints;
        self
    }

    pub fn y_axis_constraints(mut self, constraints: AxisConstraints) -> Self {
        self.y_constraints = constraints;
        self
    }

    pub fn y2_axis_constraints(mut self, constraints: AxisConstraints) -> Self {
        self.y2_constraints = constraints;
        self
    }

    pub fn y3_axis_constraints(mut self, constraints: AxisConstraints) -> Self {
        self.y3_constraints = constraints;
        self
    }

    pub fn y4_axis_constraints(mut self, constraints: AxisConstraints) -> Self {
        self.y4_constraints = constraints;
        self
    }

    pub fn x_axis_scale(mut self, scale: AxisScale) -> Self {
        self.x_scale = scale;
        self
    }

    pub fn y_axis_scale(mut self, scale: AxisScale) -> Self {
        self.y_scale = scale;
        self
    }

    pub fn y2_axis_scale(mut self, scale: AxisScale) -> Self {
        self.y2_scale = scale;
        self
    }

    pub fn y3_axis_scale(mut self, scale: AxisScale) -> Self {
        self.y3_scale = scale;
        self
    }

    pub fn y4_axis_scale(mut self, scale: AxisScale) -> Self {
        self.y4_scale = scale;
        self
    }

    pub fn x_axis_ticks(mut self, ticks: AxisTicks) -> Self {
        self.x_axis_ticks = ticks;
        self
    }

    pub fn y_axis_ticks(mut self, ticks: AxisTicks) -> Self {
        self.y_axis_ticks = ticks;
        self
    }

    pub fn y2_axis_ticks(mut self, ticks: AxisTicks) -> Self {
        self.show_y2_axis = true;
        self.y2_axis_ticks = ticks;
        self
    }

    pub fn y3_axis_ticks(mut self, ticks: AxisTicks) -> Self {
        self.show_y3_axis = true;
        self.y3_axis_ticks = ticks;
        self
    }

    pub fn y4_axis_ticks(mut self, ticks: AxisTicks) -> Self {
        self.show_y4_axis = true;
        self.y4_axis_ticks = ticks;
        self
    }

    pub fn x_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.x_axis_labels = labels.clone();
        self.tooltip_x_labels = labels;
        self
    }

    pub fn y_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y_axis_labels = labels.clone();
        self.tooltip_y_labels = labels;
        self
    }

    pub fn y2_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.show_y2_axis = true;
        self.y2_axis_labels = labels;
        self
    }

    pub fn y3_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.show_y3_axis = true;
        self.y3_axis_labels = labels;
        self
    }

    pub fn y4_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.show_y4_axis = true;
        self.y4_axis_labels = labels;
        self
    }

    pub fn tooltip_x_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.tooltip_x_labels = labels;
        self
    }

    pub fn tooltip_y_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.tooltip_y_labels = labels;
        self
    }

    pub fn state(mut self, state: Model<PlotState>) -> Self {
        self.plot_state_model = Some(state);
        self
    }

    pub fn output(mut self, output: Model<PlotOutput>) -> Self {
        self.plot_output_model = Some(output);
        self
    }

    pub fn create_node<H: UiHost>(ui: &mut fret_ui::UiTree<H>, canvas: Self) -> fret_core::NodeId {
        ui.create_node_retained(canvas)
    }

    fn axis_gaps(&self) -> (Px, Px, Px, Px, Px) {
        let min = self.style.axis_gap.0.max(0.0);
        let y = self.y_axis_thickness.0.max(min);
        let y_right = if self.show_y2_axis {
            self.y_axis_right_thickness.0.max(min)
        } else {
            0.0
        };
        let y_right2 = if self.show_y3_axis {
            self.y_axis_right2_thickness.0.max(min)
        } else {
            0.0
        };
        let y_right3 = if self.show_y4_axis {
            self.y_axis_right3_thickness.0.max(min)
        } else {
            0.0
        };
        let x = self.x_axis_thickness.0.max(min);
        (Px(y), Px(y_right), Px(y_right2), Px(y_right3), Px(x))
    }

    fn read_plot_state<H: UiHost>(&self, app: &mut H) -> PlotState {
        if let Some(state) = &self.plot_state_model {
            state
                .read(app, |_app, s| s.clone())
                .unwrap_or_else(|_| self.plot_state.clone())
        } else {
            self.plot_state.clone()
        }
    }

    fn update_plot_state<H: UiHost>(
        &mut self,
        app: &mut H,
        f: impl FnOnce(&mut PlotState),
    ) -> bool {
        if let Some(state) = &self.plot_state_model {
            state.update(app, |s, _cx| f(s)).is_ok()
        } else {
            f(&mut self.plot_state);
            true
        }
    }

    fn publish_plot_output<H: UiHost>(&mut self, app: &mut H, snapshot: PlotOutputSnapshot) {
        if self.plot_output.snapshot == snapshot {
            return;
        }

        self.plot_output.revision = self.plot_output.revision.wrapping_add(1);
        self.plot_output.snapshot = snapshot;

        if let Some(model) = &self.plot_output_model {
            let next = self.plot_output;
            let _ = model.update(app, |s, _cx| {
                *s = next;
            });
        }
    }

    fn publish_current_output_snapshot<H: UiHost>(
        &mut self,
        app: &mut H,
        layout: PlotLayout,
        state: &PlotState,
        view_bounds: DataRect,
        view_bounds_y2: Option<DataRect>,
        view_bounds_y3: Option<DataRect>,
        view_bounds_y4: Option<DataRect>,
    ) {
        let cursor_data = self.cursor_px.and_then(|cursor_px| {
            if layout.plot.size.width.0 <= 0.0 || layout.plot.size.height.0 <= 0.0 {
                return None;
            }
            let transform = PlotTransform {
                viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size),
                data: view_bounds,
                x_scale: self.x_scale,
                y_scale: self.y_scale,
            };
            let data = transform.px_to_data(cursor_px);
            (data.x.is_finite() && data.y.is_finite()).then_some(data)
        });

        self.publish_plot_output(
            app,
            PlotOutputSnapshot {
                view_bounds,
                view_bounds_y2,
                view_bounds_y3,
                view_bounds_y4,
                cursor: cursor_data,
                hover: self.hover.map(|h| PlotHoverOutput {
                    series_id: h.series_id,
                    data: h.data,
                    value: h.value,
                }),
                query: state.query,
            },
        );
    }

    fn current_view_bounds<H: UiHost>(&self, app: &mut H, state: &PlotState) -> DataRect {
        let view = if state.view_is_auto {
            let data_bounds = self.read_data_bounds(app);
            if self.style.clamp_to_data_bounds {
                expand_data_bounds_scaled(
                    data_bounds,
                    self.style.overscroll_fraction,
                    self.x_scale,
                    self.y_scale,
                )
            } else {
                sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y_scale)
            }
        } else if let Some(view) = state.view_bounds {
            sanitize_data_rect_scaled(view, self.x_scale, self.y_scale)
        } else {
            sanitize_data_rect_scaled(self.read_data_bounds(app), self.x_scale, self.y_scale)
        };

        constrain_view_bounds_scaled(
            view,
            self.x_scale,
            self.y_scale,
            self.x_constraints,
            self.y_constraints,
        )
    }

    fn current_view_bounds_y2<H: UiHost>(
        &self,
        app: &mut H,
        state: &PlotState,
        view_bounds: DataRect,
    ) -> Option<DataRect> {
        if !self.show_y2_axis {
            return None;
        }

        let data_bounds = self.read_data_bounds_y2(app)?;

        let y2_bounds = if state.view_y2_is_auto {
            if self.style.clamp_to_data_bounds {
                expand_data_bounds_scaled(
                    data_bounds,
                    self.style.overscroll_fraction,
                    self.x_scale,
                    self.y2_scale,
                )
            } else {
                sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y2_scale)
            }
        } else if let Some(view) = state.view_bounds_y2 {
            sanitize_data_rect_scaled(view, self.x_scale, self.y2_scale)
        } else {
            sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y2_scale)
        };

        let view = sanitize_data_rect_scaled(
            DataRect {
                x_min: view_bounds.x_min,
                x_max: view_bounds.x_max,
                y_min: y2_bounds.y_min,
                y_max: y2_bounds.y_max,
            },
            self.x_scale,
            self.y2_scale,
        );

        Some(constrain_view_bounds_scaled(
            view,
            self.x_scale,
            self.y2_scale,
            self.x_constraints,
            self.y2_constraints,
        ))
    }

    fn current_view_bounds_y3<H: UiHost>(
        &self,
        app: &mut H,
        state: &PlotState,
        view_bounds: DataRect,
    ) -> Option<DataRect> {
        if !self.show_y3_axis {
            return None;
        }

        let data_bounds = self.read_data_bounds_y3(app)?;

        let y3_bounds = if state.view_y3_is_auto {
            if self.style.clamp_to_data_bounds {
                expand_data_bounds_scaled(
                    data_bounds,
                    self.style.overscroll_fraction,
                    self.x_scale,
                    self.y3_scale,
                )
            } else {
                sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y3_scale)
            }
        } else if let Some(view) = state.view_bounds_y3 {
            sanitize_data_rect_scaled(view, self.x_scale, self.y3_scale)
        } else {
            sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y3_scale)
        };

        let view = sanitize_data_rect_scaled(
            DataRect {
                x_min: view_bounds.x_min,
                x_max: view_bounds.x_max,
                y_min: y3_bounds.y_min,
                y_max: y3_bounds.y_max,
            },
            self.x_scale,
            self.y3_scale,
        );

        Some(constrain_view_bounds_scaled(
            view,
            self.x_scale,
            self.y3_scale,
            self.x_constraints,
            self.y3_constraints,
        ))
    }

    fn current_view_bounds_y4<H: UiHost>(
        &self,
        app: &mut H,
        state: &PlotState,
        view_bounds: DataRect,
    ) -> Option<DataRect> {
        if !self.show_y4_axis {
            return None;
        }

        let data_bounds = self.read_data_bounds_y4(app)?;

        let y4_bounds = if state.view_y4_is_auto {
            if self.style.clamp_to_data_bounds {
                expand_data_bounds_scaled(
                    data_bounds,
                    self.style.overscroll_fraction,
                    self.x_scale,
                    self.y4_scale,
                )
            } else {
                sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y4_scale)
            }
        } else if let Some(view) = state.view_bounds_y4 {
            sanitize_data_rect_scaled(view, self.x_scale, self.y4_scale)
        } else {
            sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y4_scale)
        };

        let view = sanitize_data_rect_scaled(
            DataRect {
                x_min: view_bounds.x_min,
                x_max: view_bounds.x_max,
                y_min: y4_bounds.y_min,
                y_max: y4_bounds.y_max,
            },
            self.x_scale,
            self.y4_scale,
        );

        Some(constrain_view_bounds_scaled(
            view,
            self.x_scale,
            self.y4_scale,
            self.x_constraints,
            self.y4_constraints,
        ))
    }

    fn rebuild_paths_if_needed<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        plot: Rect,
        view_bounds: DataRect,
        view_bounds_y2: Option<DataRect>,
        view_bounds_y3: Option<DataRect>,
        view_bounds_y4: Option<DataRect>,
        hidden: &HashSet<SeriesId>,
    ) -> Vec<(SeriesId, PathId, Color)> {
        let model_revision = self.model.revision(cx.app).unwrap_or(0);
        let Ok(model) = self.model.read(cx.app, |_app, m| m.clone()) else {
            return Vec::new();
        };

        self.layer.paint_paths(
            cx,
            &model,
            PlotPaintArgs {
                model_revision,
                plot,
                view_bounds,
                view_bounds_y2,
                view_bounds_y3,
                view_bounds_y4,
                x_scale: self.x_scale,
                y_scale: self.y_scale,
                y2_scale: self.y2_scale,
                y3_scale: self.y3_scale,
                y4_scale: self.y4_scale,
                style: self.style,
                hidden,
            },
        )
    }

    fn rebuild_quads_if_needed<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        plot: Rect,
        view_bounds: DataRect,
        view_bounds_y2: Option<DataRect>,
        view_bounds_y3: Option<DataRect>,
        view_bounds_y4: Option<DataRect>,
        hidden: &HashSet<SeriesId>,
    ) -> Vec<PlotQuad> {
        let model_revision = self.model.revision(cx.app).unwrap_or(0);
        let Ok(model) = self.model.read(cx.app, |_app, m| m.clone()) else {
            return Vec::new();
        };

        self.layer.paint_quads(
            cx,
            &model,
            PlotPaintArgs {
                model_revision,
                plot,
                view_bounds,
                view_bounds_y2,
                view_bounds_y3,
                view_bounds_y4,
                x_scale: self.x_scale,
                y_scale: self.y_scale,
                y2_scale: self.y2_scale,
                y3_scale: self.y3_scale,
                y4_scale: self.y4_scale,
                style: self.style,
                hidden,
            },
        )
    }

    fn clear_axis_label_cache(&mut self, services: &mut dyn UiServices) {
        for t in self.axis_labels_x.drain(..) {
            services.text().release(t.blob);
        }
        for t in self.axis_labels_y.drain(..) {
            services.text().release(t.blob);
        }
        for t in self.axis_labels_y2.drain(..) {
            services.text().release(t.blob);
        }
        for t in self.axis_labels_y3.drain(..) {
            services.text().release(t.blob);
        }
        for t in self.axis_labels_y4.drain(..) {
            services.text().release(t.blob);
        }
        self.axis_ticks_x.clear();
        self.axis_ticks_y.clear();
        self.axis_ticks_y2.clear();
        self.axis_ticks_y3.clear();
        self.axis_ticks_y4.clear();
        self.axis_label_key = None;
    }

    fn clear_legend_cache(&mut self, services: &mut dyn UiServices) {
        for e in self.legend_entries.drain(..) {
            services.text().release(e.text.blob);
        }
        self.legend_key = None;
    }

    fn legend_layout(&self, layout: PlotLayout) -> Option<(Rect, Vec<Rect>)> {
        if self.legend_entries.len() <= 1 {
            return None;
        }
        if layout.plot.size.width.0 <= 0.0 || layout.plot.size.height.0 <= 0.0 {
            return None;
        }

        let margin = Px(8.0);
        let pad = Px(8.0);
        let gap = Px(8.0);
        let row_gap = Px(4.0);
        let swatch_w = Px(14.0);
        let swatch_h = Px(self.style.stroke_width.0.clamp(2.0, 6.0));

        let mut max_label_w = 0.0f32;
        let mut total_h = 0.0f32;
        for (i, entry) in self.legend_entries.iter().enumerate() {
            if i > 0 {
                total_h += row_gap.0;
            }
            max_label_w = max_label_w.max(entry.text.metrics.size.width.0);
            total_h += entry.text.metrics.size.height.0.max(swatch_h.0);
        }

        let legend_w = Px(pad.0 * 2.0 + swatch_w.0 + gap.0 + max_label_w);
        let legend_h = Px(pad.0 * 2.0 + total_h);

        let mut x = Px(layout.plot.origin.x.0 + layout.plot.size.width.0 - legend_w.0 - margin.0);
        let mut y = Px(layout.plot.origin.y.0 + margin.0);
        x = Px(x.0.max(layout.plot.origin.x.0));
        y = Px(y.0.max(layout.plot.origin.y.0));

        let rect = Rect::new(Point::new(x, y), Size::new(legend_w, legend_h));

        let mut rows: Vec<Rect> = Vec::with_capacity(self.legend_entries.len());
        let mut cursor_y = rect.origin.y.0 + pad.0;
        for (i, entry) in self.legend_entries.iter().enumerate() {
            let row_h = entry.text.metrics.size.height.0.max(swatch_h.0);
            rows.push(Rect::new(
                Point::new(rect.origin.x, Px(cursor_y)),
                Size::new(rect.size.width, Px(row_h)),
            ));
            cursor_y += row_h;
            if i + 1 < self.legend_entries.len() {
                cursor_y += row_gap.0;
            }
        }

        Some((rect, rows))
    }

    fn legend_swatch_column(rect: Rect) -> Rect {
        let pad = Px(8.0);
        let swatch_w = Px(14.0);
        Rect::new(
            Point::new(Px(rect.origin.x.0 + pad.0), rect.origin.y),
            Size::new(swatch_w, rect.size.height),
        )
    }

    fn hash_u64(mut state: u64, v: u64) -> u64 {
        state ^= v
            .wrapping_add(0x9e3779b97f4a7c15)
            .wrapping_add(state << 6)
            .wrapping_add(state >> 2);
        state
    }

    fn hash_f32_bits(state: u64, v: f32) -> u64 {
        Self::hash_u64(state, u64::from(v.to_bits()))
    }

    fn hash_f64_bits(state: u64, v: f64) -> u64 {
        Self::hash_u64(state, v.to_bits())
    }

    fn read_data_bounds<H: UiHost>(&self, app: &mut H) -> DataRect {
        let data_bounds = self
            .model
            .read(app, |_app, m| L::data_bounds(m))
            .unwrap_or(DataRect {
                x_min: 0.0,
                x_max: 1.0,
                y_min: 0.0,
                y_max: 1.0,
            });
        sanitize_data_rect(data_bounds)
    }

    fn read_data_bounds_y2<H: UiHost>(&self, app: &mut H) -> Option<DataRect> {
        let bounds = self
            .model
            .read(app, |_app, m| L::data_bounds_y2(m))
            .ok()
            .flatten()?;
        Some(sanitize_data_rect(bounds))
    }

    fn read_data_bounds_y3<H: UiHost>(&self, app: &mut H) -> Option<DataRect> {
        let bounds = self
            .model
            .read(app, |_app, m| L::data_bounds_y3(m))
            .ok()
            .flatten()?;
        Some(sanitize_data_rect(bounds))
    }

    fn read_data_bounds_y4<H: UiHost>(&self, app: &mut H) -> Option<DataRect> {
        let bounds = self
            .model
            .read(app, |_app, m| L::data_bounds_y4(m))
            .ok()
            .flatten()?;
        Some(sanitize_data_rect(bounds))
    }

    fn text_style_key(style: &TextStyle) -> u64 {
        let mut state = 0u64;
        state = Self::hash_u64(state, hash_value(&style.font));
        state = Self::hash_u64(state, u64::from(style.weight.0));
        state = Self::hash_f32_bits(state, style.size.0);
        state = Self::hash_u64(
            state,
            u64::from(style.line_height.map(|v| v.0.to_bits()).unwrap_or(0)),
        );
        state = Self::hash_u64(
            state,
            u64::from(style.letter_spacing_em.map(|v| v.to_bits()).unwrap_or(0)),
        );
        state
    }

    fn prepare_text(
        &mut self,
        services: &mut dyn UiServices,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> PreparedText {
        let mut state = 0u64;
        for b in text.as_bytes() {
            state = Self::hash_u64(state, u64::from(*b));
        }
        state = Self::hash_u64(state, Self::text_style_key(style));
        state = Self::hash_u64(state, u64::from(constraints.scale_factor.to_bits()));
        state = Self::hash_u64(
            state,
            u64::from(constraints.max_width.map(|v| v.0.to_bits()).unwrap_or(0)),
        );
        state = Self::hash_u64(state, hash_value(&constraints.wrap));
        state = Self::hash_u64(state, hash_value(&constraints.overflow));

        let (blob, metrics) = services.text().prepare(text, style, constraints);
        PreparedText {
            blob,
            metrics,
            key: state,
        }
    }

    fn rebuild_axis_labels_if_needed<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        layout: PlotLayout,
        view_bounds: DataRect,
        view_bounds_y2: Option<DataRect>,
        view_bounds_y3: Option<DataRect>,
        view_bounds_y4: Option<DataRect>,
        theme_revision: u64,
        font_stack_key: u64,
    ) -> bool {
        let scale_bits = cx.scale_factor.to_bits();

        let mut key = 0u64;
        key = Self::hash_u64(key, u64::from(scale_bits));
        key = Self::hash_u64(key, theme_revision);
        key = Self::hash_u64(key, font_stack_key);
        key = Self::hash_f32_bits(key, layout.plot.size.width.0);
        key = Self::hash_f32_bits(key, layout.plot.size.height.0);
        key = Self::hash_f32_bits(key, layout.y_axis_left.size.width.0);
        key = Self::hash_f32_bits(key, layout.y_axis_right.size.width.0);
        key = Self::hash_f32_bits(key, layout.y_axis_right2.size.width.0);
        key = Self::hash_f32_bits(key, layout.y_axis_right3.size.width.0);
        key = Self::hash_f32_bits(key, layout.x_axis.size.height.0);
        key = Self::hash_f64_bits(key, view_bounds.x_min);
        key = Self::hash_f64_bits(key, view_bounds.x_max);
        key = Self::hash_f64_bits(key, view_bounds.y_min);
        key = Self::hash_f64_bits(key, view_bounds.y_max);
        if let Some(y2) = view_bounds_y2 {
            key = Self::hash_f64_bits(key, y2.y_min);
            key = Self::hash_f64_bits(key, y2.y_max);
        } else {
            key = Self::hash_u64(key, 0);
        }
        if let Some(y3) = view_bounds_y3 {
            key = Self::hash_f64_bits(key, y3.y_min);
            key = Self::hash_f64_bits(key, y3.y_max);
        } else {
            key = Self::hash_u64(key, 0);
        }
        if let Some(y4) = view_bounds_y4 {
            key = Self::hash_f64_bits(key, y4.y_min);
            key = Self::hash_f64_bits(key, y4.y_max);
        } else {
            key = Self::hash_u64(key, 0);
        }
        key = Self::hash_u64(key, u64::from(self.style.axis_gap.0.to_bits()));
        key = Self::hash_u64(key, u64::from(self.style.tick_count as u32));
        key = Self::hash_u64(key, self.x_axis_ticks.key());
        key = Self::hash_u64(key, self.y_axis_ticks.key());
        key = Self::hash_u64(key, self.y2_axis_ticks.key());
        key = Self::hash_u64(key, self.y3_axis_ticks.key());
        key = Self::hash_u64(key, self.y4_axis_ticks.key());
        key = Self::hash_u64(key, self.x_scale.key());
        key = Self::hash_u64(key, self.y_scale.key());
        key = Self::hash_u64(key, self.y2_scale.key());
        key = Self::hash_u64(key, self.y3_scale.key());
        key = Self::hash_u64(key, self.y4_scale.key());
        key = Self::hash_u64(key, self.x_axis_labels.key());
        key = Self::hash_u64(key, self.y_axis_labels.key());
        key = Self::hash_u64(key, self.y2_axis_labels.key());
        key = Self::hash_u64(key, self.y3_axis_labels.key());
        key = Self::hash_u64(key, self.y4_axis_labels.key());
        key = Self::hash_u64(key, u64::from(self.show_y2_axis));
        key = Self::hash_u64(key, u64::from(self.show_y3_axis));
        key = Self::hash_u64(key, u64::from(self.show_y4_axis));

        if self.axis_label_key == Some(key) {
            return false;
        }

        self.clear_axis_label_cache(cx.services);

        let font_size = cx
            .theme()
            .metric_by_key("font.size")
            .unwrap_or(cx.theme().metrics.font_size);
        let style = TextStyle {
            font: FontId::default(),
            size: Px((font_size.0 * 0.90).max(10.0)),
            weight: FontWeight::NORMAL,
            line_height: None,
            letter_spacing_em: None,
        };

        let axis_span = |min: f64, max: f64, scale: AxisScale| -> f64 {
            let span_data = (max - min).abs();
            scale
                .to_axis(min)
                .zip(scale.to_axis(max))
                .map(|(a, b)| (b - a).abs())
                .unwrap_or(span_data)
        };

        let x_span = axis_span(view_bounds.x_min, view_bounds.x_max, self.x_scale);
        let y_span = axis_span(view_bounds.y_min, view_bounds.y_max, self.y_scale);
        let y2_span = view_bounds_y2
            .map(|b| axis_span(b.y_min, b.y_max, self.y2_scale))
            .unwrap_or(0.0);
        let y3_span = view_bounds_y3
            .map(|b| axis_span(b.y_min, b.y_max, self.y3_scale))
            .unwrap_or(0.0);
        let y4_span = view_bounds_y4
            .map(|b| axis_span(b.y_min, b.y_max, self.y4_scale))
            .unwrap_or(0.0);

        let constraints_x = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        let constraints_y = TextConstraints {
            max_width: Some(layout.y_axis_left.size.width),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        let constraints_y2 = TextConstraints {
            max_width: Some(layout.y_axis_right.size.width),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        let constraints_y3 = TextConstraints {
            max_width: Some(layout.y_axis_right2.size.width),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        let constraints_y4 = TextConstraints {
            max_width: Some(layout.y_axis_right3.size.width),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };

        let plot_w = layout.plot.size.width.0.max(0.0);
        let plot_h = layout.plot.size.height.0.max(0.0);

        let estimate_ticks = |span_px: f32, target_spacing_px: f32| -> usize {
            if !span_px.is_finite() || span_px <= 0.0 {
                return 2;
            }
            let spacing = target_spacing_px.max(1.0);
            ((span_px / spacing).floor() as usize)
                .saturating_add(1)
                .max(2)
        };

        let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size);
        let transform_y1 = PlotTransform {
            viewport: local_viewport,
            data: view_bounds,
            x_scale: self.x_scale,
            y_scale: self.y_scale,
        };

        let min_ticks = self.style.tick_count.max(2).min(128);
        let mut x_tick_count = estimate_ticks(plot_w, 70.0).max(min_ticks).min(128);
        let mut y_tick_count = estimate_ticks(plot_h, 28.0).max(min_ticks).min(128);

        let mut x_ticks: Vec<f64> = Vec::new();
        let mut y_ticks: Vec<f64> = Vec::new();

        // X axis: reduce ticks until formatted labels fit horizontally.
        for _ in 0..8 {
            x_ticks = axis_ticks_scaled(
                view_bounds.x_min,
                view_bounds.x_max,
                x_tick_count,
                self.x_axis_ticks,
                self.x_scale,
            );
            if x_ticks.len() <= 1 {
                break;
            }

            let mut max_w = 0.0f32;
            for v in &x_ticks {
                let text = self.x_axis_labels.format(*v, x_span);
                let prepared = self.prepare_text(cx.services, &text, &style, constraints_x);
                max_w = max_w.max(prepared.metrics.size.width.0);
                cx.services.text().release(prepared.blob);
            }

            let mut min_spacing_px = plot_w;
            let mut prev: Option<f32> = None;
            for v in &x_ticks {
                let Some(px) = transform_y1.data_x_to_px(*v) else {
                    continue;
                };
                let x = px.0;
                if let Some(prev) = prev {
                    let dx = (x - prev).abs();
                    if dx.is_finite() && dx > 0.0 {
                        min_spacing_px = min_spacing_px.min(dx);
                    }
                }
                prev = Some(x);
            }
            let spacing_px = min_spacing_px;
            let needed = max_w + 8.0;
            if spacing_px.is_finite() && needed.is_finite() && spacing_px >= needed {
                break;
            }

            let suggested = ((plot_w / needed.max(1.0)).floor() as usize)
                .saturating_add(1)
                .max(2);
            let next = x_tick_count.min(suggested);
            x_tick_count = if next == x_tick_count {
                x_tick_count.saturating_sub(1).max(2)
            } else {
                next
            };
            if x_tick_count <= 2 {
                break;
            }
        }

        // Y axis: reduce ticks until labels fit vertically (avoid overlap).
        for _ in 0..8 {
            y_ticks = axis_ticks_scaled(
                view_bounds.y_min,
                view_bounds.y_max,
                y_tick_count,
                self.y_axis_ticks,
                self.y_scale,
            );
            if y_ticks.len() <= 1 {
                break;
            }

            let mut max_h = 0.0f32;
            for v in &y_ticks {
                let text = self.y_axis_labels.format(*v, y_span);
                let prepared = self.prepare_text(cx.services, &text, &style, constraints_y);
                max_h = max_h.max(prepared.metrics.size.height.0);
                cx.services.text().release(prepared.blob);
            }

            let mut min_spacing_px = plot_h;
            let mut prev: Option<f32> = None;
            for v in &y_ticks {
                let Some(px) = transform_y1.data_y_to_px(*v) else {
                    continue;
                };
                let y = px.0;
                if let Some(prev) = prev {
                    let dy = (y - prev).abs();
                    if dy.is_finite() && dy > 0.0 {
                        min_spacing_px = min_spacing_px.min(dy);
                    }
                }
                prev = Some(y);
            }
            let spacing_px = min_spacing_px;
            let needed = max_h + 4.0;
            if spacing_px.is_finite() && needed.is_finite() && spacing_px >= needed {
                break;
            }

            let suggested = ((plot_h / needed.max(1.0)).floor() as usize)
                .saturating_add(1)
                .max(2);
            let next = y_tick_count.min(suggested);
            y_tick_count = if next == y_tick_count {
                y_tick_count.saturating_sub(1).max(2)
            } else {
                next
            };
            if y_tick_count <= 2 {
                break;
            }
        }

        self.axis_ticks_x = x_ticks.clone();
        self.axis_ticks_y = y_ticks.clone();
        self.axis_ticks_y2 = if self.show_y2_axis {
            if let Some(y2_bounds) = view_bounds_y2 {
                axis_ticks_scaled(
                    y2_bounds.y_min,
                    y2_bounds.y_max,
                    y_tick_count,
                    self.y2_axis_ticks,
                    self.y2_scale,
                )
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        self.axis_ticks_y3 = if self.show_y3_axis {
            if let Some(y3_bounds) = view_bounds_y3 {
                axis_ticks_scaled(
                    y3_bounds.y_min,
                    y3_bounds.y_max,
                    y_tick_count,
                    self.y3_axis_ticks,
                    self.y3_scale,
                )
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        self.axis_ticks_y4 = if self.show_y4_axis {
            if let Some(y4_bounds) = view_bounds_y4 {
                axis_ticks_scaled(
                    y4_bounds.y_min,
                    y4_bounds.y_max,
                    y_tick_count,
                    self.y4_axis_ticks,
                    self.y4_scale,
                )
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        for v in x_ticks {
            let text = if self.x_scale == AxisScale::Log10 && self.x_axis_labels.is_number_auto() {
                log10_tick_label_or_empty(v)
            } else {
                self.x_axis_labels.format(v, x_span)
            };
            let prepared = self.prepare_text(cx.services, &text, &style, constraints_x);
            self.axis_labels_x.push(prepared);
        }

        for v in y_ticks {
            let text = if self.y_scale == AxisScale::Log10 && self.y_axis_labels.is_number_auto() {
                log10_tick_label_or_empty(v)
            } else {
                self.y_axis_labels.format(v, y_span)
            };
            let prepared = self.prepare_text(cx.services, &text, &style, constraints_y);
            self.axis_labels_y.push(prepared);
        }

        if self.show_y2_axis {
            let y2_ticks = self.axis_ticks_y2.clone();
            for v in y2_ticks {
                let text =
                    if self.y2_scale == AxisScale::Log10 && self.y2_axis_labels.is_number_auto() {
                        log10_tick_label_or_empty(v)
                    } else {
                        self.y2_axis_labels.format(v, y2_span)
                    };
                let prepared = self.prepare_text(cx.services, &text, &style, constraints_y2);
                self.axis_labels_y2.push(prepared);
            }
        }

        if self.show_y3_axis {
            let y3_ticks = self.axis_ticks_y3.clone();
            for v in y3_ticks {
                let text =
                    if self.y3_scale == AxisScale::Log10 && self.y3_axis_labels.is_number_auto() {
                        log10_tick_label_or_empty(v)
                    } else {
                        self.y3_axis_labels.format(v, y3_span)
                    };
                let prepared = self.prepare_text(cx.services, &text, &style, constraints_y3);
                self.axis_labels_y3.push(prepared);
            }
        }

        if self.show_y4_axis {
            let y4_ticks = self.axis_ticks_y4.clone();
            for v in y4_ticks {
                let text =
                    if self.y4_scale == AxisScale::Log10 && self.y4_axis_labels.is_number_auto() {
                        log10_tick_label_or_empty(v)
                    } else {
                        self.y4_axis_labels.format(v, y4_span)
                    };
                let prepared = self.prepare_text(cx.services, &text, &style, constraints_y4);
                self.axis_labels_y4.push(prepared);
            }
        }

        self.axis_label_key = Some(key);

        // Axis thickness auto-fit: expand the axis gaps to fit the largest tick label.
        // This mirrors egui_plot's approach where axis thickness is cached and increased as needed.
        let mut changed = false;
        let min_axis = self.style.axis_gap.0.max(0.0);

        let content_w = layout.plot.size.width.0
            + layout.y_axis_left.size.width.0
            + layout.y_axis_right.size.width.0
            + layout.y_axis_right2.size.width.0
            + layout.y_axis_right3.size.width.0;
        let content_h = layout.plot.size.height.0 + layout.x_axis.size.height.0;

        let max_y_label_w = self
            .axis_labels_y
            .iter()
            .map(|t| t.metrics.size.width.0)
            .fold(0.0f32, f32::max);
        let max_y2_label_w = self
            .axis_labels_y2
            .iter()
            .map(|t| t.metrics.size.width.0)
            .fold(0.0f32, f32::max);
        let max_y3_label_w = self
            .axis_labels_y3
            .iter()
            .map(|t| t.metrics.size.width.0)
            .fold(0.0f32, f32::max);
        let max_y4_label_w = self
            .axis_labels_y4
            .iter()
            .map(|t| t.metrics.size.width.0)
            .fold(0.0f32, f32::max);
        let max_x_label_h = self
            .axis_labels_x
            .iter()
            .map(|t| t.metrics.size.height.0)
            .fold(0.0f32, f32::max);

        let desired_y = (max_y_label_w + 8.0).max(min_axis);
        let desired_x = (max_x_label_h + 6.0).max(min_axis);
        let desired_y2 = (max_y2_label_w + 8.0).max(min_axis);
        let desired_y3 = (max_y3_label_w + 8.0).max(min_axis);
        let desired_y4 = (max_y4_label_w + 8.0).max(min_axis);

        // Prevent axes from consuming the majority of the plot area in degenerate cases.
        let cap_y = (content_w * 0.5).max(min_axis);
        let cap_x = (content_h * 0.5).max(min_axis);

        let next_y = desired_y.min(cap_y);
        let next_x = desired_x.min(cap_x);
        let next_y2 = desired_y2.min(cap_y);
        let next_y3 = desired_y3.min(cap_y);
        let next_y4 = desired_y4.min(cap_y);

        if next_y.is_finite() && next_y > self.y_axis_thickness.0 + 0.5 {
            self.y_axis_thickness = Px(next_y);
            changed = true;
        }
        if self.show_y2_axis && next_y2.is_finite() && next_y2 > self.y_axis_right_thickness.0 + 0.5
        {
            self.y_axis_right_thickness = Px(next_y2);
            changed = true;
        }
        if self.show_y3_axis
            && next_y3.is_finite()
            && next_y3 > self.y_axis_right2_thickness.0 + 0.5
        {
            self.y_axis_right2_thickness = Px(next_y3);
            changed = true;
        }
        if self.show_y4_axis
            && next_y4.is_finite()
            && next_y4 > self.y_axis_right3_thickness.0 + 0.5
        {
            self.y_axis_right3_thickness = Px(next_y4);
            changed = true;
        }
        if next_x.is_finite() && next_x > self.x_axis_thickness.0 + 0.5 {
            self.x_axis_thickness = Px(next_x);
            changed = true;
        }

        changed
    }

    fn rebuild_legend_if_needed<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        theme_revision: u64,
        font_stack_key: u64,
    ) {
        let series: Vec<SeriesMeta> = self
            .model
            .read(cx.app, |_app, m| L::series_meta(m))
            .unwrap_or_default();
        if let Some(hovered) = self.legend_hover
            && series.iter().all(|s| s.id != hovered)
        {
            self.legend_hover = None;
        }

        if series.len() <= 1 {
            if self.legend_key.is_some() {
                self.clear_legend_cache(cx.services);
            }
            return;
        }

        let font_size = cx
            .theme()
            .metric_by_key("font.size")
            .unwrap_or(cx.theme().metrics.font_size);
        let style = TextStyle {
            font: FontId::default(),
            size: Px((font_size.0 * 0.85).max(10.0)),
            weight: FontWeight::NORMAL,
            line_height: None,
            letter_spacing_em: None,
        };
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };

        let mut key = 0u64;
        key = Self::hash_u64(key, theme_revision);
        key = Self::hash_u64(key, font_stack_key);
        key = Self::hash_u64(key, u64::from(cx.scale_factor.to_bits()));
        key = Self::hash_u64(key, u64::from(series.len() as u32));
        key = Self::hash_u64(key, Self::text_style_key(&style));
        for s in &series {
            key = Self::hash_u64(key, s.id.0);
            for b in s.label.as_bytes() {
                key = Self::hash_u64(key, u64::from(*b));
            }
        }

        if self.legend_key == Some(key) {
            return;
        }

        self.clear_legend_cache(cx.services);

        self.legend_entries = Vec::with_capacity(series.len());
        for s in series {
            let text = s.label.to_string();
            let prepared = self.prepare_text(cx.services, &text, &style, constraints);
            self.legend_entries.push(LegendEntry {
                id: s.id,
                text: prepared,
            });
        }

        self.legend_key = Some(key);
    }
}

impl<H: UiHost, L: PlotLayer + 'static> Widget<H> for PlotCanvas<L> {
    fn event(&mut self, cx: &mut fret_ui::retained_bridge::EventCx<'_, H>, event: &Event) {
        // Axis enablement is derived from the model (series -> axis assignment), so make sure
        // we don't accidentally interpret "right axis series" using the primary Y transform.
        self.ensure_required_axes_enabled(cx.app);

        match event {
            Event::KeyDown { key, modifiers, .. } => {
                let plain = !modifiers.shift
                    && !modifiers.ctrl
                    && !modifiers.alt
                    && !modifiers.alt_gr
                    && !modifiers.meta;
                let lock_mods_ok = !modifiers.alt && !modifiers.alt_gr && !modifiers.meta;
                if lock_mods_ok && *key == KeyCode::KeyL {
                    let Some(pos) = self.last_pointer_pos else {
                        return;
                    };

                    let (
                        y_axis_gap,
                        y_axis_right_gap,
                        y_axis_right2_gap,
                        y_axis_right3_gap,
                        x_axis_gap,
                    ) = self.axis_gaps();
                    let layout = PlotLayout::from_bounds(
                        cx.bounds,
                        self.style.padding,
                        y_axis_gap,
                        y_axis_right_gap,
                        y_axis_right2_gap,
                        y_axis_right3_gap,
                        x_axis_gap,
                    );
                    let Some(region) = layout.hit_test_region(pos) else {
                        return;
                    };

                    let toggle_pan = modifiers.shift && !modifiers.ctrl;
                    let toggle_zoom = modifiers.ctrl && !modifiers.shift;
                    let toggle_both = !toggle_pan && !toggle_zoom;

                    match region {
                        PlotRegion::XAxis => {
                            if toggle_both || toggle_pan {
                                self.lock_x.pan = !self.lock_x.pan;
                            }
                            if toggle_both || toggle_zoom {
                                self.lock_x.zoom = !self.lock_x.zoom;
                            }
                        }
                        PlotRegion::YAxis(axis) => {
                            let lock = match axis {
                                YAxis::Left => &mut self.lock_y,
                                YAxis::Right => &mut self.lock_y2,
                                YAxis::Right2 => &mut self.lock_y3,
                                YAxis::Right3 => &mut self.lock_y4,
                            };
                            if toggle_both || toggle_pan {
                                lock.pan = !lock.pan;
                            }
                            if toggle_both || toggle_zoom {
                                lock.zoom = !lock.zoom;
                            }
                        }
                        PlotRegion::Plot => {
                            if toggle_both || toggle_pan {
                                self.lock_x.pan = !self.lock_x.pan;
                                self.lock_y.pan = !self.lock_y.pan;
                                if self.show_y2_axis {
                                    self.lock_y2.pan = !self.lock_y2.pan;
                                }
                                if self.show_y3_axis {
                                    self.lock_y3.pan = !self.lock_y3.pan;
                                }
                                if self.show_y4_axis {
                                    self.lock_y4.pan = !self.lock_y4.pan;
                                }
                            }
                            if toggle_both || toggle_zoom {
                                self.lock_x.zoom = !self.lock_x.zoom;
                                self.lock_y.zoom = !self.lock_y.zoom;
                                if self.show_y2_axis {
                                    self.lock_y2.zoom = !self.lock_y2.zoom;
                                }
                                if self.show_y3_axis {
                                    self.lock_y3.zoom = !self.lock_y3.zoom;
                                }
                                if self.show_y4_axis {
                                    self.lock_y4.zoom = !self.lock_y4.zoom;
                                }
                            }
                        }
                    }

                    self.hover = None;
                    self.cursor_px = None;
                    self.pan_button = None;
                    self.pan_target = None;
                    self.pan_start_pos = None;
                    self.pan_last_pos = None;
                    self.box_zoom_start = None;
                    self.box_zoom_current = None;
                    self.box_zoom_button = None;
                    self.box_zoom_required_mods = None;
                    self.query_drag_button = None;
                    self.query_drag_start = None;
                    self.query_drag_current = None;

                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                } else if plain && *key == KeyCode::KeyR {
                    let _ = self.update_plot_state(cx.app, |s| {
                        s.view_is_auto = true;
                        s.view_bounds = None;
                        s.view_y2_is_auto = true;
                        s.view_bounds_y2 = None;
                        s.view_y3_is_auto = true;
                        s.view_bounds_y3 = None;
                        s.view_y4_is_auto = true;
                        s.view_bounds_y4 = None;
                        s.linked_cursor_x = None;
                        s.hidden_series.clear();
                        s.pinned_series = None;
                        s.query = None;
                    });
                    self.hover = None;
                    self.cursor_px = None;
                    self.pan_button = None;
                    self.pan_target = None;
                    self.pan_start_pos = None;
                    self.pan_last_pos = None;
                    self.box_zoom_start = None;
                    self.box_zoom_current = None;
                    self.box_zoom_button = None;
                    self.box_zoom_required_mods = None;
                    self.query_drag_button = None;
                    self.query_drag_start = None;
                    self.query_drag_current = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                } else if plain && *key == KeyCode::KeyA {
                    let _ = self.update_plot_state(cx.app, |s| {
                        s.hidden_series.clear();
                        s.pinned_series = None;
                    });
                    self.hover = None;
                    self.cursor_px = None;
                    self.legend_hover = None;
                    self.pan_button = None;
                    self.pan_target = None;
                    self.pan_start_pos = None;
                    self.pan_last_pos = None;
                    self.box_zoom_start = None;
                    self.box_zoom_current = None;
                    self.box_zoom_button = None;
                    self.box_zoom_required_mods = None;
                    self.query_drag_button = None;
                    self.query_drag_start = None;
                    self.query_drag_current = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                } else if plain && *key == KeyCode::KeyQ {
                    let query = self.read_plot_state(cx.app).query;
                    if query.is_some() {
                        let _ = self.update_plot_state(cx.app, |s| {
                            s.query = None;
                        });
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                } else if *key == KeyCode::Escape {
                    let state = self.read_plot_state(cx.app);
                    let has_active_drag = self.box_zoom_start.is_some()
                        || self.pan_button.is_some()
                        || self.query_drag_start.is_some();

                    if has_active_drag {
                        self.pan_button = None;
                        self.pan_target = None;
                        self.pan_start_pos = None;
                        self.pan_last_pos = None;
                        self.box_zoom_start = None;
                        self.box_zoom_current = None;
                        self.box_zoom_button = None;
                        self.box_zoom_required_mods = None;
                        self.query_drag_button = None;
                        self.query_drag_start = None;
                        self.query_drag_current = None;
                        self.hover = None;
                        if cx.captured == Some(cx.node) {
                            cx.release_pointer_capture();
                        }
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    } else if state.pinned_series.is_some() {
                        let _ = self.update_plot_state(cx.app, |s| {
                            s.pinned_series = None;
                        });
                        self.legend_hover = None;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    } else if state.query.is_some() {
                        let _ = self.update_plot_state(cx.app, |s| {
                            s.query = None;
                        });
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                }
            }
            Event::Pointer(PointerEvent::Down {
                position,
                button,
                modifiers,
                ..
            }) => {
                self.last_pointer_pos = Some(*position);
                let (
                    y_axis_gap,
                    y_axis_right_gap,
                    y_axis_right2_gap,
                    y_axis_right3_gap,
                    x_axis_gap,
                ) = self.axis_gaps();
                let layout = PlotLayout::from_bounds(
                    cx.bounds,
                    self.style.padding,
                    y_axis_gap,
                    y_axis_right_gap,
                    y_axis_right2_gap,
                    y_axis_right3_gap,
                    x_axis_gap,
                );
                if layout.plot.size.width.0 <= 0.0 || layout.plot.size.height.0 <= 0.0 {
                    return;
                }

                // Axis lock UI: Ctrl+Click on an axis region toggles pan+zoom lock.
                if modifiers.ctrl
                    && *button == MouseButton::Left
                    && let Some(region) = layout.hit_test_region(*position)
                    && region != PlotRegion::Plot
                {
                    match region {
                        PlotRegion::XAxis => {
                            self.lock_x.pan = !self.lock_x.pan;
                            self.lock_x.zoom = !self.lock_x.zoom;
                        }
                        PlotRegion::YAxis(axis) => {
                            let lock = match axis {
                                YAxis::Left => &mut self.lock_y,
                                YAxis::Right => &mut self.lock_y2,
                                YAxis::Right2 => &mut self.lock_y3,
                                YAxis::Right3 => &mut self.lock_y4,
                            };
                            lock.pan = !lock.pan;
                            lock.zoom = !lock.zoom;
                        }
                        PlotRegion::Plot => {}
                    }

                    self.hover = None;
                    self.cursor_px = None;
                    self.legend_hover = None;
                    self.pan_button = None;
                    self.pan_target = None;
                    self.pan_start_pos = None;
                    self.pan_last_pos = None;
                    self.box_zoom_start = None;
                    self.box_zoom_current = None;
                    self.box_zoom_button = None;
                    self.box_zoom_required_mods = None;
                    self.query_drag_button = None;
                    self.query_drag_start = None;
                    self.query_drag_current = None;

                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if *button == MouseButton::Left
                    && let Some((_legend, rows)) = self.legend_layout(layout)
                    && let Some(series_index) = rows
                        .iter()
                        .enumerate()
                        .find(|(_i, r)| contains_point(**r, *position))
                        .map(|(i, _r)| i)
                {
                    let row = rows[series_index];
                    let swatch = Self::legend_swatch_column(row);
                    let Some(entry) = self.legend_entries.get(series_index).cloned() else {
                        return;
                    };
                    let id = entry.id;
                    let state = self.read_plot_state(cx.app);
                    let mut next_hidden = state.hidden_series;
                    let mut next_pinned = state.pinned_series;

                    // Legend interaction policy:
                    // - Shift+Click: solo the series (or restore all if already solo)
                    // - Click swatch column: toggle visibility
                    // - Click label area: pin/unpin tooltip + emphasis to this series
                    if modifiers.shift {
                        let ids: Vec<SeriesId> = self.legend_entries.iter().map(|e| e.id).collect();
                        let visible_count =
                            ids.iter().filter(|sid| !next_hidden.contains(sid)).count();
                        let is_solo = visible_count == 1 && !next_hidden.contains(&id);
                        if is_solo {
                            next_hidden.clear();
                        } else {
                            next_hidden = ids.into_iter().filter(|sid| *sid != id).collect();
                        }
                        next_hidden.remove(&id);
                    } else if contains_point(swatch, *position) {
                        let total = self.legend_entries.len();
                        let hidden_count = self
                            .legend_entries
                            .iter()
                            .filter(|e| next_hidden.contains(&e.id))
                            .count();
                        let visible_count = total.saturating_sub(hidden_count);

                        let is_hidden = next_hidden.contains(&id);
                        if !is_hidden && visible_count <= 1 {
                            // Never hide the last visible series.
                        } else if is_hidden {
                            next_hidden.remove(&id);
                        } else {
                            next_hidden.insert(id);
                        }
                    } else if next_pinned == Some(id) {
                        next_pinned = None;
                    } else {
                        next_pinned = Some(id);
                        next_hidden.remove(&id);
                    }

                    let _ = self.update_plot_state(cx.app, |s| {
                        s.hidden_series = next_hidden;
                        s.pinned_series = next_pinned;
                    });

                    self.hover = None;
                    self.cursor_px = None;
                    self.legend_hover = Some(id);
                    self.pan_button = None;
                    self.pan_start_pos = None;
                    self.pan_last_pos = None;
                    self.box_zoom_start = None;
                    self.box_zoom_current = None;
                    self.box_zoom_button = None;
                    self.box_zoom_required_mods = None;
                    self.query_drag_button = None;
                    self.query_drag_start = None;
                    self.query_drag_current = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                let region = layout.hit_test_region(*position);
                if region.is_none() {
                    return;
                }

                // ImPlot-compatible box zoom cancel: a distinct button press cancels an active box
                // selection (default: LMB cancels RMB selection).
                if region == Some(PlotRegion::Plot)
                    && self.box_zoom_start.is_some()
                    && let Some(cancel) = self.input_map.box_zoom_cancel
                    && cancel.matches(*button, *modifiers)
                    && self
                        .box_zoom_button
                        .is_some_and(|active| active != cancel.button)
                {
                    self.pan_button = None;
                    self.pan_target = None;
                    self.pan_start_pos = None;
                    self.pan_last_pos = None;
                    self.box_zoom_start = None;
                    self.box_zoom_current = None;
                    self.box_zoom_button = None;
                    self.box_zoom_required_mods = None;
                    self.query_drag_start = None;
                    self.query_drag_current = None;
                    self.query_drag_button = None;
                    self.hover = None;
                    self.cursor_px = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                let start_query = self
                    .input_map
                    .query_drag
                    .is_some_and(|ch| ch.matches(*button, *modifiers));
                let start_box_primary = self.input_map.box_zoom.matches(*button, *modifiers);
                let start_box_alt = self
                    .input_map
                    .box_zoom_alt
                    .is_some_and(|ch| ch.matches(*button, *modifiers));
                let start_pan = self.input_map.pan.matches(*button, *modifiers);

                if !start_query && !start_box_primary && !start_box_alt && !start_pan {
                    return;
                }

                if start_query || start_box_primary || start_box_alt {
                    if region != Some(PlotRegion::Plot) {
                        return;
                    }
                } else if start_pan && region.is_none() {
                    return;
                }

                self.cursor_px = (region == Some(PlotRegion::Plot))
                    .then(|| local_from_absolute(layout.plot.origin, *position));
                self.hover = None;

                if start_query {
                    let local = local_from_absolute(layout.plot.origin, *position);
                    self.query_drag_start = Some(local);
                    self.query_drag_current = Some(local);
                    self.query_drag_button = Some(*button);
                    self.pan_button = None;
                    self.pan_target = None;
                    self.pan_start_pos = None;
                    self.pan_last_pos = None;
                    self.box_zoom_start = None;
                    self.box_zoom_current = None;
                    self.box_zoom_button = None;
                    self.box_zoom_required_mods = None;
                } else if start_box_primary || start_box_alt {
                    let local = local_from_absolute(layout.plot.origin, *position);
                    self.box_zoom_start = Some(local);
                    self.box_zoom_current = Some(local);
                    self.box_zoom_button = Some(*button);
                    self.box_zoom_required_mods = Some(if start_box_primary {
                        self.input_map.box_zoom.modifiers
                    } else {
                        self.input_map
                            .box_zoom_alt
                            .unwrap_or(self.input_map.box_zoom)
                            .modifiers
                    });
                    self.pan_button = None;
                    self.pan_target = None;
                    self.pan_start_pos = None;
                    self.pan_last_pos = None;
                    self.query_drag_start = None;
                    self.query_drag_current = None;
                    self.query_drag_button = None;
                } else {
                    self.pan_button = Some(*button);
                    self.pan_target = region;
                    self.pan_start_pos = Some(*position);
                    self.pan_last_pos = None;
                    self.box_zoom_start = None;
                    self.box_zoom_current = None;
                    self.box_zoom_button = None;
                    self.box_zoom_required_mods = None;
                    self.query_drag_start = None;
                    self.query_drag_current = None;
                    self.query_drag_button = None;
                }

                cx.request_focus(cx.node);
                cx.capture_pointer(cx.node);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            Event::Pointer(PointerEvent::Up {
                position,
                button,
                modifiers,
                click_count,
            }) => {
                self.last_pointer_pos = Some(*position);
                if self.pan_button == Some(*button)
                    || self.box_zoom_button == Some(*button)
                    || self.query_drag_button == Some(*button)
                    || self.input_map.fit.matches(*button, *modifiers)
                {
                    if self.input_map.fit.matches(*button, *modifiers)
                        && *click_count == 2
                        && self.pan_last_pos.is_none()
                        && self.box_zoom_start.is_none()
                        && self.query_drag_start.is_none()
                    {
                        let (
                            y_axis_gap,
                            y_axis_right_gap,
                            y_axis_right2_gap,
                            y_axis_right3_gap,
                            x_axis_gap,
                        ) = self.axis_gaps();
                        let layout = PlotLayout::from_bounds(
                            cx.bounds,
                            self.style.padding,
                            y_axis_gap,
                            y_axis_right_gap,
                            y_axis_right2_gap,
                            y_axis_right3_gap,
                            x_axis_gap,
                        );
                        if layout.plot.size.width.0 > 0.0 && layout.plot.size.height.0 > 0.0 {
                            let region = layout.hit_test_region(*position);
                            if let Some(region) = region {
                                let (fit, fit_y2, fit_y3, fit_y4) =
                                    self.fit_view_to_data_now(cx.app);
                                let show_y2_axis = self.show_y2_axis;
                                let show_y3_axis = self.show_y3_axis;
                                let show_y4_axis = self.show_y4_axis;

                                let state = self.read_plot_state(cx.app);
                                let current = self.current_view_bounds(cx.app, &state);
                                let current_y2 =
                                    self.current_view_bounds_y2(cx.app, &state, current);
                                let current_y3 =
                                    self.current_view_bounds_y3(cx.app, &state, current);
                                let current_y4 =
                                    self.current_view_bounds_y4(cx.app, &state, current);

                                let lock_x_zoom = self.lock_x.zoom;
                                let lock_y1_zoom = self.lock_y.zoom;
                                let lock_y2_zoom = self.lock_y2.zoom;
                                let lock_y3_zoom = self.lock_y3.zoom;
                                let lock_y4_zoom = self.lock_y4.zoom;

                                let x_scale = self.x_scale;
                                let y_scale = self.y_scale;
                                let y2_scale = self.y2_scale;
                                let y3_scale = self.y3_scale;
                                let y4_scale = self.y4_scale;

                                let x_constraints = self.x_constraints;
                                let y_constraints = self.y_constraints;
                                let y2_constraints = self.y2_constraints;
                                let y3_constraints = self.y3_constraints;
                                let y4_constraints = self.y4_constraints;

                                let _ = self.update_plot_state(cx.app, |s| match region {
                                    PlotRegion::Plot => {
                                        s.view_is_auto = false;
                                        s.view_bounds = Some(fit);
                                        if show_y2_axis {
                                            s.view_y2_is_auto = false;
                                            s.view_bounds_y2 = fit_y2;
                                        }
                                        if show_y3_axis {
                                            s.view_y3_is_auto = false;
                                            s.view_bounds_y3 = fit_y3;
                                        }
                                        if show_y4_axis {
                                            s.view_y4_is_auto = false;
                                            s.view_bounds_y4 = fit_y4;
                                        }
                                    }
                                    PlotRegion::XAxis => {
                                        if lock_x_zoom {
                                            return;
                                        }

                                        let mut next = current;
                                        next.x_min = fit.x_min;
                                        next.x_max = fit.x_max;
                                        next = constrain_view_bounds_scaled(
                                            next,
                                            x_scale,
                                            y_scale,
                                            x_constraints,
                                            y_constraints,
                                        );

                                        let next_y2 = current_y2.map(|mut vb| {
                                            vb.x_min = fit.x_min;
                                            vb.x_max = fit.x_max;
                                            constrain_view_bounds_scaled(
                                                vb,
                                                x_scale,
                                                y2_scale,
                                                x_constraints,
                                                y2_constraints,
                                            )
                                        });
                                        let next_y3 = current_y3.map(|mut vb| {
                                            vb.x_min = fit.x_min;
                                            vb.x_max = fit.x_max;
                                            constrain_view_bounds_scaled(
                                                vb,
                                                x_scale,
                                                y3_scale,
                                                x_constraints,
                                                y3_constraints,
                                            )
                                        });
                                        let next_y4 = current_y4.map(|mut vb| {
                                            vb.x_min = fit.x_min;
                                            vb.x_max = fit.x_max;
                                            constrain_view_bounds_scaled(
                                                vb,
                                                x_scale,
                                                y4_scale,
                                                x_constraints,
                                                y4_constraints,
                                            )
                                        });

                                        s.view_is_auto = false;
                                        s.view_bounds = Some(next);
                                        if show_y2_axis {
                                            s.view_y2_is_auto = false;
                                            s.view_bounds_y2 = next_y2;
                                        }
                                        if show_y3_axis {
                                            s.view_y3_is_auto = false;
                                            s.view_bounds_y3 = next_y3;
                                        }
                                        if show_y4_axis {
                                            s.view_y4_is_auto = false;
                                            s.view_bounds_y4 = next_y4;
                                        }
                                    }
                                    PlotRegion::YAxis(axis) => match axis {
                                        YAxis::Left => {
                                            if lock_y1_zoom {
                                                return;
                                            }

                                            let mut next = current;
                                            next.y_min = fit.y_min;
                                            next.y_max = fit.y_max;
                                            next = constrain_view_bounds_scaled(
                                                next,
                                                x_scale,
                                                y_scale,
                                                x_constraints,
                                                y_constraints,
                                            );
                                            s.view_is_auto = false;
                                            s.view_bounds = Some(next);
                                        }
                                        YAxis::Right => {
                                            if lock_y2_zoom {
                                                return;
                                            }
                                            let Some(fit_axis) = fit_y2 else {
                                                return;
                                            };
                                            let Some(mut next) = current_y2 else {
                                                return;
                                            };
                                            next.y_min = fit_axis.y_min;
                                            next.y_max = fit_axis.y_max;
                                            next = constrain_view_bounds_scaled(
                                                next,
                                                x_scale,
                                                y2_scale,
                                                x_constraints,
                                                y2_constraints,
                                            );
                                            s.view_y2_is_auto = false;
                                            s.view_bounds_y2 = Some(next);
                                        }
                                        YAxis::Right2 => {
                                            if lock_y3_zoom {
                                                return;
                                            }
                                            let Some(fit_axis) = fit_y3 else {
                                                return;
                                            };
                                            let Some(mut next) = current_y3 else {
                                                return;
                                            };
                                            next.y_min = fit_axis.y_min;
                                            next.y_max = fit_axis.y_max;
                                            next = constrain_view_bounds_scaled(
                                                next,
                                                x_scale,
                                                y3_scale,
                                                x_constraints,
                                                y3_constraints,
                                            );
                                            s.view_y3_is_auto = false;
                                            s.view_bounds_y3 = Some(next);
                                        }
                                        YAxis::Right3 => {
                                            if lock_y4_zoom {
                                                return;
                                            }
                                            let Some(fit_axis) = fit_y4 else {
                                                return;
                                            };
                                            let Some(mut next) = current_y4 else {
                                                return;
                                            };
                                            next.y_min = fit_axis.y_min;
                                            next.y_max = fit_axis.y_max;
                                            next = constrain_view_bounds_scaled(
                                                next,
                                                x_scale,
                                                y4_scale,
                                                x_constraints,
                                                y4_constraints,
                                            );
                                            s.view_y4_is_auto = false;
                                            s.view_bounds_y4 = Some(next);
                                        }
                                    },
                                });

                                self.hover = None;
                                self.cursor_px = None;
                                self.legend_hover = None;
                                self.pan_button = None;
                                self.pan_target = None;
                                self.pan_start_pos = None;
                                self.pan_last_pos = None;
                                self.box_zoom_start = None;
                                self.box_zoom_current = None;
                                self.box_zoom_button = None;
                                self.box_zoom_required_mods = None;
                                self.query_drag_button = None;
                                self.query_drag_start = None;
                                self.query_drag_current = None;
                                if cx.captured == Some(cx.node) {
                                    cx.release_pointer_capture();
                                }
                                cx.invalidate_self(Invalidation::Paint);
                                cx.request_redraw();
                                cx.stop_propagation();
                                return;
                            }
                        }
                    }

                    if self.query_drag_start.is_some() && self.query_drag_button == Some(*button) {
                        if cx.captured == Some(cx.node) {
                            cx.release_pointer_capture();
                        }

                        let (
                            y_axis_gap,
                            y_axis_right_gap,
                            y_axis_right2_gap,
                            y_axis_right3_gap,
                            x_axis_gap,
                        ) = self.axis_gaps();
                        let layout = PlotLayout::from_bounds(
                            cx.bounds,
                            self.style.padding,
                            y_axis_gap,
                            y_axis_right_gap,
                            y_axis_right2_gap,
                            y_axis_right3_gap,
                            x_axis_gap,
                        );
                        if layout.plot.size.width.0 > 0.0 && layout.plot.size.height.0 > 0.0 {
                            let start = self
                                .query_drag_start
                                .unwrap_or(Point::new(Px(0.0), Px(0.0)));
                            let end = self
                                .query_drag_current
                                .unwrap_or(Point::new(Px(0.0), Px(0.0)));

                            let w = (start.x.0 - end.x.0).abs();
                            let h = (start.y.0 - end.y.0).abs();

                            if w >= 4.0 && h >= 4.0 {
                                let state = self.read_plot_state(cx.app);
                                let view_bounds = self.current_view_bounds(cx.app, &state);
                                if let Some(next) = query_rect_from_plot_points_raw(
                                    view_bounds,
                                    layout.plot.size,
                                    start,
                                    end,
                                    self.x_scale,
                                    self.y_scale,
                                ) {
                                    let _ = self.update_plot_state(cx.app, |s| {
                                        s.query = Some(next);
                                    });
                                }
                            }
                        }

                        self.query_drag_button = None;
                        self.query_drag_start = None;
                        self.query_drag_current = None;
                        self.pan_last_pos = None;
                        self.hover = None;

                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    } else if self.box_zoom_start.is_some() && self.box_zoom_button == Some(*button)
                    {
                        if cx.captured == Some(cx.node) {
                            cx.release_pointer_capture();
                        }

                        let (
                            y_axis_gap,
                            y_axis_right_gap,
                            y_axis_right2_gap,
                            y_axis_right3_gap,
                            x_axis_gap,
                        ) = self.axis_gaps();
                        let layout = PlotLayout::from_bounds(
                            cx.bounds,
                            self.style.padding,
                            y_axis_gap,
                            y_axis_right_gap,
                            y_axis_right2_gap,
                            y_axis_right3_gap,
                            x_axis_gap,
                        );
                        if layout.plot.size.width.0 > 0.0 && layout.plot.size.height.0 > 0.0 {
                            let start = self.box_zoom_start.unwrap_or(Point::new(Px(0.0), Px(0.0)));
                            let end = self
                                .box_zoom_current
                                .unwrap_or(Point::new(Px(0.0), Px(0.0)));
                            let required =
                                self.box_zoom_required_mods.unwrap_or(ModifiersMask::NONE);
                            let (start, end) = Self::apply_box_select_modifiers(
                                layout.plot.size,
                                start,
                                end,
                                *modifiers,
                                self.input_map.box_zoom_expand_x,
                                self.input_map.box_zoom_expand_y,
                                required,
                            );

                            let w = (start.x.0 - end.x.0).abs();
                            let h = (start.y.0 - end.y.0).abs();

                            if w >= 4.0 && h >= 4.0 {
                                let all_locked = self.lock_x.zoom
                                    && self.lock_y.zoom
                                    && (!self.show_y2_axis || self.lock_y2.zoom)
                                    && (!self.show_y3_axis || self.lock_y3.zoom)
                                    && (!self.show_y4_axis || self.lock_y4.zoom);
                                if all_locked {
                                    // Axis locks prevent any view change; keep auto-fit state intact.
                                    // The selection rectangle is still useful feedback for users.
                                } else {
                                    let state = self.read_plot_state(cx.app);
                                    let view_bounds = self.current_view_bounds(cx.app, &state);
                                    let view_bounds_y2 =
                                        self.current_view_bounds_y2(cx.app, &state, view_bounds);
                                    let view_bounds_y3 =
                                        self.current_view_bounds_y3(cx.app, &state, view_bounds);
                                    let view_bounds_y4 =
                                        self.current_view_bounds_y4(cx.app, &state, view_bounds);
                                    if let Some(mut next) = data_rect_from_plot_points_scaled(
                                        view_bounds,
                                        layout.plot.size,
                                        start,
                                        end,
                                        self.x_scale,
                                        self.y_scale,
                                    ) {
                                        let mut next_y2 = (!self.lock_y2.zoom)
                                            .then(|| {
                                                view_bounds_y2.and_then(|vb| {
                                                    data_rect_from_plot_points_scaled(
                                                        vb,
                                                        layout.plot.size,
                                                        start,
                                                        end,
                                                        self.x_scale,
                                                        self.y2_scale,
                                                    )
                                                })
                                            })
                                            .flatten();
                                        let mut next_y3 = (!self.lock_y3.zoom)
                                            .then(|| {
                                                view_bounds_y3.and_then(|vb| {
                                                    data_rect_from_plot_points_scaled(
                                                        vb,
                                                        layout.plot.size,
                                                        start,
                                                        end,
                                                        self.x_scale,
                                                        self.y3_scale,
                                                    )
                                                })
                                            })
                                            .flatten();
                                        let mut next_y4 = (!self.lock_y4.zoom)
                                            .then(|| {
                                                view_bounds_y4.and_then(|vb| {
                                                    data_rect_from_plot_points_scaled(
                                                        vb,
                                                        layout.plot.size,
                                                        start,
                                                        end,
                                                        self.x_scale,
                                                        self.y4_scale,
                                                    )
                                                })
                                            })
                                            .flatten();
                                        let data_bounds = self.read_data_bounds(cx.app);
                                        if self.style.clamp_to_data_bounds {
                                            next = clamp_view_to_data_scaled(
                                                next,
                                                data_bounds,
                                                self.style.overscroll_fraction,
                                                self.x_scale,
                                                self.y_scale,
                                            );
                                            if let (Some(candidate), Some(bounds_y2)) =
                                                (next_y2.as_mut(), self.read_data_bounds_y2(cx.app))
                                            {
                                                *candidate = clamp_view_to_data_scaled(
                                                    *candidate,
                                                    bounds_y2,
                                                    self.style.overscroll_fraction,
                                                    self.x_scale,
                                                    self.y2_scale,
                                                );
                                            }
                                            if let (Some(candidate), Some(bounds_y3)) =
                                                (next_y3.as_mut(), self.read_data_bounds_y3(cx.app))
                                            {
                                                *candidate = clamp_view_to_data_scaled(
                                                    *candidate,
                                                    bounds_y3,
                                                    self.style.overscroll_fraction,
                                                    self.x_scale,
                                                    self.y3_scale,
                                                );
                                            }
                                            if let (Some(candidate), Some(bounds_y4)) =
                                                (next_y4.as_mut(), self.read_data_bounds_y4(cx.app))
                                            {
                                                *candidate = clamp_view_to_data_scaled(
                                                    *candidate,
                                                    bounds_y4,
                                                    self.style.overscroll_fraction,
                                                    self.x_scale,
                                                    self.y4_scale,
                                                );
                                            }
                                        }

                                        next = apply_axis_locks(
                                            view_bounds,
                                            next,
                                            self.lock_x.zoom,
                                            self.lock_y.zoom,
                                        );
                                        if let Some(vb_y2) = view_bounds_y2 {
                                            if let Some(candidate) = next_y2.as_mut() {
                                                *candidate = apply_axis_locks(
                                                    vb_y2,
                                                    *candidate,
                                                    self.lock_x.zoom,
                                                    self.lock_y2.zoom,
                                                );
                                            }
                                        }
                                        if let Some(vb_y3) = view_bounds_y3 {
                                            if let Some(candidate) = next_y3.as_mut() {
                                                *candidate = apply_axis_locks(
                                                    vb_y3,
                                                    *candidate,
                                                    self.lock_x.zoom,
                                                    self.lock_y3.zoom,
                                                );
                                            }
                                        }
                                        if let Some(vb_y4) = view_bounds_y4 {
                                            if let Some(candidate) = next_y4.as_mut() {
                                                *candidate = apply_axis_locks(
                                                    vb_y4,
                                                    *candidate,
                                                    self.lock_x.zoom,
                                                    self.lock_y4.zoom,
                                                );
                                            }
                                        }

                                        next = constrain_view_bounds_scaled(
                                            next,
                                            self.x_scale,
                                            self.y_scale,
                                            self.x_constraints,
                                            self.y_constraints,
                                        );
                                        if let Some(candidate) = next_y2.as_mut() {
                                            *candidate = constrain_view_bounds_scaled(
                                                *candidate,
                                                self.x_scale,
                                                self.y2_scale,
                                                self.x_constraints,
                                                self.y2_constraints,
                                            );
                                        }
                                        if let Some(candidate) = next_y3.as_mut() {
                                            *candidate = constrain_view_bounds_scaled(
                                                *candidate,
                                                self.x_scale,
                                                self.y3_scale,
                                                self.x_constraints,
                                                self.y3_constraints,
                                            );
                                        }
                                        if let Some(candidate) = next_y4.as_mut() {
                                            *candidate = constrain_view_bounds_scaled(
                                                *candidate,
                                                self.x_scale,
                                                self.y4_scale,
                                                self.x_constraints,
                                                self.y4_constraints,
                                            );
                                        }

                                        let primary_changed = next != view_bounds;
                                        let y2_changed = next_y2
                                            .zip(view_bounds_y2)
                                            .map(|(next, prev)| next != prev)
                                            .unwrap_or(
                                                next_y2.is_some() && view_bounds_y2.is_none(),
                                            );
                                        let y3_changed = next_y3
                                            .zip(view_bounds_y3)
                                            .map(|(next, prev)| next != prev)
                                            .unwrap_or(
                                                next_y3.is_some() && view_bounds_y3.is_none(),
                                            );
                                        let y4_changed = next_y4
                                            .zip(view_bounds_y4)
                                            .map(|(next, prev)| next != prev)
                                            .unwrap_or(
                                                next_y4.is_some() && view_bounds_y4.is_none(),
                                            );
                                        let show_y2_axis = self.show_y2_axis;
                                        let lock_y2_axis = self.lock_y2.zoom;
                                        let show_y3_axis = self.show_y3_axis;
                                        let lock_y3_axis = self.lock_y3.zoom;
                                        let show_y4_axis = self.show_y4_axis;
                                        let lock_y4_axis = self.lock_y4.zoom;
                                        let _ = self.update_plot_state(cx.app, |s| {
                                            if primary_changed {
                                                s.view_is_auto = false;
                                                s.view_bounds = Some(next);
                                            }
                                            if show_y2_axis
                                                && !lock_y2_axis
                                                && y2_changed
                                                && next_y2.is_some()
                                            {
                                                s.view_y2_is_auto = false;
                                                s.view_bounds_y2 = next_y2;
                                            }
                                            if show_y3_axis
                                                && !lock_y3_axis
                                                && y3_changed
                                                && next_y3.is_some()
                                            {
                                                s.view_y3_is_auto = false;
                                                s.view_bounds_y3 = next_y3;
                                            }
                                            if show_y4_axis
                                                && !lock_y4_axis
                                                && y4_changed
                                                && next_y4.is_some()
                                            {
                                                s.view_y4_is_auto = false;
                                                s.view_bounds_y4 = next_y4;
                                            }
                                        });
                                    }
                                }
                            }
                        }

                        self.box_zoom_start = None;
                        self.box_zoom_current = None;
                        self.box_zoom_button = None;
                        self.box_zoom_required_mods = None;
                        self.pan_last_pos = None;
                        self.hover = None;

                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    } else if self.pan_button == Some(*button)
                        && self.pan_start_pos.take().is_some()
                    {
                        if cx.captured == Some(cx.node) {
                            cx.release_pointer_capture();
                        }
                        self.pan_button = None;
                        self.pan_target = None;
                        self.pan_last_pos = None;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                }
            }
            Event::Pointer(PointerEvent::Wheel {
                position,
                delta,
                modifiers,
            }) => {
                self.last_pointer_pos = Some(*position);
                let (
                    y_axis_gap,
                    y_axis_right_gap,
                    y_axis_right2_gap,
                    y_axis_right3_gap,
                    x_axis_gap,
                ) = self.axis_gaps();
                let layout = PlotLayout::from_bounds(
                    cx.bounds,
                    self.style.padding,
                    y_axis_gap,
                    y_axis_right_gap,
                    y_axis_right2_gap,
                    y_axis_right3_gap,
                    x_axis_gap,
                );
                if layout.plot.size.width.0 <= 0.0 || layout.plot.size.height.0 <= 0.0 {
                    return;
                }

                let Some(region) = layout.hit_test_region(*position) else {
                    return;
                };
                if self.box_zoom_start.is_some() || self.query_drag_start.is_some() {
                    return;
                }

                if let Some(required) = self.input_map.wheel_zoom_mod
                    && !required.is_pressed(*modifiers)
                {
                    return;
                }

                let delta_y = delta.y.0;
                if !delta_y.is_finite() {
                    return;
                }

                let zoom = clamp_zoom_factors(2.0_f32.powf(delta_y * 0.0025));
                let mut zoom_x = zoom;
                let mut zoom_y1 = zoom;
                let mut zoom_y2 = zoom;
                let mut zoom_y3 = zoom;
                let mut zoom_y4 = zoom;

                match region {
                    PlotRegion::Plot => {
                        if modifiers.shift {
                            zoom_y1 = 1.0;
                            zoom_y2 = 1.0;
                            zoom_y3 = 1.0;
                            zoom_y4 = 1.0;
                        } else if modifiers.ctrl {
                            zoom_x = 1.0;
                        }
                    }
                    PlotRegion::XAxis => {
                        zoom_y1 = 1.0;
                        zoom_y2 = 1.0;
                        zoom_y3 = 1.0;
                        zoom_y4 = 1.0;
                    }
                    PlotRegion::YAxis(axis) => {
                        zoom_x = 1.0;
                        zoom_y1 = 1.0;
                        zoom_y2 = 1.0;
                        zoom_y3 = 1.0;
                        zoom_y4 = 1.0;
                        match axis {
                            YAxis::Left => zoom_y1 = zoom,
                            YAxis::Right => zoom_y2 = zoom,
                            YAxis::Right2 => zoom_y3 = zoom,
                            YAxis::Right3 => zoom_y4 = zoom,
                        }
                    }
                }

                if self.lock_x.zoom {
                    zoom_x = 1.0;
                }
                if self.lock_y.zoom {
                    zoom_y1 = 1.0;
                }
                if self.lock_y2.zoom {
                    zoom_y2 = 1.0;
                }
                if self.lock_y3.zoom {
                    zoom_y3 = 1.0;
                }
                if self.lock_y4.zoom {
                    zoom_y4 = 1.0;
                }

                let no_right_zoom = (!self.show_y2_axis || zoom_y2 == 1.0)
                    && (!self.show_y3_axis || zoom_y3 == 1.0)
                    && (!self.show_y4_axis || zoom_y4 == 1.0);
                if zoom_x == 1.0 && zoom_y1 == 1.0 && no_right_zoom {
                    return;
                }

                let state = self.read_plot_state(cx.app);
                let view_bounds = self.current_view_bounds(cx.app, &state);
                let view_bounds_y2 = self.current_view_bounds_y2(cx.app, &state, view_bounds);
                let view_bounds_y3 = self.current_view_bounds_y3(cx.app, &state, view_bounds);
                let view_bounds_y4 = self.current_view_bounds_y4(cx.app, &state, view_bounds);
                let local = local_from_absolute(layout.plot.origin, *position);
                let Some(next) = zoom_view_at_px_scaled(
                    view_bounds,
                    layout.plot.size,
                    local,
                    zoom_x,
                    zoom_y1,
                    self.x_scale,
                    self.y_scale,
                ) else {
                    return;
                };
                let mut next_y2 = (!self.lock_y2.zoom)
                    .then(|| {
                        view_bounds_y2.and_then(|vb| {
                            zoom_view_at_px_scaled(
                                vb,
                                layout.plot.size,
                                local,
                                zoom_x,
                                zoom_y2,
                                self.x_scale,
                                self.y2_scale,
                            )
                        })
                    })
                    .flatten();
                let mut next_y3 = (!self.lock_y3.zoom)
                    .then(|| {
                        view_bounds_y3.and_then(|vb| {
                            zoom_view_at_px_scaled(
                                vb,
                                layout.plot.size,
                                local,
                                zoom_x,
                                zoom_y3,
                                self.x_scale,
                                self.y3_scale,
                            )
                        })
                    })
                    .flatten();
                let mut next_y4 = (!self.lock_y4.zoom)
                    .then(|| {
                        view_bounds_y4.and_then(|vb| {
                            zoom_view_at_px_scaled(
                                vb,
                                layout.plot.size,
                                local,
                                zoom_x,
                                zoom_y4,
                                self.x_scale,
                                self.y4_scale,
                            )
                        })
                    })
                    .flatten();
                let data_bounds = self.read_data_bounds(cx.app);
                let mut next = if self.style.clamp_to_data_bounds {
                    clamp_view_to_data_scaled(
                        next,
                        data_bounds,
                        self.style.overscroll_fraction,
                        self.x_scale,
                        self.y_scale,
                    )
                } else {
                    next
                };
                if self.style.clamp_to_data_bounds {
                    if let (Some(candidate), Some(bounds_y2)) =
                        (next_y2.as_mut(), self.read_data_bounds_y2(cx.app))
                    {
                        *candidate = clamp_view_to_data_scaled(
                            *candidate,
                            bounds_y2,
                            self.style.overscroll_fraction,
                            self.x_scale,
                            self.y2_scale,
                        );
                    }
                    if let (Some(candidate), Some(bounds_y3)) =
                        (next_y3.as_mut(), self.read_data_bounds_y3(cx.app))
                    {
                        *candidate = clamp_view_to_data_scaled(
                            *candidate,
                            bounds_y3,
                            self.style.overscroll_fraction,
                            self.x_scale,
                            self.y3_scale,
                        );
                    }
                    if let (Some(candidate), Some(bounds_y4)) =
                        (next_y4.as_mut(), self.read_data_bounds_y4(cx.app))
                    {
                        *candidate = clamp_view_to_data_scaled(
                            *candidate,
                            bounds_y4,
                            self.style.overscroll_fraction,
                            self.x_scale,
                            self.y4_scale,
                        );
                    }
                }

                next = apply_axis_locks(view_bounds, next, self.lock_x.zoom, self.lock_y.zoom);
                if let (Some(vb_y2), Some(candidate)) = (view_bounds_y2, next_y2.as_mut()) {
                    *candidate =
                        apply_axis_locks(vb_y2, *candidate, self.lock_x.zoom, self.lock_y2.zoom);
                }
                if let (Some(vb_y3), Some(candidate)) = (view_bounds_y3, next_y3.as_mut()) {
                    *candidate =
                        apply_axis_locks(vb_y3, *candidate, self.lock_x.zoom, self.lock_y3.zoom);
                }
                if let (Some(vb_y4), Some(candidate)) = (view_bounds_y4, next_y4.as_mut()) {
                    *candidate =
                        apply_axis_locks(vb_y4, *candidate, self.lock_x.zoom, self.lock_y4.zoom);
                }

                next = constrain_view_bounds_scaled(
                    next,
                    self.x_scale,
                    self.y_scale,
                    self.x_constraints,
                    self.y_constraints,
                );
                if let Some(candidate) = next_y2.as_mut() {
                    *candidate = constrain_view_bounds_scaled(
                        *candidate,
                        self.x_scale,
                        self.y2_scale,
                        self.x_constraints,
                        self.y2_constraints,
                    );
                }
                if let Some(candidate) = next_y3.as_mut() {
                    *candidate = constrain_view_bounds_scaled(
                        *candidate,
                        self.x_scale,
                        self.y3_scale,
                        self.x_constraints,
                        self.y3_constraints,
                    );
                }
                if let Some(candidate) = next_y4.as_mut() {
                    *candidate = constrain_view_bounds_scaled(
                        *candidate,
                        self.x_scale,
                        self.y4_scale,
                        self.x_constraints,
                        self.y4_constraints,
                    );
                }

                let primary_changed = next != view_bounds;
                let y2_changed = next_y2
                    .zip(view_bounds_y2)
                    .map(|(next, prev)| next != prev)
                    .unwrap_or(next_y2.is_some() && view_bounds_y2.is_none());
                let y3_changed = next_y3
                    .zip(view_bounds_y3)
                    .map(|(next, prev)| next != prev)
                    .unwrap_or(next_y3.is_some() && view_bounds_y3.is_none());
                let y4_changed = next_y4
                    .zip(view_bounds_y4)
                    .map(|(next, prev)| next != prev)
                    .unwrap_or(next_y4.is_some() && view_bounds_y4.is_none());
                let show_y2_axis = self.show_y2_axis;
                let lock_y2_axis = self.lock_y2.zoom;
                let show_y3_axis = self.show_y3_axis;
                let lock_y3_axis = self.lock_y3.zoom;
                let show_y4_axis = self.show_y4_axis;
                let lock_y4_axis = self.lock_y4.zoom;
                let _ = self.update_plot_state(cx.app, |s| {
                    if primary_changed {
                        s.view_is_auto = false;
                        s.view_bounds = Some(next);
                    }
                    if show_y2_axis && !lock_y2_axis && y2_changed && next_y2.is_some() {
                        s.view_y2_is_auto = false;
                        s.view_bounds_y2 = next_y2;
                    }
                    if show_y3_axis && !lock_y3_axis && y3_changed && next_y3.is_some() {
                        s.view_y3_is_auto = false;
                        s.view_bounds_y3 = next_y3;
                    }
                    if show_y4_axis && !lock_y4_axis && y4_changed && next_y4.is_some() {
                        s.view_y4_is_auto = false;
                        s.view_bounds_y4 = next_y4;
                    }
                });
                cx.request_focus(cx.node);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            Event::Pointer(PointerEvent::Move { position, .. }) => {
                self.last_pointer_pos = Some(*position);
                let (
                    y_axis_gap,
                    y_axis_right_gap,
                    y_axis_right2_gap,
                    y_axis_right3_gap,
                    x_axis_gap,
                ) = self.axis_gaps();
                let layout = PlotLayout::from_bounds(
                    cx.bounds,
                    self.style.padding,
                    y_axis_gap,
                    y_axis_right_gap,
                    y_axis_right2_gap,
                    y_axis_right3_gap,
                    x_axis_gap,
                );
                if layout.plot.size.width.0 <= 0.0 || layout.plot.size.height.0 <= 0.0 {
                    return;
                }

                let pan_active = self.pan_button.is_some()
                    && self.pan_target.is_some()
                    && (self.pan_start_pos.is_some() || self.pan_last_pos.is_some());
                if self.box_zoom_start.is_none() && !pan_active {
                    if let Some((legend, rows)) = self.legend_layout(layout)
                        && contains_point(legend, *position)
                    {
                        let cursor_changed = self.cursor_px.take().is_some();

                        let series_index = rows
                            .iter()
                            .enumerate()
                            .find(|(_i, r)| contains_point(**r, *position))
                            .map(|(i, _r)| i);

                        let hovered_id =
                            series_index.and_then(|i| self.legend_entries.get(i).map(|e| e.id));

                        if self.legend_hover != hovered_id {
                            self.legend_hover = hovered_id;
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }

                        if self.hover.is_some() {
                            self.hover = None;
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }

                        if cursor_changed {
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }

                        cx.stop_propagation();
                        return;
                    }

                    if self.legend_hover.take().is_some() {
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                }

                if self.query_drag_start.is_some() {
                    let local = local_from_absolute(layout.plot.origin, *position);
                    self.cursor_px = Some(local);
                    self.query_drag_current = Some(local);
                    self.hover = None;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if self.box_zoom_start.is_some() {
                    let local = local_from_absolute(layout.plot.origin, *position);
                    self.cursor_px = Some(local);
                    self.box_zoom_current = Some(local);
                    self.hover = None;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if let Some(start) = self.pan_start_pos
                    && self.pan_button.is_some()
                    && let Some(target) = self.pan_target
                {
                    let last = self.pan_last_pos.unwrap_or(start);
                    self.cursor_px = None;
                    let dx_px_raw = position.x.0 - last.x.0;
                    let dy_px_raw = position.y.0 - last.y.0;

                    if dx_px_raw == 0.0 && dy_px_raw == 0.0 {
                        cx.stop_propagation();
                        return;
                    }

                    let mut dx_px = if self.lock_x.pan { 0.0 } else { dx_px_raw };
                    let mut dy_px_y1 = if self.lock_y.pan { 0.0 } else { dy_px_raw };
                    let mut dy_px_y2 = if self.lock_y2.pan { 0.0 } else { dy_px_raw };
                    let mut dy_px_y3 = if self.lock_y3.pan { 0.0 } else { dy_px_raw };
                    let mut dy_px_y4 = if self.lock_y4.pan { 0.0 } else { dy_px_raw };

                    match target {
                        PlotRegion::Plot => {}
                        PlotRegion::XAxis => {
                            dy_px_y1 = 0.0;
                            dy_px_y2 = 0.0;
                            dy_px_y3 = 0.0;
                            dy_px_y4 = 0.0;
                        }
                        PlotRegion::YAxis(axis) => {
                            dx_px = 0.0;
                            dy_px_y1 = 0.0;
                            dy_px_y2 = 0.0;
                            dy_px_y3 = 0.0;
                            dy_px_y4 = 0.0;
                            match axis {
                                YAxis::Left => {
                                    dy_px_y1 = if self.lock_y.pan { 0.0 } else { dy_px_raw }
                                }
                                YAxis::Right => {
                                    dy_px_y2 = if self.lock_y2.pan { 0.0 } else { dy_px_raw }
                                }
                                YAxis::Right2 => {
                                    dy_px_y3 = if self.lock_y3.pan { 0.0 } else { dy_px_raw }
                                }
                                YAxis::Right3 => {
                                    dy_px_y4 = if self.lock_y4.pan { 0.0 } else { dy_px_raw }
                                }
                            }
                        }
                    }

                    let no_right_pan = (!self.show_y2_axis || dy_px_y2 == 0.0)
                        && (!self.show_y3_axis || dy_px_y3 == 0.0)
                        && (!self.show_y4_axis || dy_px_y4 == 0.0);
                    if dx_px == 0.0 && dy_px_y1 == 0.0 && no_right_pan {
                        self.pan_last_pos = Some(*position);
                        cx.stop_propagation();
                        return;
                    }

                    let state = self.read_plot_state(cx.app);
                    let view_bounds = self.current_view_bounds(cx.app, &state);
                    let view_bounds_y2 = self.current_view_bounds_y2(cx.app, &state, view_bounds);
                    let view_bounds_y3 = self.current_view_bounds_y3(cx.app, &state, view_bounds);
                    let view_bounds_y4 = self.current_view_bounds_y4(cx.app, &state, view_bounds);
                    let Some(next) = pan_view_by_px_scaled(
                        view_bounds,
                        layout.plot.size,
                        dx_px,
                        dy_px_y1,
                        self.x_scale,
                        self.y_scale,
                    ) else {
                        return;
                    };
                    let mut next_y2 = (!self.lock_y2.pan)
                        .then(|| {
                            view_bounds_y2.and_then(|vb| {
                                pan_view_by_px_scaled(
                                    vb,
                                    layout.plot.size,
                                    dx_px,
                                    dy_px_y2,
                                    self.x_scale,
                                    self.y2_scale,
                                )
                            })
                        })
                        .flatten();
                    let mut next_y3 = (!self.lock_y3.pan)
                        .then(|| {
                            view_bounds_y3.and_then(|vb| {
                                pan_view_by_px_scaled(
                                    vb,
                                    layout.plot.size,
                                    dx_px,
                                    dy_px_y3,
                                    self.x_scale,
                                    self.y3_scale,
                                )
                            })
                        })
                        .flatten();
                    let mut next_y4 = (!self.lock_y4.pan)
                        .then(|| {
                            view_bounds_y4.and_then(|vb| {
                                pan_view_by_px_scaled(
                                    vb,
                                    layout.plot.size,
                                    dx_px,
                                    dy_px_y4,
                                    self.x_scale,
                                    self.y4_scale,
                                )
                            })
                        })
                        .flatten();
                    let data_bounds = self.read_data_bounds(cx.app);
                    let mut next = if self.style.clamp_to_data_bounds {
                        clamp_view_to_data_scaled(
                            next,
                            data_bounds,
                            self.style.overscroll_fraction,
                            self.x_scale,
                            self.y_scale,
                        )
                    } else {
                        next
                    };
                    if self.style.clamp_to_data_bounds {
                        if let (Some(candidate), Some(bounds_y2)) =
                            (next_y2.as_mut(), self.read_data_bounds_y2(cx.app))
                        {
                            *candidate = clamp_view_to_data_scaled(
                                *candidate,
                                bounds_y2,
                                self.style.overscroll_fraction,
                                self.x_scale,
                                self.y2_scale,
                            );
                        }
                        if let (Some(candidate), Some(bounds_y3)) =
                            (next_y3.as_mut(), self.read_data_bounds_y3(cx.app))
                        {
                            *candidate = clamp_view_to_data_scaled(
                                *candidate,
                                bounds_y3,
                                self.style.overscroll_fraction,
                                self.x_scale,
                                self.y3_scale,
                            );
                        }
                        if let (Some(candidate), Some(bounds_y4)) =
                            (next_y4.as_mut(), self.read_data_bounds_y4(cx.app))
                        {
                            *candidate = clamp_view_to_data_scaled(
                                *candidate,
                                bounds_y4,
                                self.style.overscroll_fraction,
                                self.x_scale,
                                self.y4_scale,
                            );
                        }
                    }

                    next = apply_axis_locks(view_bounds, next, self.lock_x.pan, self.lock_y.pan);
                    if let (Some(vb_y2), Some(candidate)) = (view_bounds_y2, next_y2.as_mut()) {
                        *candidate =
                            apply_axis_locks(vb_y2, *candidate, self.lock_x.pan, self.lock_y2.pan);
                    }
                    if let (Some(vb_y3), Some(candidate)) = (view_bounds_y3, next_y3.as_mut()) {
                        *candidate =
                            apply_axis_locks(vb_y3, *candidate, self.lock_x.pan, self.lock_y3.pan);
                    }
                    if let (Some(vb_y4), Some(candidate)) = (view_bounds_y4, next_y4.as_mut()) {
                        *candidate =
                            apply_axis_locks(vb_y4, *candidate, self.lock_x.pan, self.lock_y4.pan);
                    }

                    next = constrain_view_bounds_scaled(
                        next,
                        self.x_scale,
                        self.y_scale,
                        self.x_constraints,
                        self.y_constraints,
                    );
                    if let Some(candidate) = next_y2.as_mut() {
                        *candidate = constrain_view_bounds_scaled(
                            *candidate,
                            self.x_scale,
                            self.y2_scale,
                            self.x_constraints,
                            self.y2_constraints,
                        );
                    }
                    if let Some(candidate) = next_y3.as_mut() {
                        *candidate = constrain_view_bounds_scaled(
                            *candidate,
                            self.x_scale,
                            self.y3_scale,
                            self.x_constraints,
                            self.y3_constraints,
                        );
                    }
                    if let Some(candidate) = next_y4.as_mut() {
                        *candidate = constrain_view_bounds_scaled(
                            *candidate,
                            self.x_scale,
                            self.y4_scale,
                            self.x_constraints,
                            self.y4_constraints,
                        );
                    }

                    let primary_changed = next != view_bounds;
                    let y2_changed = next_y2
                        .zip(view_bounds_y2)
                        .map(|(next, prev)| next != prev)
                        .unwrap_or(next_y2.is_some() && view_bounds_y2.is_none());
                    let y3_changed = next_y3
                        .zip(view_bounds_y3)
                        .map(|(next, prev)| next != prev)
                        .unwrap_or(next_y3.is_some() && view_bounds_y3.is_none());
                    let y4_changed = next_y4
                        .zip(view_bounds_y4)
                        .map(|(next, prev)| next != prev)
                        .unwrap_or(next_y4.is_some() && view_bounds_y4.is_none());
                    let show_y2_axis = self.show_y2_axis;
                    let lock_y2_axis = self.lock_y2.pan;
                    let show_y3_axis = self.show_y3_axis;
                    let lock_y3_axis = self.lock_y3.pan;
                    let show_y4_axis = self.show_y4_axis;
                    let lock_y4_axis = self.lock_y4.pan;
                    let _ = self.update_plot_state(cx.app, |s| {
                        if primary_changed {
                            s.view_is_auto = false;
                            s.view_bounds = Some(next);
                        }
                        if show_y2_axis && !lock_y2_axis && y2_changed && next_y2.is_some() {
                            s.view_y2_is_auto = false;
                            s.view_bounds_y2 = next_y2;
                        }
                        if show_y3_axis && !lock_y3_axis && y3_changed && next_y3.is_some() {
                            s.view_y3_is_auto = false;
                            s.view_bounds_y3 = next_y3;
                        }
                        if show_y4_axis && !lock_y4_axis && y4_changed && next_y4.is_some() {
                            s.view_y4_is_auto = false;
                            s.view_bounds_y4 = next_y4;
                        }
                    });
                    self.pan_last_pos = Some(*position);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                let inside = contains_point(layout.plot, *position);

                let prev_cursor = self.cursor_px;
                self.cursor_px = inside.then(|| local_from_absolute(layout.plot.origin, *position));
                if prev_cursor != self.cursor_px {
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }

                let next_hover = if inside {
                    let model_revision = self.model.revision(cx.app).unwrap_or(0);
                    let scale_factor = self.last_scale_factor;

                    let state = self.read_plot_state(cx.app);
                    let view_bounds = self.current_view_bounds(cx.app, &state);
                    let view_bounds_y2 = self.current_view_bounds_y2(cx.app, &state, view_bounds);
                    let view_bounds_y3 = self.current_view_bounds_y3(cx.app, &state, view_bounds);
                    let view_bounds_y4 = self.current_view_bounds_y4(cx.app, &state, view_bounds);
                    let hidden = &state.hidden_series;
                    let pinned = state.pinned_series;

                    let local = local_from_absolute(layout.plot.origin, *position);

                    self.model
                        .read(cx.app, |_app, m| m.clone())
                        .ok()
                        .and_then(|model| {
                            self.layer.hit_test(
                                &model,
                                PlotHitTestArgs {
                                    model_revision,
                                    plot_size: layout.plot.size,
                                    view_bounds,
                                    view_bounds_y2,
                                    view_bounds_y3,
                                    view_bounds_y4,
                                    x_scale: self.x_scale,
                                    y_scale: self.y_scale,
                                    y2_scale: self.y2_scale,
                                    y3_scale: self.y3_scale,
                                    y4_scale: self.y4_scale,
                                    scale_factor,
                                    local,
                                    style: self.style,
                                    hover_threshold: self.style.hover_threshold,
                                    hidden,
                                    pinned,
                                },
                            )
                        })
                } else {
                    None
                };

                if self.hover != next_hover {
                    self.hover = next_hover;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }

                // Publish interaction output eagerly so linked-plot coordinators can react to
                // pointer movement without waiting for the next paint.
                let state = self.read_plot_state(cx.app);
                let view_bounds = self.current_view_bounds(cx.app, &state);
                let view_bounds_y2 = self.current_view_bounds_y2(cx.app, &state, view_bounds);
                let view_bounds_y3 = self.current_view_bounds_y3(cx.app, &state, view_bounds);
                let view_bounds_y4 = self.current_view_bounds_y4(cx.app, &state, view_bounds);
                self.publish_current_output_snapshot(
                    cx.app,
                    layout,
                    &state,
                    view_bounds,
                    view_bounds_y2,
                    view_bounds_y3,
                    view_bounds_y4,
                );
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(&self.model, Invalidation::Paint);
        if let Some(state) = &self.plot_state_model {
            cx.observe_model(state, Invalidation::Paint);
        }
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(&self.model, Invalidation::Paint);
        if let Some(state) = &self.plot_state_model {
            cx.observe_model(state, Invalidation::Paint);
        }
        cx.observe_global::<TextFontStackKey>(Invalidation::Paint);
        self.last_scale_factor = cx.scale_factor;

        self.ensure_required_axes_enabled(cx.app);

        let theme = cx.theme().snapshot();
        let font_stack_key = cx
            .app
            .global::<TextFontStackKey>()
            .map(|k| k.0)
            .unwrap_or(0);
        let background = self
            .style
            .background
            .unwrap_or(theme.colors.panel_background);
        let border = self.style.border.unwrap_or(theme.colors.panel_border);

        let axis_color = self.style.axis_color.unwrap_or(theme.colors.panel_border);
        let grid_color = self.style.grid_color.unwrap_or(Color {
            a: 0.35,
            ..theme.colors.panel_border
        });
        let label_color = self.style.label_color.unwrap_or(theme.colors.text_muted);
        let crosshair_color = self.style.crosshair_color.unwrap_or(Color {
            a: 0.65,
            ..theme.colors.accent
        });
        let selection_border = crosshair_color;
        let selection_fill = Color {
            a: (crosshair_color.a * 0.18).clamp(0.06, 0.22),
            ..crosshair_color
        };
        let tooltip_background = self
            .style
            .tooltip_background
            .unwrap_or(theme.colors.menu_background);
        let tooltip_border = self
            .style
            .tooltip_border
            .unwrap_or(theme.colors.menu_border);
        let tooltip_text_color = self
            .style
            .tooltip_text_color
            .unwrap_or(theme.colors.text_primary);

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background,
            border: fret_core::Edges::all(self.style.border_width),
            border_color: border,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });

        let min_axis = self.style.axis_gap;
        self.x_axis_thickness = Px(self.x_axis_thickness.0.max(min_axis.0));
        self.y_axis_thickness = Px(self.y_axis_thickness.0.max(min_axis.0));
        if self.show_y2_axis {
            self.y_axis_right_thickness = Px(self.y_axis_right_thickness.0.max(min_axis.0));
        }
        if self.show_y3_axis {
            self.y_axis_right2_thickness = Px(self.y_axis_right2_thickness.0.max(min_axis.0));
        }
        if self.show_y4_axis {
            self.y_axis_right3_thickness = Px(self.y_axis_right3_thickness.0.max(min_axis.0));
        }

        // Layout can depend on text metrics (axis thickness). Converge in up to two passes.
        let mut layout = PlotLayout::from_bounds(
            cx.bounds,
            self.style.padding,
            self.y_axis_thickness,
            self.y_axis_right_thickness,
            self.y_axis_right2_thickness,
            self.y_axis_right3_thickness,
            self.x_axis_thickness,
        );
        let state = self.read_plot_state(cx.app);
        let view_bounds = self.current_view_bounds(cx.app, &state);
        let view_bounds_y2 = self.current_view_bounds_y2(cx.app, &state, view_bounds);
        let view_bounds_y3 = self.current_view_bounds_y3(cx.app, &state, view_bounds);
        let view_bounds_y4 = self.current_view_bounds_y4(cx.app, &state, view_bounds);

        // Axis labels can expand axis thickness; keep plot-local interaction points stable by
        // shifting stored coordinates when the plot origin moves.
        for _ in 0..2 {
            let changed = self.rebuild_axis_labels_if_needed(
                cx,
                layout,
                view_bounds,
                view_bounds_y2,
                view_bounds_y3,
                view_bounds_y4,
                theme.revision,
                font_stack_key,
            );
            if !changed {
                break;
            }

            let next_layout = PlotLayout::from_bounds(
                cx.bounds,
                self.style.padding,
                self.y_axis_thickness,
                self.y_axis_right_thickness,
                self.y_axis_right2_thickness,
                self.y_axis_right3_thickness,
                self.x_axis_thickness,
            );

            let dx = next_layout.plot.origin.x.0 - layout.plot.origin.x.0;
            let dy = next_layout.plot.origin.y.0 - layout.plot.origin.y.0;
            if dx != 0.0 || dy != 0.0 {
                let delta = Point::new(Px(dx), Px(dy));

                let shift = |p: &mut Point| {
                    p.x.0 -= delta.x.0;
                    p.y.0 -= delta.y.0;
                };

                if let Some(p) = self.cursor_px.as_mut() {
                    shift(p);
                }
                if let Some(p) = self.box_zoom_start.as_mut() {
                    shift(p);
                }
                if let Some(p) = self.box_zoom_current.as_mut() {
                    shift(p);
                }
                if let Some(p) = self.query_drag_start.as_mut() {
                    shift(p);
                }
                if let Some(p) = self.query_drag_current.as_mut() {
                    shift(p);
                }

                self.hover = None;
            }

            layout = next_layout;
        }

        self.publish_current_output_snapshot(
            cx.app,
            layout,
            &state,
            view_bounds,
            view_bounds_y2,
            view_bounds_y3,
            view_bounds_y4,
        );
        self.rebuild_legend_if_needed(cx, theme.revision, font_stack_key);

        // Grid + series + hover are clipped to the plot area.
        cx.scene.push(SceneOp::PushClipRect { rect: layout.plot });

        if layout.plot.size.width.0 > 0.0 && layout.plot.size.height.0 > 0.0 {
            // Grid: align to axis ticks so labels and grid are consistent (ImPlot-style).
            let x_ticks = &self.axis_ticks_x;
            let y_ticks = &self.axis_ticks_y;

            let transform = PlotTransform {
                viewport: layout.plot,
                data: view_bounds,
                x_scale: self.x_scale,
                y_scale: self.y_scale,
            };

            for v in x_ticks.iter().copied() {
                let Some(x) = transform.data_x_to_px(v) else {
                    continue;
                };
                let x = Px(x.0.round());
                let background =
                    if self.x_scale == AxisScale::Log10 && log10_decade_exponent(v).is_none() {
                        dim_color(grid_color, 0.45)
                    } else {
                        grid_color
                    };
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: Rect::new(
                        Point::new(x, layout.plot.origin.y),
                        Size::new(Px(1.0), layout.plot.size.height),
                    ),
                    background,
                    border: fret_core::Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });
            }

            for v in y_ticks.iter().copied() {
                let Some(y) = transform.data_y_to_px(v) else {
                    continue;
                };
                let y = Px(y.0.round());
                let background =
                    if self.y_scale == AxisScale::Log10 && log10_decade_exponent(v).is_none() {
                        dim_color(grid_color, 0.45)
                    } else {
                        grid_color
                    };
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: Rect::new(
                        Point::new(layout.plot.origin.x, y),
                        Size::new(layout.plot.size.width, Px(1.0)),
                    ),
                    background,
                    border: fret_core::Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });
            }

            let emphasized = self.style.emphasize_hovered_series;
            let dim_alpha = self.style.dimmed_series_alpha;
            let series_meta: Vec<SeriesMeta> = self
                .model
                .read(cx.app, |_app, m| L::series_meta(m))
                .unwrap_or_default();
            let pinned = state
                .pinned_series
                .filter(|id| series_meta.iter().any(|s| s.id == *id));
            let hidden = &state.hidden_series;

            let emphasized_series = pinned
                .or(self.hover.map(|h| h.series_id))
                .or(self.legend_hover);

            for quad in self.rebuild_quads_if_needed(
                cx,
                layout.plot,
                view_bounds,
                view_bounds_y2,
                view_bounds_y3,
                view_bounds_y4,
                hidden,
            ) {
                cx.scene.push(SceneOp::Quad {
                    order: quad.order,
                    rect: offset_rect(quad.rect_local, layout.plot.origin),
                    background: quad.background,
                    border: fret_core::Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });
            }

            for (series_id, path, color) in self.rebuild_paths_if_needed(
                cx,
                layout.plot,
                view_bounds,
                view_bounds_y2,
                view_bounds_y3,
                view_bounds_y4,
                hidden,
            ) {
                let color = if emphasized {
                    if let Some(emphasized) = emphasized_series
                        && emphasized != series_id
                    {
                        dim_color(color, dim_alpha)
                    } else {
                        color
                    }
                } else {
                    color
                };
                cx.scene.push(SceneOp::Path {
                    order: DrawOrder(2),
                    origin: layout.plot.origin,
                    path,
                    color,
                });
            }

            if let Some(cursor) = self.cursor_px {
                let x = Px((layout.plot.origin.x.0 + cursor.x.0).round());
                let y = Px((layout.plot.origin.y.0 + cursor.y.0).round());
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(3),
                    rect: Rect::new(
                        Point::new(x, layout.plot.origin.y),
                        Size::new(Px(1.0), layout.plot.size.height),
                    ),
                    background: crosshair_color,
                    border: fret_core::Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });

                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(3),
                    rect: Rect::new(
                        Point::new(layout.plot.origin.x, y),
                        Size::new(layout.plot.size.width, Px(1.0)),
                    ),
                    background: crosshair_color,
                    border: fret_core::Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });
            }
            // Linked cursor (typically driven by `LinkedPlotGroup`).
            if self.cursor_px.is_none()
                && let Some(linked_x) = state.linked_cursor_x
                && linked_x.is_finite()
            {
                let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size);
                let transform = PlotTransform {
                    viewport: local_viewport,
                    data: view_bounds,
                    x_scale: self.x_scale,
                    y_scale: self.y_scale,
                };
                if let Some(px) = transform.data_x_to_px(linked_x) {
                    let x0 = px.0.clamp(0.0, layout.plot.size.width.0);
                    let x = Px((layout.plot.origin.x.0 + x0).round());
                    let linked_color = Color {
                        a: (crosshair_color.a * 0.55).clamp(0.05, 1.0),
                        ..crosshair_color
                    };
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(3),
                        rect: Rect::new(
                            Point::new(x, layout.plot.origin.y),
                            Size::new(Px(1.0), layout.plot.size.height),
                        ),
                        background: linked_color,
                        border: fret_core::Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: fret_core::Corners::all(Px(0.0)),
                    });
                }
            }

            if let Some(hover) = self.hover {
                let hx = Px((layout.plot.origin.x.0 + hover.plot_px.x.0).round());
                let hy = Px((layout.plot.origin.y.0 + hover.plot_px.y.0).round());

                let dot_size = Px(6.0);
                let dot_origin =
                    Point::new(Px(hx.0 - dot_size.0 * 0.5), Px(hy.0 - dot_size.0 * 0.5));
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(4),
                    rect: Rect::new(dot_origin, Size::new(dot_size, dot_size)),
                    background: crosshair_color,
                    border: fret_core::Edges::all(Px(1.0)),
                    border_color: tooltip_border,
                    corner_radii: fret_core::Corners::all(Px(dot_size.0 * 0.5)),
                });
            }

            if let Some(query) = state.query {
                let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size);
                let transform = PlotTransform {
                    viewport: local_viewport,
                    data: view_bounds,
                    x_scale: self.x_scale,
                    y_scale: self.y_scale,
                };

                let a = transform.data_to_px(DataPoint {
                    x: query.x_min,
                    y: query.y_min,
                });
                let b = transform.data_to_px(DataPoint {
                    x: query.x_max,
                    y: query.y_max,
                });

                let x0 = a.x.0.min(b.x.0).clamp(0.0, layout.plot.size.width.0);
                let x1 = a.x.0.max(b.x.0).clamp(0.0, layout.plot.size.width.0);
                let y0 = a.y.0.min(b.y.0).clamp(0.0, layout.plot.size.height.0);
                let y1 = a.y.0.max(b.y.0).clamp(0.0, layout.plot.size.height.0);
                let w = x1 - x0;
                let h = y1 - y0;
                if w >= 1.0 && h >= 1.0 {
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(5),
                        rect: Rect::new(
                            Point::new(
                                Px(layout.plot.origin.x.0 + x0),
                                Px(layout.plot.origin.y.0 + y0),
                            ),
                            Size::new(Px(w), Px(h)),
                        ),
                        background: selection_fill,
                        border: fret_core::Edges::all(Px(1.0)),
                        border_color: selection_border,
                        corner_radii: fret_core::Corners::all(Px(0.0)),
                    });
                }
            }

            if let (Some(start), Some(end)) = (self.query_drag_start, self.query_drag_current) {
                let x0 = start.x.0.min(end.x.0).clamp(0.0, layout.plot.size.width.0);
                let x1 = start.x.0.max(end.x.0).clamp(0.0, layout.plot.size.width.0);
                let y0 = start.y.0.min(end.y.0).clamp(0.0, layout.plot.size.height.0);
                let y1 = start.y.0.max(end.y.0).clamp(0.0, layout.plot.size.height.0);
                let w = x1 - x0;
                let h = y1 - y0;
                if w >= 1.0 && h >= 1.0 {
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(5),
                        rect: Rect::new(
                            Point::new(
                                Px(layout.plot.origin.x.0 + x0),
                                Px(layout.plot.origin.y.0 + y0),
                            ),
                            Size::new(Px(w), Px(h)),
                        ),
                        background: selection_fill,
                        border: fret_core::Edges::all(Px(1.0)),
                        border_color: selection_border,
                        corner_radii: fret_core::Corners::all(Px(0.0)),
                    });
                }
            }

            if let (Some(start), Some(end)) = (self.box_zoom_start, self.box_zoom_current) {
                let x0 = start.x.0.min(end.x.0).clamp(0.0, layout.plot.size.width.0);
                let x1 = start.x.0.max(end.x.0).clamp(0.0, layout.plot.size.width.0);
                let y0 = start.y.0.min(end.y.0).clamp(0.0, layout.plot.size.height.0);
                let y1 = start.y.0.max(end.y.0).clamp(0.0, layout.plot.size.height.0);
                let w = x1 - x0;
                let h = y1 - y0;
                if w >= 1.0 && h >= 1.0 {
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(5),
                        rect: Rect::new(
                            Point::new(
                                Px(layout.plot.origin.x.0 + x0),
                                Px(layout.plot.origin.y.0 + y0),
                            ),
                            Size::new(Px(w), Px(h)),
                        ),
                        background: selection_fill,
                        border: fret_core::Edges::all(Px(1.0)),
                        border_color: selection_border,
                        corner_radii: fret_core::Corners::all(Px(0.0)),
                    });
                }
            }
        }

        cx.scene.push(SceneOp::PopClip);

        // Legend (P0: shown when there is more than one series; can be moved to overlays later).
        if let Some((rect, rows)) = self.legend_layout(layout) {
            let series_overrides: Vec<Option<Color>> = self
                .model
                .read(cx.app, |_app, m| {
                    L::series_meta(m)
                        .into_iter()
                        .map(|s| s.stroke_color)
                        .collect()
                })
                .unwrap_or_default();
            let series_count = self.legend_entries.len();

            let pad = Px(8.0);
            let gap = Px(8.0);
            let swatch_w = Px(14.0);
            let swatch_h = Px(self.style.stroke_width.0.clamp(2.0, 6.0));
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(6),
                rect,
                background: tooltip_background,
                border: fret_core::Edges::all(Px(1.0)),
                border_color: tooltip_border,
                corner_radii: fret_core::Corners::all(Px(6.0)),
            });

            for (i, entry) in self.legend_entries.iter().enumerate() {
                let row = rows.get(i).copied().unwrap_or(rect);
                let row_h = row.size.height;

                let hovered_row = self.legend_hover == Some(entry.id);
                let pinned_row = state.pinned_series == Some(entry.id);
                if hovered_row || pinned_row {
                    let a = if pinned_row { 0.16 } else { 0.10 };
                    let highlight = Color {
                        a,
                        ..crosshair_color
                    };
                    let inset_x = Px(2.0);
                    let highlight_rect = Rect::new(
                        Point::new(Px(row.origin.x.0 + inset_x.0), row.origin.y),
                        Size::new(
                            Px((row.size.width.0 - inset_x.0 * 2.0).max(0.0)),
                            row.size.height,
                        ),
                    );
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(6),
                        rect: highlight_rect,
                        background: highlight,
                        border: fret_core::Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: fret_core::Corners::all(Px(4.0)),
                    });
                }

                let override_color = series_overrides.get(i).copied().flatten();
                let color = resolve_series_color(i, self.style, series_count, override_color);

                let visible = !state.hidden_series.contains(&entry.id);
                let swatch_color = if visible {
                    color
                } else {
                    Color {
                        a: (color.a * 0.20).clamp(0.05, 0.35),
                        ..color
                    }
                };
                let text_color = if visible {
                    tooltip_text_color
                } else {
                    Color {
                        a: (tooltip_text_color.a * 0.55).clamp(0.25, 0.85),
                        ..tooltip_text_color
                    }
                };

                let row_mid = row.origin.y.0 + row_h.0 * 0.5;
                let swatch_x = Px(row.origin.x.0 + pad.0);
                let swatch_y = Px(row_mid - swatch_h.0 * 0.5);
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(7),
                    rect: Rect::new(
                        Point::new(swatch_x, swatch_y),
                        Size::new(swatch_w, swatch_h),
                    ),
                    background: swatch_color,
                    border: fret_core::Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });

                let text_x = Px(swatch_x.0 + swatch_w.0 + gap.0);
                let text_top = row.origin.y.0 + (row_h.0 - entry.text.metrics.size.height.0) * 0.5;
                let origin = Point::new(text_x, Px(text_top + entry.text.metrics.baseline.0));
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(7),
                    origin,
                    text: entry.text.blob,
                    color: text_color,
                });
            }
        }

        // Axes.
        if layout.plot.size.width.0 > 0.0 && layout.plot.size.height.0 > 0.0 {
            // Y axis line.
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(10),
                rect: Rect::new(
                    layout.plot.origin,
                    Size::new(Px(1.0), layout.plot.size.height),
                ),
                background: axis_color,
                border: fret_core::Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });

            // Right-side Y axis line (Y2/Y3/Y4 share the plot edge).
            let any_right_axis = self.show_y2_axis || self.show_y3_axis || self.show_y4_axis;
            let any_right_width = layout.y_axis_right.size.width.0 > 0.0
                || layout.y_axis_right2.size.width.0 > 0.0
                || layout.y_axis_right3.size.width.0 > 0.0;
            if any_right_axis && any_right_width {
                let x = Px(layout.plot.origin.x.0 + layout.plot.size.width.0 - 1.0);
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(10),
                    rect: Rect::new(
                        Point::new(x, layout.plot.origin.y),
                        Size::new(Px(1.0), layout.plot.size.height),
                    ),
                    background: axis_color,
                    border: fret_core::Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });
            }

            // X axis line.
            let y = Px(layout.plot.origin.y.0 + layout.plot.size.height.0 - 1.0);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(10),
                rect: Rect::new(
                    Point::new(layout.plot.origin.x, y),
                    Size::new(layout.plot.size.width, Px(1.0)),
                ),
                background: axis_color,
                border: fret_core::Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        }

        // Axis labels: tick density adapts to viewport + label spacing.
        let x_ticks = &self.axis_ticks_x;
        let y_ticks = &self.axis_ticks_y;
        let y2_ticks = &self.axis_ticks_y2;
        let y3_ticks = &self.axis_ticks_y3;
        let y4_ticks = &self.axis_ticks_y4;

        let transform_y1 = PlotTransform {
            viewport: layout.plot,
            data: view_bounds,
            x_scale: self.x_scale,
            y_scale: self.y_scale,
        };
        let transform_y2 = view_bounds_y2.map(|b| PlotTransform {
            viewport: layout.plot,
            data: b,
            x_scale: self.x_scale,
            y_scale: self.y2_scale,
        });
        let transform_y3 = view_bounds_y3.map(|b| PlotTransform {
            viewport: layout.plot,
            data: b,
            x_scale: self.x_scale,
            y_scale: self.y3_scale,
        });
        let transform_y4 = view_bounds_y4.map(|b| PlotTransform {
            viewport: layout.plot,
            data: b,
            x_scale: self.x_scale,
            y_scale: self.y4_scale,
        });

        for (i, label) in self.axis_labels_x.iter().enumerate() {
            if label.metrics.size.width.0 <= 0.0 {
                continue;
            }
            let Some(v) = x_ticks.get(i).copied() else {
                continue;
            };
            let Some(x) = transform_y1.data_x_to_px(v) else {
                continue;
            };
            let x = Px(x.0.round());

            let top = layout.x_axis.origin.y.0 + 2.0;
            let origin = Point::new(
                Px(x.0 - (label.metrics.size.width.0 * 0.5)),
                Px(top + label.metrics.baseline.0),
            );

            cx.scene.push(SceneOp::Text {
                order: DrawOrder(11),
                origin,
                text: label.blob,
                color: label_color,
            });
        }

        for (i, label) in self.axis_labels_y.iter().enumerate() {
            if label.metrics.size.width.0 <= 0.0 {
                continue;
            }
            let Some(v) = y_ticks.get(i).copied() else {
                continue;
            };
            let Some(y) = transform_y1.data_y_to_px(v) else {
                continue;
            };
            let y = Px(y.0.round());

            let origin_x = layout.y_axis_left.origin.x.0 + layout.y_axis_left.size.width.0
                - label.metrics.size.width.0
                - 4.0;
            let top = y.0 - (label.metrics.size.height.0 * 0.5);
            let origin = Point::new(
                Px(origin_x.max(layout.y_axis_left.origin.x.0)),
                Px(top + label.metrics.baseline.0),
            );

            cx.scene.push(SceneOp::Text {
                order: DrawOrder(11),
                origin,
                text: label.blob,
                color: label_color,
            });
        }

        if self.show_y2_axis {
            if let Some(transform_y2) = transform_y2 {
                for (i, label) in self.axis_labels_y2.iter().enumerate() {
                    if label.metrics.size.width.0 <= 0.0 {
                        continue;
                    }
                    let Some(v) = y2_ticks.get(i).copied() else {
                        continue;
                    };
                    let Some(y) = transform_y2.data_y_to_px(v) else {
                        continue;
                    };
                    let y = Px(y.0.round());

                    let origin_x = layout.y_axis_right.origin.x.0 + 4.0;
                    let top = y.0 - (label.metrics.size.height.0 * 0.5);
                    let origin = Point::new(Px(origin_x), Px(top + label.metrics.baseline.0));

                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(11),
                        origin,
                        text: label.blob,
                        color: label_color,
                    });
                }
            }
        }

        if self.show_y3_axis {
            if let Some(transform_y3) = transform_y3 {
                for (i, label) in self.axis_labels_y3.iter().enumerate() {
                    if label.metrics.size.width.0 <= 0.0 {
                        continue;
                    }
                    let Some(v) = y3_ticks.get(i).copied() else {
                        continue;
                    };
                    let Some(y) = transform_y3.data_y_to_px(v) else {
                        continue;
                    };
                    let y = Px(y.0.round());

                    let origin_x = layout.y_axis_right2.origin.x.0 + 4.0;
                    let top = y.0 - (label.metrics.size.height.0 * 0.5);
                    let origin = Point::new(Px(origin_x), Px(top + label.metrics.baseline.0));

                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(11),
                        origin,
                        text: label.blob,
                        color: label_color,
                    });
                }
            }
        }

        if self.show_y4_axis {
            if let Some(transform_y4) = transform_y4 {
                for (i, label) in self.axis_labels_y4.iter().enumerate() {
                    if label.metrics.size.width.0 <= 0.0 {
                        continue;
                    }
                    let Some(v) = y4_ticks.get(i).copied() else {
                        continue;
                    };
                    let Some(y) = transform_y4.data_y_to_px(v) else {
                        continue;
                    };
                    let y = Px(y.0.round());

                    let origin_x = layout.y_axis_right3.origin.x.0 + 4.0;
                    let top = y.0 - (label.metrics.size.height.0 * 0.5);
                    let origin = Point::new(Px(origin_x), Px(top + label.metrics.baseline.0));

                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(11),
                        origin,
                        text: label.blob,
                        color: label_color,
                    });
                }
            }
        }

        // Tooltip (P0: drawn in the same scene; can be moved to overlays later).
        //
        // Behavior:
        // - Selection tooltips are always shown while dragging (query/box-zoom).
        // - Series tooltips are shown only when hovering near a series.
        // - Mouse coordinate readout is controlled via `LinePlotStyle.mouse_readout`.
        let x_span = (view_bounds.x_max - view_bounds.x_min).abs();
        let y_span = (view_bounds.y_max - view_bounds.y_min).abs();

        let selection_tooltip =
            if let (Some(start), Some(end)) = (self.query_drag_start, self.query_drag_current) {
                query_rect_from_plot_points_raw(
                    view_bounds,
                    layout.plot.size,
                    start,
                    end,
                    self.x_scale,
                    self.y_scale,
                )
                .map(|rect| {
                    let x0 = self.tooltip_x_labels.format(rect.x_min, x_span);
                    let x1 = self.tooltip_x_labels.format(rect.x_max, x_span);
                    let y0 = self.tooltip_y_labels.format(rect.y_min, y_span);
                    let y1 = self.tooltip_y_labels.format(rect.y_max, y_span);
                    let text = format!("query\nx=[{x0}, {x1}]\ny=[{y0}, {y1}]");
                    (end, text)
                })
            } else if let (Some(start), Some(end)) = (self.box_zoom_start, self.box_zoom_current) {
                query_rect_from_plot_points_raw(
                    view_bounds,
                    layout.plot.size,
                    start,
                    end,
                    self.x_scale,
                    self.y_scale,
                )
                .map(|rect| {
                    let x0 = self.tooltip_x_labels.format(rect.x_min, x_span);
                    let x1 = self.tooltip_x_labels.format(rect.x_max, x_span);
                    let y0 = self.tooltip_y_labels.format(rect.y_min, y_span);
                    let y1 = self.tooltip_y_labels.format(rect.y_max, y_span);
                    let text = format!("zoom\nx=[{x0}, {x1}]\ny=[{y0}, {y1}]");
                    (end, text)
                })
            } else {
                None
            };

        let cursor_px = self.cursor_px;
        let cursor_data = cursor_px.and_then(|cursor_px| {
            if layout.plot.size.width.0 <= 0.0 || layout.plot.size.height.0 <= 0.0 {
                return None;
            }
            let transform = PlotTransform {
                viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size),
                data: view_bounds,
                x_scale: self.x_scale,
                y_scale: self.y_scale,
            };
            let data = transform.px_to_data(cursor_px);
            (data.x.is_finite() && data.y.is_finite()).then_some(data)
        });

        let linked_x = state.linked_cursor_x.filter(|x| x.is_finite());
        let linked_overlay_active =
            linked_x.is_some() && self.style.linked_cursor_readout == MouseReadoutMode::Overlay;

        if linked_overlay_active {
            let linked_x = linked_x.expect("checked above");

            let hidden = &state.hidden_series;
            let readout_args = PlotCursorReadoutArgs {
                x: linked_x,
                plot_size: layout.plot.size,
                view_bounds,
                x_scale: self.x_scale,
                y_scale: self.y_scale,
                scale_factor: cx.scale_factor,
                hidden,
            };
            let mut readout_rows = self
                .model
                .read(cx.app, |_app, m| L::cursor_readout(m, readout_args))
                .unwrap_or_default();
            apply_readout_policy(
                &mut readout_rows,
                state.pinned_series,
                self.legend_hover,
                self.style.linked_cursor_readout_policy,
            );

            let x_text = self.tooltip_x_labels.format(linked_x, x_span);
            let mut text = format!("x={x_text}");
            for row in readout_rows {
                let y_text = row
                    .y
                    .filter(|y| y.is_finite())
                    .map(|y| match row.y_axis {
                        YAxis::Right if self.show_y2_axis => {
                            let span = view_bounds_y2
                                .map(|b| (b.y_max - b.y_min).abs())
                                .unwrap_or(y_span);
                            self.y2_axis_labels.format(y, span)
                        }
                        YAxis::Right2 if self.show_y3_axis => {
                            let span = view_bounds_y3
                                .map(|b| (b.y_max - b.y_min).abs())
                                .unwrap_or(y_span);
                            self.y3_axis_labels.format(y, span)
                        }
                        YAxis::Right3 if self.show_y4_axis => {
                            let span = view_bounds_y4
                                .map(|b| (b.y_max - b.y_min).abs())
                                .unwrap_or(y_span);
                            self.y4_axis_labels.format(y, span)
                        }
                        _ => self.tooltip_y_labels.format(y, y_span),
                    })
                    .unwrap_or_else(|| "NA".to_string());
                text.push_str(&format!("\n{}: y={y_text}", row.label));
            }

            let font_size = cx
                .theme()
                .metric_by_key("font.size")
                .unwrap_or(cx.theme().metrics.font_size);
            let style = TextStyle {
                font: FontId::default(),
                size: Px((font_size.0 * 0.90).max(10.0)),
                weight: FontWeight::NORMAL,
                line_height: None,
                letter_spacing_em: None,
            };
            let constraints = TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: cx.scale_factor,
            };

            let mut key = 0u64;
            key = Self::hash_u64(key, theme.revision);
            key = Self::hash_u64(key, font_stack_key);
            key = Self::hash_u64(key, u64::from(cx.scale_factor.to_bits()));
            for b in text.as_bytes() {
                key = Self::hash_u64(key, u64::from(*b));
            }
            key = Self::hash_u64(key, Self::text_style_key(&style));

            let needs = self
                .linked_cursor_readout_text
                .as_ref()
                .is_none_or(|t| t.key != key);
            if needs {
                if let Some(prev) = self.linked_cursor_readout_text.take() {
                    cx.services.text().release(prev.blob);
                }
                let prepared = self.prepare_text(cx.services, &text, &style, constraints);
                self.linked_cursor_readout_text = Some(PreparedText {
                    blob: prepared.blob,
                    metrics: prepared.metrics,
                    key,
                });
            }

            if let Some(tt) = self.linked_cursor_readout_text {
                let pad = Px(6.0);
                let margin = Px(6.0);
                let w = Px(tt.metrics.size.width.0 + pad.0 * 2.0);
                let h = Px(tt.metrics.size.height.0 + pad.0 * 2.0);
                let Some(rect) = overlay_rect_in_plot(
                    layout.plot,
                    Size::new(w, h),
                    self.style.linked_cursor_readout_anchor,
                    margin,
                ) else {
                    return;
                };
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(12),
                    rect,
                    background: tooltip_background,
                    border: fret_core::Edges::all(Px(1.0)),
                    border_color: tooltip_border,
                    corner_radii: fret_core::Corners::all(Px(6.0)),
                });

                let origin = Point::new(
                    Px(rect.origin.x.0 + pad.0),
                    Px(rect.origin.y.0 + pad.0 + tt.metrics.baseline.0),
                );
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(13),
                    origin,
                    text: tt.blob,
                    color: tooltip_text_color,
                });
            }
        }

        if self.style.mouse_readout == MouseReadoutMode::Overlay && !linked_overlay_active {
            if let Some(cursor_data) = cursor_data {
                let x_text = self.tooltip_x_labels.format(cursor_data.x, x_span);
                let y_text = self.tooltip_y_labels.format(cursor_data.y, y_span);
                let text = format!("x={x_text}  y={y_text}");

                let font_size = cx
                    .theme()
                    .metric_by_key("font.size")
                    .unwrap_or(cx.theme().metrics.font_size);
                let style = TextStyle {
                    font: FontId::default(),
                    size: Px((font_size.0 * 0.90).max(10.0)),
                    weight: FontWeight::NORMAL,
                    line_height: None,
                    letter_spacing_em: None,
                };
                let constraints = TextConstraints {
                    max_width: None,
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    scale_factor: cx.scale_factor,
                };

                let mut key = 0u64;
                key = Self::hash_u64(key, theme.revision);
                key = Self::hash_u64(key, font_stack_key);
                key = Self::hash_u64(key, u64::from(cx.scale_factor.to_bits()));
                for b in text.as_bytes() {
                    key = Self::hash_u64(key, u64::from(*b));
                }
                key = Self::hash_u64(key, Self::text_style_key(&style));

                let needs = self
                    .mouse_readout_text
                    .as_ref()
                    .is_none_or(|t| t.key != key);
                if needs {
                    if let Some(prev) = self.mouse_readout_text.take() {
                        cx.services.text().release(prev.blob);
                    }
                    let prepared = self.prepare_text(cx.services, &text, &style, constraints);
                    self.mouse_readout_text = Some(PreparedText {
                        blob: prepared.blob,
                        metrics: prepared.metrics,
                        key,
                    });
                }

                if let Some(tt) = self.mouse_readout_text {
                    let margin = Px(6.0);
                    let w = Px(tt.metrics.size.width.0);
                    let h = Px(tt.metrics.size.height.0);
                    let Some(rect) = overlay_rect_in_plot(
                        layout.plot,
                        Size::new(w, h),
                        self.style.mouse_readout_anchor,
                        margin,
                    ) else {
                        return;
                    };
                    let origin =
                        Point::new(rect.origin.x, Px(rect.origin.y.0 + tt.metrics.baseline.0));
                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(12),
                        origin,
                        text: tt.blob,
                        color: label_color,
                    });
                }
            }
        }

        let tooltip = selection_tooltip.or_else(|| {
            self.hover
                .map(|hover| {
                    let (series_count, series_label, y_axis) = self
                        .model
                        .read(cx.app, |_app, m| {
                            let series_count = L::series_meta(m).len();
                            let series_label = L::series_label(m, hover.series_id);
                            let y_axis = L::series_y_axis(m, hover.series_id);
                            (series_count, series_label, y_axis)
                        })
                        .unwrap_or((0, None, YAxis::Left));

                    let x_text = self.tooltip_x_labels.format(hover.data.x, x_span);
                    let y_text = if y_axis == YAxis::Right && self.show_y2_axis {
                        let span = view_bounds_y2
                            .map(|b| (b.y_max - b.y_min).abs())
                            .unwrap_or(y_span);
                        self.y2_axis_labels.format(hover.data.y, span)
                    } else if y_axis == YAxis::Right2 && self.show_y3_axis {
                        let span = view_bounds_y3
                            .map(|b| (b.y_max - b.y_min).abs())
                            .unwrap_or(y_span);
                        self.y3_axis_labels.format(hover.data.y, span)
                    } else if y_axis == YAxis::Right3 && self.show_y4_axis {
                        let span = view_bounds_y4
                            .map(|b| (b.y_max - b.y_min).abs())
                            .unwrap_or(y_span);
                        self.y4_axis_labels.format(hover.data.y, span)
                    } else {
                        self.tooltip_y_labels.format(hover.data.y, y_span)
                    };
                    let text = if series_count > 1 {
                        if let Some(label) = series_label.as_deref() {
                            format!("{label}  x={x_text}  y={y_text}")
                        } else {
                            format!("s={}  x={x_text}  y={y_text}", hover.series_id.0)
                        }
                    } else {
                        format!("x={x_text}  y={y_text}")
                    };
                    let text = if series_count == 0
                        && let Some(label) = series_label.as_deref()
                    {
                        format!("{label}  {text}")
                    } else {
                        text
                    };
                    let text = if let Some(v) = hover.value
                        && v.is_finite()
                    {
                        let v_text = if v.abs() < 10_000.0 {
                            format!("{v:.4}")
                        } else {
                            format!("{v:.4e}")
                        };
                        format!("{text}  value={v_text}")
                    } else {
                        text
                    };
                    (hover.plot_px, text)
                })
                .or_else(|| {
                    if self.style.mouse_readout != MouseReadoutMode::Tooltip {
                        return None;
                    }

                    let cursor_px = cursor_px?;
                    let cursor_data = cursor_data?;

                    let hidden = &state.hidden_series;
                    let readout_args = PlotCursorReadoutArgs {
                        x: cursor_data.x,
                        plot_size: layout.plot.size,
                        view_bounds,
                        x_scale: self.x_scale,
                        y_scale: self.y_scale,
                        scale_factor: cx.scale_factor,
                        hidden,
                    };
                    let mut readout_rows = self
                        .model
                        .read(cx.app, |_app, m| L::cursor_readout(m, readout_args))
                        .unwrap_or_default();

                    if let Some(pinned) = state.pinned_series {
                        readout_rows.retain(|r| r.series_id == pinned);
                    }

                    let x_text = self.tooltip_x_labels.format(cursor_data.x, x_span);
                    let y_text = self.tooltip_y_labels.format(cursor_data.y, y_span);
                    let mut text = format!("x={x_text}  y={y_text}");
                    for row in readout_rows {
                        let y_text = row
                            .y
                            .filter(|y| y.is_finite())
                            .map(|y| match row.y_axis {
                                YAxis::Right if self.show_y2_axis => {
                                    let span = view_bounds_y2
                                        .map(|b| (b.y_max - b.y_min).abs())
                                        .unwrap_or(y_span);
                                    self.y2_axis_labels.format(y, span)
                                }
                                YAxis::Right2 if self.show_y3_axis => {
                                    let span = view_bounds_y3
                                        .map(|b| (b.y_max - b.y_min).abs())
                                        .unwrap_or(y_span);
                                    self.y3_axis_labels.format(y, span)
                                }
                                YAxis::Right3 if self.show_y4_axis => {
                                    let span = view_bounds_y4
                                        .map(|b| (b.y_max - b.y_min).abs())
                                        .unwrap_or(y_span);
                                    self.y4_axis_labels.format(y, span)
                                }
                                _ => self.tooltip_y_labels.format(y, y_span),
                            })
                            .unwrap_or_else(|| "NA".to_string());
                        text.push_str(&format!("\n{}: y={y_text}", row.label));
                    }

                    if let Some(query) = state.query {
                        let x0 = self.tooltip_x_labels.format(query.x_min, x_span);
                        let x1 = self.tooltip_x_labels.format(query.x_max, x_span);
                        let y0 = self.tooltip_y_labels.format(query.y_min, y_span);
                        let y1 = self.tooltip_y_labels.format(query.y_max, y_span);
                        text.push_str(&format!("\nquery: x=[{x0}, {x1}]  y=[{y0}, {y1}]"));
                    }

                    Some((cursor_px, text))
                })
                .or_else(|| {
                    let linked_x = state.linked_cursor_x?;
                    if self.style.linked_cursor_readout != MouseReadoutMode::Tooltip {
                        return None;
                    }
                    if !linked_x.is_finite() {
                        return None;
                    }

                    let transform = PlotTransform {
                        viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size),
                        data: view_bounds,
                        x_scale: self.x_scale,
                        y_scale: self.y_scale,
                    };
                    let Some(linked_x_px) = transform.data_x_to_px(linked_x) else {
                        return None;
                    };

                    let anchor_local = Point::new(
                        Px(linked_x_px.0.clamp(0.0, layout.plot.size.width.0)),
                        Px(0.0),
                    );

                    let hidden = &state.hidden_series;
                    let readout_args = PlotCursorReadoutArgs {
                        x: linked_x,
                        plot_size: layout.plot.size,
                        view_bounds,
                        x_scale: self.x_scale,
                        y_scale: self.y_scale,
                        scale_factor: cx.scale_factor,
                        hidden,
                    };
                    let mut readout_rows = self
                        .model
                        .read(cx.app, |_app, m| L::cursor_readout(m, readout_args))
                        .unwrap_or_default();
                    apply_readout_policy(
                        &mut readout_rows,
                        state.pinned_series,
                        self.legend_hover,
                        self.style.linked_cursor_readout_policy,
                    );

                    let x_text = self.tooltip_x_labels.format(linked_x, x_span);
                    let mut text = format!("x={x_text}");
                    for row in readout_rows {
                        let y_text = row
                            .y
                            .filter(|y| y.is_finite())
                            .map(|y| match row.y_axis {
                                YAxis::Right if self.show_y2_axis => {
                                    let span = view_bounds_y2
                                        .map(|b| (b.y_max - b.y_min).abs())
                                        .unwrap_or(y_span);
                                    self.y2_axis_labels.format(y, span)
                                }
                                YAxis::Right2 if self.show_y3_axis => {
                                    let span = view_bounds_y3
                                        .map(|b| (b.y_max - b.y_min).abs())
                                        .unwrap_or(y_span);
                                    self.y3_axis_labels.format(y, span)
                                }
                                YAxis::Right3 if self.show_y4_axis => {
                                    let span = view_bounds_y4
                                        .map(|b| (b.y_max - b.y_min).abs())
                                        .unwrap_or(y_span);
                                    self.y4_axis_labels.format(y, span)
                                }
                                _ => self.tooltip_y_labels.format(y, y_span),
                            })
                            .unwrap_or_else(|| "NA".to_string());
                        text.push_str(&format!("\n{}: y={y_text}", row.label));
                    }

                    if let Some(query) = state.query {
                        let x0 = self.tooltip_x_labels.format(query.x_min, x_span);
                        let x1 = self.tooltip_x_labels.format(query.x_max, x_span);
                        let y0 = self.tooltip_y_labels.format(query.y_min, y_span);
                        let y1 = self.tooltip_y_labels.format(query.y_max, y_span);
                        text.push_str(&format!("\nquery: x=[{x0}, {x1}]  y=[{y0}, {y1}]"));
                    }

                    Some((anchor_local, text))
                })
        });

        if let Some((anchor_local, text)) = tooltip {
            let font_size = cx
                .theme()
                .metric_by_key("font.size")
                .unwrap_or(cx.theme().metrics.font_size);
            let style = TextStyle {
                font: FontId::default(),
                size: Px((font_size.0 * 0.90).max(10.0)),
                weight: FontWeight::NORMAL,
                line_height: None,
                letter_spacing_em: None,
            };
            let constraints = TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: cx.scale_factor,
            };

            let mut key = 0u64;
            key = Self::hash_u64(key, theme.revision);
            key = Self::hash_u64(key, font_stack_key);
            key = Self::hash_u64(key, u64::from(cx.scale_factor.to_bits()));
            for b in text.as_bytes() {
                key = Self::hash_u64(key, u64::from(*b));
            }
            key = Self::hash_u64(key, Self::text_style_key(&style));

            let needs = self.tooltip_text.as_ref().is_none_or(|t| t.key != key);
            if needs {
                if let Some(prev) = self.tooltip_text.take() {
                    cx.services.text().release(prev.blob);
                }
                let prepared = self.prepare_text(cx.services, &text, &style, constraints);
                self.tooltip_text = Some(PreparedText {
                    blob: prepared.blob,
                    metrics: prepared.metrics,
                    key,
                });
            }

            if let Some(tt) = self.tooltip_text {
                let anchor = Point::new(
                    Px(layout.plot.origin.x.0 + anchor_local.x.0),
                    Px(layout.plot.origin.y.0 + anchor_local.y.0),
                );
                let pad = Px(6.0);
                let gap = Px(10.0);
                let w = Px(tt.metrics.size.width.0 + pad.0 * 2.0);
                let h = Px(tt.metrics.size.height.0 + pad.0 * 2.0);

                let mut x = Px(anchor.x.0 + gap.0);
                let mut y = Px(anchor.y.0 + gap.0);
                if x.0 + w.0 > cx.bounds.origin.x.0 + cx.bounds.size.width.0 {
                    x = Px(anchor.x.0 - gap.0 - w.0);
                }
                if y.0 + h.0 > cx.bounds.origin.y.0 + cx.bounds.size.height.0 {
                    y = Px(anchor.y.0 - gap.0 - h.0);
                }
                x = Px(x.0.max(cx.bounds.origin.x.0));
                y = Px(y.0.max(cx.bounds.origin.y.0));

                let rect = Rect::new(Point::new(x, y), Size::new(w, h));
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(20),
                    rect,
                    background: tooltip_background,
                    border: fret_core::Edges::all(Px(1.0)),
                    border_color: tooltip_border,
                    corner_radii: fret_core::Corners::all(Px(6.0)),
                });

                let origin = Point::new(
                    Px(rect.origin.x.0 + pad.0),
                    Px(rect.origin.y.0 + pad.0 + tt.metrics.baseline.0),
                );
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(21),
                    origin,
                    text: tt.blob,
                    color: tooltip_text_color,
                });
            }
        }
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Viewport);
        cx.set_label("Plot");
    }

    fn cleanup_resources(&mut self, services: &mut dyn UiServices) {
        self.layer.cleanup_resources(services);
        self.clear_axis_label_cache(services);
        self.clear_legend_cache(services);
        if let Some(t) = self.tooltip_text.take() {
            services.text().release(t.blob);
        }
        if let Some(t) = self.mouse_readout_text.take() {
            services.text().release(t.blob);
        }
        if let Some(t) = self.linked_cursor_readout_text.take() {
            services.text().release(t.blob);
        }
    }
}

impl PlotLayer for LinePlotLayer {
    type Model = LinePlotModel;

    fn data_bounds(model: &Self::Model) -> DataRect {
        model.data_bounds
    }

    fn data_bounds_y2(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y2
    }

    fn data_bounds_y3(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y3
    }

    fn data_bounds_y4(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y4
    }

    fn series_meta(model: &Self::Model) -> Vec<SeriesMeta> {
        model
            .series
            .iter()
            .map(|s| SeriesMeta {
                id: s.id,
                label: s.label.clone(),
                y_axis: s.y_axis,
                stroke_color: s.stroke_color,
            })
            .collect()
    }

    fn series_label(model: &Self::Model, series_id: SeriesId) -> Option<String> {
        model
            .series
            .iter()
            .find(|s| s.id == series_id)
            .map(|s| s.label.to_string())
    }

    fn series_y_axis(model: &Self::Model, series_id: SeriesId) -> YAxis {
        model
            .series
            .iter()
            .find(|s| s.id == series_id)
            .map(|s| s.y_axis)
            .unwrap_or(YAxis::Left)
    }

    fn cursor_readout(
        model: &Self::Model,
        args: PlotCursorReadoutArgs<'_>,
    ) -> Vec<PlotCursorReadoutRow> {
        let PlotCursorReadoutArgs {
            x,
            plot_size,
            view_bounds,
            x_scale,
            y_scale,
            scale_factor,
            hidden,
        } = args;

        if !x.is_finite() {
            return Vec::new();
        }

        let transform = PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let view_x = view_x_range(transform);
        let view_x = (view_x.start().is_finite() && view_x.end().is_finite()).then_some(view_x);
        let budget = device_point_budget(transform, scale_factor);

        let mut out: Vec<PlotCursorReadoutRow> = Vec::new();
        for s in &model.series {
            if hidden.contains(&s.id) {
                continue;
            }
            let y = cursor_readout_y_at_x(&*s.data, x, view_x.clone(), budget);
            out.push(PlotCursorReadoutRow {
                series_id: s.id,
                label: s.label.clone(),
                y_axis: s.y_axis,
                y,
            });
        }
        out
    }

    fn paint_paths<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        model: &Self::Model,
        args: PlotPaintArgs<'_>,
    ) -> Vec<(SeriesId, PathId, Color)> {
        let PlotPaintArgs {
            model_revision,
            plot,
            view_bounds,
            view_bounds_y2,
            view_bounds_y3,
            view_bounds_y4,
            x_scale,
            y_scale,
            y2_scale,
            y3_scale,
            y4_scale,
            style,
            hidden,
        } = args;

        let scale_factor_bits = cx.scale_factor.to_bits();
        let viewport_w_bits = plot.size.width.0.to_bits();
        let viewport_h_bits = plot.size.height.0.to_bits();
        let view_key_y1 = data_rect_key_scaled(view_bounds, x_scale, y_scale);
        let view_key_y2 = view_bounds_y2.map(|b| data_rect_key_scaled(b, x_scale, y2_scale));
        let view_key_y3 = view_bounds_y3.map(|b| data_rect_key_scaled(b, x_scale, y3_scale));
        let view_key_y4 = view_bounds_y4.map(|b| data_rect_key_scaled(b, x_scale, y4_scale));

        let view_key_for_axis = |axis: YAxis| match axis {
            YAxis::Left => view_key_y1,
            YAxis::Right => view_key_y2.unwrap_or(view_key_y1),
            YAxis::Right2 => view_key_y3.unwrap_or(view_key_y1),
            YAxis::Right3 => view_key_y4.unwrap_or(view_key_y1),
        };

        let series = &model.series;
        let series_count = series.len();

        if series_count == 0 {
            for cached in self.cached_paths.drain(..) {
                if let Some(id) = cached.id {
                    cx.services.path().release(id);
                }
            }
            return Vec::new();
        }

        let cached_ok = self.cached_paths.len() == series_count
            && self.cached_paths.iter().enumerate().all(|(i, c)| {
                series.get(i).is_some_and(|s| {
                    let expected_view_key = view_key_for_axis(s.y_axis);

                    s.id == c.series_id && c.view_key == expected_view_key
                }) && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
                    && c.stroke_width == style.stroke_width
            });

        if cached_ok {
            let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
            for (i, s) in series.iter().enumerate() {
                if hidden.contains(&s.id) {
                    continue;
                }
                let Some(id) = self.cached_paths.get(i).and_then(|c| c.id) else {
                    continue;
                };
                let style = series_style(s, i, style, series_count);
                out.push((s.id, id, style.stroke_color));
            }
            return out;
        }

        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.id {
                cx.services.path().release(id);
            }
        }

        let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size);
        let transform_y1 = PlotTransform {
            viewport: local_viewport,
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let transform_y2 = view_bounds_y2.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y2_scale,
        });
        let transform_y3 = view_bounds_y3.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y3_scale,
        });
        let transform_y4 = view_bounds_y4.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y4_scale,
        });

        let transform_for_axis = |axis: YAxis| match axis {
            YAxis::Left => transform_y1,
            YAxis::Right => transform_y2.unwrap_or(transform_y1),
            YAxis::Right2 => transform_y3.unwrap_or(transform_y1),
            YAxis::Right3 => transform_y4.unwrap_or(transform_y1),
        };

        let path_style = PathStyle::Stroke(fret_core::StrokeStyle {
            width: style.stroke_width,
        });
        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };

        let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
        self.cached_paths = Vec::with_capacity(series_count);

        for (series_index, s) in series.iter().enumerate() {
            let series_id = s.id;
            if hidden.contains(&series_id) {
                let view_key = view_key_for_axis(s.y_axis);
                self.cached_paths.push(CachedPath {
                    id: None,
                    series_id,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width: style.stroke_width,
                    view_key,
                    samples: Vec::new(),
                });
                continue;
            }

            let transform = transform_for_axis(s.y_axis);
            let view_key = view_key_for_axis(s.y_axis);

            let (commands, samples) =
                decimate_polyline(transform, &*s.data, cx.scale_factor, series_id);
            let id = if commands.is_empty() {
                None
            } else {
                let (id, _metrics) = cx
                    .services
                    .path()
                    .prepare(&commands, path_style, constraints);
                Some(id)
            };

            self.cached_paths.push(CachedPath {
                id,
                series_id,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width: style.stroke_width,
                view_key,
                samples,
            });

            if let Some(id) = id {
                let style = series_style(s, series_index, style, series_count);
                out.push((series_id, id, style.stroke_color));
            }
        }

        out
    }

    fn hit_test(&mut self, model: &Self::Model, args: PlotHitTestArgs<'_>) -> Option<PlotHover> {
        let series: Vec<(SeriesId, YAxis, &dyn SeriesData)> = model
            .series
            .iter()
            .map(|s| (s.id, s.y_axis, &*s.data))
            .collect();
        hit_test_polyline_series_data(&self.cached_paths, &series, args)
    }

    fn cleanup_resources(&mut self, services: &mut dyn UiServices) {
        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.id {
                services.path().release(id);
            }
        }
    }
}

impl PlotLayer for ScatterPlotLayer {
    type Model = ScatterPlotModel;

    fn data_bounds(model: &Self::Model) -> DataRect {
        model.data_bounds
    }

    fn data_bounds_y2(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y2
    }

    fn data_bounds_y3(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y3
    }

    fn data_bounds_y4(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y4
    }

    fn series_meta(model: &Self::Model) -> Vec<SeriesMeta> {
        model
            .series
            .iter()
            .map(|s| SeriesMeta {
                id: s.id,
                label: s.label.clone(),
                y_axis: s.y_axis,
                stroke_color: s.stroke_color,
            })
            .collect()
    }

    fn series_label(model: &Self::Model, series_id: SeriesId) -> Option<String> {
        model
            .series
            .iter()
            .find(|s| s.id == series_id)
            .map(|s| s.label.to_string())
    }

    fn series_y_axis(model: &Self::Model, series_id: SeriesId) -> YAxis {
        model
            .series
            .iter()
            .find(|s| s.id == series_id)
            .map(|s| s.y_axis)
            .unwrap_or(YAxis::Left)
    }

    fn cursor_readout(
        model: &Self::Model,
        args: PlotCursorReadoutArgs<'_>,
    ) -> Vec<PlotCursorReadoutRow> {
        let PlotCursorReadoutArgs {
            x,
            plot_size,
            view_bounds,
            x_scale,
            y_scale,
            scale_factor,
            hidden,
        } = args;

        if !x.is_finite() {
            return Vec::new();
        }

        let transform = PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let view_x = view_x_range(transform);
        let view_x = (view_x.start().is_finite() && view_x.end().is_finite()).then_some(view_x);
        let budget = device_point_budget(transform, scale_factor);

        let mut out: Vec<PlotCursorReadoutRow> = Vec::new();
        for s in &model.series {
            if hidden.contains(&s.id) {
                continue;
            }
            let y = cursor_readout_y_at_x(&*s.data, x, view_x.clone(), budget);
            out.push(PlotCursorReadoutRow {
                series_id: s.id,
                label: s.label.clone(),
                y_axis: s.y_axis,
                y,
            });
        }
        out
    }

    fn paint_paths<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        model: &Self::Model,
        args: PlotPaintArgs<'_>,
    ) -> Vec<(SeriesId, PathId, Color)> {
        let PlotPaintArgs {
            model_revision,
            plot,
            view_bounds,
            view_bounds_y2,
            view_bounds_y3,
            view_bounds_y4,
            x_scale,
            y_scale,
            y2_scale,
            y3_scale,
            y4_scale,
            style,
            hidden,
        } = args;

        let scale_factor_bits = cx.scale_factor.to_bits();
        let viewport_w_bits = plot.size.width.0.to_bits();
        let viewport_h_bits = plot.size.height.0.to_bits();
        let view_key_y1 = data_rect_key_scaled(view_bounds, x_scale, y_scale);
        let view_key_y2 = view_bounds_y2.map(|b| data_rect_key_scaled(b, x_scale, y2_scale));
        let view_key_y3 = view_bounds_y3.map(|b| data_rect_key_scaled(b, x_scale, y3_scale));
        let view_key_y4 = view_bounds_y4.map(|b| data_rect_key_scaled(b, x_scale, y4_scale));

        let view_key_for_axis = |axis: YAxis| match axis {
            YAxis::Left => view_key_y1,
            YAxis::Right => view_key_y2.unwrap_or(view_key_y1),
            YAxis::Right2 => view_key_y3.unwrap_or(view_key_y1),
            YAxis::Right3 => view_key_y4.unwrap_or(view_key_y1),
        };

        let series = &model.series;
        let series_count = series.len();

        if series_count == 0 {
            for cached in self.cached_paths.drain(..) {
                if let Some(id) = cached.id {
                    cx.services.path().release(id);
                }
            }
            return Vec::new();
        }

        let cached_ok = self.cached_paths.len() == series_count
            && self.cached_paths.iter().enumerate().all(|(i, c)| {
                series.get(i).is_some_and(|s| {
                    let expected_view_key = view_key_for_axis(s.y_axis);
                    s.id == c.series_id && c.view_key == expected_view_key
                }) && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
                    && c.stroke_width == style.stroke_width
            });

        if cached_ok {
            let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
            for (i, s) in series.iter().enumerate() {
                if hidden.contains(&s.id) {
                    continue;
                }
                let Some(id) = self.cached_paths.get(i).and_then(|c| c.id) else {
                    continue;
                };
                let color = resolve_series_color(i, style, series_count, s.stroke_color);
                out.push((s.id, id, color));
            }
            return out;
        }

        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.id {
                cx.services.path().release(id);
            }
        }

        let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size);
        let transform_y1 = PlotTransform {
            viewport: local_viewport,
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let transform_y2 = view_bounds_y2.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y2_scale,
        });
        let transform_y3 = view_bounds_y3.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y3_scale,
        });
        let transform_y4 = view_bounds_y4.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y4_scale,
        });

        let transform_for_axis = |axis: YAxis| match axis {
            YAxis::Left => transform_y1,
            YAxis::Right => transform_y2.unwrap_or(transform_y1),
            YAxis::Right2 => transform_y3.unwrap_or(transform_y1),
            YAxis::Right3 => transform_y4.unwrap_or(transform_y1),
        };

        let marker_radius = Px((style.stroke_width.0 * 3.0).clamp(2.0, 6.0));
        let path_style = PathStyle::Stroke(fret_core::StrokeStyle {
            width: style.stroke_width,
        });
        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };

        let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
        self.cached_paths = Vec::with_capacity(series_count);

        for (series_index, s) in series.iter().enumerate() {
            let series_id = s.id;
            if hidden.contains(&series_id) {
                let view_key = view_key_for_axis(s.y_axis);
                self.cached_paths.push(CachedPath {
                    id: None,
                    series_id,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width: style.stroke_width,
                    view_key,
                    samples: Vec::new(),
                });
                continue;
            }

            let transform = transform_for_axis(s.y_axis);
            let view_key = view_key_for_axis(s.y_axis);

            let samples = decimate_points(transform, &*s.data, cx.scale_factor, series_id);
            let commands = scatter_marker_commands(&samples, marker_radius);
            let id = if commands.is_empty() {
                None
            } else {
                let (id, _metrics) = cx
                    .services
                    .path()
                    .prepare(&commands, path_style, constraints);
                Some(id)
            };

            self.cached_paths.push(CachedPath {
                id,
                series_id,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width: style.stroke_width,
                view_key,
                samples,
            });

            if let Some(id) = id {
                let color = resolve_series_color(series_index, style, series_count, s.stroke_color);
                out.push((series_id, id, color));
            }
        }

        out
    }

    fn hit_test(&mut self, model: &Self::Model, args: PlotHitTestArgs<'_>) -> Option<PlotHover> {
        let series: Vec<(SeriesId, YAxis, &dyn SeriesData)> = model
            .series
            .iter()
            .map(|s| (s.id, s.y_axis, &*s.data))
            .collect();
        hit_test_series_data(&self.cached_paths, &series, args)
    }

    fn cleanup_resources(&mut self, services: &mut dyn UiServices) {
        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.id {
                services.path().release(id);
            }
        }
    }
}

impl PlotLayer for ErrorBarsPlotLayer {
    type Model = ErrorBarsPlotModel;

    fn data_bounds(model: &Self::Model) -> DataRect {
        model.data_bounds
    }

    fn data_bounds_y2(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y2
    }

    fn data_bounds_y3(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y3
    }

    fn data_bounds_y4(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y4
    }

    fn series_meta(model: &Self::Model) -> Vec<SeriesMeta> {
        model
            .series
            .iter()
            .map(|s| SeriesMeta {
                id: s.id,
                label: s.label.clone(),
                y_axis: s.y_axis,
                stroke_color: s.stroke_color,
            })
            .collect()
    }

    fn series_label(model: &Self::Model, series_id: SeriesId) -> Option<String> {
        model
            .series
            .iter()
            .find(|s| s.id == series_id)
            .map(|s| s.label.to_string())
    }

    fn series_y_axis(model: &Self::Model, series_id: SeriesId) -> YAxis {
        model
            .series
            .iter()
            .find(|s| s.id == series_id)
            .map(|s| s.y_axis)
            .unwrap_or(YAxis::Left)
    }

    fn cursor_readout(
        model: &Self::Model,
        args: PlotCursorReadoutArgs<'_>,
    ) -> Vec<PlotCursorReadoutRow> {
        let PlotCursorReadoutArgs {
            x,
            plot_size,
            view_bounds,
            x_scale,
            y_scale,
            scale_factor,
            hidden,
        } = args;

        if !x.is_finite() {
            return Vec::new();
        }

        let transform = PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let view_x = view_x_range(transform);
        let view_x = (view_x.start().is_finite() && view_x.end().is_finite()).then_some(view_x);
        let budget = device_point_budget(transform, scale_factor);

        let mut out: Vec<PlotCursorReadoutRow> = Vec::new();
        for s in &model.series {
            if hidden.contains(&s.id) {
                continue;
            }
            let y = cursor_readout_y_at_x(&*s.data, x, view_x.clone(), budget);
            out.push(PlotCursorReadoutRow {
                series_id: s.id,
                label: s.label.clone(),
                y_axis: s.y_axis,
                y,
            });
        }
        out
    }

    fn paint_paths<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        model: &Self::Model,
        args: PlotPaintArgs<'_>,
    ) -> Vec<(SeriesId, PathId, Color)> {
        let PlotPaintArgs {
            model_revision,
            plot,
            view_bounds,
            view_bounds_y2,
            view_bounds_y3,
            view_bounds_y4,
            x_scale,
            y_scale,
            y2_scale,
            y3_scale,
            y4_scale,
            style,
            hidden,
        } = args;

        let scale_factor_bits = cx.scale_factor.to_bits();
        let viewport_w_bits = plot.size.width.0.to_bits();
        let viewport_h_bits = plot.size.height.0.to_bits();
        let view_key_y1 = data_rect_key_scaled(view_bounds, x_scale, y_scale);
        let view_key_y2 = view_bounds_y2.map(|b| data_rect_key_scaled(b, x_scale, y2_scale));
        let view_key_y3 = view_bounds_y3.map(|b| data_rect_key_scaled(b, x_scale, y3_scale));
        let view_key_y4 = view_bounds_y4.map(|b| data_rect_key_scaled(b, x_scale, y4_scale));

        let view_key_for_axis = |axis: YAxis| match axis {
            YAxis::Left => view_key_y1,
            YAxis::Right => view_key_y2.unwrap_or(view_key_y1),
            YAxis::Right2 => view_key_y3.unwrap_or(view_key_y1),
            YAxis::Right3 => view_key_y4.unwrap_or(view_key_y1),
        };

        let series = &model.series;
        let series_count = series.len();

        if series_count == 0 {
            for cached in self.cached_paths.drain(..) {
                if let Some(id) = cached.id {
                    cx.services.path().release(id);
                }
            }
            return Vec::new();
        }

        let cached_ok = self.cached_paths.len() == series_count
            && self.cached_paths.iter().enumerate().all(|(i, c)| {
                series.get(i).is_some_and(|s| {
                    let expected_view_key = view_key_for_axis(s.y_axis);
                    s.id == c.series_id && c.view_key == expected_view_key
                }) && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
                    && c.stroke_width == style.stroke_width
            });

        if cached_ok {
            let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
            for (i, s) in series.iter().enumerate() {
                if hidden.contains(&s.id) {
                    continue;
                }
                let Some(id) = self.cached_paths.get(i).and_then(|c| c.id) else {
                    continue;
                };
                let color = resolve_series_color(i, style, series_count, s.stroke_color);
                out.push((s.id, id, color));
            }
            return out;
        }

        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.id {
                cx.services.path().release(id);
            }
        }

        let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size);
        let transform_y1 = PlotTransform {
            viewport: local_viewport,
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let transform_y2 = view_bounds_y2.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y2_scale,
        });
        let transform_y3 = view_bounds_y3.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y3_scale,
        });
        let transform_y4 = view_bounds_y4.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y4_scale,
        });

        let transform_for_axis = |axis: YAxis| match axis {
            YAxis::Left => transform_y1,
            YAxis::Right => transform_y2.unwrap_or(transform_y1),
            YAxis::Right2 => transform_y3.unwrap_or(transform_y1),
            YAxis::Right3 => transform_y4.unwrap_or(transform_y1),
        };

        let path_style = PathStyle::Stroke(fret_core::StrokeStyle {
            width: style.stroke_width,
        });
        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };

        let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
        self.cached_paths = Vec::with_capacity(series_count);

        for (series_index, s) in series.iter().enumerate() {
            let series_id = s.id;
            if hidden.contains(&series_id) {
                let view_key = view_key_for_axis(s.y_axis);
                self.cached_paths.push(CachedPath {
                    id: None,
                    series_id,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width: style.stroke_width,
                    view_key,
                    samples: Vec::new(),
                });
                continue;
            }

            let transform = transform_for_axis(s.y_axis);
            let view_key = view_key_for_axis(s.y_axis);

            let samples = decimate_points(transform, &*s.data, cx.scale_factor, series_id);
            let commands = error_bars_commands(s, transform, &samples);
            let id = if commands.is_empty() {
                None
            } else {
                let (id, _metrics) = cx
                    .services
                    .path()
                    .prepare(&commands, path_style, constraints);
                Some(id)
            };

            self.cached_paths.push(CachedPath {
                id,
                series_id,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width: style.stroke_width,
                view_key,
                samples,
            });

            if let Some(id) = id {
                let color = resolve_series_color(series_index, style, series_count, s.stroke_color);
                out.push((series_id, id, color));
            }
        }

        out
    }

    fn hit_test(&mut self, model: &Self::Model, args: PlotHitTestArgs<'_>) -> Option<PlotHover> {
        let series: Vec<(SeriesId, YAxis, &dyn SeriesData)> = model
            .series
            .iter()
            .map(|s| (s.id, s.y_axis, &*s.data))
            .collect();
        hit_test_series_data(&self.cached_paths, &series, args)
    }

    fn cleanup_resources(&mut self, services: &mut dyn UiServices) {
        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.id {
                services.path().release(id);
            }
        }
    }
}

impl PlotLayer for CandlestickPlotLayer {
    type Model = CandlestickPlotModel;

    fn data_bounds(model: &Self::Model) -> DataRect {
        model.data_bounds
    }

    fn data_bounds_y2(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y2
    }

    fn data_bounds_y3(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y3
    }

    fn data_bounds_y4(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y4
    }

    fn series_meta(model: &Self::Model) -> Vec<SeriesMeta> {
        model
            .series
            .iter()
            .map(|s| SeriesMeta {
                id: s.id,
                label: s.label.clone(),
                y_axis: s.y_axis,
                stroke_color: s.wick_color,
            })
            .collect()
    }

    fn series_label(model: &Self::Model, series_id: SeriesId) -> Option<String> {
        model
            .series
            .iter()
            .find(|s| s.id == series_id)
            .map(|s| s.label.to_string())
    }

    fn series_y_axis(model: &Self::Model, series_id: SeriesId) -> YAxis {
        model
            .series
            .iter()
            .find(|s| s.id == series_id)
            .map(|s| s.y_axis)
            .unwrap_or(YAxis::Left)
    }

    fn cursor_readout(
        model: &Self::Model,
        args: PlotCursorReadoutArgs<'_>,
    ) -> Vec<PlotCursorReadoutRow> {
        let PlotCursorReadoutArgs {
            x,
            plot_size,
            view_bounds,
            x_scale,
            y_scale,
            scale_factor,
            hidden,
        } = args;

        if !x.is_finite() {
            return Vec::new();
        }

        let transform = PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let view_x = view_x_range(transform);
        let view_x = (view_x.start().is_finite() && view_x.end().is_finite()).then_some(view_x);
        let budget = device_point_budget(transform, scale_factor);

        let mut out: Vec<PlotCursorReadoutRow> = Vec::new();
        for s in &model.series {
            if hidden.contains(&s.id) {
                continue;
            }
            let y = cursor_readout_y_at_x(&*s.close_series, x, view_x.clone(), budget);
            out.push(PlotCursorReadoutRow {
                series_id: s.id,
                label: s.label.clone(),
                y_axis: s.y_axis,
                y,
            });
        }
        out
    }

    fn paint_paths<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        model: &Self::Model,
        args: PlotPaintArgs<'_>,
    ) -> Vec<(SeriesId, PathId, Color)> {
        let PlotPaintArgs {
            model_revision,
            plot,
            view_bounds,
            view_bounds_y2,
            view_bounds_y3,
            view_bounds_y4,
            x_scale,
            y_scale,
            y2_scale,
            y3_scale,
            y4_scale,
            style,
            hidden,
        } = args;

        let scale_factor_bits = cx.scale_factor.to_bits();
        let viewport_w_bits = plot.size.width.0.to_bits();
        let viewport_h_bits = plot.size.height.0.to_bits();
        let view_key_y1 = data_rect_key_scaled(view_bounds, x_scale, y_scale);
        let view_key_y2 = view_bounds_y2.map(|b| data_rect_key_scaled(b, x_scale, y2_scale));
        let view_key_y3 = view_bounds_y3.map(|b| data_rect_key_scaled(b, x_scale, y3_scale));
        let view_key_y4 = view_bounds_y4.map(|b| data_rect_key_scaled(b, x_scale, y4_scale));

        let view_key_for_axis = |axis: YAxis| match axis {
            YAxis::Left => view_key_y1,
            YAxis::Right => view_key_y2.unwrap_or(view_key_y1),
            YAxis::Right2 => view_key_y3.unwrap_or(view_key_y1),
            YAxis::Right3 => view_key_y4.unwrap_or(view_key_y1),
        };

        let series = &model.series;
        let series_count = series.len();

        if series_count == 0 {
            for cached in self.cached_paths.drain(..) {
                if let Some(id) = cached.wick_id {
                    cx.services.path().release(id);
                }
                if let Some(id) = cached.up_id {
                    cx.services.path().release(id);
                }
                if let Some(id) = cached.down_id {
                    cx.services.path().release(id);
                }
            }
            return Vec::new();
        }

        let cached_ok = self.cached_paths.len() == series_count
            && self.cached_paths.iter().enumerate().all(|(i, c)| {
                series.get(i).is_some_and(|s| {
                    let expected_view_key = view_key_for_axis(s.y_axis);
                    s.id == c.series_id && c.view_key == expected_view_key
                }) && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
                    && c.stroke_width == style.stroke_width
            });

        let default_up = Color {
            r: 0.25,
            g: 0.80,
            b: 0.45,
            a: 0.85,
        };
        let default_down = Color {
            r: 0.90,
            g: 0.35,
            b: 0.45,
            a: 0.85,
        };

        if cached_ok {
            let mut out: Vec<(SeriesId, PathId, Color)> =
                Vec::with_capacity(series_count.saturating_mul(3));
            for (i, s) in series.iter().enumerate() {
                if hidden.contains(&s.id) {
                    continue;
                }

                let wick =
                    s.wick_color
                        .unwrap_or(resolve_series_color(i, style, series_count, None));
                let up = s.up_fill.unwrap_or(default_up);
                let down = s.down_fill.unwrap_or(default_down);

                let Some(cached) = self.cached_paths.get(i) else {
                    continue;
                };
                if let Some(id) = cached.wick_id {
                    out.push((s.id, id, wick));
                }
                if let Some(id) = cached.up_id {
                    out.push((s.id, id, up));
                }
                if let Some(id) = cached.down_id {
                    out.push((s.id, id, down));
                }
            }
            return out;
        }

        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.wick_id {
                cx.services.path().release(id);
            }
            if let Some(id) = cached.up_id {
                cx.services.path().release(id);
            }
            if let Some(id) = cached.down_id {
                cx.services.path().release(id);
            }
        }

        let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size);
        let transform_y1 = PlotTransform {
            viewport: local_viewport,
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let transform_y2 = view_bounds_y2.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y2_scale,
        });
        let transform_y3 = view_bounds_y3.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y3_scale,
        });
        let transform_y4 = view_bounds_y4.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y4_scale,
        });

        let transform_for_axis = |axis: YAxis| match axis {
            YAxis::Left => transform_y1,
            YAxis::Right => transform_y2.unwrap_or(transform_y1),
            YAxis::Right2 => transform_y3.unwrap_or(transform_y1),
            YAxis::Right3 => transform_y4.unwrap_or(transform_y1),
        };
        let view_for_axis = |axis: YAxis| match axis {
            YAxis::Left => view_bounds,
            YAxis::Right => view_bounds_y2.unwrap_or(view_bounds),
            YAxis::Right2 => view_bounds_y3.unwrap_or(view_bounds),
            YAxis::Right3 => view_bounds_y4.unwrap_or(view_bounds),
        };

        let wick_style = PathStyle::Stroke(fret_core::StrokeStyle {
            width: style.stroke_width,
        });
        let fill_style = PathStyle::Fill(fret_core::FillStyle::default());
        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };

        let mut out: Vec<(SeriesId, PathId, Color)> =
            Vec::with_capacity(series_count.saturating_mul(3));
        self.cached_paths = Vec::with_capacity(series_count);

        for (series_index, s) in series.iter().enumerate() {
            let series_id = s.id;
            if hidden.contains(&series_id) {
                let view_key = view_key_for_axis(s.y_axis);
                self.cached_paths.push(CachedCandlestickPath {
                    wick_id: None,
                    up_id: None,
                    down_id: None,
                    series_id,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width: style.stroke_width,
                    view_key,
                    samples: Vec::new(),
                });
                continue;
            }

            let transform = transform_for_axis(s.y_axis);
            let view_key = view_key_for_axis(s.y_axis);
            let view = view_for_axis(s.y_axis);

            let samples = decimate_samples(transform, &*s.close_series, cx.scale_factor, series_id);
            let (wick_cmds, up_cmds, down_cmds) =
                candlestick_paths(s, transform, view, cx.scale_factor, style.stroke_width);

            let wick_id = if wick_cmds.is_empty() {
                None
            } else {
                let (id, _metrics) =
                    cx.services
                        .path()
                        .prepare(&wick_cmds, wick_style, constraints);
                Some(id)
            };
            let up_id = if up_cmds.is_empty() {
                None
            } else {
                let (id, _metrics) = cx
                    .services
                    .path()
                    .prepare(&up_cmds, fill_style, constraints);
                Some(id)
            };
            let down_id = if down_cmds.is_empty() {
                None
            } else {
                let (id, _metrics) =
                    cx.services
                        .path()
                        .prepare(&down_cmds, fill_style, constraints);
                Some(id)
            };

            self.cached_paths.push(CachedCandlestickPath {
                wick_id,
                up_id,
                down_id,
                series_id,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width: style.stroke_width,
                view_key,
                samples,
            });

            let wick = s.wick_color.unwrap_or(resolve_series_color(
                series_index,
                style,
                series_count,
                None,
            ));
            let up = s.up_fill.unwrap_or(default_up);
            let down = s.down_fill.unwrap_or(default_down);

            if let Some(id) = wick_id {
                out.push((series_id, id, wick));
            }
            if let Some(id) = up_id {
                out.push((series_id, id, up));
            }
            if let Some(id) = down_id {
                out.push((series_id, id, down));
            }
        }

        out
    }

    fn hit_test(&mut self, model: &Self::Model, args: PlotHitTestArgs<'_>) -> Option<PlotHover> {
        let PlotHitTestArgs {
            model_revision,
            plot_size,
            view_bounds,
            view_bounds_y2,
            view_bounds_y3,
            view_bounds_y4,
            x_scale,
            y_scale,
            y2_scale,
            y3_scale,
            y4_scale,
            scale_factor,
            local,
            style,
            hover_threshold,
            hidden,
            pinned,
        } = args;

        let threshold = hover_threshold.0.max(0.0);
        let threshold2 = threshold * threshold;

        let scale_factor_bits = scale_factor.to_bits();
        let viewport_w_bits = plot_size.width.0.to_bits();
        let viewport_h_bits = plot_size.height.0.to_bits();
        let view_key_y1 = data_rect_key_scaled(view_bounds, x_scale, y_scale);
        let view_key_y2 = view_bounds_y2.map(|b| data_rect_key_scaled(b, x_scale, y2_scale));
        let view_key_y3 = view_bounds_y3.map(|b| data_rect_key_scaled(b, x_scale, y3_scale));
        let view_key_y4 = view_bounds_y4.map(|b| data_rect_key_scaled(b, x_scale, y4_scale));

        let view_key_for_axis = |axis: YAxis| match axis {
            YAxis::Left => view_key_y1,
            YAxis::Right => view_key_y2.unwrap_or(view_key_y1),
            YAxis::Right2 => view_key_y3.unwrap_or(view_key_y1),
            YAxis::Right3 => view_key_y4.unwrap_or(view_key_y1),
        };

        let series: Vec<(SeriesId, YAxis, &dyn SeriesData)> = model
            .series
            .iter()
            .map(|s| (s.id, s.y_axis, &*s.close_series))
            .collect();

        let series_count = series.len();
        if series_count == 0 {
            return None;
        }

        let cached_ok = self.cached_paths.len() == series_count
            && self.cached_paths.iter().enumerate().all(|(i, c)| {
                series.get(i).is_some_and(|(id, axis, _data)| {
                    let expected_view_key = view_key_for_axis(*axis);
                    *id == c.series_id && c.view_key == expected_view_key
                }) && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
                    && c.stroke_width == style.stroke_width
            });

        let mut best: Option<(SamplePoint, f32)> = None;
        let mut consider_sample = |s: SamplePoint| {
            let dx = s.plot_px.x.0 - local.x.0;
            let dy = s.plot_px.y.0 - local.y.0;
            let d2 = dx * dx + dy * dy;
            if !d2.is_finite() {
                return;
            }
            if best.is_none_or(|b| d2 < b.1) {
                best = Some((s, d2));
            }
        };

        if cached_ok {
            for cached in self.cached_paths.iter() {
                if hidden.contains(&cached.series_id) {
                    continue;
                }
                if let Some(pinned) = pinned
                    && cached.series_id != pinned
                {
                    continue;
                }
                for s in cached.samples.iter().copied() {
                    consider_sample(s);
                }
            }
        } else {
            let transform_y1 = PlotTransform {
                viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
                data: view_bounds,
                x_scale,
                y_scale,
            };
            let transform_y2 = view_bounds_y2.map(|b| PlotTransform {
                viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
                data: b,
                x_scale,
                y_scale: y2_scale,
            });
            let transform_y3 = view_bounds_y3.map(|b| PlotTransform {
                viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
                data: b,
                x_scale,
                y_scale: y3_scale,
            });
            let transform_y4 = view_bounds_y4.map(|b| PlotTransform {
                viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
                data: b,
                x_scale,
                y_scale: y4_scale,
            });

            let transform_for_axis = |axis: YAxis| match axis {
                YAxis::Left => transform_y1,
                YAxis::Right => transform_y2.unwrap_or(transform_y1),
                YAxis::Right2 => transform_y3.unwrap_or(transform_y1),
                YAxis::Right3 => transform_y4.unwrap_or(transform_y1),
            };

            for (series_id, axis, data) in series.iter().copied() {
                if hidden.contains(&series_id) {
                    continue;
                }
                if let Some(pinned) = pinned
                    && pinned != series_id
                {
                    continue;
                }
                let transform = transform_for_axis(axis);
                for sample in decimate_samples(transform, data, scale_factor, series_id) {
                    consider_sample(sample);
                }
            }
        }

        best.and_then(|(s, d2)| {
            (d2 <= threshold2).then_some(PlotHover {
                series_id: s.series_id,
                index: s.index,
                data: s.data,
                plot_px: s.plot_px,
                value: None,
            })
        })
    }

    fn cleanup_resources(&mut self, services: &mut dyn UiServices) {
        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.wick_id {
                services.path().release(id);
            }
            if let Some(id) = cached.up_id {
                services.path().release(id);
            }
            if let Some(id) = cached.down_id {
                services.path().release(id);
            }
        }
    }
}

impl PlotLayer for StairsPlotLayer {
    type Model = LinePlotModel;

    fn data_bounds(model: &Self::Model) -> DataRect {
        model.data_bounds
    }

    fn data_bounds_y2(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y2
    }

    fn data_bounds_y3(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y3
    }

    fn data_bounds_y4(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y4
    }

    fn series_meta(model: &Self::Model) -> Vec<SeriesMeta> {
        model
            .series
            .iter()
            .map(|s| SeriesMeta {
                id: s.id,
                label: s.label.clone(),
                y_axis: s.y_axis,
                stroke_color: s.stroke_color,
            })
            .collect()
    }

    fn series_label(model: &Self::Model, series_id: SeriesId) -> Option<String> {
        model
            .series
            .iter()
            .find(|s| s.id == series_id)
            .map(|s| s.label.to_string())
    }

    fn series_y_axis(model: &Self::Model, series_id: SeriesId) -> YAxis {
        model
            .series
            .iter()
            .find(|s| s.id == series_id)
            .map(|s| s.y_axis)
            .unwrap_or(YAxis::Left)
    }

    fn cursor_readout(
        model: &Self::Model,
        args: PlotCursorReadoutArgs<'_>,
    ) -> Vec<PlotCursorReadoutRow> {
        let PlotCursorReadoutArgs {
            x,
            plot_size,
            view_bounds,
            x_scale,
            y_scale,
            scale_factor,
            hidden,
        } = args;

        if !x.is_finite() {
            return Vec::new();
        }

        let transform = PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let view_x = view_x_range(transform);
        let view_x = (view_x.start().is_finite() && view_x.end().is_finite()).then_some(view_x);
        let budget = device_point_budget(transform, scale_factor);

        let mut out: Vec<PlotCursorReadoutRow> = Vec::new();
        for s in &model.series {
            if hidden.contains(&s.id) {
                continue;
            }
            let y = cursor_readout_y_at_x(&*s.data, x, view_x.clone(), budget);
            out.push(PlotCursorReadoutRow {
                series_id: s.id,
                label: s.label.clone(),
                y_axis: s.y_axis,
                y,
            });
        }
        out
    }

    fn paint_paths<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        model: &Self::Model,
        args: PlotPaintArgs<'_>,
    ) -> Vec<(SeriesId, PathId, Color)> {
        let PlotPaintArgs {
            model_revision,
            plot,
            view_bounds,
            view_bounds_y2,
            view_bounds_y3,
            view_bounds_y4,
            x_scale,
            y_scale,
            y2_scale,
            y3_scale,
            y4_scale,
            style,
            hidden,
        } = args;

        let scale_factor_bits = cx.scale_factor.to_bits();
        let viewport_w_bits = plot.size.width.0.to_bits();
        let viewport_h_bits = plot.size.height.0.to_bits();
        let view_key_y1 = data_rect_key_scaled(view_bounds, x_scale, y_scale);
        let view_key_y2 = view_bounds_y2.map(|b| data_rect_key_scaled(b, x_scale, y2_scale));
        let view_key_y3 = view_bounds_y3.map(|b| data_rect_key_scaled(b, x_scale, y3_scale));
        let view_key_y4 = view_bounds_y4.map(|b| data_rect_key_scaled(b, x_scale, y4_scale));

        let view_key_for_axis = |axis: YAxis| match axis {
            YAxis::Left => view_key_y1,
            YAxis::Right => view_key_y2.unwrap_or(view_key_y1),
            YAxis::Right2 => view_key_y3.unwrap_or(view_key_y1),
            YAxis::Right3 => view_key_y4.unwrap_or(view_key_y1),
        };

        let series = &model.series;
        let series_count = series.len();

        if series_count == 0 {
            for cached in self.cached_paths.drain(..) {
                if let Some(id) = cached.id {
                    cx.services.path().release(id);
                }
            }
            return Vec::new();
        }

        let cached_ok = self.cached_paths.len() == series_count
            && self.cached_paths.iter().enumerate().all(|(i, c)| {
                series.get(i).is_some_and(|s| {
                    let expected_view_key = view_key_for_axis(s.y_axis);
                    s.id == c.series_id && c.view_key == expected_view_key
                }) && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
                    && c.stroke_width == style.stroke_width
            });

        if cached_ok {
            let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
            for (i, s) in series.iter().enumerate() {
                if hidden.contains(&s.id) {
                    continue;
                }
                let Some(id) = self.cached_paths.get(i).and_then(|c| c.id) else {
                    continue;
                };
                let style = series_style(s, i, style, series_count);
                out.push((s.id, id, style.stroke_color));
            }
            return out;
        }

        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.id {
                cx.services.path().release(id);
            }
        }

        let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size);
        let transform_y1 = PlotTransform {
            viewport: local_viewport,
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let transform_y2 = view_bounds_y2.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y2_scale,
        });
        let transform_y3 = view_bounds_y3.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y3_scale,
        });
        let transform_y4 = view_bounds_y4.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y4_scale,
        });

        let transform_for_axis = |axis: YAxis| match axis {
            YAxis::Left => transform_y1,
            YAxis::Right => transform_y2.unwrap_or(transform_y1),
            YAxis::Right2 => transform_y3.unwrap_or(transform_y1),
            YAxis::Right3 => transform_y4.unwrap_or(transform_y1),
        };

        let path_style = PathStyle::Stroke(fret_core::StrokeStyle {
            width: style.stroke_width,
        });
        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };

        let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
        self.cached_paths = Vec::with_capacity(series_count);

        for (series_index, s) in series.iter().enumerate() {
            let series_id = s.id;
            if hidden.contains(&series_id) {
                let view_key = view_key_for_axis(s.y_axis);
                self.cached_paths.push(CachedPath {
                    id: None,
                    series_id,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width: style.stroke_width,
                    view_key,
                    samples: Vec::new(),
                });
                continue;
            }

            let transform = transform_for_axis(s.y_axis);
            let view_key = view_key_for_axis(s.y_axis);

            let (polyline, samples) =
                decimate_polyline(transform, &*s.data, cx.scale_factor, series_id);
            let commands = step_commands_from_polyline(&polyline, self.step_mode);

            let id = if commands.is_empty() {
                None
            } else {
                let (id, _metrics) = cx
                    .services
                    .path()
                    .prepare(&commands, path_style, constraints);
                Some(id)
            };

            self.cached_paths.push(CachedPath {
                id,
                series_id,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width: style.stroke_width,
                view_key,
                samples,
            });

            if let Some(id) = id {
                let style = series_style(s, series_index, style, series_count);
                out.push((series_id, id, style.stroke_color));
            }
        }

        out
    }

    fn hit_test(&mut self, model: &Self::Model, args: PlotHitTestArgs<'_>) -> Option<PlotHover> {
        let series: Vec<(SeriesId, YAxis, &dyn SeriesData)> = model
            .series
            .iter()
            .map(|s| (s.id, s.y_axis, &*s.data))
            .collect();
        hit_test_series_data(&self.cached_paths, &series, args)
    }

    fn cleanup_resources(&mut self, services: &mut dyn UiServices) {
        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.id {
                services.path().release(id);
            }
        }
    }
}

impl PlotLayer for BarsPlotLayer {
    type Model = BarsPlotModel;

    fn data_bounds(model: &Self::Model) -> DataRect {
        model.data_bounds
    }

    fn data_bounds_y2(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y2
    }

    fn data_bounds_y3(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y3
    }

    fn data_bounds_y4(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y4
    }

    fn series_meta(model: &Self::Model) -> Vec<SeriesMeta> {
        model
            .series
            .iter()
            .map(|s| SeriesMeta {
                id: s.id,
                label: s.label.clone(),
                y_axis: s.y_axis,
                stroke_color: s.fill_color,
            })
            .collect()
    }

    fn series_label(model: &Self::Model, series_id: SeriesId) -> Option<String> {
        model
            .series
            .iter()
            .find(|s| s.id == series_id)
            .map(|s| s.label.to_string())
    }

    fn series_y_axis(model: &Self::Model, series_id: SeriesId) -> YAxis {
        model
            .series
            .iter()
            .find(|s| s.id == series_id)
            .map(|s| s.y_axis)
            .unwrap_or(YAxis::Left)
    }

    fn cursor_readout(
        model: &Self::Model,
        args: PlotCursorReadoutArgs<'_>,
    ) -> Vec<PlotCursorReadoutRow> {
        let PlotCursorReadoutArgs {
            x,
            plot_size,
            view_bounds,
            x_scale,
            y_scale,
            scale_factor,
            hidden,
        } = args;

        if !x.is_finite() {
            return Vec::new();
        }

        let transform = PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let view_x = view_x_range(transform);
        let view_x = (view_x.start().is_finite() && view_x.end().is_finite()).then_some(view_x);
        let budget = device_point_budget(transform, scale_factor);

        let mut out: Vec<PlotCursorReadoutRow> = Vec::new();
        for s in &model.series {
            if hidden.contains(&s.id) {
                continue;
            }
            let y = cursor_readout_y_at_x(&*s.data, x, view_x.clone(), budget);
            out.push(PlotCursorReadoutRow {
                series_id: s.id,
                label: s.label.clone(),
                y_axis: s.y_axis,
                y,
            });
        }
        out
    }

    fn paint_paths<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        model: &Self::Model,
        args: PlotPaintArgs<'_>,
    ) -> Vec<(SeriesId, PathId, Color)> {
        let PlotPaintArgs {
            model_revision,
            plot,
            view_bounds,
            view_bounds_y2,
            view_bounds_y3,
            view_bounds_y4,
            x_scale,
            y_scale,
            y2_scale,
            y3_scale,
            y4_scale,
            style,
            hidden,
        } = args;

        let scale_factor_bits = cx.scale_factor.to_bits();
        let viewport_w_bits = plot.size.width.0.to_bits();
        let viewport_h_bits = plot.size.height.0.to_bits();
        let view_key_y1 = data_rect_key_scaled(view_bounds, x_scale, y_scale);
        let view_key_y2 = view_bounds_y2.map(|b| data_rect_key_scaled(b, x_scale, y2_scale));
        let view_key_y3 = view_bounds_y3.map(|b| data_rect_key_scaled(b, x_scale, y3_scale));
        let view_key_y4 = view_bounds_y4.map(|b| data_rect_key_scaled(b, x_scale, y4_scale));

        let view_key_for_axis = |axis: YAxis| match axis {
            YAxis::Left => view_key_y1,
            YAxis::Right => view_key_y2.unwrap_or(view_key_y1),
            YAxis::Right2 => view_key_y3.unwrap_or(view_key_y1),
            YAxis::Right3 => view_key_y4.unwrap_or(view_key_y1),
        };

        let series = &model.series;
        let series_count = series.len();

        if series_count == 0 {
            for cached in self.cached_paths.drain(..) {
                if let Some(id) = cached.id {
                    cx.services.path().release(id);
                }
            }
            return Vec::new();
        }

        let cached_ok = self.cached_paths.len() == series_count
            && self.cached_paths.iter().enumerate().all(|(i, c)| {
                series.get(i).is_some_and(|s| {
                    let expected_view_key = view_key_for_axis(s.y_axis);
                    s.id == c.series_id && c.view_key == expected_view_key
                }) && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
                    && c.stroke_width == style.stroke_width
            });

        if cached_ok {
            let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
            for (i, s) in series.iter().enumerate() {
                if hidden.contains(&s.id) {
                    continue;
                }
                let Some(id) = self.cached_paths.get(i).and_then(|c| c.id) else {
                    continue;
                };
                let color = resolve_series_color(i, style, series_count, s.fill_color);
                out.push((s.id, id, color));
            }
            return out;
        }

        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.id {
                cx.services.path().release(id);
            }
        }

        let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size);
        let transform_y1 = PlotTransform {
            viewport: local_viewport,
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let transform_y2 = view_bounds_y2.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y2_scale,
        });
        let transform_y3 = view_bounds_y3.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y3_scale,
        });
        let transform_y4 = view_bounds_y4.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y4_scale,
        });

        let transform_for_axis = |axis: YAxis| match axis {
            YAxis::Left => transform_y1,
            YAxis::Right => transform_y2.unwrap_or(transform_y1),
            YAxis::Right2 => transform_y3.unwrap_or(transform_y1),
            YAxis::Right3 => transform_y4.unwrap_or(transform_y1),
        };

        let path_style = PathStyle::Fill(fret_core::FillStyle::default());
        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };

        let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
        self.cached_paths = Vec::with_capacity(series_count);

        for (series_index, s) in series.iter().enumerate() {
            let series_id = s.id;
            if hidden.contains(&series_id) {
                let view_key = view_key_for_axis(s.y_axis);
                self.cached_paths.push(CachedPath {
                    id: None,
                    series_id,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width: style.stroke_width,
                    view_key,
                    samples: Vec::new(),
                });
                continue;
            }

            let transform = transform_for_axis(s.y_axis);
            let view_key = view_key_for_axis(s.y_axis);

            let samples = decimate_points(transform, &*s.data, cx.scale_factor, series_id);
            let commands = bars_path_commands(transform, &samples, s.bar_width, s.baseline);

            let id = if commands.is_empty() {
                None
            } else {
                let (id, _metrics) = cx
                    .services
                    .path()
                    .prepare(&commands, path_style, constraints);
                Some(id)
            };

            self.cached_paths.push(CachedPath {
                id,
                series_id,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width: style.stroke_width,
                view_key,
                samples,
            });

            if let Some(id) = id {
                let color = resolve_series_color(series_index, style, series_count, s.fill_color);
                out.push((series_id, id, color));
            }
        }

        out
    }

    fn hit_test(&mut self, model: &Self::Model, args: PlotHitTestArgs<'_>) -> Option<PlotHover> {
        let series: Vec<(SeriesId, YAxis, &dyn SeriesData)> = model
            .series
            .iter()
            .map(|s| (s.id, s.y_axis, &*s.data))
            .collect();
        hit_test_series_data(&self.cached_paths, &series, args)
    }

    fn cleanup_resources(&mut self, services: &mut dyn UiServices) {
        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.id {
                services.path().release(id);
            }
        }
    }
}

impl PlotLayer for HeatmapPlotLayer {
    type Model = HeatmapPlotModel;

    fn data_bounds(model: &Self::Model) -> DataRect {
        model.data_bounds
    }

    fn series_meta(_model: &Self::Model) -> Vec<SeriesMeta> {
        Vec::new()
    }

    fn series_label(_model: &Self::Model, _series_id: SeriesId) -> Option<String> {
        Some("heatmap".to_string())
    }

    fn paint_paths<H: UiHost>(
        &mut self,
        _cx: &mut PaintCx<'_, H>,
        _model: &Self::Model,
        _args: PlotPaintArgs<'_>,
    ) -> Vec<(SeriesId, PathId, Color)> {
        Vec::new()
    }

    fn paint_quads<H: UiHost>(
        &mut self,
        _cx: &mut PaintCx<'_, H>,
        model: &Self::Model,
        args: PlotPaintArgs<'_>,
    ) -> Vec<PlotQuad> {
        let PlotPaintArgs {
            model_revision,
            plot,
            view_bounds,
            x_scale,
            y_scale,
            ..
        } = args;

        if model.cols == 0 || model.rows == 0 {
            self.cache_key = None;
            self.cached_quads.clear();
            return Vec::new();
        }

        let view_key = data_rect_key_scaled(view_bounds, x_scale, y_scale);
        let cache_key = HeatmapCacheKey {
            model_revision,
            view_key,
            cols: model.cols,
            rows: model.rows,
            viewport_w_bits: plot.size.width.0.to_bits(),
            viewport_h_bits: plot.size.height.0.to_bits(),
            value_min_bits: model.value_min.to_bits(),
            value_max_bits: model.value_max.to_bits(),
        };

        if self.cache_key == Some(cache_key) {
            return self.cached_quads.clone();
        }

        self.rebuild_mips_if_needed(model_revision, model);

        fn lerp(a: f32, b: f32, t: f32) -> f32 {
            a + (b - a) * t
        }

        fn heatmap_color(t: f32) -> Color {
            // Simple blue -> cyan -> green -> yellow -> red ramp (portable and predictable).
            let t = t.clamp(0.0, 1.0);
            let (r, g, b) = if t < 0.25 {
                let u = t / 0.25;
                (0.0, lerp(0.1, 1.0, u), 1.0)
            } else if t < 0.50 {
                let u = (t - 0.25) / 0.25;
                (0.0, 1.0, lerp(1.0, 0.0, u))
            } else if t < 0.75 {
                let u = (t - 0.50) / 0.25;
                (lerp(0.0, 1.0, u), 1.0, 0.0)
            } else {
                let u = (t - 0.75) / 0.25;
                (1.0, lerp(1.0, 0.0, u), 0.0)
            };
            Color { r, g, b, a: 1.0 }
        }

        let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size);
        let transform = PlotTransform {
            viewport: local_viewport,
            data: view_bounds,
            x_scale,
            y_scale,
        };

        let dx = (model.data_bounds.x_max - model.data_bounds.x_min) / (model.cols as f64);
        let dy = (model.data_bounds.y_max - model.data_bounds.y_min) / (model.rows as f64);
        if !dx.is_finite() || !dy.is_finite() || dx <= 0.0 || dy <= 0.0 {
            self.cache_key = Some(cache_key);
            self.cached_quads.clear();
            return Vec::new();
        }

        let clip_min_x = view_bounds.x_min.max(model.data_bounds.x_min);
        let clip_max_x = view_bounds.x_max.min(model.data_bounds.x_max);
        let clip_min_y = view_bounds.y_min.max(model.data_bounds.y_min);
        let clip_max_y = view_bounds.y_max.min(model.data_bounds.y_max);

        if clip_max_x <= clip_min_x || clip_max_y <= clip_min_y {
            self.cache_key = Some(cache_key);
            self.cached_quads.clear();
            return Vec::new();
        }

        let col0 = (((clip_min_x - model.data_bounds.x_min) / dx).floor() as isize)
            .clamp(0, model.cols.saturating_sub(1) as isize) as usize;
        let col1 = (((clip_max_x - model.data_bounds.x_min) / dx).ceil() as isize)
            .clamp(0, model.cols as isize) as usize;

        let row0 = (((clip_min_y - model.data_bounds.y_min) / dy).floor() as isize)
            .clamp(0, model.rows.saturating_sub(1) as isize) as usize;
        let row1 = (((clip_max_y - model.data_bounds.y_min) / dy).ceil() as isize)
            .clamp(0, model.rows as isize) as usize;

        let denom = (model.value_max - model.value_min).max(1.0e-12);

        let visible_cols = col1.saturating_sub(col0);
        let visible_rows = row1.saturating_sub(row0);
        if visible_cols == 0 || visible_rows == 0 {
            self.cache_key = Some(cache_key);
            self.cached_quads.clear();
            return Vec::new();
        }

        const MAX_HEATMAP_QUADS: usize = 50_000;
        let max_level = self.mips.len();
        let level = select_heatmap_mip_level(
            visible_cols,
            visible_rows,
            plot.size.width.0,
            plot.size.height.0,
            MAX_HEATMAP_QUADS,
            max_level,
        );

        let scale = 1usize << level.min(usize::BITS as usize - 1);
        let (grid_cols, grid_rows, values) = self.mip_level_values(level, model);

        let level_col0 = (col0 / scale).min(grid_cols);
        let level_col1 = ceil_div_usize(col1, scale).min(grid_cols);
        let level_row0 = (row0 / scale).min(grid_rows);
        let level_row1 = ceil_div_usize(row1, scale).min(grid_rows);

        if level_col1 <= level_col0 || level_row1 <= level_row0 {
            self.cache_key = Some(cache_key);
            self.cached_quads.clear();
            return Vec::new();
        }

        let quad_cols = level_col1.saturating_sub(level_col0);
        let quad_rows = level_row1.saturating_sub(level_row0);
        let mut quads: Vec<PlotQuad> = Vec::with_capacity(quad_cols.saturating_mul(quad_rows));

        for row_l in level_row0..level_row1 {
            let row_base0 = row_l.saturating_mul(scale);
            let row_base1 = (row_l.saturating_add(1).saturating_mul(scale)).min(model.rows);

            let y0 = model.data_bounds.y_min + (row_base0 as f64) * dy;
            let y1 = model.data_bounds.y_min + (row_base1 as f64) * dy;
            let (Some(py0), Some(py1)) = (transform.data_y_to_px(y0), transform.data_y_to_px(y1))
            else {
                continue;
            };
            let top = py0.0.min(py1.0);
            let bottom = py0.0.max(py1.0);
            if !top.is_finite() || !bottom.is_finite() || bottom <= top {
                continue;
            }

            for col_l in level_col0..level_col1 {
                let idx = row_l.saturating_mul(grid_cols).saturating_add(col_l);
                let Some(v) = values.get(idx).copied() else {
                    continue;
                };
                if !v.is_finite() {
                    continue;
                }

                let col_base0 = col_l.saturating_mul(scale);
                let col_base1 = (col_l.saturating_add(1).saturating_mul(scale)).min(model.cols);

                let x0 = model.data_bounds.x_min + (col_base0 as f64) * dx;
                let x1 = model.data_bounds.x_min + (col_base1 as f64) * dx;
                let (Some(px0), Some(px1)) =
                    (transform.data_x_to_px(x0), transform.data_x_to_px(x1))
                else {
                    continue;
                };
                let left = px0.0.min(px1.0);
                let right = px0.0.max(px1.0);
                if !left.is_finite() || !right.is_finite() || right <= left {
                    continue;
                }

                let t = ((v - model.value_min) / denom).clamp(0.0, 1.0);
                let color = heatmap_color(t);

                quads.push(PlotQuad {
                    rect_local: Rect::new(
                        Point::new(Px(left), Px(top)),
                        Size::new(Px(right - left), Px(bottom - top)),
                    ),
                    background: color,
                    order: DrawOrder(2),
                });

                if quads.len() >= MAX_HEATMAP_QUADS {
                    break;
                }
            }

            if quads.len() >= MAX_HEATMAP_QUADS {
                break;
            }
        }

        self.cache_key = Some(cache_key);
        self.cached_quads = quads.clone();
        quads
    }

    fn hit_test(&mut self, model: &Self::Model, args: PlotHitTestArgs<'_>) -> Option<PlotHover> {
        if model.cols == 0 || model.rows == 0 {
            return None;
        }

        let PlotHitTestArgs {
            plot_size,
            view_bounds,
            x_scale,
            y_scale,
            local,
            ..
        } = args;

        let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size);
        let transform = PlotTransform {
            viewport: local_viewport,
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let data = transform.px_to_data(local);
        if !data.x.is_finite() || !data.y.is_finite() {
            return None;
        }

        let dx = (model.data_bounds.x_max - model.data_bounds.x_min) / (model.cols as f64);
        let dy = (model.data_bounds.y_max - model.data_bounds.y_min) / (model.rows as f64);
        if !dx.is_finite() || !dy.is_finite() || dx <= 0.0 || dy <= 0.0 {
            return None;
        }

        let col_f = (data.x - model.data_bounds.x_min) / dx;
        let row_f = (data.y - model.data_bounds.y_min) / dy;
        if !col_f.is_finite() || !row_f.is_finite() {
            return None;
        }

        let col = col_f.floor() as isize;
        let row = row_f.floor() as isize;
        if col < 0 || row < 0 || col >= model.cols as isize || row >= model.rows as isize {
            return None;
        }
        let (col, row) = (col as usize, row as usize);

        let v = model.value_at(col, row)?;
        if !v.is_finite() {
            return None;
        }

        let cx = model.data_bounds.x_min + (col as f64 + 0.5) * dx;
        let cy = model.data_bounds.y_min + (row as f64 + 0.5) * dy;
        let plot_px = transform.data_to_px(DataPoint { x: cx, y: cy });

        Some(PlotHover {
            series_id: SeriesId::from_label("heatmap"),
            index: row.saturating_mul(model.cols).saturating_add(col),
            data: DataPoint { x: cx, y: cy },
            plot_px,
            value: Some(f64::from(v)),
        })
    }

    fn cleanup_resources(&mut self, _services: &mut dyn UiServices) {
        self.cache_key = None;
        self.cached_quads.clear();
        self.mip_key = None;
        self.mips.clear();
    }
}

impl PlotLayer for AreaPlotLayer {
    type Model = AreaPlotModel;

    fn data_bounds(model: &Self::Model) -> DataRect {
        model.data_bounds
    }

    fn data_bounds_y2(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y2
    }

    fn data_bounds_y3(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y3
    }

    fn data_bounds_y4(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y4
    }

    fn series_meta(model: &Self::Model) -> Vec<SeriesMeta> {
        model
            .series
            .iter()
            .map(|s| SeriesMeta {
                id: s.id,
                label: s.label.clone(),
                y_axis: s.y_axis,
                stroke_color: s.stroke_color.or(s.fill_color),
            })
            .collect()
    }

    fn series_label(model: &Self::Model, series_id: SeriesId) -> Option<String> {
        model
            .series
            .iter()
            .find(|s| s.id == series_id)
            .map(|s| s.label.to_string())
    }

    fn series_y_axis(model: &Self::Model, series_id: SeriesId) -> YAxis {
        model
            .series
            .iter()
            .find(|s| s.id == series_id)
            .map(|s| s.y_axis)
            .unwrap_or(YAxis::Left)
    }

    fn cursor_readout(
        model: &Self::Model,
        args: PlotCursorReadoutArgs<'_>,
    ) -> Vec<PlotCursorReadoutRow> {
        let PlotCursorReadoutArgs {
            x,
            plot_size,
            view_bounds,
            x_scale,
            y_scale,
            scale_factor,
            hidden,
        } = args;

        if !x.is_finite() {
            return Vec::new();
        }

        let transform = PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let view_x = view_x_range(transform);
        let view_x = (view_x.start().is_finite() && view_x.end().is_finite()).then_some(view_x);
        let budget = device_point_budget(transform, scale_factor);

        let mut out: Vec<PlotCursorReadoutRow> = Vec::new();
        for s in &model.series {
            if hidden.contains(&s.id) {
                continue;
            }
            let y = cursor_readout_y_at_x(&*s.data, x, view_x.clone(), budget);
            out.push(PlotCursorReadoutRow {
                series_id: s.id,
                label: s.label.clone(),
                y_axis: s.y_axis,
                y,
            });
        }
        out
    }

    fn paint_paths<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        model: &Self::Model,
        args: PlotPaintArgs<'_>,
    ) -> Vec<(SeriesId, PathId, Color)> {
        let PlotPaintArgs {
            model_revision,
            plot,
            view_bounds,
            view_bounds_y2,
            view_bounds_y3,
            view_bounds_y4,
            x_scale,
            y_scale,
            y2_scale,
            y3_scale,
            y4_scale,
            style,
            hidden,
        } = args;

        let scale_factor_bits = cx.scale_factor.to_bits();
        let viewport_w_bits = plot.size.width.0.to_bits();
        let viewport_h_bits = plot.size.height.0.to_bits();
        let view_key_y1 = data_rect_key_scaled(view_bounds, x_scale, y_scale);
        let view_key_y2 = view_bounds_y2.map(|b| data_rect_key_scaled(b, x_scale, y2_scale));
        let view_key_y3 = view_bounds_y3.map(|b| data_rect_key_scaled(b, x_scale, y3_scale));
        let view_key_y4 = view_bounds_y4.map(|b| data_rect_key_scaled(b, x_scale, y4_scale));

        let view_key_for_axis = |axis: YAxis| match axis {
            YAxis::Left => view_key_y1,
            YAxis::Right => view_key_y2.unwrap_or(view_key_y1),
            YAxis::Right2 => view_key_y3.unwrap_or(view_key_y1),
            YAxis::Right3 => view_key_y4.unwrap_or(view_key_y1),
        };

        let series = &model.series;
        let series_count = series.len();

        if series_count == 0 {
            for cached in self.cached_paths.drain(..) {
                if let Some(id) = cached.fill_id {
                    cx.services.path().release(id);
                }
                if let Some(id) = cached.stroke_id {
                    cx.services.path().release(id);
                }
            }
            return Vec::new();
        }

        let cached_ok = self.cached_paths.len() == series_count
            && self.cached_paths.iter().enumerate().all(|(i, c)| {
                let Some(s) = series.get(i) else {
                    return false;
                };
                s.id == c.series_id
                    && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
                    && c.stroke_width == style.stroke_width
                    && {
                        let expected_view_key = view_key_for_axis(s.y_axis);
                        c.view_key == expected_view_key
                    }
                    && c.baseline_bits == s.baseline.to_bits()
                    && c.fill_alpha_bits == s.fill_alpha.to_bits()
            });

        if cached_ok {
            let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count * 2);
            for (i, s) in series.iter().enumerate() {
                if hidden.contains(&s.id) {
                    continue;
                }

                let base_fill = resolve_series_color(i, style, series_count, s.fill_color);
                let base_stroke =
                    resolve_series_color(i, style, series_count, s.stroke_color.or(s.fill_color));

                let fill_alpha = s.fill_alpha.clamp(0.0, 1.0);
                let fill = Color {
                    a: (base_fill.a * fill_alpha).clamp(0.0, 1.0),
                    ..base_fill
                };

                if let Some(id) = self.cached_paths.get(i).and_then(|c| c.fill_id) {
                    out.push((s.id, id, fill));
                }
                if let Some(id) = self.cached_paths.get(i).and_then(|c| c.stroke_id) {
                    out.push((s.id, id, base_stroke));
                }
            }
            return out;
        }

        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.fill_id {
                cx.services.path().release(id);
            }
            if let Some(id) = cached.stroke_id {
                cx.services.path().release(id);
            }
        }

        let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size);
        let transform_y1 = PlotTransform {
            viewport: local_viewport,
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let transform_y2 = view_bounds_y2.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y2_scale,
        });
        let transform_y3 = view_bounds_y3.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y3_scale,
        });
        let transform_y4 = view_bounds_y4.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y4_scale,
        });

        let fill_style = PathStyle::Fill(fret_core::FillStyle::default());
        let stroke_style = PathStyle::Stroke(fret_core::StrokeStyle {
            width: style.stroke_width,
        });
        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };

        let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count * 2);
        self.cached_paths = Vec::with_capacity(series_count);

        for (series_index, s) in series.iter().enumerate() {
            let series_id = s.id;
            if hidden.contains(&series_id) {
                let view_key = view_key_for_axis(s.y_axis);
                self.cached_paths.push(CachedAreaPath {
                    fill_id: None,
                    stroke_id: None,
                    series_id,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width: style.stroke_width,
                    view_key,
                    baseline_bits: s.baseline.to_bits(),
                    fill_alpha_bits: s.fill_alpha.to_bits(),
                    samples: Vec::new(),
                });
                continue;
            }

            let (transform, view_bounds, view_key) = match s.y_axis {
                YAxis::Left => (transform_y1, view_bounds, view_key_y1),
                YAxis::Right => (
                    transform_y2.unwrap_or(transform_y1),
                    view_bounds_y2.unwrap_or(view_bounds),
                    view_key_y2.unwrap_or(view_key_y1),
                ),
                YAxis::Right2 => (
                    transform_y3.unwrap_or(transform_y1),
                    view_bounds_y3.unwrap_or(view_bounds),
                    view_key_y3.unwrap_or(view_key_y1),
                ),
                YAxis::Right3 => (
                    transform_y4.unwrap_or(transform_y1),
                    view_bounds_y4.unwrap_or(view_bounds),
                    view_key_y4.unwrap_or(view_key_y1),
                ),
            };

            let (line_commands, samples) =
                decimate_polyline(transform, &*s.data, cx.scale_factor, series_id);

            let baseline_y = transform.data_to_px(DataPoint {
                x: view_bounds.x_min,
                y: f64::from(s.baseline),
            });
            let fill_commands = area_fill_commands_from_polyline(&line_commands, baseline_y.y);

            let fill_id = if fill_commands.is_empty() {
                None
            } else {
                let (id, _metrics) =
                    cx.services
                        .path()
                        .prepare(&fill_commands, fill_style, constraints);
                Some(id)
            };

            let stroke_id = if line_commands.is_empty() || style.stroke_width.0 <= 0.0 {
                None
            } else {
                let (id, _metrics) =
                    cx.services
                        .path()
                        .prepare(&line_commands, stroke_style, constraints);
                Some(id)
            };

            self.cached_paths.push(CachedAreaPath {
                fill_id,
                stroke_id,
                series_id,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width: style.stroke_width,
                view_key,
                baseline_bits: s.baseline.to_bits(),
                fill_alpha_bits: s.fill_alpha.to_bits(),
                samples,
            });

            let base_fill = resolve_series_color(series_index, style, series_count, s.fill_color);
            let base_stroke = resolve_series_color(
                series_index,
                style,
                series_count,
                s.stroke_color.or(s.fill_color),
            );
            let fill_alpha = s.fill_alpha.clamp(0.0, 1.0);
            let fill = Color {
                a: (base_fill.a * fill_alpha).clamp(0.0, 1.0),
                ..base_fill
            };

            if let Some(id) = fill_id {
                out.push((series_id, id, fill));
            }
            if let Some(id) = stroke_id {
                out.push((series_id, id, base_stroke));
            }
        }

        out
    }

    fn hit_test(&mut self, model: &Self::Model, args: PlotHitTestArgs<'_>) -> Option<PlotHover> {
        let PlotHitTestArgs {
            model_revision,
            plot_size,
            view_bounds,
            view_bounds_y2,
            view_bounds_y3,
            view_bounds_y4,
            x_scale,
            y_scale,
            y2_scale,
            y3_scale,
            y4_scale,
            scale_factor,
            local,
            style,
            hover_threshold,
            hidden,
            pinned,
        } = args;

        let threshold = hover_threshold.0.max(0.0);
        let threshold2 = threshold * threshold;

        let scale_factor_bits = scale_factor.to_bits();
        let viewport_w_bits = plot_size.width.0.to_bits();
        let viewport_h_bits = plot_size.height.0.to_bits();
        let view_key_y1 = data_rect_key_scaled(view_bounds, x_scale, y_scale);
        let view_key_y2 = view_bounds_y2.map(|b| data_rect_key_scaled(b, x_scale, y2_scale));
        let view_key_y3 = view_bounds_y3.map(|b| data_rect_key_scaled(b, x_scale, y3_scale));
        let view_key_y4 = view_bounds_y4.map(|b| data_rect_key_scaled(b, x_scale, y4_scale));

        let view_key_for_axis = |axis: YAxis| match axis {
            YAxis::Left => view_key_y1,
            YAxis::Right => view_key_y2.unwrap_or(view_key_y1),
            YAxis::Right2 => view_key_y3.unwrap_or(view_key_y1),
            YAxis::Right3 => view_key_y4.unwrap_or(view_key_y1),
        };

        let series = &model.series;
        let series_count = series.len();
        if series_count == 0 {
            return None;
        }

        let cached_ok = self.cached_paths.len() == series_count
            && self.cached_paths.iter().enumerate().all(|(i, c)| {
                let Some(s) = series.get(i) else {
                    return false;
                };
                s.id == c.series_id
                    && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
                    && c.stroke_width == style.stroke_width
                    && {
                        let expected_view_key = view_key_for_axis(s.y_axis);
                        c.view_key == expected_view_key
                    }
            });

        let transform_y1 = PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let transform_y2 = view_bounds_y2.map(|b| PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: b,
            x_scale,
            y_scale: y2_scale,
        });
        let transform_y3 = view_bounds_y3.map(|b| PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: b,
            x_scale,
            y_scale: y3_scale,
        });
        let transform_y4 = view_bounds_y4.map(|b| PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: b,
            x_scale,
            y_scale: y4_scale,
        });

        let transform_for_axis = |axis: YAxis| match axis {
            YAxis::Left => transform_y1,
            YAxis::Right => transform_y2.unwrap_or(transform_y1),
            YAxis::Right2 => transform_y3.unwrap_or(transform_y1),
            YAxis::Right3 => transform_y4.unwrap_or(transform_y1),
        };

        let mut best: Option<(PlotHover, f32)> = None;

        if cached_ok {
            for (cached, s) in self.cached_paths.iter().zip(series.iter()) {
                if hidden.contains(&cached.series_id) {
                    continue;
                }
                if let Some(pinned) = pinned
                    && cached.series_id != pinned
                {
                    continue;
                }
                let transform = transform_for_axis(s.y_axis);

                let mut prev: Option<SamplePoint> = None;
                for sp in cached.samples.iter().copied() {
                    if let Some(p) = prev {
                        consider_hover_segment(&mut best, local, p, sp, transform);
                    }
                    consider_hover_point(&mut best, local, sp);
                    prev = Some(sp);
                }
            }
        } else {
            for s in series {
                if hidden.contains(&s.id) {
                    continue;
                }
                if let Some(pinned) = pinned
                    && pinned != s.id
                {
                    continue;
                }
                let transform = transform_for_axis(s.y_axis);
                let samples = decimate_samples(transform, &*s.data, scale_factor, s.id);
                let mut prev: Option<SamplePoint> = None;
                for sp in samples.into_iter() {
                    if let Some(p) = prev {
                        consider_hover_segment(&mut best, local, p, sp, transform);
                    }
                    consider_hover_point(&mut best, local, sp);
                    prev = Some(sp);
                }
            }
        }

        best.and_then(|(hover, d2)| (d2 <= threshold2).then_some(hover))
    }

    fn cleanup_resources(&mut self, services: &mut dyn UiServices) {
        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.fill_id {
                services.path().release(id);
            }
            if let Some(id) = cached.stroke_id {
                services.path().release(id);
            }
        }
    }
}

impl PlotLayer for ShadedPlotLayer {
    type Model = ShadedPlotModel;

    fn data_bounds(model: &Self::Model) -> DataRect {
        model.data_bounds
    }

    fn data_bounds_y2(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y2
    }

    fn data_bounds_y3(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y3
    }

    fn data_bounds_y4(model: &Self::Model) -> Option<DataRect> {
        model.data_bounds_y4
    }

    fn series_meta(model: &Self::Model) -> Vec<SeriesMeta> {
        model
            .series
            .iter()
            .map(|s| SeriesMeta {
                id: s.id,
                label: s.label.clone(),
                y_axis: s.y_axis,
                stroke_color: s.stroke_color.or(s.fill_color),
            })
            .collect()
    }

    fn series_label(model: &Self::Model, series_id: SeriesId) -> Option<String> {
        model
            .series
            .iter()
            .find(|s| s.id == series_id)
            .map(|s| s.label.to_string())
    }

    fn series_y_axis(model: &Self::Model, series_id: SeriesId) -> YAxis {
        model
            .series
            .iter()
            .find(|s| s.id == series_id)
            .map(|s| s.y_axis)
            .unwrap_or(YAxis::Left)
    }

    fn cursor_readout(
        model: &Self::Model,
        args: PlotCursorReadoutArgs<'_>,
    ) -> Vec<PlotCursorReadoutRow> {
        let PlotCursorReadoutArgs {
            x,
            plot_size,
            view_bounds,
            x_scale,
            y_scale,
            scale_factor,
            hidden,
        } = args;

        if !x.is_finite() {
            return Vec::new();
        }

        let transform = PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let view_x = view_x_range(transform);
        let view_x = (view_x.start().is_finite() && view_x.end().is_finite()).then_some(view_x);
        let budget = device_point_budget(transform, scale_factor);

        let mut out: Vec<PlotCursorReadoutRow> = Vec::new();
        for s in &model.series {
            if hidden.contains(&s.id) {
                continue;
            }

            let upper_y = cursor_readout_y_at_x(&*s.upper, x, view_x.clone(), budget);
            let lower_y = cursor_readout_y_at_x(&*s.lower, x, view_x.clone(), budget);

            out.push(PlotCursorReadoutRow {
                series_id: s.id,
                label: Arc::from(format!("{} (upper)", s.label)),
                y_axis: s.y_axis,
                y: upper_y,
            });
            out.push(PlotCursorReadoutRow {
                series_id: s.id,
                label: Arc::from(format!("{} (lower)", s.label)),
                y_axis: s.y_axis,
                y: lower_y,
            });
        }
        out
    }

    fn paint_paths<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        model: &Self::Model,
        args: PlotPaintArgs<'_>,
    ) -> Vec<(SeriesId, PathId, Color)> {
        let PlotPaintArgs {
            model_revision,
            plot,
            view_bounds,
            view_bounds_y2,
            view_bounds_y3,
            view_bounds_y4,
            x_scale,
            y_scale,
            y2_scale,
            y3_scale,
            y4_scale,
            style,
            hidden,
        } = args;

        let scale_factor_bits = cx.scale_factor.to_bits();
        let viewport_w_bits = plot.size.width.0.to_bits();
        let viewport_h_bits = plot.size.height.0.to_bits();
        let view_key_y1 = data_rect_key_scaled(view_bounds, x_scale, y_scale);
        let view_key_y2 = view_bounds_y2.map(|b| data_rect_key_scaled(b, x_scale, y2_scale));
        let view_key_y3 = view_bounds_y3.map(|b| data_rect_key_scaled(b, x_scale, y3_scale));
        let view_key_y4 = view_bounds_y4.map(|b| data_rect_key_scaled(b, x_scale, y4_scale));

        let view_key_for_axis = |axis: YAxis| match axis {
            YAxis::Left => view_key_y1,
            YAxis::Right => view_key_y2.unwrap_or(view_key_y1),
            YAxis::Right2 => view_key_y3.unwrap_or(view_key_y1),
            YAxis::Right3 => view_key_y4.unwrap_or(view_key_y1),
        };

        let series = &model.series;
        let series_count = series.len();

        if series_count == 0 {
            for cached in self.cached_paths.drain(..) {
                if let Some(id) = cached.fill_id {
                    cx.services.path().release(id);
                }
                if let Some(id) = cached.upper_stroke_id {
                    cx.services.path().release(id);
                }
                if let Some(id) = cached.lower_stroke_id {
                    cx.services.path().release(id);
                }
            }
            return Vec::new();
        }

        let cached_ok = self.cached_paths.len() == series_count
            && self.cached_paths.iter().enumerate().all(|(i, c)| {
                let Some(s) = series.get(i) else {
                    return false;
                };
                s.id == c.series_id
                    && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
                    && c.stroke_width == style.stroke_width
                    && {
                        let expected_view_key = view_key_for_axis(s.y_axis);
                        c.view_key == expected_view_key
                    }
                    && c.fill_alpha_bits == s.fill_alpha.to_bits()
            });

        if cached_ok {
            let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count * 3);
            for (i, s) in series.iter().enumerate() {
                if hidden.contains(&s.id) {
                    continue;
                }

                let base_fill = resolve_series_color(i, style, series_count, s.fill_color);
                let base_stroke =
                    resolve_series_color(i, style, series_count, s.stroke_color.or(s.fill_color));

                let fill_alpha = s.fill_alpha.clamp(0.0, 1.0);
                let fill = Color {
                    a: (base_fill.a * fill_alpha).clamp(0.0, 1.0),
                    ..base_fill
                };

                if let Some(id) = self.cached_paths.get(i).and_then(|c| c.fill_id) {
                    out.push((s.id, id, fill));
                }
                if let Some(id) = self.cached_paths.get(i).and_then(|c| c.upper_stroke_id) {
                    out.push((s.id, id, base_stroke));
                }
                if let Some(id) = self.cached_paths.get(i).and_then(|c| c.lower_stroke_id) {
                    out.push((s.id, id, base_stroke));
                }
            }
            return out;
        }

        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.fill_id {
                cx.services.path().release(id);
            }
            if let Some(id) = cached.upper_stroke_id {
                cx.services.path().release(id);
            }
            if let Some(id) = cached.lower_stroke_id {
                cx.services.path().release(id);
            }
        }

        let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size);
        let transform_y1 = PlotTransform {
            viewport: local_viewport,
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let transform_y2 = view_bounds_y2.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y2_scale,
        });
        let transform_y3 = view_bounds_y3.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y3_scale,
        });
        let transform_y4 = view_bounds_y4.map(|b| PlotTransform {
            viewport: local_viewport,
            data: b,
            x_scale,
            y_scale: y4_scale,
        });

        let transform_for_axis = |axis: YAxis| match axis {
            YAxis::Left => transform_y1,
            YAxis::Right => transform_y2.unwrap_or(transform_y1),
            YAxis::Right2 => transform_y3.unwrap_or(transform_y1),
            YAxis::Right3 => transform_y4.unwrap_or(transform_y1),
        };

        let fill_style = PathStyle::Fill(fret_core::FillStyle::default());
        let stroke_style = PathStyle::Stroke(fret_core::StrokeStyle {
            width: style.stroke_width,
        });
        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };

        let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count * 3);
        self.cached_paths = Vec::with_capacity(series_count);

        for (series_index, s) in series.iter().enumerate() {
            let series_id = s.id;
            if hidden.contains(&series_id) {
                let view_key = view_key_for_axis(s.y_axis);
                self.cached_paths.push(CachedShadedPath {
                    fill_id: None,
                    upper_stroke_id: None,
                    lower_stroke_id: None,
                    series_id,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width: style.stroke_width,
                    view_key,
                    fill_alpha_bits: s.fill_alpha.to_bits(),
                    samples: Vec::new(),
                });
                continue;
            }

            let transform = transform_for_axis(s.y_axis);
            let view_key = view_key_for_axis(s.y_axis);

            let (fill_commands, upper_line_commands, lower_line_commands, samples) =
                decimate_shaded_band(transform, &*s.upper, &*s.lower, cx.scale_factor, series_id);

            let fill_id = if fill_commands.is_empty() {
                None
            } else {
                let (id, _metrics) =
                    cx.services
                        .path()
                        .prepare(&fill_commands, fill_style, constraints);
                Some(id)
            };

            let upper_stroke_id = if upper_line_commands.is_empty() || style.stroke_width.0 <= 0.0 {
                None
            } else {
                let (id, _metrics) =
                    cx.services
                        .path()
                        .prepare(&upper_line_commands, stroke_style, constraints);
                Some(id)
            };

            let lower_stroke_id = if lower_line_commands.is_empty() || style.stroke_width.0 <= 0.0 {
                None
            } else {
                let (id, _metrics) =
                    cx.services
                        .path()
                        .prepare(&lower_line_commands, stroke_style, constraints);
                Some(id)
            };

            self.cached_paths.push(CachedShadedPath {
                fill_id,
                upper_stroke_id,
                lower_stroke_id,
                series_id,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width: style.stroke_width,
                view_key,
                fill_alpha_bits: s.fill_alpha.to_bits(),
                samples,
            });

            let base_fill = resolve_series_color(series_index, style, series_count, s.fill_color);
            let base_stroke = resolve_series_color(
                series_index,
                style,
                series_count,
                s.stroke_color.or(s.fill_color),
            );

            let fill_alpha = s.fill_alpha.clamp(0.0, 1.0);
            let fill = Color {
                a: (base_fill.a * fill_alpha).clamp(0.0, 1.0),
                ..base_fill
            };

            if let Some(id) = fill_id {
                out.push((series_id, id, fill));
            }
            if let Some(id) = upper_stroke_id {
                out.push((series_id, id, base_stroke));
            }
            if let Some(id) = lower_stroke_id {
                out.push((series_id, id, base_stroke));
            }
        }

        out
    }

    fn hit_test(&mut self, model: &Self::Model, args: PlotHitTestArgs<'_>) -> Option<PlotHover> {
        let PlotHitTestArgs {
            model_revision,
            plot_size,
            view_bounds,
            view_bounds_y2,
            view_bounds_y3,
            view_bounds_y4,
            x_scale,
            y_scale,
            y2_scale,
            y3_scale,
            y4_scale,
            scale_factor,
            local,
            style,
            hover_threshold,
            hidden,
            pinned,
        } = args;

        let threshold = hover_threshold.0.max(0.0);
        let threshold2 = threshold * threshold;

        let scale_factor_bits = scale_factor.to_bits();
        let viewport_w_bits = plot_size.width.0.to_bits();
        let viewport_h_bits = plot_size.height.0.to_bits();
        let view_key_y1 = data_rect_key_scaled(view_bounds, x_scale, y_scale);
        let view_key_y2 = view_bounds_y2.map(|b| data_rect_key_scaled(b, x_scale, y2_scale));
        let view_key_y3 = view_bounds_y3.map(|b| data_rect_key_scaled(b, x_scale, y3_scale));
        let view_key_y4 = view_bounds_y4.map(|b| data_rect_key_scaled(b, x_scale, y4_scale));

        let view_key_for_axis = |axis: YAxis| match axis {
            YAxis::Left => view_key_y1,
            YAxis::Right => view_key_y2.unwrap_or(view_key_y1),
            YAxis::Right2 => view_key_y3.unwrap_or(view_key_y1),
            YAxis::Right3 => view_key_y4.unwrap_or(view_key_y1),
        };

        let series = &model.series;
        let series_count = series.len();
        if series_count == 0 {
            return None;
        }

        let cached_ok = self.cached_paths.len() == series_count
            && self.cached_paths.iter().enumerate().all(|(i, c)| {
                let Some(s) = series.get(i) else {
                    return false;
                };
                s.id == c.series_id
                    && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
                    && c.stroke_width == style.stroke_width
                    && {
                        let expected_view_key = view_key_for_axis(s.y_axis);
                        c.view_key == expected_view_key
                    }
            });

        let transform_y1 = PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let transform_y2 = view_bounds_y2.map(|b| PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: b,
            x_scale,
            y_scale: y2_scale,
        });
        let transform_y3 = view_bounds_y3.map(|b| PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: b,
            x_scale,
            y_scale: y3_scale,
        });
        let transform_y4 = view_bounds_y4.map(|b| PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: b,
            x_scale,
            y_scale: y4_scale,
        });

        let transform_for_axis = |axis: YAxis| match axis {
            YAxis::Left => transform_y1,
            YAxis::Right => transform_y2.unwrap_or(transform_y1),
            YAxis::Right2 => transform_y3.unwrap_or(transform_y1),
            YAxis::Right3 => transform_y4.unwrap_or(transform_y1),
        };

        let mut best: Option<(PlotHover, f32)> = None;

        if cached_ok {
            for (cached, s) in self.cached_paths.iter().zip(series.iter()) {
                if hidden.contains(&cached.series_id) {
                    continue;
                }
                if let Some(pinned) = pinned
                    && cached.series_id != pinned
                {
                    continue;
                }

                let transform = transform_for_axis(s.y_axis);

                let mut prev: Option<SamplePoint> = None;
                for sp in cached.samples.iter().copied() {
                    if let Some(p) = prev {
                        consider_hover_segment(&mut best, local, p, sp, transform);
                    }
                    consider_hover_point(&mut best, local, sp);
                    prev = Some(sp);
                }
            }
        } else {
            for s in series {
                if hidden.contains(&s.id) {
                    continue;
                }
                if let Some(pinned) = pinned
                    && pinned != s.id
                {
                    continue;
                }

                let transform = transform_for_axis(s.y_axis);
                let (_fill, _upper, _lower, samples) =
                    decimate_shaded_band(transform, &*s.upper, &*s.lower, scale_factor, s.id);
                let mut prev: Option<SamplePoint> = None;
                for sp in samples.into_iter() {
                    if let Some(p) = prev {
                        consider_hover_segment(&mut best, local, p, sp, transform);
                    }
                    consider_hover_point(&mut best, local, sp);
                    prev = Some(sp);
                }
            }
        }

        best.and_then(|(hover, d2)| (d2 <= threshold2).then_some(hover))
    }

    fn cleanup_resources(&mut self, services: &mut dyn UiServices) {
        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.fill_id {
                services.path().release(id);
            }
            if let Some(id) = cached.upper_stroke_id {
                services.path().release(id);
            }
            if let Some(id) = cached.lower_stroke_id {
                services.path().release(id);
            }
        }
    }
}

fn hash_value<T: Hash>(v: &T) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    v.hash(&mut h);
    h.finish()
}

fn point_segment_closest_px(p: Point, a: Point, b: Point) -> Option<(Point, f32)> {
    let ax = a.x.0;
    let ay = a.y.0;
    let bx = b.x.0;
    let by = b.y.0;
    let px = p.x.0;
    let py = p.y.0;

    if !ax.is_finite() || !ay.is_finite() || !bx.is_finite() || !by.is_finite() {
        return None;
    }
    if !px.is_finite() || !py.is_finite() {
        return None;
    }

    let abx = bx - ax;
    let aby = by - ay;
    let apx = px - ax;
    let apy = py - ay;

    let denom = abx * abx + aby * aby;
    if !denom.is_finite() || denom <= 0.0 {
        return Some((a, 0.0));
    }

    let mut t = (apx * abx + apy * aby) / denom;
    if !t.is_finite() {
        return None;
    }
    t = t.clamp(0.0, 1.0);

    let qx = ax + abx * t;
    let qy = ay + aby * t;
    if !qx.is_finite() || !qy.is_finite() {
        return None;
    }

    Some((Point::new(Px(qx), Px(qy)), t))
}

fn consider_hover_point(best: &mut Option<(PlotHover, f32)>, local: Point, s: SamplePoint) {
    let dx = s.plot_px.x.0 - local.x.0;
    let dy = s.plot_px.y.0 - local.y.0;
    let d2 = dx * dx + dy * dy;
    if !d2.is_finite() {
        return;
    }

    let hover = PlotHover {
        series_id: s.series_id,
        index: s.index,
        data: s.data,
        plot_px: s.plot_px,
        value: None,
    };

    if best.is_none_or(|b| d2 < b.1) {
        *best = Some((hover, d2));
    }
}

fn consider_hover_segment(
    best: &mut Option<(PlotHover, f32)>,
    local: Point,
    prev: SamplePoint,
    curr: SamplePoint,
    transform: PlotTransform,
) {
    if !curr.connects_to_prev {
        return;
    }

    let Some((q, t)) = point_segment_closest_px(local, prev.plot_px, curr.plot_px) else {
        return;
    };

    let dx = q.x.0 - local.x.0;
    let dy = q.y.0 - local.y.0;
    let d2 = dx * dx + dy * dy;
    if !d2.is_finite() {
        return;
    }

    let idx = if t <= 0.5 { prev.index } else { curr.index };
    let data = transform.px_to_data(q);
    if !data.x.is_finite() || !data.y.is_finite() {
        return;
    }

    if best.is_none_or(|b| d2 < b.1) {
        *best = Some((
            PlotHover {
                series_id: curr.series_id,
                index: idx,
                data,
                plot_px: q,
                value: None,
            },
            d2,
        ));
    }
}

fn hit_test_polyline_series_data(
    cached_paths: &[CachedPath],
    series: &[(SeriesId, YAxis, &dyn SeriesData)],
    args: PlotHitTestArgs<'_>,
) -> Option<PlotHover> {
    let PlotHitTestArgs {
        model_revision,
        plot_size,
        view_bounds,
        view_bounds_y2,
        view_bounds_y3,
        view_bounds_y4,
        x_scale,
        y_scale,
        y2_scale,
        y3_scale,
        y4_scale,
        scale_factor,
        local,
        style,
        hover_threshold,
        hidden,
        pinned,
    } = args;

    let threshold = hover_threshold.0.max(0.0);
    let threshold2 = threshold * threshold;

    let scale_factor_bits = scale_factor.to_bits();
    let viewport_w_bits = plot_size.width.0.to_bits();
    let viewport_h_bits = plot_size.height.0.to_bits();
    let view_key_y1 = data_rect_key_scaled(view_bounds, x_scale, y_scale);
    let view_key_y2 = view_bounds_y2.map(|b| data_rect_key_scaled(b, x_scale, y2_scale));
    let view_key_y3 = view_bounds_y3.map(|b| data_rect_key_scaled(b, x_scale, y3_scale));
    let view_key_y4 = view_bounds_y4.map(|b| data_rect_key_scaled(b, x_scale, y4_scale));

    let view_key_for_axis = |axis: YAxis| match axis {
        YAxis::Left => view_key_y1,
        YAxis::Right => view_key_y2.unwrap_or(view_key_y1),
        YAxis::Right2 => view_key_y3.unwrap_or(view_key_y1),
        YAxis::Right3 => view_key_y4.unwrap_or(view_key_y1),
    };

    let series_count = series.len();
    if series_count == 0 {
        return None;
    }

    let cached_ok = cached_paths.len() == series_count
        && cached_paths.iter().enumerate().all(|(i, c)| {
            series.get(i).is_some_and(|(id, axis, _data)| {
                let expected_view_key = view_key_for_axis(*axis);
                *id == c.series_id && c.view_key == expected_view_key
            }) && c.model_revision == model_revision
                && c.scale_factor_bits == scale_factor_bits
                && c.viewport_w_bits == viewport_w_bits
                && c.viewport_h_bits == viewport_h_bits
                && c.stroke_width == style.stroke_width
        });

    let transform_y1 = PlotTransform {
        viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
        data: view_bounds,
        x_scale,
        y_scale,
    };
    let transform_y2 = view_bounds_y2.map(|b| PlotTransform {
        viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
        data: b,
        x_scale,
        y_scale: y2_scale,
    });
    let transform_y3 = view_bounds_y3.map(|b| PlotTransform {
        viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
        data: b,
        x_scale,
        y_scale: y3_scale,
    });
    let transform_y4 = view_bounds_y4.map(|b| PlotTransform {
        viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
        data: b,
        x_scale,
        y_scale: y4_scale,
    });

    let transform_for_axis = |axis: YAxis| match axis {
        YAxis::Left => transform_y1,
        YAxis::Right => transform_y2.unwrap_or(transform_y1),
        YAxis::Right2 => transform_y3.unwrap_or(transform_y1),
        YAxis::Right3 => transform_y4.unwrap_or(transform_y1),
    };

    let mut best: Option<(PlotHover, f32)> = None;

    if cached_ok {
        for (cached, (series_id, axis, _data)) in cached_paths.iter().zip(series.iter().copied()) {
            if hidden.contains(&cached.series_id) {
                continue;
            }
            if let Some(pinned) = pinned
                && cached.series_id != pinned
            {
                continue;
            }
            let transform = transform_for_axis(axis);
            debug_assert_eq!(cached.series_id, series_id);

            let mut prev: Option<SamplePoint> = None;
            for s in cached.samples.iter().copied() {
                if let Some(p) = prev {
                    consider_hover_segment(&mut best, local, p, s, transform);
                }
                consider_hover_point(&mut best, local, s);
                prev = Some(s);
            }
        }
    } else {
        for (series_id, axis, data) in series.iter().copied() {
            if hidden.contains(&series_id) {
                continue;
            }
            if let Some(pinned) = pinned
                && pinned != series_id
            {
                continue;
            }
            let transform = if axis == YAxis::Right {
                transform_y2.unwrap_or(transform_y1)
            } else {
                transform_y1
            };

            // Fast path for monotonic-X slice-backed series: only consider a small X window around
            // the cursor instead of scanning all visible samples.
            if data.is_sorted_by_x()
                && let Some(slice) = data.as_slice()
                && threshold.is_finite()
                && threshold > 0.0
            {
                let left = Px(local.x.0 - threshold);
                let right = Px(local.x.0 + threshold);
                let x0 = transform.px_to_data(Point::new(left, local.y)).x;
                let x1 = transform.px_to_data(Point::new(right, local.y)).x;
                if x0.is_finite() && x1.is_finite() {
                    let x_min = x0.min(x1);
                    let x_max = x0.max(x1);
                    let (base, window) = visible_sorted_slice(slice, x_min, x_max);

                    let mut prev: Option<SamplePoint> = None;
                    for (i, p) in window.iter().copied().enumerate() {
                        if !p.x.is_finite() || !p.y.is_finite() {
                            prev = None;
                            continue;
                        }
                        let plot_px = transform.data_to_px(p);
                        if !plot_px.x.0.is_finite() || !plot_px.y.0.is_finite() {
                            prev = None;
                            continue;
                        }

                        let s = SamplePoint {
                            series_id,
                            index: base + i,
                            data: p,
                            plot_px,
                            connects_to_prev: prev.is_some(),
                        };
                        if let Some(p) = prev {
                            consider_hover_segment(&mut best, local, p, s, transform);
                        }
                        consider_hover_point(&mut best, local, s);
                        prev = Some(s);
                    }
                    continue;
                }
            }

            let samples = decimate_samples(transform, data, scale_factor, series_id);
            let mut prev: Option<SamplePoint> = None;
            for s in samples.into_iter() {
                if let Some(p) = prev {
                    consider_hover_segment(&mut best, local, p, s, transform);
                }
                consider_hover_point(&mut best, local, s);
                prev = Some(s);
            }
        }
    }

    best.and_then(|(hover, d2)| (d2 <= threshold2).then_some(hover))
}

fn hit_test_series_data(
    cached_paths: &[CachedPath],
    series: &[(SeriesId, YAxis, &dyn SeriesData)],
    args: PlotHitTestArgs<'_>,
) -> Option<PlotHover> {
    let PlotHitTestArgs {
        model_revision,
        plot_size,
        view_bounds,
        view_bounds_y2,
        view_bounds_y3,
        view_bounds_y4,
        x_scale,
        y_scale,
        y2_scale,
        y3_scale,
        y4_scale,
        scale_factor,
        local,
        style,
        hover_threshold,
        hidden,
        pinned,
    } = args;

    let threshold = hover_threshold.0.max(0.0);
    let threshold2 = threshold * threshold;

    let scale_factor_bits = scale_factor.to_bits();
    let viewport_w_bits = plot_size.width.0.to_bits();
    let viewport_h_bits = plot_size.height.0.to_bits();
    let view_key_y1 = data_rect_key_scaled(view_bounds, x_scale, y_scale);
    let view_key_y2 = view_bounds_y2.map(|b| data_rect_key_scaled(b, x_scale, y2_scale));
    let view_key_y3 = view_bounds_y3.map(|b| data_rect_key_scaled(b, x_scale, y3_scale));
    let view_key_y4 = view_bounds_y4.map(|b| data_rect_key_scaled(b, x_scale, y4_scale));

    let view_key_for_axis = |axis: YAxis| match axis {
        YAxis::Left => view_key_y1,
        YAxis::Right => view_key_y2.unwrap_or(view_key_y1),
        YAxis::Right2 => view_key_y3.unwrap_or(view_key_y1),
        YAxis::Right3 => view_key_y4.unwrap_or(view_key_y1),
    };

    let series_count = series.len();
    if series_count == 0 {
        return None;
    }

    let cached_ok = cached_paths.len() == series_count
        && cached_paths.iter().enumerate().all(|(i, c)| {
            series.get(i).is_some_and(|(id, axis, _data)| {
                let expected_view_key = view_key_for_axis(*axis);
                *id == c.series_id && c.view_key == expected_view_key
            }) && c.model_revision == model_revision
                && c.scale_factor_bits == scale_factor_bits
                && c.viewport_w_bits == viewport_w_bits
                && c.viewport_h_bits == viewport_h_bits
                && c.stroke_width == style.stroke_width
        });

    let mut best: Option<(SamplePoint, f32)> = None;
    let mut consider_sample = |s: SamplePoint| {
        let dx = s.plot_px.x.0 - local.x.0;
        let dy = s.plot_px.y.0 - local.y.0;
        let d2 = dx * dx + dy * dy;
        if !d2.is_finite() {
            return;
        }
        if best.is_none_or(|b| d2 < b.1) {
            best = Some((s, d2));
        }
    };

    if cached_ok {
        for cached in cached_paths {
            if hidden.contains(&cached.series_id) {
                continue;
            }
            if let Some(pinned) = pinned
                && cached.series_id != pinned
            {
                continue;
            }
            for s in cached.samples.iter().copied() {
                consider_sample(s);
            }
        }
    } else {
        let transform_y1 = PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let transform_y2 = view_bounds_y2.map(|b| PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: b,
            x_scale,
            y_scale: y2_scale,
        });
        let transform_y3 = view_bounds_y3.map(|b| PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: b,
            x_scale,
            y_scale: y3_scale,
        });
        let transform_y4 = view_bounds_y4.map(|b| PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: b,
            x_scale,
            y_scale: y4_scale,
        });

        let transform_for_axis = |axis: YAxis| match axis {
            YAxis::Left => transform_y1,
            YAxis::Right => transform_y2.unwrap_or(transform_y1),
            YAxis::Right2 => transform_y3.unwrap_or(transform_y1),
            YAxis::Right3 => transform_y4.unwrap_or(transform_y1),
        };

        for (series_id, axis, data) in series.iter().copied() {
            if hidden.contains(&series_id) {
                continue;
            }
            if let Some(pinned) = pinned
                && pinned != series_id
            {
                continue;
            }
            let transform = transform_for_axis(axis);

            // Fast path for monotonic-X slice-backed series.
            if data.is_sorted_by_x()
                && let Some(slice) = data.as_slice()
                && threshold.is_finite()
                && threshold > 0.0
            {
                let left = Px(local.x.0 - threshold);
                let right = Px(local.x.0 + threshold);
                let x0 = transform.px_to_data(Point::new(left, local.y)).x;
                let x1 = transform.px_to_data(Point::new(right, local.y)).x;
                if x0.is_finite() && x1.is_finite() {
                    let x_min = x0.min(x1);
                    let x_max = x0.max(x1);
                    let (base, window) = visible_sorted_slice(slice, x_min, x_max);
                    for (i, p) in window.iter().copied().enumerate() {
                        if !p.x.is_finite() || !p.y.is_finite() {
                            continue;
                        }
                        let plot_px = transform.data_to_px(p);
                        if !plot_px.x.0.is_finite() || !plot_px.y.0.is_finite() {
                            continue;
                        }
                        consider_sample(SamplePoint {
                            series_id,
                            index: base + i,
                            data: p,
                            plot_px,
                            connects_to_prev: false,
                        });
                    }
                    continue;
                }
            }

            for sample in decimate_samples(transform, data, scale_factor, series_id) {
                consider_sample(sample);
            }
        }
    }

    best.and_then(|(s, d2)| {
        (d2 <= threshold2).then_some(PlotHover {
            series_id: s.series_id,
            index: s.index,
            data: s.data,
            plot_px: s.plot_px,
            value: None,
        })
    })
}

fn bars_path_commands(
    transform: PlotTransform,
    samples: &[SamplePoint],
    bar_width: f32,
    baseline: f32,
) -> Vec<fret_core::PathCommand> {
    if samples.is_empty() {
        return Vec::new();
    }

    let w = f64::from(bar_width.abs());
    if !w.is_finite() || w <= 0.0 {
        return Vec::new();
    }

    let baseline = f64::from(baseline);

    let mut out: Vec<fret_core::PathCommand> = Vec::new();

    for s in samples {
        let x = s.data.x;
        let y = s.data.y;
        if !x.is_finite() || !y.is_finite() || !baseline.is_finite() {
            continue;
        }

        let x0 = x - w * 0.5;
        let x1 = x + w * 0.5;

        let p00 = transform.data_to_px(DataPoint { x: x0, y: baseline });
        let p10 = transform.data_to_px(DataPoint { x: x1, y: baseline });
        let p01 = transform.data_to_px(DataPoint { x: x0, y });
        let p11 = transform.data_to_px(DataPoint { x: x1, y });

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

        out.push(fret_core::PathCommand::MoveTo(a));
        out.push(fret_core::PathCommand::LineTo(b));
        out.push(fret_core::PathCommand::LineTo(c));
        out.push(fret_core::PathCommand::LineTo(d));
        out.push(fret_core::PathCommand::Close);
    }

    out
}

fn area_fill_commands_from_polyline(
    polyline: &[fret_core::PathCommand],
    baseline_y: Px,
) -> Vec<fret_core::PathCommand> {
    if polyline.is_empty() {
        return Vec::new();
    }
    if !baseline_y.0.is_finite() {
        return Vec::new();
    }

    let mut out: Vec<fret_core::PathCommand> = Vec::new();
    let mut segment: Vec<Point> = Vec::new();

    let mut flush_segment = |segment: &mut Vec<Point>| {
        if segment.len() < 2 {
            segment.clear();
            return;
        }

        let first = segment[0];
        let last = *segment.last().expect("len>=2");

        let base0 = Point::new(first.x, baseline_y);
        let base1 = Point::new(last.x, baseline_y);

        out.push(fret_core::PathCommand::MoveTo(base0));
        out.push(fret_core::PathCommand::LineTo(first));
        for p in segment.iter().copied().skip(1) {
            out.push(fret_core::PathCommand::LineTo(p));
        }
        out.push(fret_core::PathCommand::LineTo(base1));
        out.push(fret_core::PathCommand::Close);

        segment.clear();
    };

    for cmd in polyline {
        match *cmd {
            fret_core::PathCommand::MoveTo(p) => {
                flush_segment(&mut segment);
                segment.push(p);
            }
            fret_core::PathCommand::LineTo(p) => {
                segment.push(p);
            }
            _ => {}
        }
    }

    flush_segment(&mut segment);
    out
}
