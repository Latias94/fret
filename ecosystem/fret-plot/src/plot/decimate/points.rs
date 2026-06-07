use fret_core::geometry::Point;

use crate::cartesian::{DataPoint, PlotTransform};
use crate::series::{SeriesData, SeriesId};

use super::SamplePoint;
use super::common::{device_point_budget, view_x_range, visible_sorted_slice};

/// Produces a decimated point cloud suitable for scatter-like plots.
///
/// Strategy: bucket by device-pixel X (plot-local), then keep a single representative point per
/// bucket (closest-to-center in X) to keep the output bounded to O(plot_width_px).
pub(crate) fn decimate_points(
    transform: PlotTransform,
    points: &dyn SeriesData,
    scale_factor: f32,
    series_id: SeriesId,
) -> Vec<SamplePoint> {
    let view_range = view_x_range(transform);
    let budget = device_point_budget(transform, scale_factor);

    // Bucket state for a single device pixel column.
    #[derive(Clone, Copy)]
    struct BucketBest {
        idx: usize,
        data: DataPoint,
        plot_px: Point,
        dx2: f32,
    }

    let scale = scale_factor.max(1.0);
    let mut buckets: std::collections::HashMap<i32, BucketBest> = std::collections::HashMap::new();

    let mut consider = |idx: usize, p: DataPoint| {
        if !p.x.is_finite() || !p.y.is_finite() {
            return;
        }

        let plot_px = transform.data_to_px(p);
        if !plot_px.x.0.is_finite() || !plot_px.y.0.is_finite() {
            return;
        }

        let device_x = (plot_px.x.0 * scale).floor() as i32;

        // Prefer points closest to the center of the device pixel column.
        let center = (device_x as f32 + 0.5) / scale;
        let dx = plot_px.x.0 - center;
        let dx2 = dx * dx;
        if !dx2.is_finite() {
            return;
        }

        let cand = BucketBest {
            idx,
            data: p,
            plot_px,
            dx2,
        };

        buckets
            .entry(device_x)
            .and_modify(|b| {
                if dx2 < b.dx2 {
                    *b = cand;
                }
            })
            .or_insert(cand);
    };

    if let Some(sampled) = points.sample_range(view_range.clone(), budget) {
        for (i, p) in sampled.into_iter().enumerate() {
            consider(i, p);
        }
    } else if let Some(slice) = points.as_slice() {
        let lo = *view_range.start();
        let hi = *view_range.end();
        if points.is_sorted_by_x() {
            let (base, visible) = visible_sorted_slice(slice, lo, hi);
            for (i, p) in visible.iter().copied().enumerate() {
                consider(base + i, p);
            }
        } else {
            for (idx, p) in slice.iter().copied().enumerate() {
                if !p.x.is_finite() {
                    continue;
                }
                if p.x < lo || p.x > hi {
                    continue;
                }
                consider(idx, p);
            }
        }
    } else {
        // Getter-backed fallback.
        let lo = *view_range.start();
        let hi = *view_range.end();
        let limit = if points.is_sorted_by_x() {
            points.len()
        } else {
            points.len().min(budget.saturating_mul(16).max(1024))
        };
        for idx in 0..limit {
            let Some(p) = points.get(idx) else {
                continue;
            };
            if points.is_sorted_by_x() {
                if p.x < lo {
                    continue;
                }
                if p.x > hi {
                    break;
                }
            } else if p.x.is_finite() && (p.x < lo || p.x > hi) {
                continue;
            }
            consider(idx, p);
        }
    }

    let mut out: Vec<SamplePoint> = buckets
        .into_values()
        .map(|b| SamplePoint {
            series_id,
            index: b.idx,
            data: b.data,
            plot_px: b.plot_px,
            connects_to_prev: false,
        })
        .collect();
    out.sort_by(|a, b| a.plot_px.x.0.total_cmp(&b.plot_px.x.0));
    out
}
