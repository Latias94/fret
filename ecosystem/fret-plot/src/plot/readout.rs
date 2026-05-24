use std::collections::HashSet;
use std::ops::RangeInclusive;
use std::sync::Arc;

use fret_core::{Px, Rect, Size};

use crate::cartesian::{AxisScale, DataPoint, DataRect, PlotTransform};
use crate::models::{LinePlotModel, YAxis};
use crate::series::{SeriesData, SeriesId};

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

#[derive(Clone)]
pub struct PlotCursorReadoutSeries<'a> {
    pub id: SeriesId,
    pub label: Arc<str>,
    pub y_axis: YAxis,
    pub data: &'a dyn SeriesData,
}

pub fn line_plot_cursor_readout(
    model: &LinePlotModel,
    args: PlotCursorReadoutArgs<'_>,
) -> Vec<PlotCursorReadoutRow> {
    plot_cursor_readout(
        model.series.iter().map(|series| PlotCursorReadoutSeries {
            id: series.id,
            label: series.label.clone(),
            y_axis: series.y_axis,
            data: &*series.data,
        }),
        args,
    )
}

pub fn plot_cursor_readout<'a>(
    series: impl IntoIterator<Item = PlotCursorReadoutSeries<'a>>,
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
        viewport: Rect::new(fret_core::Point::new(Px(0.0), Px(0.0)), plot_size),
        data: view_bounds,
        x_scale,
        y_scale,
    };
    let view_x = view_x_range(transform);
    let view_x = (view_x.start().is_finite() && view_x.end().is_finite()).then_some(view_x);
    let budget = device_point_budget(transform, scale_factor);

    let mut out: Vec<PlotCursorReadoutRow> = Vec::new();
    for series in series {
        if hidden.contains(&series.id) {
            continue;
        }
        let y = cursor_readout_y_at_x(series.data, x, view_x.clone(), budget);
        out.push(PlotCursorReadoutRow {
            series_id: series.id,
            label: series.label,
            y_axis: series.y_axis,
            y,
        });
    }
    out
}

/// Estimates a series' Y value at the given X coordinate for cursor readouts.
///
/// Strategy:
/// - If the series reports `sorted_by_x`, do an O(log N) lookup + linear interpolation.
/// - Otherwise, first try `SeriesData::sample_range` (view-dependent, bounded by `budget`) and
///   interpolate within the sampled polyline.
/// - As a last resort, do a budgeted scan to find the nearest-X point (O(budget)).
pub fn cursor_readout_y_at_x(
    series: &dyn SeriesData,
    x: f64,
    view_x_range: Option<RangeInclusive<f64>>,
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

fn view_x_range(transform: PlotTransform) -> RangeInclusive<f64> {
    let x0 = transform.data.x_min;
    let x1 = transform.data.x_max;
    if x0 <= x1 { x0..=x1 } else { x1..=x0 }
}

fn device_point_budget(transform: PlotTransform, scale_factor: f32) -> usize {
    let w = transform.viewport.size.width.0.max(0.0);
    let device_w = (w * scale_factor.max(1.0)).max(1.0);
    (device_w as usize).saturating_mul(2).max(64)
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
    let stride = len.div_ceil(budget).max(1);

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
        if left >= lo
            && let Some(p) = series.get(left)
            && p.x.is_finite()
            && p.y.is_finite()
        {
            return Some((left, p));
        }

        let right = center.saturating_add(step);
        if step > 0
            && right < hi
            && let Some(p) = series.get(right)
            && p.x.is_finite()
            && p.y.is_finite()
        {
            return Some((right, p));
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

#[cfg(test)]
mod tests {
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
                _x_range: RangeInclusive<f64>,
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
