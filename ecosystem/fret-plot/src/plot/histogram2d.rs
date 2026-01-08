use crate::cartesian::{DataPoint, DataRect};

#[derive(Debug, Clone)]
pub struct Histogram2DGrid {
    pub data_bounds: DataRect,
    pub cols: usize,
    pub rows: usize,
    pub values: Vec<f32>,
}

impl Histogram2DGrid {
    pub fn value_range(&self) -> (f32, f32) {
        let mut min_v: Option<f32> = None;
        let mut max_v: Option<f32> = None;
        for v in self.values.iter().copied() {
            if !v.is_finite() {
                continue;
            }
            min_v = Some(min_v.map_or(v, |m| m.min(v)));
            max_v = Some(max_v.map_or(v, |m| m.max(v)));
        }
        match min_v.zip(max_v) {
            Some((min_v, max_v)) if min_v.is_finite() && max_v.is_finite() && max_v >= min_v => {
                (min_v, max_v)
            }
            _ => (0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Histogram2DBins {
    pub cols: usize,
    pub rows: usize,
}

impl Histogram2DBins {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self { cols, rows }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Histogram2DConfig {
    pub bins: Histogram2DBins,
    pub data_bounds: DataRect,
}

impl Histogram2DConfig {
    pub fn new(data_bounds: DataRect, cols: usize, rows: usize) -> Self {
        Self {
            bins: Histogram2DBins::new(cols, rows),
            data_bounds,
        }
    }
}

pub fn histogram2d_counts(
    config: Histogram2DConfig,
    points: impl IntoIterator<Item = DataPoint>,
) -> Histogram2DGrid {
    let cols = config.bins.cols;
    let rows = config.bins.rows;
    let mut values = vec![0.0f32; cols.saturating_mul(rows)];

    let bounds = config.data_bounds;
    let span_x = bounds.x_max - bounds.x_min;
    let span_y = bounds.y_max - bounds.y_min;
    if !span_x.is_finite() || !span_y.is_finite() || span_x <= 0.0 || span_y <= 0.0 {
        return Histogram2DGrid {
            data_bounds: bounds,
            cols,
            rows,
            values,
        };
    }

    for p in points.into_iter() {
        if !p.x.is_finite() || !p.y.is_finite() {
            continue;
        }
        if p.x < bounds.x_min || p.x > bounds.x_max || p.y < bounds.y_min || p.y > bounds.y_max {
            continue;
        }

        let nx = ((p.x - bounds.x_min) / span_x).clamp(0.0, 1.0);
        let ny = ((p.y - bounds.y_min) / span_y).clamp(0.0, 1.0);

        let col = ((nx * cols as f64).floor() as isize).clamp(0, cols.saturating_sub(1) as isize)
            as usize;
        let row = ((ny * rows as f64).floor() as isize).clamp(0, rows.saturating_sub(1) as isize)
            as usize;
        let idx = row.saturating_mul(cols).saturating_add(col);
        if let Some(v) = values.get_mut(idx) {
            *v += 1.0;
        }
    }

    Histogram2DGrid {
        data_bounds: bounds,
        cols,
        rows,
        values,
    }
}

#[derive(Debug, Clone)]
pub struct Histogram2DAccumulator {
    config: Histogram2DConfig,
    values: Vec<f32>,
}

impl Histogram2DAccumulator {
    pub fn new(config: Histogram2DConfig) -> Self {
        let cols = config.bins.cols;
        let rows = config.bins.rows;
        Self {
            config,
            values: vec![0.0f32; cols.saturating_mul(rows)],
        }
    }

    pub fn clear(&mut self) {
        for v in &mut self.values {
            *v = 0.0;
        }
    }

    pub fn add_points(&mut self, points: impl IntoIterator<Item = DataPoint>) {
        let cols = self.config.bins.cols;
        let rows = self.config.bins.rows;
        if cols == 0 || rows == 0 {
            return;
        }

        let bounds = self.config.data_bounds;
        let span_x = bounds.x_max - bounds.x_min;
        let span_y = bounds.y_max - bounds.y_min;
        if !span_x.is_finite() || !span_y.is_finite() || span_x <= 0.0 || span_y <= 0.0 {
            return;
        }

        for p in points.into_iter() {
            if !p.x.is_finite() || !p.y.is_finite() {
                continue;
            }
            if p.x < bounds.x_min || p.x > bounds.x_max || p.y < bounds.y_min || p.y > bounds.y_max
            {
                continue;
            }

            let nx = ((p.x - bounds.x_min) / span_x).clamp(0.0, 1.0);
            let ny = ((p.y - bounds.y_min) / span_y).clamp(0.0, 1.0);
            let col = ((nx * cols as f64).floor() as isize)
                .clamp(0, cols.saturating_sub(1) as isize) as usize;
            let row = ((ny * rows as f64).floor() as isize)
                .clamp(0, rows.saturating_sub(1) as isize) as usize;
            let idx = row.saturating_mul(cols).saturating_add(col);
            if let Some(v) = self.values.get_mut(idx) {
                *v += 1.0;
            }
        }
    }

    pub fn snapshot_grid(&self) -> Histogram2DGrid {
        Histogram2DGrid {
            data_bounds: self.config.data_bounds,
            cols: self.config.bins.cols,
            rows: self.config.bins.rows,
            values: self.values.clone(),
        }
    }
}
