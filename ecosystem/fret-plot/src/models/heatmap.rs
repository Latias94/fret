use std::sync::Arc;

use crate::cartesian::DataRect;
use crate::plot::view::sanitize_data_rect;

#[derive(Debug, Clone)]
pub struct HeatmapPlotModel {
    /// Grid domain in data space.
    pub data_bounds: DataRect,
    pub cols: usize,
    pub rows: usize,
    /// Row-major values, length == cols * rows.
    pub values: Arc<[f32]>,
    pub value_min: f32,
    pub value_max: f32,
}

impl HeatmapPlotModel {
    pub fn new(
        data_bounds: DataRect,
        cols: usize,
        rows: usize,
        values: impl Into<Arc<[f32]>>,
    ) -> Self {
        let values: Arc<[f32]> = values.into();
        let expected = cols.saturating_mul(rows);
        debug_assert_eq!(values.len(), expected, "values.len != cols*rows");

        let mut min_v: Option<f32> = None;
        let mut max_v: Option<f32> = None;
        for v in values.iter().copied() {
            if !v.is_finite() {
                continue;
            }
            min_v = Some(min_v.map_or(v, |m| m.min(v)));
            max_v = Some(max_v.map_or(v, |m| m.max(v)));
        }

        let (value_min, value_max) = match min_v.zip(max_v) {
            Some((min_v, max_v)) if min_v.is_finite() && max_v.is_finite() && max_v >= min_v => {
                (min_v, max_v)
            }
            _ => (0.0, 1.0),
        };

        Self {
            data_bounds: sanitize_data_rect(data_bounds),
            cols,
            rows,
            values,
            value_min,
            value_max,
        }
    }

    pub fn value_at(&self, col: usize, row: usize) -> Option<f32> {
        if col >= self.cols || row >= self.rows {
            return None;
        }
        let idx = row.saturating_mul(self.cols).saturating_add(col);
        self.values.get(idx).copied()
    }
}

#[derive(Debug, Clone)]
pub struct Histogram2DPlotModel {
    /// Grid domain in data space.
    pub data_bounds: DataRect,
    pub cols: usize,
    pub rows: usize,
    /// Row-major bin values, length == cols * rows.
    pub values: Arc<[f32]>,
    pub value_min: f32,
    pub value_max: f32,
}

impl Histogram2DPlotModel {
    pub fn new(
        data_bounds: DataRect,
        cols: usize,
        rows: usize,
        values: impl Into<Arc<[f32]>>,
    ) -> Self {
        let values: Arc<[f32]> = values.into();
        let expected = cols.saturating_mul(rows);
        debug_assert_eq!(values.len(), expected, "values.len != cols*rows");

        let mut min_v: Option<f32> = None;
        let mut max_v: Option<f32> = None;
        for v in values.iter().copied() {
            if !v.is_finite() {
                continue;
            }
            min_v = Some(min_v.map_or(v, |m| m.min(v)));
            max_v = Some(max_v.map_or(v, |m| m.max(v)));
        }

        let (value_min, value_max) = match min_v.zip(max_v) {
            Some((min_v, max_v)) if min_v.is_finite() && max_v.is_finite() && max_v >= min_v => {
                (min_v, max_v)
            }
            _ => (0.0, 1.0),
        };

        Self {
            data_bounds: sanitize_data_rect(data_bounds),
            cols,
            rows,
            values,
            value_min,
            value_max,
        }
    }

    pub fn value_at(&self, col: usize, row: usize) -> Option<f32> {
        if col >= self.cols || row >= self.rows {
            return None;
        }
        let idx = row.saturating_mul(self.cols).saturating_add(col);
        self.values.get(idx).copied()
    }
}
