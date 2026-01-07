use core::ops::Range;

use fret_core::{Point, Px, Rect};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct DataBounds {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl DataBounds {
    pub fn is_valid(&self) -> bool {
        self.x_min.is_finite()
            && self.x_max.is_finite()
            && self.y_min.is_finite()
            && self.y_max.is_finite()
            && self.x_max > self.x_min
            && self.y_max > self.y_min
    }

    pub fn clamp_non_degenerate(&mut self) {
        if !self.x_min.is_finite() || !self.x_max.is_finite() || self.x_max <= self.x_min {
            self.x_min = 0.0;
            self.x_max = 1.0;
        }
        if !self.y_min.is_finite() || !self.y_max.is_finite() || self.y_max <= self.y_min {
            self.y_min = 0.0;
            self.y_max = 1.0;
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct LodScratch {
    buckets: Vec<Bucket>,
    tmp_indices: Vec<usize>,
}

impl LodScratch {
    pub fn clear(&mut self) {
        self.reset_buckets();
        self.tmp_indices.clear();
    }

    pub fn ensure_bucket_count(&mut self, count: usize) {
        if self.buckets.len() < count {
            self.buckets.resize_with(count, Bucket::default);
        }
        self.buckets.truncate(count);
    }

    pub fn reset_buckets(&mut self) {
        for bucket in &mut self.buckets {
            bucket.clear();
        }
    }

    pub fn tmp_indices_mut(&mut self) -> &mut Vec<usize> {
        &mut self.tmp_indices
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Bucket {
    first: Option<usize>,
    last: Option<usize>,
    min: Option<usize>,
    max: Option<usize>,
}

impl Bucket {
    fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Default, Clone)]
pub struct MinMaxPerPixelCursor {
    pub next_index: usize,
}

pub fn compute_bounds(x: &[f64], y: &[f64]) -> Option<DataBounds> {
    let mut bounds = DataBounds {
        x_min: f64::INFINITY,
        x_max: f64::NEG_INFINITY,
        y_min: f64::INFINITY,
        y_max: f64::NEG_INFINITY,
    };

    let len = x.len().min(y.len());
    for i in 0..len {
        let xi = x[i];
        let yi = y[i];
        if !xi.is_finite() || !yi.is_finite() {
            continue;
        }
        bounds.x_min = bounds.x_min.min(xi);
        bounds.x_max = bounds.x_max.max(xi);
        bounds.y_min = bounds.y_min.min(yi);
        bounds.y_max = bounds.y_max.max(yi);
    }

    if bounds.x_min.is_finite()
        && bounds.x_max.is_finite()
        && bounds.y_min.is_finite()
        && bounds.y_max.is_finite()
    {
        Some(bounds)
    } else {
        None
    }
}

pub fn minmax_per_pixel_step(
    cursor: &mut MinMaxPerPixelCursor,
    scratch: &mut LodScratch,
    x: &[f64],
    y: &[f64],
    bounds: &DataBounds,
    viewport: Rect,
    max_points_to_process: usize,
) -> bool {
    let width_px = viewport.size.width.0.max(1.0).ceil() as usize;
    scratch.ensure_bucket_count(width_px);

    let len = x.len().min(y.len());
    if cursor.next_index >= len {
        return true;
    }

    let x_span = bounds.x_max - bounds.x_min;
    if x_span <= 0.0 || !x_span.is_finite() {
        cursor.next_index = len;
        return true;
    }

    let end = (cursor.next_index + max_points_to_process).min(len);
    for i in cursor.next_index..end {
        let xi = x[i];
        let yi = y[i];
        if !xi.is_finite() || !yi.is_finite() {
            continue;
        }

        let t = (xi - bounds.x_min) / x_span;
        if !t.is_finite() {
            continue;
        }
        let bucket =
            ((t.clamp(0.0, 1.0) * (width_px - 1) as f64).round() as usize).min(width_px - 1);

        let b = &mut scratch.buckets[bucket];
        if b.first.is_none() {
            b.first = Some(i);
        }
        b.last = Some(i);

        let min_index = b.min.unwrap_or(i);
        if yi < y[min_index] {
            b.min = Some(i);
        } else if b.min.is_none() {
            b.min = Some(i);
        }

        let max_index = b.max.unwrap_or(i);
        if yi > y[max_index] {
            b.max = Some(i);
        } else if b.max.is_none() {
            b.max = Some(i);
        }
    }

    cursor.next_index = end;
    cursor.next_index >= len
}

pub fn minmax_per_pixel_finalize(
    scratch: &mut LodScratch,
    x: &[f64],
    y: &[f64],
    bounds: &DataBounds,
    viewport: Rect,
    out_points: &mut Vec<Point>,
) -> Range<usize> {
    let start = out_points.len();

    let x_span = bounds.x_max - bounds.x_min;
    let y_span = bounds.y_max - bounds.y_min;
    let x_span = if x_span.is_finite() && x_span > 0.0 {
        x_span
    } else {
        1.0
    };
    let y_span = if y_span.is_finite() && y_span > 0.0 {
        y_span
    } else {
        1.0
    };

    let to_px = |xi: f64, yi: f64| -> Point {
        let tx = (xi - bounds.x_min) / x_span;
        let ty = (yi - bounds.y_min) / y_span;

        let px_x = viewport.origin.x.0 + (tx as f32) * viewport.size.width.0;
        let px_y = viewport.origin.y.0 + (1.0 - (ty as f32)) * viewport.size.height.0;

        Point::new(Px(px_x), Px(px_y))
    };

    let buckets = &scratch.buckets;
    let indices = &mut scratch.tmp_indices;

    for bucket in buckets {
        indices.clear();
        if let Some(i) = bucket.first {
            indices.push(i);
        }
        if let Some(i) = bucket.min {
            indices.push(i);
        }
        if let Some(i) = bucket.max {
            indices.push(i);
        }
        if let Some(i) = bucket.last {
            indices.push(i);
        }
        indices.sort_unstable();
        indices.dedup();

        for &i in indices.iter() {
            let xi = x.get(i).copied().unwrap_or(f64::NAN);
            let yi = y.get(i).copied().unwrap_or(f64::NAN);
            if !xi.is_finite() || !yi.is_finite() {
                continue;
            }
            out_points.push(to_px(xi, yi));
        }
    }

    start..out_points.len()
}
