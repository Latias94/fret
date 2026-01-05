use fret_core::PathCommand;
use fret_core::geometry::{Point, Px};

use crate::cartesian::{DataPoint, PlotTransform};
use crate::series::{SeriesData, SeriesId};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SamplePoint {
    pub(crate) series_id: SeriesId,
    pub(crate) index: usize,
    pub(crate) data: DataPoint,
    /// Point in plot-local logical pixels (origin at plot rect origin).
    pub(crate) plot_px: Point,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct BandPoint {
    index: usize,
    upper: DataPoint,
    lower: DataPoint,
    upper_px: Point,
    lower_px: Point,
}

pub(crate) fn decimate_shaded_band(
    transform: PlotTransform,
    upper: &dyn SeriesData,
    lower: &dyn SeriesData,
    scale_factor: f32,
    series_id: SeriesId,
) -> (
    Vec<PathCommand>,
    Vec<PathCommand>,
    Vec<PathCommand>,
    Vec<SamplePoint>,
) {
    let mut fill_commands: Vec<PathCommand> = Vec::new();
    let mut upper_commands: Vec<PathCommand> = Vec::new();
    let mut lower_commands: Vec<PathCommand> = Vec::new();
    let mut samples: Vec<SamplePoint> = Vec::new();

    let mut segment: Vec<BandPoint> = Vec::new();

    let bucket_of = |x: Px| -> i32 {
        let x = x.0 * scale_factor.max(1.0);
        if !x.is_finite() { 0 } else { x.floor() as i32 }
    };

    let mut flush_segment = |segment: &mut Vec<BandPoint>| {
        if segment.is_empty() {
            return;
        }

        if segment.len() == 1 {
            let p = segment[0];
            upper_commands.push(PathCommand::MoveTo(p.upper_px));
            lower_commands.push(PathCommand::MoveTo(p.lower_px));
            samples.push(SamplePoint {
                series_id,
                index: p.index,
                data: p.upper,
                plot_px: p.upper_px,
            });
            samples.push(SamplePoint {
                series_id,
                index: p.index,
                data: p.lower,
                plot_px: p.lower_px,
            });
            segment.clear();
            return;
        }

        let first = segment[0];
        let last = *segment.last().expect("non-empty segment");

        upper_commands.push(PathCommand::MoveTo(first.upper_px));
        lower_commands.push(PathCommand::MoveTo(first.lower_px));
        samples.push(SamplePoint {
            series_id,
            index: first.index,
            data: first.upper,
            plot_px: first.upper_px,
        });
        samples.push(SamplePoint {
            series_id,
            index: first.index,
            data: first.lower,
            plot_px: first.lower_px,
        });

        let mut band_points: Vec<BandPoint> = Vec::new();
        band_points.push(first);

        let mut last_emitted_idx = first.index;
        let mut last_upper_px = first.upper_px;
        let mut last_lower_px = first.lower_px;

        {
            let mut emit = |p: BandPoint| {
                if p.index <= last_emitted_idx {
                    return;
                }

                if p.upper_px == last_upper_px && p.lower_px == last_lower_px {
                    last_emitted_idx = p.index;
                    return;
                }

                upper_commands.push(PathCommand::LineTo(p.upper_px));
                lower_commands.push(PathCommand::LineTo(p.lower_px));
                samples.push(SamplePoint {
                    series_id,
                    index: p.index,
                    data: p.upper,
                    plot_px: p.upper_px,
                });
                samples.push(SamplePoint {
                    series_id,
                    index: p.index,
                    data: p.lower,
                    plot_px: p.lower_px,
                });
                band_points.push(p);

                last_emitted_idx = p.index;
                last_upper_px = p.upper_px;
                last_lower_px = p.lower_px;
            };

            let mut current_bucket: Option<i32> = None;
            let mut min_upper: Option<BandPoint> = None;
            let mut max_upper: Option<BandPoint> = None;
            let mut min_lower: Option<BandPoint> = None;
            let mut max_lower: Option<BandPoint> = None;

            let mut flush_bucket =
                |min_upper: Option<BandPoint>,
                 max_upper: Option<BandPoint>,
                 min_lower: Option<BandPoint>,
                 max_lower: Option<BandPoint>| {
                    let mut candidates: Vec<BandPoint> = Vec::new();
                    for p in [min_upper, max_upper, min_lower, max_lower] {
                        if let Some(p) = p {
                            candidates.push(p);
                        }
                    }

                    candidates.sort_by_key(|p| p.index);
                    candidates.dedup_by_key(|p| p.index);

                    for p in candidates {
                        emit(p);
                    }
                };

            for p in segment
                .iter()
                .copied()
                .skip(1)
                .take(segment.len().saturating_sub(2))
            {
                let b = bucket_of(p.upper_px.x);
                if current_bucket != Some(b) {
                    flush_bucket(
                        min_upper.take(),
                        max_upper.take(),
                        min_lower.take(),
                        max_lower.take(),
                    );
                    current_bucket = Some(b);
                    min_upper = Some(p);
                    max_upper = Some(p);
                    min_lower = Some(p);
                    max_lower = Some(p);
                    continue;
                }

                if let Some(m) = min_upper
                    && p.upper_px.y.0.is_finite()
                    && m.upper_px.y.0.is_finite()
                    && p.upper_px.y.0 < m.upper_px.y.0
                {
                    min_upper = Some(p);
                }
                if let Some(m) = max_upper
                    && p.upper_px.y.0.is_finite()
                    && m.upper_px.y.0.is_finite()
                    && p.upper_px.y.0 > m.upper_px.y.0
                {
                    max_upper = Some(p);
                }
                if let Some(m) = min_lower
                    && p.lower_px.y.0.is_finite()
                    && m.lower_px.y.0.is_finite()
                    && p.lower_px.y.0 < m.lower_px.y.0
                {
                    min_lower = Some(p);
                }
                if let Some(m) = max_lower
                    && p.lower_px.y.0.is_finite()
                    && m.lower_px.y.0.is_finite()
                    && p.lower_px.y.0 > m.lower_px.y.0
                {
                    max_lower = Some(p);
                }
            }

            flush_bucket(
                min_upper.take(),
                max_upper.take(),
                min_lower.take(),
                max_lower.take(),
            );
        }

        if last.index > last_emitted_idx {
            if last.upper_px == last_upper_px && last.lower_px == last_lower_px {
                // Keep hover indices monotonic even if the endpoint collapses.
                samples.push(SamplePoint {
                    series_id,
                    index: last.index,
                    data: last.upper,
                    plot_px: last.upper_px,
                });
                samples.push(SamplePoint {
                    series_id,
                    index: last.index,
                    data: last.lower,
                    plot_px: last.lower_px,
                });
            } else {
                upper_commands.push(PathCommand::LineTo(last.upper_px));
                lower_commands.push(PathCommand::LineTo(last.lower_px));
                samples.push(SamplePoint {
                    series_id,
                    index: last.index,
                    data: last.upper,
                    plot_px: last.upper_px,
                });
                samples.push(SamplePoint {
                    series_id,
                    index: last.index,
                    data: last.lower,
                    plot_px: last.lower_px,
                });
                band_points.push(last);
            }
        }

        if band_points.len() >= 2 {
            fill_commands.push(PathCommand::MoveTo(band_points[0].upper_px));
            for p in band_points.iter().copied().skip(1) {
                fill_commands.push(PathCommand::LineTo(p.upper_px));
            }
            for p in band_points.iter().rev().copied() {
                fill_commands.push(PathCommand::LineTo(p.lower_px));
            }
            fill_commands.push(PathCommand::Close);
        }

        segment.clear();
    };

    let len = upper.len().min(lower.len());
    for idx in 0..len {
        let (Some(u), Some(l)) = (upper.get(idx), lower.get(idx)) else {
            flush_segment(&mut segment);
            continue;
        };
        if !u.x.is_finite() || !u.y.is_finite() || !l.x.is_finite() || !l.y.is_finite() {
            flush_segment(&mut segment);
            continue;
        }

        let u_px = transform.data_to_px(u);
        let l_px = transform.data_to_px(l);
        if !u_px.x.0.is_finite()
            || !u_px.y.0.is_finite()
            || !l_px.x.0.is_finite()
            || !l_px.y.0.is_finite()
        {
            flush_segment(&mut segment);
            continue;
        }

        segment.push(BandPoint {
            index: idx,
            upper: u,
            lower: l,
            upper_px: u_px,
            lower_px: l_px,
        });
    }

    flush_segment(&mut segment);

    (fill_commands, upper_commands, lower_commands, samples)
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

    let mut flush_segment = |segment: &mut Vec<SamplePoint>| {
        if segment.is_empty() {
            return;
        }

        if segment.len() == 1 {
            let p = segment[0];
            commands.push(PathCommand::MoveTo(p.plot_px));
            samples.push(p);
            segment.clear();
            return;
        }

        let first = segment[0];
        let last = *segment.last().expect("non-empty segment");

        commands.push(PathCommand::MoveTo(first.plot_px));
        samples.push(first);

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
                samples.push(p);
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
            samples.push(last);
        } else if last.index > last_emitted_idx && last.plot_px == last_emitted_point {
            // Keep sample indices monotonic for hover even if the point collapses.
            samples.push(last);
        }

        segment.clear();
    };

    if let Some(slice) = points.as_slice() {
        for (idx, p) in slice.iter().copied().enumerate() {
            if !p.x.is_finite() || !p.y.is_finite() {
                flush_segment(&mut segment);
                continue;
            }
            let px = transform.data_to_px(p);
            if !px.x.0.is_finite() || !px.y.0.is_finite() {
                flush_segment(&mut segment);
                continue;
            }
            segment.push(SamplePoint {
                series_id,
                index: idx,
                data: p,
                plot_px: px,
            });
        }
    } else {
        for idx in 0..points.len() {
            let Some(p) = points.get(idx) else {
                flush_segment(&mut segment);
                continue;
            };
            if !p.x.is_finite() || !p.y.is_finite() {
                flush_segment(&mut segment);
                continue;
            }
            let px = transform.data_to_px(p);
            if !px.x.0.is_finite() || !px.y.0.is_finite() {
                flush_segment(&mut segment);
                continue;
            }
            segment.push(SamplePoint {
                series_id,
                index: idx,
                data: p,
                plot_px: px,
            });
        }
    }

    flush_segment(&mut segment);

    (commands, samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_core::geometry::{Rect, Size};

    use crate::cartesian::DataRect;
    use crate::series::{GetterSeriesData, OwnedSeriesData};

    fn transform(viewport_w: f32, viewport_h: f32, data: DataRect) -> PlotTransform {
        PlotTransform {
            viewport: Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(viewport_w), Px(viewport_h)),
            ),
            data,
        }
    }

    #[test]
    fn preserves_spikes_with_min_max_per_bucket() {
        let points: Vec<DataPoint> = (0..100)
            .map(|i| DataPoint {
                x: i as f32,
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
                y: f32::NAN,
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
}
