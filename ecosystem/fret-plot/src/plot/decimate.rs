#![cfg_attr(test, allow(dead_code))]

use fret_core::PathCommand;
use fret_core::geometry::Point;

use crate::cartesian::{DataPoint, PlotTransform};
use crate::series::{SeriesData, SeriesId};

mod band;
mod common;
mod points;
mod polyline;

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

pub(crate) fn decimate_points(
    transform: PlotTransform,
    points: &dyn SeriesData,
    scale_factor: f32,
    series_id: SeriesId,
) -> Vec<SamplePoint> {
    points::decimate_points(transform, points, scale_factor, series_id)
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
    polyline::decimate_polyline(transform, points, scale_factor, series_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use fret_core::geometry::{Px, Rect, Size};

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
