use fret_core::PathCommand;
use fret_core::geometry::Px;

use crate::cartesian::{DataPoint, PlotTransform};
use crate::series::{SeriesData, SeriesId};

use super::SamplePoint;
use super::common::{device_point_budget, view_x_range, visible_sorted_slice};

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
