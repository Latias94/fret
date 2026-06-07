#![cfg_attr(test, allow(dead_code))]

use fret_core::PathCommand;
use fret_core::geometry::{Point, Px};

use crate::cartesian::{DataPoint, PlotTransform};
use crate::series::{SeriesData, SeriesId};

mod band;

pub(crate) use band::decimate_shaded_band;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SamplePoint {
    pub(crate) series_id: SeriesId,
    pub(crate) index: usize,
    pub(crate) data: DataPoint,
    /// Point in plot-local logical pixels (origin at plot rect origin).
    pub(crate) plot_px: Point,
    /// Whether this point is connected to the previous emitted point in the same sample stream.
    ///
    /// This is used for hit testing against line segments. A `false` value indicates a segment
    /// boundary (e.g. due to missing/non-finite data).
    pub(crate) connects_to_prev: bool,
}

pub(crate) fn decimate_samples(
    transform: PlotTransform,
    points: &dyn SeriesData,
    scale_factor: f32,
    series_id: SeriesId,
) -> Vec<SamplePoint> {
    let (_commands, samples) = decimate_polyline(transform, points, scale_factor, series_id);
    samples
}

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

fn flush_polyline_segment(
    commands: &mut Vec<PathCommand>,
    samples: &mut Vec<SamplePoint>,
    segment: &mut Vec<SamplePoint>,
    scale_factor: f32,
) {
    if segment.is_empty() {
        return;
    }

    if segment.len() == 1 {
        let p = segment[0];
        commands.push(PathCommand::MoveTo(p.plot_px));
        samples.push(SamplePoint {
            connects_to_prev: false,
            ..p
        });
        segment.clear();
        return;
    }

    let first = segment[0];
    let last = *segment.last().expect("non-empty segment");

    commands.push(PathCommand::MoveTo(first.plot_px));
    samples.push(SamplePoint {
        connects_to_prev: false,
        ..first
    });

    let mut last_emitted_idx = first.index;
    let mut last_emitted_point = first.plot_px;

    let bucket_of = |x: Px| -> i32 {
        let x = x.0 * scale_factor.max(1.0);
        if !x.is_finite() { 0 } else { x.floor() as i32 }
    };

    let mut current_bucket: Option<i32> = None;
    let mut min: Option<SamplePoint> = None;
    let mut max: Option<SamplePoint> = None;

    let mut flush_bucket = |min: Option<SamplePoint>, max: Option<SamplePoint>| {
        let (Some(min), Some(max)) = (min, max) else {
            return;
        };

        let mut a = min;
        let mut b = max;
        if a.index > b.index {
            std::mem::swap(&mut a, &mut b);
        }

        for p in [a, b] {
            if p.index <= last_emitted_idx {
                continue;
            }
            if p.plot_px == last_emitted_point {
                last_emitted_idx = p.index;
                continue;
            }
            commands.push(PathCommand::LineTo(p.plot_px));
            samples.push(SamplePoint {
                connects_to_prev: true,
                ..p
            });
            last_emitted_idx = p.index;
            last_emitted_point = p.plot_px;
        }
    };

    // Exclude endpoints from bucketing (they are emitted explicitly).
    for p in segment
        .iter()
        .copied()
        .skip(1)
        .take(segment.len().saturating_sub(2))
    {
        let b = bucket_of(p.plot_px.x);
        if current_bucket != Some(b) {
            flush_bucket(min.take(), max.take());
            current_bucket = Some(b);
            min = Some(p);
            max = Some(p);
            continue;
        }

        if let Some(m) = min
            && p.plot_px.y.0.is_finite()
            && m.plot_px.y.0.is_finite()
            && p.plot_px.y.0 < m.plot_px.y.0
        {
            min = Some(p);
        }
        if let Some(m) = max
            && p.plot_px.y.0.is_finite()
            && m.plot_px.y.0.is_finite()
            && p.plot_px.y.0 > m.plot_px.y.0
        {
            max = Some(p);
        }
    }

    flush_bucket(min.take(), max.take());

    if last.index > last_emitted_idx && last.plot_px != last_emitted_point {
        commands.push(PathCommand::LineTo(last.plot_px));
        samples.push(SamplePoint {
            connects_to_prev: true,
            ..last
        });
    } else if last.index > last_emitted_idx && last.plot_px == last_emitted_point {
        // Keep sample indices monotonic for hover even if the point collapses.
        samples.push(SamplePoint {
            connects_to_prev: true,
            ..last
        });
    }

    segment.clear();
}

fn push_poly_point(
    commands: &mut Vec<PathCommand>,
    samples: &mut Vec<SamplePoint>,
    segment: &mut Vec<SamplePoint>,
    transform: PlotTransform,
    scale_factor: f32,
    series_id: SeriesId,
    index: usize,
    p: DataPoint,
) {
    if !p.x.is_finite() || !p.y.is_finite() {
        flush_polyline_segment(commands, samples, segment, scale_factor);
        return;
    }
    let px = transform.data_to_px(p);
    if !px.x.0.is_finite() || !px.y.0.is_finite() {
        flush_polyline_segment(commands, samples, segment, scale_factor);
        return;
    }
    segment.push(SamplePoint {
        series_id,
        index,
        data: p,
        plot_px: px,
        connects_to_prev: false,
    });
}

/// Produces a decimated polyline suitable for large datasets.
///
/// Strategy: bucket by device-pixel X (plot-local), then emit min/max Y points per bucket to
/// preserve spikes while bounding the output size to O(plot_width_px).
pub(crate) fn decimate_polyline(
    transform: PlotTransform,
    points: &dyn SeriesData,
    scale_factor: f32,
    series_id: SeriesId,
) -> (Vec<PathCommand>, Vec<SamplePoint>) {
    let mut commands: Vec<PathCommand> = Vec::new();
    let mut samples: Vec<SamplePoint> = Vec::new();

    let mut segment: Vec<SamplePoint> = Vec::new();

    let view_range = view_x_range(transform);
    let budget = device_point_budget(transform, scale_factor);

    if let Some(sampled) = points.sample_range(view_range.clone(), budget) {
        for (i, p) in sampled.into_iter().enumerate() {
            push_poly_point(
                &mut commands,
                &mut samples,
                &mut segment,
                transform,
                scale_factor,
                series_id,
                i,
                p,
            );
        }
    } else if let Some(slice) = points.as_slice() {
        if points.is_sorted_by_x() {
            let (base, visible) =
                visible_sorted_slice(slice, *view_range.start(), *view_range.end());
            for (i, p) in visible.iter().copied().enumerate() {
                push_poly_point(
                    &mut commands,
                    &mut samples,
                    &mut segment,
                    transform,
                    scale_factor,
                    series_id,
                    base + i,
                    p,
                );
            }
        } else {
            for (idx, p) in slice.iter().copied().enumerate() {
                push_poly_point(
                    &mut commands,
                    &mut samples,
                    &mut segment,
                    transform,
                    scale_factor,
                    series_id,
                    idx,
                    p,
                );
            }
        }
    } else if points.is_sorted_by_x() {
        let mut started = false;
        let lo = *view_range.start();
        let hi = *view_range.end();
        for idx in 0..points.len() {
            let Some(p) = points.get(idx) else {
                flush_polyline_segment(&mut commands, &mut samples, &mut segment, scale_factor);
                started = false;
                continue;
            };
            if p.x.is_finite() {
                if !started && p.x < lo {
                    continue;
                }
                if started && p.x > hi {
                    break;
                }
                if p.x >= lo {
                    started = true;
                }
            }
            push_poly_point(
                &mut commands,
                &mut samples,
                &mut segment,
                transform,
                scale_factor,
                series_id,
                idx,
                p,
            );
        }
    } else {
        for idx in 0..points.len() {
            let Some(p) = points.get(idx) else {
                flush_polyline_segment(&mut commands, &mut samples, &mut segment, scale_factor);
                continue;
            };
            push_poly_point(
                &mut commands,
                &mut samples,
                &mut segment,
                transform,
                scale_factor,
                series_id,
                idx,
                p,
            );
        }
    }

    flush_polyline_segment(&mut commands, &mut samples, &mut segment, scale_factor);

    (commands, samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use fret_core::geometry::{Rect, Size};

    use crate::cartesian::{AxisScale, DataRect};
    use crate::series::{GetterSeriesData, OwnedSeriesData};

    fn transform(viewport_w: f32, viewport_h: f32, data: DataRect) -> PlotTransform {
        PlotTransform {
            viewport: Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(viewport_w), Px(viewport_h)),
            ),
            data,
            x_scale: AxisScale::Linear,
            y_scale: AxisScale::Linear,
        }
    }

    #[test]
    fn preserves_spikes_with_min_max_per_bucket() {
        let points: Vec<DataPoint> = (0..100)
            .map(|i| DataPoint {
                x: i as f64,
                y: 0.0,
            })
            .collect();
        let mut points = points;
        points[40].y = 10.0;
        points[60].y = -10.0;

        let data_bounds = DataRect {
            x_min: 0.0,
            x_max: 99.0,
            y_min: -10.0,
            y_max: 10.0,
        };

        // Collapse X heavily so most points fall into a small set of pixel buckets.
        let transform = transform(8.0, 80.0, data_bounds);
        let series = OwnedSeriesData::new(points);

        let (_commands, samples) = decimate_polyline(transform, &series, 1.0, SeriesId(123));
        let indices: Vec<usize> = samples.iter().map(|s| s.index).collect();

        assert!(indices.contains(&40), "expected the spike to be sampled");
        assert!(indices.contains(&60), "expected the valley to be sampled");

        assert!(samples.windows(2).all(|w| w[0].index <= w[1].index));
    }

    #[test]
    fn breaks_segments_on_non_finite_points() {
        let points = vec![
            DataPoint { x: 0.0, y: 0.0 },
            DataPoint { x: 1.0, y: 1.0 },
            DataPoint { x: 2.0, y: 2.0 },
            DataPoint {
                x: 3.0,
                y: f64::NAN,
            },
            DataPoint { x: 4.0, y: 4.0 },
            DataPoint { x: 5.0, y: 5.0 },
        ];

        let data_bounds = DataRect {
            x_min: 0.0,
            x_max: 5.0,
            y_min: 0.0,
            y_max: 5.0,
        };
        let transform = transform(100.0, 100.0, data_bounds);
        let series = OwnedSeriesData::new(points);

        let (commands, _samples) = decimate_polyline(transform, &series, 1.0, SeriesId(1));
        let move_tos = commands
            .iter()
            .filter(|c| matches!(c, PathCommand::MoveTo(_)))
            .count();
        assert_eq!(
            move_tos, 2,
            "expected two subpaths due to NaN discontinuity"
        );
    }

    #[test]
    fn getter_none_breaks_segments() {
        let series = GetterSeriesData::new(6, |i| match i {
            0 => Some(DataPoint { x: 0.0, y: 0.0 }),
            1 => Some(DataPoint { x: 1.0, y: 1.0 }),
            2 => None,
            3 => Some(DataPoint { x: 3.0, y: 3.0 }),
            4 => Some(DataPoint { x: 4.0, y: 4.0 }),
            _ => Some(DataPoint { x: 5.0, y: 5.0 }),
        });

        let data_bounds = DataRect {
            x_min: 0.0,
            x_max: 5.0,
            y_min: 0.0,
            y_max: 5.0,
        };
        let transform = transform(100.0, 100.0, data_bounds);

        let (commands, _samples) = decimate_polyline(transform, &series, 1.0, SeriesId(2));
        let move_tos = commands
            .iter()
            .filter(|c| matches!(c, PathCommand::MoveTo(_)))
            .count();
        assert_eq!(
            move_tos, 2,
            "expected two subpaths due to missing getter point"
        );
    }

    #[test]
    fn shaded_band_resamples_by_x_for_sorted_series() {
        let upper_points = Arc::new(vec![
            DataPoint { x: 0.0, y: 0.0 },
            DataPoint { x: 1.0, y: 1.0 },
            DataPoint { x: 2.0, y: 0.0 },
        ]);
        let lower_points = Arc::new(vec![
            DataPoint { x: 0.0, y: -1.0 },
            DataPoint { x: 0.5, y: -0.5 },
            DataPoint { x: 2.0, y: -1.0 },
        ]);

        let upper = GetterSeriesData::new(upper_points.len(), {
            let points = upper_points.clone();
            move |i| points.get(i).copied()
        })
        .sorted_by_x(true);
        let lower = GetterSeriesData::new(lower_points.len(), {
            let points = lower_points.clone();
            move |i| points.get(i).copied()
        })
        .sorted_by_x(true);

        let data_bounds = DataRect {
            x_min: 0.0,
            x_max: 2.0,
            y_min: -2.0,
            y_max: 2.0,
        };
        let transform = transform(1000.0, 100.0, data_bounds);

        let (fill, _upper_cmds, _lower_cmds, samples) =
            decimate_shaded_band(transform, &upper, &lower, 1.0, SeriesId(7));

        assert!(!fill.is_empty(), "expected a filled band path");
        assert!(
            samples.iter().any(|s| s.data.x == 0.5),
            "expected the union X grid to include x=0.5"
        );

        let mut found_upper_interpolated = false;
        for s in &samples {
            if s.data.x == 0.5 && (s.data.y - 0.5).abs() <= 1e-4 {
                found_upper_interpolated = true;
                break;
            }
        }
        assert!(
            found_upper_interpolated,
            "expected upper Y to be interpolated at x=0.5"
        );
    }

    #[test]
    fn shaded_band_breaks_segments_on_missing_points() {
        let upper = GetterSeriesData::new(5, |i| match i {
            0 => Some(DataPoint { x: 0.0, y: 0.0 }),
            1 => Some(DataPoint { x: 1.0, y: 1.0 }),
            2 => None,
            3 => Some(DataPoint { x: 3.0, y: 0.0 }),
            _ => Some(DataPoint { x: 4.0, y: 1.0 }),
        })
        .sorted_by_x(true);

        let lower = GetterSeriesData::new(5, |i| match i {
            0 => Some(DataPoint { x: 0.0, y: -1.0 }),
            1 => Some(DataPoint { x: 1.0, y: -1.0 }),
            2 => Some(DataPoint { x: 2.0, y: -1.0 }),
            3 => Some(DataPoint { x: 3.0, y: -1.0 }),
            _ => Some(DataPoint { x: 4.0, y: -1.0 }),
        })
        .sorted_by_x(true);

        let data_bounds = DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: -2.0,
            y_max: 2.0,
        };
        let transform = transform(1000.0, 100.0, data_bounds);

        let (fill, _upper_cmds, _lower_cmds, _samples) =
            decimate_shaded_band(transform, &upper, &lower, 1.0, SeriesId(9));
        let move_tos = fill
            .iter()
            .filter(|c| matches!(c, PathCommand::MoveTo(_)))
            .count();
        assert_eq!(move_tos, 2, "expected two shaded band segments");
    }
}
