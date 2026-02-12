//! Plot layer implementations for retained plots.
//!
//! This module contains:
//! - the `PlotLayer` trait and paint/hit-test helpers,
//! - concrete 2D plot layers (line/scatter/bars/area/shaded/heatmap/etc),
//! - convenience `*PlotCanvas` aliases for `PlotCanvas<L>`.

use fret_canvas::budget::{InteractionBudget, WorkBudget};
use fret_canvas::cache::{
    PathCache, SceneOpTileCache, TileCacheKeyBuilder, TileCoord, TileGrid2D,
    warm_scene_op_tiles_u64_with,
};
use fret_canvas::diagnostics::{CanvasCacheKey, CanvasCacheStatsRegistry};
use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{Color, DrawOrder, SceneOp};
use fret_core::{PathConstraints, PathId, PathStyle, UiServices};
use fret_runtime::Model;
use fret_ui::UiHost;
use fret_ui::retained_bridge::PaintCx;
use slotmap::Key;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use super::canvas::PlotCanvas;
use super::models::*;
use super::style::*;

use crate::cartesian::{AxisScale, DataPoint, DataRect, PlotTransform};
use crate::plot::colormap::ColorMapLut;
use crate::plot::decimate::{
    SamplePoint, decimate_points, decimate_polyline, decimate_samples, decimate_shaded_band,
    device_point_budget, view_x_range, visible_sorted_slice,
};
use crate::plot::histogram::histogram_bins;
use crate::plot::view::data_rect_key_scaled;
use crate::series::{SeriesData, SeriesId};

fn report_layer_path_cache_stats<H: UiHost>(
    cx: &mut PaintCx<'_, H>,
    layer_name: &'static str,
    cache: &PathCache,
) {
    let Some(window) = cx.window else {
        return;
    };

    let frame_id = cx.app.frame_id().0;
    let key = CanvasCacheKey {
        window: window.data().as_ffi(),
        node: cx.node.data().as_ffi(),
        name: layer_name,
    };

    cx.app
        .with_global_mut(CanvasCacheStatsRegistry::default, |registry, _app| {
            registry.record_path_cache(key, frame_id, cache.len(), cache.stats());
        });
}

fn report_layer_tile_cache_stats<H: UiHost>(
    cx: &mut PaintCx<'_, H>,
    layer_name: &'static str,
    cache: &SceneOpTileCache<u64>,
    requested_tiles: usize,
    budget_limit: u32,
    budget_used: u32,
    skipped_tiles: u32,
) {
    let Some(window) = cx.window else {
        return;
    };

    let frame_id = cx.app.frame_id().0;
    let key = CanvasCacheKey {
        window: window.data().as_ffi(),
        node: cx.node.data().as_ffi(),
        name: layer_name,
    };

    cx.app
        .with_global_mut(CanvasCacheStatsRegistry::default, |registry, _app| {
            registry.record_scene_op_tile_cache_with_budget(
                key,
                frame_id,
                cache.entries_len(),
                requested_tiles,
                budget_limit,
                budget_used,
                skipped_tiles,
                cache.stats(),
            );
        });
}

struct ResolvedSeriesStyle {
    stroke_color: Color,
    stroke_width: Px,
}

pub(super) fn resolve_series_color(
    series_index: usize,
    plot_style: LinePlotStyle,
    series_count: usize,
    override_color: Option<Color>,
) -> Color {
    if series_count <= 1 {
        return override_color.unwrap_or(plot_style.stroke_color);
    }
    override_color
        .unwrap_or(plot_style.series_palette[series_index % plot_style.series_palette.len()])
}

fn resolve_stroke_width(plot_style: LinePlotStyle, override_width: Option<Px>) -> Px {
    let Some(w) = override_width else {
        return plot_style.stroke_width;
    };
    if !w.0.is_finite() {
        return plot_style.stroke_width;
    }
    Px(w.0.max(0.0))
}

fn resolve_marker_radius(override_radius: Option<Px>, stroke_width: Px) -> Px {
    let Some(r) = override_radius else {
        return Px((stroke_width.0 * 3.0).clamp(2.0, 6.0));
    };
    if !r.0.is_finite() {
        return Px((stroke_width.0 * 3.0).clamp(2.0, 6.0));
    }
    Px(r.0.max(0.0))
}

fn series_style(
    series: &LineSeries,
    series_index: usize,
    plot_style: LinePlotStyle,
    series_count: usize,
) -> ResolvedSeriesStyle {
    ResolvedSeriesStyle {
        stroke_color: resolve_series_color(
            series_index,
            plot_style,
            series_count,
            series.stroke_color,
        ),
        stroke_width: resolve_stroke_width(plot_style, series.stroke_width),
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
    use crate::series::Series;

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

fn scatter_marker_commands_with_shape(
    samples: &[SamplePoint],
    radius: Px,
    shape: MarkerShape,
) -> Vec<fret_core::PathCommand> {
    if samples.is_empty() {
        return Vec::new();
    }

    let r = radius.0.max(0.0);
    let mut out: Vec<fret_core::PathCommand> = Vec::new();
    out.reserve(samples.len().saturating_mul(8));

    for s in samples {
        let x = s.plot_px.x.0;
        let y = s.plot_px.y.0;
        if !x.is_finite() || !y.is_finite() {
            continue;
        }

        push_marker_commands(&mut out, Px(x), Px(y), Px(r), shape);
    }

    out
}

fn resolve_marker_shape(override_shape: Option<MarkerShape>) -> MarkerShape {
    override_shape.unwrap_or_default()
}

fn push_marker_commands(
    out: &mut Vec<fret_core::PathCommand>,
    x: Px,
    y: Px,
    radius: Px,
    shape: MarkerShape,
) {
    let x = x.0;
    let y = y.0;
    let r = radius.0.max(0.0);
    if !x.is_finite() || !y.is_finite() || !r.is_finite() || r <= 0.0 {
        return;
    }

    match shape {
        MarkerShape::Plus => {
            out.push(fret_core::PathCommand::MoveTo(Point::new(Px(x - r), Px(y))));
            out.push(fret_core::PathCommand::LineTo(Point::new(Px(x + r), Px(y))));
            out.push(fret_core::PathCommand::MoveTo(Point::new(Px(x), Px(y - r))));
            out.push(fret_core::PathCommand::LineTo(Point::new(Px(x), Px(y + r))));
        }
        MarkerShape::X => {
            out.push(fret_core::PathCommand::MoveTo(Point::new(
                Px(x - r),
                Px(y - r),
            )));
            out.push(fret_core::PathCommand::LineTo(Point::new(
                Px(x + r),
                Px(y + r),
            )));
            out.push(fret_core::PathCommand::MoveTo(Point::new(
                Px(x - r),
                Px(y + r),
            )));
            out.push(fret_core::PathCommand::LineTo(Point::new(
                Px(x + r),
                Px(y - r),
            )));
        }
        MarkerShape::Square => {
            let p0 = Point::new(Px(x - r), Px(y - r));
            let p1 = Point::new(Px(x + r), Px(y - r));
            let p2 = Point::new(Px(x + r), Px(y + r));
            let p3 = Point::new(Px(x - r), Px(y + r));
            out.push(fret_core::PathCommand::MoveTo(p0));
            out.push(fret_core::PathCommand::LineTo(p1));
            out.push(fret_core::PathCommand::LineTo(p2));
            out.push(fret_core::PathCommand::LineTo(p3));
            out.push(fret_core::PathCommand::LineTo(p0));
        }
        MarkerShape::Diamond => {
            let p0 = Point::new(Px(x), Px(y - r));
            let p1 = Point::new(Px(x + r), Px(y));
            let p2 = Point::new(Px(x), Px(y + r));
            let p3 = Point::new(Px(x - r), Px(y));
            out.push(fret_core::PathCommand::MoveTo(p0));
            out.push(fret_core::PathCommand::LineTo(p1));
            out.push(fret_core::PathCommand::LineTo(p2));
            out.push(fret_core::PathCommand::LineTo(p3));
            out.push(fret_core::PathCommand::LineTo(p0));
        }
        MarkerShape::TriangleUp => {
            let p0 = Point::new(Px(x), Px(y - r));
            let p1 = Point::new(Px(x + r), Px(y + r));
            let p2 = Point::new(Px(x - r), Px(y + r));
            out.push(fret_core::PathCommand::MoveTo(p0));
            out.push(fret_core::PathCommand::LineTo(p1));
            out.push(fret_core::PathCommand::LineTo(p2));
            out.push(fret_core::PathCommand::LineTo(p0));
        }
        MarkerShape::TriangleDown => {
            let p0 = Point::new(Px(x), Px(y + r));
            let p1 = Point::new(Px(x + r), Px(y - r));
            let p2 = Point::new(Px(x - r), Px(y - r));
            out.push(fret_core::PathCommand::MoveTo(p0));
            out.push(fret_core::PathCommand::LineTo(p1));
            out.push(fret_core::PathCommand::LineTo(p2));
            out.push(fret_core::PathCommand::LineTo(p0));
        }
        MarkerShape::Circle => {
            let segments = 12usize;
            let step = (std::f32::consts::PI * 2.0) / segments as f32;
            let t0 = 0.0f32;
            let p0 = Point::new(Px(x + r * t0.cos()), Px(y + r * t0.sin()));
            out.push(fret_core::PathCommand::MoveTo(p0));

            for i in 1..=segments {
                let t = step * i as f32;
                let px = x + r * t.cos();
                let py = y + r * t.sin();
                if !px.is_finite() || !py.is_finite() {
                    continue;
                }
                out.push(fret_core::PathCommand::LineTo(Point::new(Px(px), Px(py))));
            }
        }
    }
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
    let marker = if series.show_markers {
        series.marker_radius.0.max(0.0)
    } else {
        0.0
    };

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

        if marker > 0.0 {
            let Some(y_px) = transform.data_y_to_px(p.y) else {
                return;
            };

            push_marker_commands(&mut out, x_px, y_px, Px(marker), series.marker_shape);
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
    series_id: SeriesId,
    model_revision: u64,
    scale_factor_bits: u32,
    viewport_w_bits: u32,
    viewport_h_bits: u32,
    stroke_width: Px,
    marker_radius: Px,
    marker_shape: MarkerShape,
    cap_size: Px,
    view_key: u64,
    samples: Vec<SamplePoint>,
}

#[derive(Debug)]
struct CachedCandlestickPath {
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

fn series_path_cache_key(series_id: SeriesId, variant: u8) -> u64 {
    use std::collections::hash_map::DefaultHasher;

    let mut hasher = DefaultHasher::new();
    series_id.hash(&mut hasher);
    variant.hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotHover {
    pub series_id: SeriesId,
    pub index: usize,
    pub data: DataPoint,
    pub plot_px: Point,
    pub value: Option<f64>,
}

#[derive(Debug, Clone)]
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

    fn quads_scene_cache_policy(&self) -> PlotQuadsSceneCachePolicy {
        PlotQuadsSceneCachePolicy::Disabled
    }

    /// Optional fast path for layers that can emit quad `SceneOp`s directly (e.g. tiled caches).
    ///
    /// When this returns `true`, the caller should skip the default `paint_quads` path.
    fn paint_quads_scene_ops_tiled<H: UiHost>(
        &mut self,
        _cx: &mut PaintCx<'_, H>,
        _model: &Self::Model,
        _args: PlotPaintArgs<'_>,
        _plot_origin: Point,
    ) -> bool {
        false
    }

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

    fn heatmap_value_range(_model: &Self::Model) -> Option<(f32, f32)> {
        None
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotQuadsSceneCachePolicy {
    Disabled,
    Enabled,
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
    pub view_interacting: bool,
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
    path_cache: PathCache,
    cached_paths: Vec<CachedPath>,
}

pub type LinePlotCanvas = PlotCanvas<LinePlotLayer>;

impl PlotCanvas<LinePlotLayer> {
    pub fn new(model: Model<LinePlotModel>) -> Self {
        Self::with_layer(model, LinePlotLayer::default())
    }
}

#[derive(Debug, Default)]
pub struct StemsPlotLayer {
    path_cache: PathCache,
    cached_paths: Vec<CachedPath>,
}

pub type StemsPlotCanvas = PlotCanvas<StemsPlotLayer>;

impl PlotCanvas<StemsPlotLayer> {
    pub fn new(model: Model<StemsPlotModel>) -> Self {
        Self::with_layer(model, StemsPlotLayer::default())
    }
}

#[derive(Debug, Default)]
pub struct ScatterPlotLayer {
    path_cache: PathCache,
    cached_paths: Vec<CachedPath>,
}

pub type ScatterPlotCanvas = PlotCanvas<ScatterPlotLayer>;

impl PlotCanvas<ScatterPlotLayer> {
    pub fn new(model: Model<ScatterPlotModel>) -> Self {
        Self::with_layer(model, ScatterPlotLayer::default())
    }
}

#[derive(Debug, Default)]
pub struct ErrorBarsPlotLayer {
    path_cache: PathCache,
    cached_paths: Vec<CachedPath>,
}

pub type ErrorBarsPlotCanvas = PlotCanvas<ErrorBarsPlotLayer>;

impl PlotCanvas<ErrorBarsPlotLayer> {
    pub fn new(model: Model<ErrorBarsPlotModel>) -> Self {
        Self::with_layer(model, ErrorBarsPlotLayer::default())
    }
}

#[derive(Debug, Default)]
pub struct CandlestickPlotLayer {
    path_cache: PathCache,
    cached_paths: Vec<CachedCandlestickPath>,
}

pub type CandlestickPlotCanvas = PlotCanvas<CandlestickPlotLayer>;

impl PlotCanvas<CandlestickPlotLayer> {
    pub fn new(model: Model<CandlestickPlotModel>) -> Self {
        Self::with_layer(model, CandlestickPlotLayer::default())
    }
}

#[derive(Debug, Default)]
pub struct StairsPlotLayer {
    path_cache: PathCache,
    cached_paths: Vec<CachedPath>,
    step_mode: StepMode,
}

pub type StairsPlotCanvas = PlotCanvas<StairsPlotLayer>;

impl PlotCanvas<StairsPlotLayer> {
    pub fn new(model: Model<LinePlotModel>) -> Self {
        Self::with_layer(
            model,
            StairsPlotLayer {
                path_cache: PathCache::default(),
                cached_paths: Vec::new(),
                step_mode: StepMode::default(),
            },
        )
    }

    pub fn step_mode(self, mode: StepMode) -> Self {
        self.with_layer_mut(|layer| {
            layer.step_mode = mode;
        })
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
    colormap_key: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HeatmapMipKey {
    model_revision: u64,
    cols: usize,
    rows: usize,
    values_ptr: usize,
}

#[derive(Debug, Clone)]
struct GridMipLevel {
    cols: usize,
    rows: usize,
    values: Vec<f32>,
}

#[derive(Debug, Default)]
pub struct HeatmapPlotLayer {
    cache_key: Option<HeatmapCacheKey>,
    cached_quads: Vec<PlotQuad>,
    mip_key: Option<HeatmapMipKey>,
    mips: Vec<GridMipLevel>,
    colormap_lut: ColorMapLut,
    tile_ops_cache: SceneOpTileCache<u64>,
    tile_scratch: Vec<TileCoord>,
}

fn ceil_div_usize(a: usize, b: usize) -> usize {
    if b == 0 {
        return a;
    }
    a / b + usize::from(a % b != 0)
}

fn select_grid_mip_level(
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

fn downsample_grid_avg(
    prev_cols: usize,
    prev_rows: usize,
    prev: &[f32],
) -> (usize, usize, Vec<f32>) {
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

    (next_cols, next_rows, next)
}

fn build_grid_mips(cols: usize, rows: usize, base: &[f32]) -> Vec<GridMipLevel> {
    let mut mips: Vec<GridMipLevel> = Vec::new();

    let mut prev_cols = cols;
    let mut prev_rows = rows;
    while prev_cols > 1 || prev_rows > 1 {
        let (next_cols, next_rows, next) = {
            let prev_values: &[f32] = if mips.is_empty() {
                base
            } else {
                &mips.last().expect("non-empty").values
            };
            downsample_grid_avg(prev_cols, prev_rows, prev_values)
        };

        mips.push(GridMipLevel {
            cols: next_cols,
            rows: next_rows,
            values: next,
        });

        prev_cols = next_cols;
        prev_rows = next_rows;
    }

    mips
}

fn grid_mip_level_values<'a>(
    level: usize,
    base_cols: usize,
    base_rows: usize,
    base_values: &'a [f32],
    mips: &'a [GridMipLevel],
) -> (usize, usize, &'a [f32]) {
    if level == 0 {
        return (base_cols, base_rows, base_values);
    }

    let mip = mips
        .get(level.saturating_sub(1))
        .expect("level > 0 implies mip exists");
    (mip.cols, mip.rows, &mip.values)
}

fn grid_heatmap_tile_ops(
    clip_world: Rect,
    tile_origin_world: Point,
    scale_x: f64,
    scale_y: f64,
    x_scale: AxisScale,
    y_scale: AxisScale,
    data_bounds: DataRect,
    cols: usize,
    rows: usize,
    dx: f64,
    dy: f64,
    value_min: f32,
    denom: f32,
    grid_cols: usize,
    grid_rows: usize,
    values: &[f32],
    scale: usize,
    heatmap_color: &impl Fn(f32) -> Color,
) -> Vec<SceneOp> {
    let clip_x0 = f64::from(clip_world.origin.x.0);
    let clip_x1 = f64::from(clip_world.origin.x.0 + clip_world.size.width.0);
    let clip_y0 = f64::from(clip_world.origin.y.0);
    let clip_y1 = f64::from(clip_world.origin.y.0 + clip_world.size.height.0);

    if !clip_x0.is_finite()
        || !clip_x1.is_finite()
        || !clip_y0.is_finite()
        || !clip_y1.is_finite()
        || clip_x1 <= clip_x0
        || clip_y1 <= clip_y0
    {
        return Vec::new();
    }

    if !scale_x.is_finite() || !scale_y.is_finite() || scale_x <= 0.0 || scale_y <= 0.0 {
        return Vec::new();
    }

    let vx0 = clip_x0 / scale_x;
    let vx1 = clip_x1 / scale_x;
    let vy0 = -clip_y0 / scale_y;
    let vy1 = -clip_y1 / scale_y;

    let Some(data_x0) = x_scale.from_axis(vx0.min(vx1)) else {
        return Vec::new();
    };
    let Some(data_x1) = x_scale.from_axis(vx0.max(vx1)) else {
        return Vec::new();
    };
    let Some(data_y0) = y_scale.from_axis(vy0.min(vy1)) else {
        return Vec::new();
    };
    let Some(data_y1) = y_scale.from_axis(vy0.max(vy1)) else {
        return Vec::new();
    };

    let col0 = (((data_x0 - data_bounds.x_min) / dx).floor() as isize)
        .clamp(0, cols.saturating_sub(1) as isize) as usize;
    let col1 =
        (((data_x1 - data_bounds.x_min) / dx).ceil() as isize).clamp(0, cols as isize) as usize;

    let row0 = (((data_y0 - data_bounds.y_min) / dy).floor() as isize)
        .clamp(0, rows.saturating_sub(1) as isize) as usize;
    let row1 =
        (((data_y1 - data_bounds.y_min) / dy).ceil() as isize).clamp(0, rows as isize) as usize;

    let col0_l = (col0 / scale).min(grid_cols);
    let col1_l = ceil_div_usize(col1, scale).min(grid_cols);
    let row0_l = (row0 / scale).min(grid_rows);
    let row1_l = ceil_div_usize(row1, scale).min(grid_rows);

    if col1_l <= col0_l || row1_l <= row0_l {
        return Vec::new();
    }

    let tile_ox = tile_origin_world.x.0;
    let tile_oy = tile_origin_world.y.0;

    let quad_cols = col1_l.saturating_sub(col0_l);
    let quad_rows = row1_l.saturating_sub(row0_l);
    let mut ops: Vec<SceneOp> = Vec::with_capacity(quad_cols.saturating_mul(quad_rows));

    for row_l in row0_l..row1_l {
        let row_base0 = row_l.saturating_mul(scale);
        let row_base1 = (row_l.saturating_add(1).saturating_mul(scale)).min(rows);

        let y0_data = data_bounds.y_min + (row_base0 as f64) * dy;
        let y1_data = data_bounds.y_min + (row_base1 as f64) * dy;
        let (Some(vy0), Some(vy1)) = (y_scale.to_axis(y0_data), y_scale.to_axis(y1_data)) else {
            continue;
        };
        let wy0 = -(vy0 * scale_y);
        let wy1 = -(vy1 * scale_y);
        let top = wy0.min(wy1);
        let bottom = wy0.max(wy1);
        if !top.is_finite() || !bottom.is_finite() || bottom <= top {
            continue;
        }

        for col_l in col0_l..col1_l {
            let idx = row_l.saturating_mul(grid_cols).saturating_add(col_l);
            let Some(v) = values.get(idx).copied() else {
                continue;
            };
            if !v.is_finite() {
                continue;
            }

            let col_base0 = col_l.saturating_mul(scale);
            let col_base1 = (col_l.saturating_add(1).saturating_mul(scale)).min(cols);

            let x0_data = data_bounds.x_min + (col_base0 as f64) * dx;
            let x1_data = data_bounds.x_min + (col_base1 as f64) * dx;
            let (Some(vx0), Some(vx1)) = (x_scale.to_axis(x0_data), x_scale.to_axis(x1_data))
            else {
                continue;
            };
            let wx0 = vx0 * scale_x;
            let wx1 = vx1 * scale_x;
            let left = wx0.min(wx1);
            let right = wx0.max(wx1);
            if !left.is_finite() || !right.is_finite() || right <= left {
                continue;
            }

            let t = ((v - value_min) / denom).clamp(0.0, 1.0);
            let color = heatmap_color(t);

            ops.push(SceneOp::Quad {
                order: DrawOrder(2),
                rect: Rect::new(
                    Point::new(Px(left as f32 - tile_ox), Px(top as f32 - tile_oy)),
                    Size::new(Px((right - left) as f32), Px((bottom - top) as f32)),
                ),
                background: fret_core::Paint::Solid(color),

                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT,

                corner_radii: Corners::all(Px(0.0)),
            });
        }
    }

    #[cfg(debug_assertions)]
    {
        debug_assert!(
            ops.iter().all(|op| {
                !matches!(
                    op,
                    SceneOp::Text { .. }
                        | SceneOp::Path { .. }
                        | SceneOp::SvgMaskIcon { .. }
                        | SceneOp::SvgImage { .. }
                )
            }),
            "Heatmap tile ops are cached/replayed; do not cache hosted resource ops without touching their caches on replay"
        );
    }

    ops
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
        self.mips = build_grid_mips(model.cols, model.rows, &model.values);
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

        let level = select_grid_mip_level(
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
        let level = select_grid_mip_level(64, 64, 1024.0, 768.0, 50_000, 8);
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

pub type HeatmapPlotCanvas = PlotCanvas<HeatmapPlotLayer>;

impl PlotCanvas<HeatmapPlotLayer> {
    pub fn new(model: Model<HeatmapPlotModel>) -> Self {
        Self::with_layer(model, HeatmapPlotLayer::default()).heatmap_colorbar(true)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Histogram2DCacheKey {
    model_revision: u64,
    view_key: u64,
    cols: usize,
    rows: usize,
    viewport_w_bits: u32,
    viewport_h_bits: u32,
    value_min_bits: u32,
    value_max_bits: u32,
    colormap_key: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Histogram2DMipKey {
    model_revision: u64,
    cols: usize,
    rows: usize,
    values_ptr: usize,
}

#[derive(Debug, Default)]
pub struct Histogram2DPlotLayer {
    cache_key: Option<Histogram2DCacheKey>,
    cached_quads: Vec<PlotQuad>,
    mip_key: Option<Histogram2DMipKey>,
    mips: Vec<GridMipLevel>,
    colormap_lut: ColorMapLut,
    tile_ops_cache: SceneOpTileCache<u64>,
    tile_scratch: Vec<TileCoord>,
}

pub type Histogram2DPlotCanvas = PlotCanvas<Histogram2DPlotLayer>;

impl PlotCanvas<Histogram2DPlotLayer> {
    pub fn new(model: Model<Histogram2DPlotModel>) -> Self {
        Self::with_layer(model, Histogram2DPlotLayer::default()).heatmap_colorbar(true)
    }
}

#[derive(Debug, Default)]
pub struct HistogramPlotLayer {
    path_cache: PathCache,
    cached_paths: Vec<CachedPath>,
}

pub type HistogramPlotCanvas = PlotCanvas<HistogramPlotLayer>;

impl PlotCanvas<HistogramPlotLayer> {
    pub fn new(model: Model<HistogramPlotModel>) -> Self {
        Self::with_layer(model, HistogramPlotLayer::default())
    }
}

#[derive(Debug, Default)]
pub struct BarsPlotLayer {
    path_cache: PathCache,
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
    path_cache: PathCache,
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
    path_cache: PathCache,
    cached_paths: Vec<CachedShadedPath>,
}

pub type ShadedPlotCanvas = PlotCanvas<ShadedPlotLayer>;

impl PlotCanvas<ShadedPlotLayer> {
    pub fn new(model: Model<ShadedPlotModel>) -> Self {
        Self::with_layer(model, ShadedPlotLayer::default())
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
        self.path_cache.begin_frame();
        report_layer_path_cache_stats(cx, "fret-plot.line.paths", &self.path_cache);
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
            ..
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
            self.path_cache.clear(cx.services);
            self.cached_paths.clear();
            return Vec::new();
        }

        let cached_ok = self.cached_paths.len() == series_count
            && self.cached_paths.iter().enumerate().all(|(i, c)| {
                series.get(i).is_some_and(|s| {
                    let expected_view_key = view_key_for_axis(s.y_axis);
                    let expected_stroke_width = resolve_stroke_width(style, s.stroke_width);

                    s.id == c.series_id
                        && c.view_key == expected_view_key
                        && c.stroke_width == expected_stroke_width
                }) && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
            });

        if cached_ok {
            let constraints = PathConstraints {
                scale_factor: cx.scale_factor,
            };
            let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
            for (i, s) in series.iter().enumerate() {
                if hidden.contains(&s.id) {
                    continue;
                }
                let key = series_path_cache_key(s.id, 0);
                let Some((id, _metrics)) = self.path_cache.get(key, constraints) else {
                    continue;
                };
                let style = series_style(s, i, style, series_count);
                out.push((s.id, id, style.stroke_color));
            }
            return out;
        }

        self.cached_paths.clear();

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

        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };

        let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
        self.cached_paths = Vec::with_capacity(series_count);

        for (series_index, s) in series.iter().enumerate() {
            let resolved = series_style(s, series_index, style, series_count);
            let path_style = PathStyle::Stroke(fret_core::StrokeStyle {
                width: resolved.stroke_width,
            });

            let series_id = s.id;
            if hidden.contains(&series_id) {
                let view_key = view_key_for_axis(s.y_axis);
                self.cached_paths.push(CachedPath {
                    series_id,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width: resolved.stroke_width,
                    marker_radius: Px(0.0),
                    marker_shape: MarkerShape::default(),
                    cap_size: Px(0.0),
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
                let key = series_path_cache_key(series_id, 0);
                let (id, _metrics) =
                    self.path_cache
                        .prepare(cx.services, key, &commands, path_style, constraints);
                Some(id)
            };

            self.cached_paths.push(CachedPath {
                series_id,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width: resolved.stroke_width,
                marker_radius: Px(0.0),
                marker_shape: MarkerShape::default(),
                cap_size: Px(0.0),
                view_key,
                samples,
            });

            if let Some(id) = id {
                out.push((series_id, id, resolved.stroke_color));
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
        self.path_cache.clear(services);
        self.cached_paths.clear();
    }
}

impl PlotLayer for StemsPlotLayer {
    type Model = StemsPlotModel;

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
        self.path_cache.begin_frame();
        report_layer_path_cache_stats(cx, "fret-plot.stems.paths", &self.path_cache);
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
            ..
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
            self.path_cache.clear(cx.services);
            self.cached_paths.clear();
            return Vec::new();
        }

        let cached_ok = self.cached_paths.len() == series_count
            && self.cached_paths.iter().enumerate().all(|(i, c)| {
                series.get(i).is_some_and(|s| {
                    let expected_view_key = view_key_for_axis(s.y_axis);
                    let expected_stroke_width = resolve_stroke_width(style, s.stroke_width);
                    s.id == c.series_id
                        && c.view_key == expected_view_key
                        && c.stroke_width == expected_stroke_width
                }) && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
            });

        if cached_ok {
            let constraints = PathConstraints {
                scale_factor: cx.scale_factor,
            };
            let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
            for (i, s) in series.iter().enumerate() {
                if hidden.contains(&s.id) {
                    continue;
                }
                let key = series_path_cache_key(s.id, 0);
                let Some((id, _metrics)) = self.path_cache.get(key, constraints) else {
                    continue;
                };
                let color = resolve_series_color(i, style, series_count, s.stroke_color);
                out.push((s.id, id, color));
            }
            return out;
        }

        self.cached_paths.clear();

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

        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };

        let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
        self.cached_paths = Vec::with_capacity(series_count);

        for (series_index, s) in series.iter().enumerate() {
            let stroke_width = resolve_stroke_width(style, s.stroke_width);
            let series_id = s.id;
            let path_style = PathStyle::Stroke(fret_core::StrokeStyle {
                width: stroke_width,
            });

            if hidden.contains(&series_id) {
                let view_key = view_key_for_axis(s.y_axis);
                self.cached_paths.push(CachedPath {
                    series_id,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width,
                    marker_radius: Px(0.0),
                    marker_shape: MarkerShape::default(),
                    cap_size: Px(0.0),
                    view_key,
                    samples: Vec::new(),
                });
                continue;
            }

            let transform = transform_for_axis(s.y_axis);
            let view_key = view_key_for_axis(s.y_axis);

            let baseline_y = f64::from(s.baseline);
            let baseline_y_px = transform.data_y_to_px(baseline_y);

            let samples = decimate_points(transform, &*s.data, cx.scale_factor, series_id);
            let mut commands: Vec<fret_core::PathCommand> = Vec::new();
            commands.reserve(samples.len().saturating_mul(2));

            if let Some(y0) = baseline_y_px {
                for sp in &samples {
                    let x = sp.plot_px.x;
                    let y = sp.plot_px.y;
                    if !x.0.is_finite() || !y.0.is_finite() {
                        continue;
                    }
                    commands.push(fret_core::PathCommand::MoveTo(Point::new(x, y0)));
                    commands.push(fret_core::PathCommand::LineTo(Point::new(x, y)));
                }
            }

            let id = if commands.is_empty() || stroke_width.0 <= 0.0 {
                None
            } else {
                let key = series_path_cache_key(series_id, 0);
                let (id, _metrics) =
                    self.path_cache
                        .prepare(cx.services, key, &commands, path_style, constraints);
                Some(id)
            };

            self.cached_paths.push(CachedPath {
                series_id,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width,
                marker_radius: Px(0.0),
                marker_shape: MarkerShape::default(),
                cap_size: Px(0.0),
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
                series.get(i).is_some_and(|s| {
                    let expected_view_key = view_key_for_axis(s.y_axis);
                    let expected_stroke_width = resolve_stroke_width(style, s.stroke_width);
                    s.id == c.series_id
                        && c.view_key == expected_view_key
                        && c.stroke_width == expected_stroke_width
                }) && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
            });

        let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size);
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

        let mut best: Option<(PlotHover, f32)> = None;
        let mut consider_stem = |series: &StemsSeries, sp: SamplePoint| {
            let transform = transform_for_axis(series.y_axis);
            let baseline_y = f64::from(series.baseline);
            let Some(y0) = transform.data_y_to_px(baseline_y) else {
                return;
            };

            let a = Point::new(sp.plot_px.x, y0);
            let b = sp.plot_px;
            let Some((q, _t)) = point_segment_closest_px(local, a, b) else {
                return;
            };

            let dx = q.x.0 - local.x.0;
            let dy = q.y.0 - local.y.0;
            let d2 = dx * dx + dy * dy;
            if !d2.is_finite() {
                return;
            }

            let hover = PlotHover {
                series_id: sp.series_id,
                index: sp.index,
                data: sp.data,
                plot_px: sp.plot_px,
                value: None,
            };

            if best.is_none_or(|b| d2 < b.1) {
                best = Some((hover, d2));
            }
        };

        if cached_ok {
            for (series_index, s) in series.iter().enumerate() {
                let cached = &self.cached_paths[series_index];
                if hidden.contains(&cached.series_id) {
                    continue;
                }
                if let Some(pinned) = pinned
                    && cached.series_id != pinned
                {
                    continue;
                }
                for sp in cached.samples.iter().copied() {
                    consider_stem(s, sp);
                }
            }
        } else {
            for s in series {
                if hidden.contains(&s.id) {
                    continue;
                }
                if let Some(pinned) = pinned
                    && s.id != pinned
                {
                    continue;
                }
                let transform = transform_for_axis(s.y_axis);
                let samples = decimate_points(transform, &*s.data, scale_factor, s.id);
                for sp in samples.into_iter() {
                    consider_stem(s, sp);
                }
            }
        }

        best.and_then(|(hover, d2)| (d2 <= threshold2).then_some(hover))
    }

    fn cleanup_resources(&mut self, services: &mut dyn UiServices) {
        self.path_cache.clear(services);
        self.cached_paths.clear();
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
        self.path_cache.begin_frame();
        report_layer_path_cache_stats(cx, "fret-plot.scatter.paths", &self.path_cache);
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
            ..
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
            self.path_cache.clear(cx.services);
            self.cached_paths.clear();
            return Vec::new();
        }

        let cached_ok = self.cached_paths.len() == series_count
            && self.cached_paths.iter().enumerate().all(|(i, c)| {
                series.get(i).is_some_and(|s| {
                    let expected_view_key = view_key_for_axis(s.y_axis);
                    let expected_stroke_width = resolve_stroke_width(style, s.stroke_width);
                    let expected_marker_radius =
                        resolve_marker_radius(s.marker_radius, expected_stroke_width);
                    let expected_marker_shape = resolve_marker_shape(s.marker_shape);

                    s.id == c.series_id
                        && c.view_key == expected_view_key
                        && c.stroke_width == expected_stroke_width
                        && c.marker_radius == expected_marker_radius
                        && c.marker_shape == expected_marker_shape
                }) && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
            });

        if cached_ok {
            let constraints = PathConstraints {
                scale_factor: cx.scale_factor,
            };
            let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
            for (i, s) in series.iter().enumerate() {
                if hidden.contains(&s.id) {
                    continue;
                }
                let key = series_path_cache_key(s.id, 0);
                let Some((id, _metrics)) = self.path_cache.get(key, constraints) else {
                    continue;
                };
                let color = resolve_series_color(i, style, series_count, s.stroke_color);
                out.push((s.id, id, color));
            }
            return out;
        }

        self.cached_paths.clear();

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

        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };

        let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
        self.cached_paths = Vec::with_capacity(series_count);

        for (series_index, s) in series.iter().enumerate() {
            let series_id = s.id;
            let stroke_width = resolve_stroke_width(style, s.stroke_width);
            let marker_radius = resolve_marker_radius(s.marker_radius, stroke_width);
            let marker_shape = resolve_marker_shape(s.marker_shape);
            let path_style = PathStyle::Stroke(fret_core::StrokeStyle {
                width: stroke_width,
            });

            if hidden.contains(&series_id) {
                let view_key = view_key_for_axis(s.y_axis);
                self.cached_paths.push(CachedPath {
                    series_id,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width,
                    marker_radius,
                    marker_shape,
                    cap_size: Px(0.0),
                    view_key,
                    samples: Vec::new(),
                });
                continue;
            }

            let transform = transform_for_axis(s.y_axis);
            let view_key = view_key_for_axis(s.y_axis);

            let samples = decimate_points(transform, &*s.data, cx.scale_factor, series_id);
            let commands =
                scatter_marker_commands_with_shape(&samples, marker_radius, marker_shape);
            let id = if commands.is_empty() {
                None
            } else {
                let key = series_path_cache_key(series_id, 0);
                let (id, _metrics) =
                    self.path_cache
                        .prepare(cx.services, key, &commands, path_style, constraints);
                Some(id)
            };

            self.cached_paths.push(CachedPath {
                series_id,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width,
                marker_radius,
                marker_shape,
                cap_size: Px(0.0),
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
        self.path_cache.clear(services);
        self.cached_paths.clear();
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
        self.path_cache.begin_frame();
        report_layer_path_cache_stats(cx, "fret-plot.error_bars.paths", &self.path_cache);
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
            ..
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
            self.path_cache.clear(cx.services);
            self.cached_paths.clear();
            return Vec::new();
        }

        let cached_ok = self.cached_paths.len() == series_count
            && self.cached_paths.iter().enumerate().all(|(i, c)| {
                series.get(i).is_some_and(|s| {
                    let expected_view_key = view_key_for_axis(s.y_axis);
                    let expected_stroke_width = resolve_stroke_width(style, s.stroke_width);
                    let expected_marker_radius = if s.show_markers {
                        s.marker_radius
                    } else {
                        Px(0.0)
                    };
                    let expected_cap_size = if s.show_caps { s.cap_size } else { Px(0.0) };

                    s.id == c.series_id
                        && c.view_key == expected_view_key
                        && c.stroke_width == expected_stroke_width
                        && c.marker_radius == expected_marker_radius
                        && c.marker_shape == s.marker_shape
                        && c.cap_size == expected_cap_size
                }) && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
            });

        if cached_ok {
            let constraints = PathConstraints {
                scale_factor: cx.scale_factor,
            };
            let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
            for (i, s) in series.iter().enumerate() {
                if hidden.contains(&s.id) {
                    continue;
                }
                let key = series_path_cache_key(s.id, 0);
                let Some((id, _metrics)) = self.path_cache.get(key, constraints) else {
                    continue;
                };
                let color = resolve_series_color(i, style, series_count, s.stroke_color);
                out.push((s.id, id, color));
            }
            return out;
        }

        self.cached_paths.clear();

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

        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };

        let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
        self.cached_paths = Vec::with_capacity(series_count);

        for (series_index, s) in series.iter().enumerate() {
            let series_id = s.id;
            let stroke_width = resolve_stroke_width(style, s.stroke_width);
            let marker_radius = if s.show_markers {
                s.marker_radius
            } else {
                Px(0.0)
            };
            let cap_size = if s.show_caps { s.cap_size } else { Px(0.0) };
            let path_style = PathStyle::Stroke(fret_core::StrokeStyle {
                width: stroke_width,
            });

            if hidden.contains(&series_id) {
                let view_key = view_key_for_axis(s.y_axis);
                self.cached_paths.push(CachedPath {
                    series_id,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width,
                    marker_radius,
                    marker_shape: s.marker_shape,
                    cap_size,
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
                let key = series_path_cache_key(series_id, 0);
                let (id, _metrics) =
                    self.path_cache
                        .prepare(cx.services, key, &commands, path_style, constraints);
                Some(id)
            };

            self.cached_paths.push(CachedPath {
                series_id,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width,
                marker_radius,
                marker_shape: s.marker_shape,
                cap_size,
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
        self.path_cache.clear(services);
        self.cached_paths.clear();
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
        self.path_cache.begin_frame();
        report_layer_path_cache_stats(cx, "fret-plot.candlestick.paths", &self.path_cache);
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
            ..
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
            self.path_cache.clear(cx.services);
            self.cached_paths.clear();
            return Vec::new();
        }

        let cached_ok = self.cached_paths.len() == series_count
            && self.cached_paths.iter().enumerate().all(|(i, c)| {
                series.get(i).is_some_and(|s| {
                    let expected_view_key = view_key_for_axis(s.y_axis);
                    let expected_stroke_width = resolve_stroke_width(style, s.stroke_width);
                    s.id == c.series_id
                        && c.view_key == expected_view_key
                        && c.stroke_width == expected_stroke_width
                }) && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
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
            let constraints = PathConstraints {
                scale_factor: cx.scale_factor,
            };
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

                if let Some((id, _metrics)) = self
                    .path_cache
                    .get(series_path_cache_key(s.id, 0), constraints)
                {
                    out.push((s.id, id, wick));
                }
                if let Some((id, _metrics)) = self
                    .path_cache
                    .get(series_path_cache_key(s.id, 1), constraints)
                {
                    out.push((s.id, id, up));
                }
                if let Some((id, _metrics)) = self
                    .path_cache
                    .get(series_path_cache_key(s.id, 2), constraints)
                {
                    out.push((s.id, id, down));
                }
            }
            return out;
        }

        self.cached_paths.clear();

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

        let fill_style = PathStyle::Fill(fret_core::FillStyle::default());
        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };

        let mut out: Vec<(SeriesId, PathId, Color)> =
            Vec::with_capacity(series_count.saturating_mul(3));
        self.cached_paths = Vec::with_capacity(series_count);

        for (series_index, s) in series.iter().enumerate() {
            let series_id = s.id;
            let stroke_width = resolve_stroke_width(style, s.stroke_width);
            let wick_style = PathStyle::Stroke(fret_core::StrokeStyle {
                width: stroke_width,
            });

            if hidden.contains(&series_id) {
                let view_key = view_key_for_axis(s.y_axis);
                self.cached_paths.push(CachedCandlestickPath {
                    series_id,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width,
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
                candlestick_paths(s, transform, view, cx.scale_factor, stroke_width);

            let wick_id = if wick_cmds.is_empty() {
                None
            } else {
                let key = series_path_cache_key(series_id, 0);
                let (id, _metrics) =
                    self.path_cache
                        .prepare(cx.services, key, &wick_cmds, wick_style, constraints);
                Some(id)
            };
            let up_id = if up_cmds.is_empty() {
                None
            } else {
                let key = series_path_cache_key(series_id, 1);
                let (id, _metrics) =
                    self.path_cache
                        .prepare(cx.services, key, &up_cmds, fill_style, constraints);
                Some(id)
            };
            let down_id = if down_cmds.is_empty() {
                None
            } else {
                let key = series_path_cache_key(series_id, 2);
                let (id, _metrics) =
                    self.path_cache
                        .prepare(cx.services, key, &down_cmds, fill_style, constraints);
                Some(id)
            };

            self.cached_paths.push(CachedCandlestickPath {
                series_id,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width,
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
                    let expected_stroke_width = model
                        .series
                        .get(i)
                        .map(|s| resolve_stroke_width(style, s.stroke_width))
                        .unwrap_or(style.stroke_width);

                    *id == c.series_id
                        && c.view_key == expected_view_key
                        && c.stroke_width == expected_stroke_width
                }) && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
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
        self.path_cache.clear(services);
        self.cached_paths.clear();
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
        self.path_cache.begin_frame();
        report_layer_path_cache_stats(cx, "fret-plot.stairs.paths", &self.path_cache);
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
            ..
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
            self.path_cache.clear(cx.services);
            self.cached_paths.clear();
            return Vec::new();
        }

        let cached_ok = self.cached_paths.len() == series_count
            && self.cached_paths.iter().enumerate().all(|(i, c)| {
                series.get(i).is_some_and(|s| {
                    let expected_view_key = view_key_for_axis(s.y_axis);
                    let expected_stroke_width = resolve_stroke_width(style, s.stroke_width);

                    s.id == c.series_id
                        && c.view_key == expected_view_key
                        && c.stroke_width == expected_stroke_width
                }) && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
            });

        if cached_ok {
            let constraints = PathConstraints {
                scale_factor: cx.scale_factor,
            };
            let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
            for (i, s) in series.iter().enumerate() {
                if hidden.contains(&s.id) {
                    continue;
                }
                let key = series_path_cache_key(s.id, 0);
                let Some((id, _metrics)) = self.path_cache.get(key, constraints) else {
                    continue;
                };
                let style = series_style(s, i, style, series_count);
                out.push((s.id, id, style.stroke_color));
            }
            return out;
        }

        self.cached_paths.clear();

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

        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };

        let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
        self.cached_paths = Vec::with_capacity(series_count);

        for (series_index, s) in series.iter().enumerate() {
            let resolved = series_style(s, series_index, style, series_count);
            let path_style = PathStyle::Stroke(fret_core::StrokeStyle {
                width: resolved.stroke_width,
            });

            let series_id = s.id;
            if hidden.contains(&series_id) {
                let view_key = view_key_for_axis(s.y_axis);
                self.cached_paths.push(CachedPath {
                    series_id,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width: resolved.stroke_width,
                    marker_radius: Px(0.0),
                    marker_shape: MarkerShape::default(),
                    cap_size: Px(0.0),
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
                let key = series_path_cache_key(series_id, 0);
                let (id, _metrics) =
                    self.path_cache
                        .prepare(cx.services, key, &commands, path_style, constraints);
                Some(id)
            };

            self.cached_paths.push(CachedPath {
                series_id,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width: resolved.stroke_width,
                marker_radius: Px(0.0),
                marker_shape: MarkerShape::default(),
                cap_size: Px(0.0),
                view_key,
                samples,
            });

            if let Some(id) = id {
                out.push((series_id, id, resolved.stroke_color));
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
        self.path_cache.clear(services);
        self.cached_paths.clear();
    }
}

fn histogram_series_samples(
    transform: PlotTransform,
    series: &HistogramSeries,
) -> Option<(Vec<SamplePoint>, f32)> {
    let bins = histogram_bins(&series.values, series.bin_count, series.range)?;
    if bins.is_empty() {
        return None;
    }

    let gap = series.bar_gap_fraction.clamp(0.0, 0.95);
    let bar_width = (bins.bin_width as f32) * (1.0 - gap);
    if !bar_width.is_finite() || bar_width <= 0.0 {
        return None;
    }

    let mut samples: Vec<SamplePoint> = Vec::with_capacity(bins.len());
    for (i, count) in bins.counts.iter().copied().enumerate() {
        if !count.is_finite() || count <= 0.0 {
            continue;
        }

        let x = bins.center_x(i);
        if !x.is_finite() {
            continue;
        }

        let data = DataPoint { x, y: count };
        let plot_px = transform.data_to_px(data);
        if !plot_px.x.0.is_finite() || !plot_px.y.0.is_finite() {
            continue;
        }

        samples.push(SamplePoint {
            series_id: series.id,
            index: i,
            data,
            plot_px,
            connects_to_prev: false,
        });
    }

    Some((samples, bar_width))
}

fn histogram_y_at_x(series: &HistogramSeries, x: f64) -> Option<f64> {
    if !x.is_finite() {
        return None;
    }

    let bins = histogram_bins(&series.values, series.bin_count, series.range)?;
    if bins.is_empty() {
        return None;
    }
    if x < bins.x_min || x > bins.x_max {
        return None;
    }

    let mut idx = ((x - bins.x_min) / bins.bin_width).floor() as isize;
    if idx == bins.len() as isize {
        idx = bins.len().saturating_sub(1) as isize;
    }
    if idx < 0 {
        return None;
    }
    let idx = idx as usize;
    bins.counts.get(idx).copied().filter(|v| v.is_finite())
}

impl PlotLayer for HistogramPlotLayer {
    type Model = HistogramPlotModel;

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
        let PlotCursorReadoutArgs { x, hidden, .. } = args;
        if !x.is_finite() {
            return Vec::new();
        }

        let mut out: Vec<PlotCursorReadoutRow> = Vec::new();
        for s in &model.series {
            if hidden.contains(&s.id) {
                continue;
            }
            let y = histogram_y_at_x(s, x);
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
        self.path_cache.begin_frame();
        report_layer_path_cache_stats(cx, "fret-plot.histogram.paths", &self.path_cache);
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
            ..
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
            self.path_cache.clear(cx.services);
            self.cached_paths.clear();
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
            let constraints = PathConstraints {
                scale_factor: cx.scale_factor,
            };
            let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
            for (i, s) in series.iter().enumerate() {
                if hidden.contains(&s.id) {
                    continue;
                }
                let key = series_path_cache_key(s.id, 0);
                let Some((id, _metrics)) = self.path_cache.get(key, constraints) else {
                    continue;
                };
                let color = resolve_series_color(i, style, series_count, s.fill_color);
                out.push((s.id, id, color));
            }
            return out;
        }

        self.cached_paths.clear();

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
                    series_id,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width: style.stroke_width,
                    marker_radius: Px(0.0),
                    marker_shape: MarkerShape::default(),
                    cap_size: Px(0.0),
                    view_key,
                    samples: Vec::new(),
                });
                continue;
            }

            let transform = transform_for_axis(s.y_axis);
            let view_key = view_key_for_axis(s.y_axis);

            let (samples, bar_width) =
                histogram_series_samples(transform, s).unwrap_or((Vec::new(), 0.0));
            let commands = bars_path_commands(transform, &samples, bar_width, 0.0);

            let id = if commands.is_empty() {
                None
            } else {
                let key = series_path_cache_key(series_id, 0);
                let (id, _metrics) =
                    self.path_cache
                        .prepare(cx.services, key, &commands, path_style, constraints);
                Some(id)
            };

            self.cached_paths.push(CachedPath {
                series_id,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width: style.stroke_width,
                marker_radius: Px(0.0),
                marker_shape: MarkerShape::default(),
                cap_size: Px(0.0),
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
                series.get(i).is_some_and(|s| {
                    let expected_view_key = view_key_for_axis(s.y_axis);
                    s.id == c.series_id && c.view_key == expected_view_key
                }) && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
                    && c.stroke_width == style.stroke_width
            });

        let mut best: Option<(PlotHover, f32)> = None;

        if cached_ok {
            for cached in &self.cached_paths {
                if hidden.contains(&cached.series_id) {
                    continue;
                }
                if let Some(pinned) = pinned
                    && cached.series_id != pinned
                {
                    continue;
                }
                for s in cached.samples.iter().copied() {
                    consider_hover_point(&mut best, local, s, threshold2);
                }
            }
        } else {
            let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size);
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

            for s in series {
                if hidden.contains(&s.id) {
                    continue;
                }
                if let Some(pinned) = pinned
                    && s.id != pinned
                {
                    continue;
                }
                let transform = transform_for_axis(s.y_axis);
                if let Some((samples, _bar_width)) = histogram_series_samples(transform, s) {
                    for sp in samples {
                        consider_hover_point(&mut best, local, sp, threshold2);
                    }
                }
            }
        }

        best.and_then(|(hover, d2)| (d2 <= threshold2).then_some(hover))
    }

    fn cleanup_resources(&mut self, services: &mut dyn UiServices) {
        self.path_cache.clear(services);
        self.cached_paths.clear();
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
        self.path_cache.begin_frame();
        report_layer_path_cache_stats(cx, "fret-plot.bars.paths", &self.path_cache);
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
            ..
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
            self.path_cache.clear(cx.services);
            self.cached_paths.clear();
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
            let constraints = PathConstraints {
                scale_factor: cx.scale_factor,
            };
            let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
            for (i, s) in series.iter().enumerate() {
                if hidden.contains(&s.id) {
                    continue;
                }
                let key = series_path_cache_key(s.id, 0);
                let Some((id, _metrics)) = self.path_cache.get(key, constraints) else {
                    continue;
                };
                let color = resolve_series_color(i, style, series_count, s.fill_color);
                out.push((s.id, id, color));
            }
            return out;
        }

        self.cached_paths.clear();

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
                    series_id,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width: style.stroke_width,
                    marker_radius: Px(0.0),
                    marker_shape: MarkerShape::default(),
                    cap_size: Px(0.0),
                    view_key,
                    samples: Vec::new(),
                });
                continue;
            }

            let transform = transform_for_axis(s.y_axis);
            let view_key = view_key_for_axis(s.y_axis);

            let samples = decimate_points(transform, &*s.data, cx.scale_factor, series_id);
            let commands = bars_path_commands_with_baselines(
                transform,
                &samples,
                s.bar_width,
                s.baseline,
                s.baseline_by_index.as_deref(),
            );

            let id = if commands.is_empty() {
                None
            } else {
                let key = series_path_cache_key(series_id, 0);
                let (id, _metrics) =
                    self.path_cache
                        .prepare(cx.services, key, &commands, path_style, constraints);
                Some(id)
            };

            self.cached_paths.push(CachedPath {
                series_id,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width: style.stroke_width,
                marker_radius: Px(0.0),
                marker_shape: MarkerShape::default(),
                cap_size: Px(0.0),
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
        self.path_cache.clear(services);
        self.cached_paths.clear();
    }
}

impl PlotLayer for HeatmapPlotLayer {
    type Model = HeatmapPlotModel;

    fn quads_scene_cache_policy(&self) -> PlotQuadsSceneCachePolicy {
        PlotQuadsSceneCachePolicy::Enabled
    }

    fn data_bounds(model: &Self::Model) -> DataRect {
        model.data_bounds
    }

    fn series_meta(_model: &Self::Model) -> Vec<SeriesMeta> {
        Vec::new()
    }

    fn series_label(_model: &Self::Model, _series_id: SeriesId) -> Option<String> {
        Some("heatmap".to_string())
    }

    fn heatmap_value_range(model: &Self::Model) -> Option<(f32, f32)> {
        Some((model.value_min, model.value_max))
    }

    fn paint_paths<H: UiHost>(
        &mut self,
        _cx: &mut PaintCx<'_, H>,
        _model: &Self::Model,
        _args: PlotPaintArgs<'_>,
    ) -> Vec<(SeriesId, PathId, Color)> {
        Vec::new()
    }

    fn paint_quads_scene_ops_tiled<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        model: &Self::Model,
        args: PlotPaintArgs<'_>,
        plot_origin: Point,
    ) -> bool {
        const TILE_SIZE_CANVAS: f32 = 512.0;
        const TILE_MAX_AGE_FRAMES: u64 = 240;
        const TILE_MAX_ENTRIES: usize = 512;

        let PlotPaintArgs {
            model_revision,
            plot,
            view_bounds,
            x_scale,
            y_scale,
            style,
            view_interacting,
            ..
        } = args;

        if model.cols == 0 || model.rows == 0 {
            self.tile_ops_cache.clear();
            self.tile_scratch.clear();
            return true;
        }

        self.rebuild_mips_if_needed(model_revision, model);

        let dx = (model.data_bounds.x_max - model.data_bounds.x_min) / (model.cols as f64);
        let dy = (model.data_bounds.y_max - model.data_bounds.y_min) / (model.rows as f64);
        if !dx.is_finite() || !dy.is_finite() || dx <= 0.0 || dy <= 0.0 {
            self.tile_ops_cache.clear();
            self.tile_scratch.clear();
            return true;
        }

        let denom = (model.value_max - model.value_min).max(1.0e-12);

        let clip_min_x = view_bounds.x_min.max(model.data_bounds.x_min);
        let clip_max_x = view_bounds.x_max.min(model.data_bounds.x_max);
        let clip_min_y = view_bounds.y_min.max(model.data_bounds.y_min);
        let clip_max_y = view_bounds.y_max.min(model.data_bounds.y_max);

        if clip_max_x <= clip_min_x || clip_max_y <= clip_min_y {
            return true;
        }

        let col0 = (((clip_min_x - model.data_bounds.x_min) / dx).floor() as isize)
            .clamp(0, model.cols.saturating_sub(1) as isize) as usize;
        let col1 = (((clip_max_x - model.data_bounds.x_min) / dx).ceil() as isize)
            .clamp(0, model.cols as isize) as usize;

        let row0 = (((clip_min_y - model.data_bounds.y_min) / dy).floor() as isize)
            .clamp(0, model.rows.saturating_sub(1) as isize) as usize;
        let row1 = (((clip_max_y - model.data_bounds.y_min) / dy).ceil() as isize)
            .clamp(0, model.rows as isize) as usize;

        let visible_cols = col1.saturating_sub(col0);
        let visible_rows = row1.saturating_sub(row0);
        if visible_cols == 0 || visible_rows == 0 {
            return true;
        }

        const MAX_HEATMAP_QUADS: usize = 50_000;
        let max_level = self.mips.len();
        let level = select_grid_mip_level(
            visible_cols,
            visible_rows,
            plot.size.width.0,
            plot.size.height.0,
            MAX_HEATMAP_QUADS,
            max_level,
        );
        let scale = 1usize << level.min(usize::BITS as usize - 1);

        let (grid_cols, grid_rows, values) =
            grid_mip_level_values(level, model.cols, model.rows, &model.values, &self.mips);

        self.colormap_lut.ensure(style.heatmap_colormap, 256);
        let heatmap_color = |t: f32| self.colormap_lut.sample(t);

        let Some(vx_min) = x_scale.to_axis(view_bounds.x_min) else {
            return true;
        };
        let Some(vx_max) = x_scale.to_axis(view_bounds.x_max) else {
            return true;
        };
        let Some(vy_min) = y_scale.to_axis(view_bounds.y_min) else {
            return true;
        };
        let Some(vy_max) = y_scale.to_axis(view_bounds.y_max) else {
            return true;
        };

        let view_w_axis = vx_max - vx_min;
        let view_h_axis = vy_max - vy_min;
        if !view_w_axis.is_finite()
            || !view_h_axis.is_finite()
            || view_w_axis <= 0.0
            || view_h_axis <= 0.0
        {
            return true;
        }

        let scale_x = (plot.size.width.0 as f64) / view_w_axis;
        let scale_y = (plot.size.height.0 as f64) / view_h_axis;

        let translation = Point::new(
            Px((vx_min * scale_x) as f32),
            Px((-(vy_max) * scale_y) as f32),
        );

        let view_rect_world = Rect::new(translation, plot.size);

        self.tile_ops_cache.begin_frame();
        self.tile_ops_cache
            .prune(TILE_MAX_AGE_FRAMES, TILE_MAX_ENTRIES);

        let tile_grid = TileGrid2D::new(TILE_SIZE_CANVAS);
        tile_grid.tiles_in_rect(view_rect_world, &mut self.tile_scratch);
        tile_grid.sort_tiles_center_first(view_rect_world, &mut self.tile_scratch);

        let base_key = {
            let mut b = TileCacheKeyBuilder::new("fret-plot.heatmap.tile.v1");
            b.add_u64(model_revision);
            b.add_u64(model.cols as u64);
            b.add_u64(model.rows as u64);
            b.add_u64(x_scale.key());
            b.add_u64(y_scale.key());
            b.add_u64(u64::from(style.heatmap_colormap.key()));
            b.add_f64_bits(f64::from(plot.size.width.0));
            b.add_f64_bits(f64::from(plot.size.height.0));
            b.add_f64_bits(scale_x);
            b.add_f64_bits(scale_y);
            b.add_u32(model.value_min.to_bits());
            b.add_u32(model.value_max.to_bits());
            b.add_u64(level as u64);
            b.add_u32(TILE_SIZE_CANVAS.to_bits());
            b.finish()
        };

        let plot_origin_x = plot_origin.x.0;
        let plot_origin_y = plot_origin.y.0;
        let translation_x = translation.x.0;
        let translation_y = translation.y.0;

        let tile_budget_limit = InteractionBudget::new(
            style.tile_warmup_tiles_per_frame_idle,
            style.tile_warmup_tiles_per_frame_interactive,
        )
        .select(view_interacting);
        let mut tile_budget = WorkBudget::new(tile_budget_limit);
        let warmup = warm_scene_op_tiles_u64_with(
            &mut self.tile_ops_cache,
            cx.scene,
            &self.tile_scratch,
            base_key,
            1,
            &mut tile_budget,
            |tile| {
                let tile_origin_world = tile.origin(TILE_SIZE_CANVAS);
                let screen_tile_origin = Point::new(
                    Px(tile_origin_world.x.0 - translation_x),
                    Px(tile_origin_world.y.0 - translation_y),
                );
                Point::new(
                    Px(plot_origin_x + screen_tile_origin.x.0),
                    Px(plot_origin_y + screen_tile_origin.y.0),
                )
            },
            |_ops| {},
            |tile| {
                let tile_origin_world = tile.origin(TILE_SIZE_CANVAS);
                let tile_rect_world = Rect::new(
                    tile_origin_world,
                    Size::new(Px(TILE_SIZE_CANVAS), Px(TILE_SIZE_CANVAS)),
                );
                grid_heatmap_tile_ops(
                    tile_rect_world,
                    tile_origin_world,
                    scale_x,
                    scale_y,
                    x_scale,
                    y_scale,
                    model.data_bounds,
                    model.cols,
                    model.rows,
                    dx,
                    dy,
                    model.value_min,
                    denom,
                    grid_cols,
                    grid_rows,
                    values,
                    scale,
                    &heatmap_color,
                )
            },
        );
        let skipped_tiles = warmup.skipped_tiles;

        report_layer_tile_cache_stats(
            cx,
            "fret-plot.heatmap.tiles",
            &self.tile_ops_cache,
            self.tile_scratch.len(),
            tile_budget_limit,
            tile_budget.used(),
            skipped_tiles,
        );
        if skipped_tiles > 0 {
            // Continue warming tiles incrementally to avoid a single frame spike.
            cx.request_redraw();
        }
        true
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
            style,
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
            colormap_key: style.heatmap_colormap.key(),
        };

        if self.cache_key == Some(cache_key) {
            return self.cached_quads.clone();
        }

        self.rebuild_mips_if_needed(model_revision, model);

        self.colormap_lut.ensure(style.heatmap_colormap, 256);
        let heatmap_color = |t: f32| self.colormap_lut.sample(t);

        let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size);
        let transform = PlotTransform {
            viewport: local_viewport,
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let Some(transform) = transform.prepare() else {
            self.cache_key = Some(cache_key);
            self.cached_quads.clear();
            return Vec::new();
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
        let level = select_grid_mip_level(
            visible_cols,
            visible_rows,
            plot.size.width.0,
            plot.size.height.0,
            MAX_HEATMAP_QUADS,
            max_level,
        );

        let scale = 1usize << level.min(usize::BITS as usize - 1);
        let (grid_cols, grid_rows, values) =
            grid_mip_level_values(level, model.cols, model.rows, &model.values, &self.mips);

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
        self.tile_ops_cache.clear();
        self.tile_scratch.clear();
    }
}

impl Histogram2DPlotLayer {
    fn rebuild_mips_if_needed(&mut self, model_revision: u64, model: &Histogram2DPlotModel) {
        let mip_key = Histogram2DMipKey {
            model_revision,
            cols: model.cols,
            rows: model.rows,
            values_ptr: model.values.as_ptr() as usize,
        };

        if self.mip_key == Some(mip_key) {
            return;
        }

        self.mip_key = Some(mip_key);
        self.mips = build_grid_mips(model.cols, model.rows, &model.values);
    }
}

impl PlotLayer for Histogram2DPlotLayer {
    type Model = Histogram2DPlotModel;

    fn quads_scene_cache_policy(&self) -> PlotQuadsSceneCachePolicy {
        PlotQuadsSceneCachePolicy::Enabled
    }

    fn data_bounds(model: &Self::Model) -> DataRect {
        model.data_bounds
    }

    fn series_meta(_model: &Self::Model) -> Vec<SeriesMeta> {
        Vec::new()
    }

    fn series_label(_model: &Self::Model, _series_id: SeriesId) -> Option<String> {
        Some("histogram2d".to_string())
    }

    fn heatmap_value_range(model: &Self::Model) -> Option<(f32, f32)> {
        Some((model.value_min, model.value_max))
    }

    fn paint_paths<H: UiHost>(
        &mut self,
        _cx: &mut PaintCx<'_, H>,
        _model: &Self::Model,
        _args: PlotPaintArgs<'_>,
    ) -> Vec<(SeriesId, PathId, Color)> {
        Vec::new()
    }

    fn paint_quads_scene_ops_tiled<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        model: &Self::Model,
        args: PlotPaintArgs<'_>,
        plot_origin: Point,
    ) -> bool {
        const TILE_SIZE_CANVAS: f32 = 512.0;
        const TILE_MAX_AGE_FRAMES: u64 = 240;
        const TILE_MAX_ENTRIES: usize = 512;

        let PlotPaintArgs {
            model_revision,
            plot,
            view_bounds,
            x_scale,
            y_scale,
            style,
            view_interacting,
            ..
        } = args;

        if model.cols == 0 || model.rows == 0 {
            self.tile_ops_cache.clear();
            self.tile_scratch.clear();
            return true;
        }

        self.rebuild_mips_if_needed(model_revision, model);

        let dx = (model.data_bounds.x_max - model.data_bounds.x_min) / (model.cols as f64);
        let dy = (model.data_bounds.y_max - model.data_bounds.y_min) / (model.rows as f64);
        if !dx.is_finite() || !dy.is_finite() || dx <= 0.0 || dy <= 0.0 {
            self.tile_ops_cache.clear();
            self.tile_scratch.clear();
            return true;
        }

        let denom = (model.value_max - model.value_min).max(1.0e-12);

        let clip_min_x = view_bounds.x_min.max(model.data_bounds.x_min);
        let clip_max_x = view_bounds.x_max.min(model.data_bounds.x_max);
        let clip_min_y = view_bounds.y_min.max(model.data_bounds.y_min);
        let clip_max_y = view_bounds.y_max.min(model.data_bounds.y_max);

        if clip_max_x <= clip_min_x || clip_max_y <= clip_min_y {
            return true;
        }

        let col0 = (((clip_min_x - model.data_bounds.x_min) / dx).floor() as isize)
            .clamp(0, model.cols.saturating_sub(1) as isize) as usize;
        let col1 = (((clip_max_x - model.data_bounds.x_min) / dx).ceil() as isize)
            .clamp(0, model.cols as isize) as usize;

        let row0 = (((clip_min_y - model.data_bounds.y_min) / dy).floor() as isize)
            .clamp(0, model.rows.saturating_sub(1) as isize) as usize;
        let row1 = (((clip_max_y - model.data_bounds.y_min) / dy).ceil() as isize)
            .clamp(0, model.rows as isize) as usize;

        let visible_cols = col1.saturating_sub(col0);
        let visible_rows = row1.saturating_sub(row0);
        if visible_cols == 0 || visible_rows == 0 {
            return true;
        }

        const MAX_HISTOGRAM2D_QUADS: usize = 50_000;
        const MAX_HISTOGRAM2D_MIP_LEVEL: usize = 16;

        let level = select_grid_mip_level(
            visible_cols,
            visible_rows,
            plot.size.width.0,
            plot.size.height.0,
            MAX_HISTOGRAM2D_QUADS,
            MAX_HISTOGRAM2D_MIP_LEVEL.min(self.mips.len()),
        );
        let scale = 1usize << level.min(usize::BITS as usize - 1);

        let (grid_cols, grid_rows, values) =
            grid_mip_level_values(level, model.cols, model.rows, &model.values, &self.mips);

        self.colormap_lut.ensure(style.heatmap_colormap, 256);
        let heatmap_color = |t: f32| self.colormap_lut.sample(t);

        let Some(vx_min) = x_scale.to_axis(view_bounds.x_min) else {
            return true;
        };
        let Some(vx_max) = x_scale.to_axis(view_bounds.x_max) else {
            return true;
        };
        let Some(vy_min) = y_scale.to_axis(view_bounds.y_min) else {
            return true;
        };
        let Some(vy_max) = y_scale.to_axis(view_bounds.y_max) else {
            return true;
        };

        let view_w_axis = vx_max - vx_min;
        let view_h_axis = vy_max - vy_min;
        if !view_w_axis.is_finite()
            || !view_h_axis.is_finite()
            || view_w_axis <= 0.0
            || view_h_axis <= 0.0
        {
            return true;
        }

        let scale_x = (plot.size.width.0 as f64) / view_w_axis;
        let scale_y = (plot.size.height.0 as f64) / view_h_axis;

        let translation = Point::new(
            Px((vx_min * scale_x) as f32),
            Px((-(vy_max) * scale_y) as f32),
        );

        let view_rect_world = Rect::new(translation, plot.size);

        self.tile_ops_cache.begin_frame();
        self.tile_ops_cache
            .prune(TILE_MAX_AGE_FRAMES, TILE_MAX_ENTRIES);

        let tile_grid = TileGrid2D::new(TILE_SIZE_CANVAS);
        tile_grid.tiles_in_rect(view_rect_world, &mut self.tile_scratch);
        tile_grid.sort_tiles_center_first(view_rect_world, &mut self.tile_scratch);

        let base_key = {
            let mut b = TileCacheKeyBuilder::new("fret-plot.histogram2d.tile.v1");
            b.add_u64(model_revision);
            b.add_u64(model.cols as u64);
            b.add_u64(model.rows as u64);
            b.add_u64(x_scale.key());
            b.add_u64(y_scale.key());
            b.add_u64(u64::from(style.heatmap_colormap.key()));
            b.add_f64_bits(f64::from(plot.size.width.0));
            b.add_f64_bits(f64::from(plot.size.height.0));
            b.add_f64_bits(scale_x);
            b.add_f64_bits(scale_y);
            b.add_u32(model.value_min.to_bits());
            b.add_u32(model.value_max.to_bits());
            b.add_u64(level as u64);
            b.add_u32(TILE_SIZE_CANVAS.to_bits());
            b.finish()
        };

        let plot_origin_x = plot_origin.x.0;
        let plot_origin_y = plot_origin.y.0;
        let translation_x = translation.x.0;
        let translation_y = translation.y.0;

        let tile_budget_limit = InteractionBudget::new(
            style.tile_warmup_tiles_per_frame_idle,
            style.tile_warmup_tiles_per_frame_interactive,
        )
        .select(view_interacting);
        let mut tile_budget = WorkBudget::new(tile_budget_limit);
        let warmup = warm_scene_op_tiles_u64_with(
            &mut self.tile_ops_cache,
            cx.scene,
            &self.tile_scratch,
            base_key,
            1,
            &mut tile_budget,
            |tile| {
                let tile_origin_world = tile.origin(TILE_SIZE_CANVAS);
                let screen_tile_origin = Point::new(
                    Px(tile_origin_world.x.0 - translation_x),
                    Px(tile_origin_world.y.0 - translation_y),
                );
                Point::new(
                    Px(plot_origin_x + screen_tile_origin.x.0),
                    Px(plot_origin_y + screen_tile_origin.y.0),
                )
            },
            |_ops| {},
            |tile| {
                let tile_origin_world = tile.origin(TILE_SIZE_CANVAS);
                let tile_rect_world = Rect::new(
                    tile_origin_world,
                    Size::new(Px(TILE_SIZE_CANVAS), Px(TILE_SIZE_CANVAS)),
                );
                grid_heatmap_tile_ops(
                    tile_rect_world,
                    tile_origin_world,
                    scale_x,
                    scale_y,
                    x_scale,
                    y_scale,
                    model.data_bounds,
                    model.cols,
                    model.rows,
                    dx,
                    dy,
                    model.value_min,
                    denom,
                    grid_cols,
                    grid_rows,
                    values,
                    scale,
                    &heatmap_color,
                )
            },
        );
        let skipped_tiles = warmup.skipped_tiles;

        report_layer_tile_cache_stats(
            cx,
            "fret-plot.histogram2d.tiles",
            &self.tile_ops_cache,
            self.tile_scratch.len(),
            tile_budget_limit,
            tile_budget.used(),
            skipped_tiles,
        );
        if skipped_tiles > 0 {
            // Continue warming tiles incrementally to avoid a single frame spike.
            cx.request_redraw();
        }
        true
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
            style,
            ..
        } = args;

        if model.cols == 0 || model.rows == 0 {
            self.cache_key = None;
            self.cached_quads.clear();
            return Vec::new();
        }

        let view_key = data_rect_key_scaled(view_bounds, x_scale, y_scale);
        let cache_key = Histogram2DCacheKey {
            model_revision,
            view_key,
            cols: model.cols,
            rows: model.rows,
            viewport_w_bits: plot.size.width.0.to_bits(),
            viewport_h_bits: plot.size.height.0.to_bits(),
            value_min_bits: model.value_min.to_bits(),
            value_max_bits: model.value_max.to_bits(),
            colormap_key: style.heatmap_colormap.key(),
        };

        if self.cache_key == Some(cache_key) {
            return self.cached_quads.clone();
        }

        self.rebuild_mips_if_needed(model_revision, model);

        self.colormap_lut.ensure(style.heatmap_colormap, 256);
        let heatmap_color = |t: f32| self.colormap_lut.sample(t);

        let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size);
        let transform = PlotTransform {
            viewport: local_viewport,
            data: view_bounds,
            x_scale,
            y_scale,
        };
        let Some(transform) = transform.prepare() else {
            self.cache_key = Some(cache_key);
            self.cached_quads.clear();
            return Vec::new();
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

        const MAX_HISTOGRAM2D_QUADS: usize = 50_000;
        const MAX_HISTOGRAM2D_MIP_LEVEL: usize = 16;

        let level = select_grid_mip_level(
            visible_cols,
            visible_rows,
            plot.size.width.0,
            plot.size.height.0,
            MAX_HISTOGRAM2D_QUADS,
            MAX_HISTOGRAM2D_MIP_LEVEL.min(self.mips.len()),
        );
        let scale = 1usize << level.min(usize::BITS as usize - 1);

        let col0_l = col0 / scale;
        let col1_l = ceil_div_usize(col1, scale).max(col0_l.saturating_add(1));
        let row0_l = row0 / scale;
        let row1_l = ceil_div_usize(row1, scale).max(row0_l.saturating_add(1));

        let (grid_cols, grid_rows, values) =
            grid_mip_level_values(level, model.cols, model.rows, &model.values, &self.mips);
        let col1_l = col1_l.min(grid_cols);
        let row1_l = row1_l.min(grid_rows);

        let mut quads: Vec<PlotQuad> = Vec::with_capacity(
            col1_l
                .saturating_sub(col0_l)
                .saturating_mul(row1_l.saturating_sub(row0_l)),
        );

        for row_l in row0_l..row1_l {
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

            for col_l in col0_l..col1_l {
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

                if quads.len() >= MAX_HISTOGRAM2D_QUADS {
                    break;
                }
            }

            if quads.len() >= MAX_HISTOGRAM2D_QUADS {
                break;
            }
        }

        self.cache_key = Some(cache_key);
        self.cached_quads = quads.clone();
        quads
    }

    fn hit_test(&mut self, model: &Self::Model, args: PlotHitTestArgs<'_>) -> Option<PlotHover> {
        let PlotHitTestArgs {
            plot_size,
            view_bounds,
            x_scale,
            y_scale,
            local,
            ..
        } = args;

        if plot_size.width.0 <= 0.0
            || plot_size.height.0 <= 0.0
            || model.cols == 0
            || model.rows == 0
        {
            return None;
        }

        let transform = PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
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
            series_id: SeriesId::from_label("histogram2d"),
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
        self.tile_ops_cache.clear();
        self.tile_scratch.clear();
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
        self.path_cache.begin_frame();
        report_layer_path_cache_stats(cx, "fret-plot.area.paths", &self.path_cache);
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
            ..
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
            self.path_cache.clear(cx.services);
            self.cached_paths.clear();
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
                    && c.stroke_width == resolve_stroke_width(style, s.stroke_width)
                    && {
                        let expected_view_key = view_key_for_axis(s.y_axis);
                        c.view_key == expected_view_key
                    }
                    && c.baseline_bits == s.baseline.to_bits()
                    && c.fill_alpha_bits == s.fill_alpha.to_bits()
            });

        if cached_ok {
            let constraints = PathConstraints {
                scale_factor: cx.scale_factor,
            };
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

                let fill_key = series_path_cache_key(s.id, 0);
                if let Some((id, _metrics)) = self.path_cache.get(fill_key, constraints) {
                    out.push((s.id, id, fill));
                }
                let stroke_key = series_path_cache_key(s.id, 1);
                if let Some((id, _metrics)) = self.path_cache.get(stroke_key, constraints) {
                    out.push((s.id, id, base_stroke));
                }
            }
            return out;
        }

        self.cached_paths.clear();

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
        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };

        let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count * 2);
        self.cached_paths = Vec::with_capacity(series_count);

        for (series_index, s) in series.iter().enumerate() {
            let series_id = s.id;
            let stroke_width = resolve_stroke_width(style, s.stroke_width);
            let stroke_style = PathStyle::Stroke(fret_core::StrokeStyle {
                width: stroke_width,
            });

            if hidden.contains(&series_id) {
                let view_key = view_key_for_axis(s.y_axis);
                self.cached_paths.push(CachedAreaPath {
                    series_id,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width,
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
                let key = series_path_cache_key(series_id, 0);
                let (id, _metrics) = self.path_cache.prepare(
                    cx.services,
                    key,
                    &fill_commands,
                    fill_style,
                    constraints,
                );
                Some(id)
            };

            let stroke_id = if line_commands.is_empty() || stroke_width.0 <= 0.0 {
                None
            } else {
                let key = series_path_cache_key(series_id, 1);
                let (id, _metrics) = self.path_cache.prepare(
                    cx.services,
                    key,
                    &line_commands,
                    stroke_style,
                    constraints,
                );
                Some(id)
            };

            self.cached_paths.push(CachedAreaPath {
                series_id,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width,
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
                    && c.stroke_width == resolve_stroke_width(style, s.stroke_width)
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
                        consider_hover_segment(&mut best, local, p, sp, transform, threshold2);
                    }
                    consider_hover_point(&mut best, local, sp, threshold2);
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
                        consider_hover_segment(&mut best, local, p, sp, transform, threshold2);
                    }
                    consider_hover_point(&mut best, local, sp, threshold2);
                    prev = Some(sp);
                }
            }
        }

        best.and_then(|(hover, d2)| (d2 <= threshold2).then_some(hover))
    }

    fn cleanup_resources(&mut self, services: &mut dyn UiServices) {
        self.path_cache.clear(services);
        self.cached_paths.clear();
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
        self.path_cache.begin_frame();
        report_layer_path_cache_stats(cx, "fret-plot.shaded.paths", &self.path_cache);
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
            ..
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
            self.path_cache.clear(cx.services);
            self.cached_paths.clear();
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
                    && c.stroke_width == resolve_stroke_width(style, s.stroke_width)
                    && {
                        let expected_view_key = view_key_for_axis(s.y_axis);
                        c.view_key == expected_view_key
                    }
                    && c.fill_alpha_bits == s.fill_alpha.to_bits()
            });

        if cached_ok {
            let constraints = PathConstraints {
                scale_factor: cx.scale_factor,
            };
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

                let fill_key = series_path_cache_key(s.id, 0);
                if let Some((id, _metrics)) = self.path_cache.get(fill_key, constraints) {
                    out.push((s.id, id, fill));
                }
                let upper_key = series_path_cache_key(s.id, 1);
                if let Some((id, _metrics)) = self.path_cache.get(upper_key, constraints) {
                    out.push((s.id, id, base_stroke));
                }
                let lower_key = series_path_cache_key(s.id, 2);
                if let Some((id, _metrics)) = self.path_cache.get(lower_key, constraints) {
                    out.push((s.id, id, base_stroke));
                }
            }
            return out;
        }

        self.cached_paths.clear();

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
        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };

        let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count * 3);
        self.cached_paths = Vec::with_capacity(series_count);

        for (series_index, s) in series.iter().enumerate() {
            let series_id = s.id;
            let stroke_width = resolve_stroke_width(style, s.stroke_width);
            let stroke_style = PathStyle::Stroke(fret_core::StrokeStyle {
                width: stroke_width,
            });

            if hidden.contains(&series_id) {
                let view_key = view_key_for_axis(s.y_axis);
                self.cached_paths.push(CachedShadedPath {
                    series_id,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width,
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
                let key = series_path_cache_key(series_id, 0);
                let (id, _metrics) = self.path_cache.prepare(
                    cx.services,
                    key,
                    &fill_commands,
                    fill_style,
                    constraints,
                );
                Some(id)
            };

            let upper_stroke_id = if upper_line_commands.is_empty() || stroke_width.0 <= 0.0 {
                None
            } else {
                let key = series_path_cache_key(series_id, 1);
                let (id, _metrics) = self.path_cache.prepare(
                    cx.services,
                    key,
                    &upper_line_commands,
                    stroke_style,
                    constraints,
                );
                Some(id)
            };

            let lower_stroke_id = if lower_line_commands.is_empty() || stroke_width.0 <= 0.0 {
                None
            } else {
                let key = series_path_cache_key(series_id, 2);
                let (id, _metrics) = self.path_cache.prepare(
                    cx.services,
                    key,
                    &lower_line_commands,
                    stroke_style,
                    constraints,
                );
                Some(id)
            };

            self.cached_paths.push(CachedShadedPath {
                series_id,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width,
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
                    && c.stroke_width == resolve_stroke_width(style, s.stroke_width)
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
                        consider_hover_segment(&mut best, local, p, sp, transform, threshold2);
                    }
                    consider_hover_point(&mut best, local, sp, threshold2);
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
                        consider_hover_segment(&mut best, local, p, sp, transform, threshold2);
                    }
                    consider_hover_point(&mut best, local, sp, threshold2);
                    prev = Some(sp);
                }
            }
        }

        best.and_then(|(hover, d2)| (d2 <= threshold2).then_some(hover))
    }

    fn cleanup_resources(&mut self, services: &mut dyn UiServices) {
        self.path_cache.clear(services);
        self.cached_paths.clear();
    }
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

fn consider_hover_point(
    best: &mut Option<(PlotHover, f32)>,
    local: Point,
    s: SamplePoint,
    threshold2: f32,
) {
    let dx = s.plot_px.x.0 - local.x.0;
    let dy = s.plot_px.y.0 - local.y.0;
    let d2 = dx * dx + dy * dy;
    if !d2.is_finite() {
        return;
    }
    if d2 > threshold2 {
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
    threshold2: f32,
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
    if d2 > threshold2 {
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

    let base_threshold = hover_threshold.0.max(0.0);
    let effective_threshold_for_stroke_width = |stroke_width: Px| -> f32 {
        let stroke = if stroke_width.0.is_finite() {
            stroke_width.0.max(0.0)
        } else {
            0.0
        };
        base_threshold.max(stroke * 2.0 + 2.0)
    };

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
            let threshold = effective_threshold_for_stroke_width(cached.stroke_width);
            let threshold2 = threshold * threshold;

            let mut prev: Option<SamplePoint> = None;
            for s in cached.samples.iter().copied() {
                if let Some(p) = prev {
                    consider_hover_segment(&mut best, local, p, s, transform, threshold2);
                }
                consider_hover_point(&mut best, local, s, threshold2);
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
            let transform = transform_for_axis(axis);
            let threshold = effective_threshold_for_stroke_width(style.stroke_width);
            let threshold2 = threshold * threshold;

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
                            consider_hover_segment(&mut best, local, p, s, transform, threshold2);
                        }
                        consider_hover_point(&mut best, local, s, threshold2);
                        prev = Some(s);
                    }
                    continue;
                }
            }

            let samples = decimate_samples(transform, data, scale_factor, series_id);
            let mut prev: Option<SamplePoint> = None;
            for s in samples.into_iter() {
                if let Some(p) = prev {
                    consider_hover_segment(&mut best, local, p, s, transform, threshold2);
                }
                consider_hover_point(&mut best, local, s, threshold2);
                prev = Some(s);
            }
        }
    }

    best.map(|(hover, _d2)| hover)
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
        style: _style,
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
    bars_path_commands_with_baselines(transform, samples, bar_width, baseline, None)
}

fn bars_path_commands_with_baselines(
    transform: PlotTransform,
    samples: &[SamplePoint],
    bar_width: f32,
    baseline: f32,
    baselines: Option<&[f64]>,
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
        let baseline = baselines
            .and_then(|b| b.get(s.index).copied())
            .unwrap_or(baseline);
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
