use crate::cartesian::{DataPoint, PlotTransform};

/// Returns the visible X range in data space for the given plot transform.
pub(crate) fn view_x_range(transform: PlotTransform) -> std::ops::RangeInclusive<f64> {
    let x0 = transform.data.x_min;
    let x1 = transform.data.x_max;
    if x0 <= x1 { x0..=x1 } else { x1..=x0 }
}

/// Returns a reasonable point budget for view-dependent sampling.
///
/// This is tuned for generator-like series that can cheaply resample by X range and for
/// downsampling strategies that bucket by device-pixel X.
pub(crate) fn device_point_budget(transform: PlotTransform, scale_factor: f32) -> usize {
    let w = transform.viewport.size.width.0.max(0.0);
    let device_w = (w * scale_factor.max(1.0)).max(1.0);
    // Roughly "2 points per device pixel" is usually enough to preserve spikes after min/max
    // bucketing, while keeping generator series bounded.
    (device_w as usize).saturating_mul(2).max(64)
}

pub(crate) fn visible_sorted_slice(
    points: &[DataPoint],
    x_min: f64,
    x_max: f64,
) -> (usize, &[DataPoint]) {
    if points.is_empty() {
        return (0, points);
    }

    // If the slice contains NaNs in X, binary search is not well-defined. Fall back to full slice.
    if points.iter().any(|p| p.x.is_nan()) {
        return (0, points);
    }

    let (lo, hi) = if x_min <= x_max {
        (x_min, x_max)
    } else {
        (x_max, x_min)
    };

    let start = points.partition_point(|p| p.x < lo);
    let end = points.partition_point(|p| p.x <= hi);

    let start = start.saturating_sub(1);
    let end = (end + 1).min(points.len());

    (start, &points[start..end])
}
