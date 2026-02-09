use core::ops::Range;

use fret_core::{Point, Px, Rect};

use crate::engine::lod::DataBounds;
use crate::transform::RowSelection;

#[derive(Debug, Default, Clone)]
pub struct LodScratch {
    buckets: Vec<Bucket>,
    tmp_indices: Vec<usize>,
    tmp_candidates: Vec<Candidate>,
}

impl LodScratch {
    pub fn clear(&mut self) {
        self.reset_buckets();
        self.tmp_indices.clear();
        self.tmp_candidates.clear();
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
struct Candidate {
    index: usize,
    y: f64,
    y_clamped: f64,
    seq: u32,
}

#[derive(Debug, Default, Clone, Copy)]
struct Bucket {
    first: Option<Candidate>,
    last: Option<Candidate>,
    min: Option<Candidate>,
    max: Option<Candidate>,
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

#[derive(Debug, Default, Clone)]
pub struct SegmentedMinMaxPerPixelCursor {
    pub next_index: usize,
    in_segment: bool,
    segment_points_seen: u32,
}

#[derive(Debug, Default, Clone)]
pub struct SegmentedMinMaxPerPixelSelectionCursor {
    pub next_view_index: usize,
    in_segment: bool,
    segment_points_seen: u32,
}

#[derive(Debug, Clone)]
pub struct SegmentedMinMaxPerPixelStepResult {
    pub done: bool,
    pub segment: Range<usize>,
    pub segment_points_seen: u32,
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
    row_range: core::ops::Range<usize>,
    max_points_to_process: usize,
) -> bool {
    minmax_per_pixel_step_with(
        cursor,
        scratch,
        x,
        bounds,
        viewport,
        row_range,
        max_points_to_process,
        |i| y.get(i).copied().unwrap_or(f64::NAN),
    )
}

pub fn minmax_per_pixel_step_selection(
    cursor: &mut MinMaxPerPixelCursor,
    scratch: &mut LodScratch,
    x: &[f64],
    y: &[f64],
    bounds: &DataBounds,
    viewport: Rect,
    selection: &RowSelection,
    max_points_to_process: usize,
) -> bool {
    minmax_per_pixel_step_selection_with(
        cursor,
        scratch,
        x,
        bounds,
        viewport,
        selection,
        max_points_to_process,
        |i| y.get(i).copied().unwrap_or(f64::NAN),
    )
}

pub fn minmax_per_pixel_step_selection_with(
    cursor: &mut MinMaxPerPixelCursor,
    scratch: &mut LodScratch,
    x: &[f64],
    bounds: &DataBounds,
    viewport: Rect,
    selection: &RowSelection,
    max_points_to_process: usize,
    mut y_at: impl FnMut(usize) -> f64,
) -> bool {
    let width_px = viewport.size.width.0.max(1.0).ceil() as usize;
    scratch.ensure_bucket_count(width_px);

    let len = x.len();
    let end_limit = selection.view_len(len);
    if cursor.next_index >= end_limit {
        return true;
    }

    let x_span = bounds.x_max - bounds.x_min;
    if x_span <= 0.0 || !x_span.is_finite() {
        cursor.next_index = end_limit;
        return true;
    }

    let mut processed = 0usize;
    while cursor.next_index < end_limit && processed < max_points_to_process {
        let view_index = cursor.next_index;
        cursor.next_index += 1;
        processed += 1;

        let Some(i) = selection.get_raw_index(len, view_index) else {
            continue;
        };

        let xi = x.get(i).copied().unwrap_or(f64::NAN);
        let yi = y_at(i);
        if !xi.is_finite() || !yi.is_finite() {
            continue;
        }
        if xi < bounds.x_min || xi > bounds.x_max {
            continue;
        }

        let t = (xi - bounds.x_min) / x_span;
        if !t.is_finite() {
            continue;
        }
        let bucket =
            ((t.clamp(0.0, 1.0) * (width_px - 1) as f64).round() as usize).min(width_px - 1);

        let b = &mut scratch.buckets[bucket];
        let yi_clamped = yi.clamp(bounds.y_min, bounds.y_max);
        let seq = view_index.min(u32::MAX as usize) as u32;
        let c = Candidate {
            index: i,
            y: yi,
            y_clamped: yi_clamped,
            seq,
        };

        if b.first.is_none() {
            b.first = Some(c);
        }
        b.last = Some(c);

        let min_y = b.min.map(|m| m.y_clamped).unwrap_or(yi_clamped);
        if yi_clamped < min_y || b.min.is_none() {
            b.min = Some(c);
        }

        let max_y = b.max.map(|m| m.y_clamped).unwrap_or(yi_clamped);
        if yi_clamped > max_y || b.max.is_none() {
            b.max = Some(c);
        }
    }

    cursor.next_index >= end_limit
}

pub fn minmax_per_pixel_step_with(
    cursor: &mut MinMaxPerPixelCursor,
    scratch: &mut LodScratch,
    x: &[f64],
    bounds: &DataBounds,
    viewport: Rect,
    row_range: core::ops::Range<usize>,
    max_points_to_process: usize,
    mut y_at: impl FnMut(usize) -> f64,
) -> bool {
    let width_px = viewport.size.width.0.max(1.0).ceil() as usize;
    scratch.ensure_bucket_count(width_px);

    let len = x.len();
    if cursor.next_index == 0 {
        cursor.next_index = row_range.start.min(len);
    }

    let end_limit = row_range.end.min(len);
    if cursor.next_index >= end_limit {
        return true;
    }

    let x_span = bounds.x_max - bounds.x_min;
    if x_span <= 0.0 || !x_span.is_finite() {
        cursor.next_index = len;
        return true;
    }

    let end = (cursor.next_index + max_points_to_process).min(end_limit);
    for i in cursor.next_index..end {
        let xi = x[i];
        let yi = y_at(i);
        if !xi.is_finite() || !yi.is_finite() {
            continue;
        }
        if xi < bounds.x_min || xi > bounds.x_max {
            continue;
        }

        let t = (xi - bounds.x_min) / x_span;
        if !t.is_finite() {
            continue;
        }
        let bucket =
            ((t.clamp(0.0, 1.0) * (width_px - 1) as f64).round() as usize).min(width_px - 1);

        let b = &mut scratch.buckets[bucket];
        let yi_clamped = yi.clamp(bounds.y_min, bounds.y_max);
        let seq = i.min(u32::MAX as usize) as u32;
        let c = Candidate {
            index: i,
            y: yi,
            y_clamped: yi_clamped,
            seq,
        };

        if b.first.is_none() {
            b.first = Some(c);
        }
        b.last = Some(c);

        let min_y = b.min.map(|m| m.y_clamped).unwrap_or(yi_clamped);
        if yi_clamped < min_y || b.min.is_none() {
            b.min = Some(c);
        }

        let max_y = b.max.map(|m| m.y_clamped).unwrap_or(yi_clamped);
        if yi_clamped > max_y || b.max.is_none() {
            b.max = Some(c);
        }
    }

    cursor.next_index = end;
    cursor.next_index >= end_limit
}

pub fn minmax_per_pixel_step_segmented_with(
    cursor: &mut SegmentedMinMaxPerPixelCursor,
    scratch: &mut LodScratch,
    x: &[f64],
    bounds: &DataBounds,
    viewport: Rect,
    selection: &RowSelection,
    max_points_to_process: usize,
    out_points: &mut Vec<Point>,
    out_indices: &mut Vec<u32>,
    mut y_at: impl FnMut(usize) -> f64,
    mut is_gap: impl FnMut(usize, f64, f64) -> bool,
) -> Option<SegmentedMinMaxPerPixelStepResult> {
    let width_px = viewport.size.width.0.max(1.0).ceil() as usize;
    scratch.ensure_bucket_count(width_px);

    let len = x.len();
    if cursor.next_index == 0 {
        cursor.in_segment = false;
        cursor.segment_points_seen = 0;
    }

    let end_limit = selection.view_len(len);
    if cursor.next_index >= end_limit {
        cursor.in_segment = false;
        cursor.segment_points_seen = 0;
        return None;
    }

    let x_span = bounds.x_max - bounds.x_min;
    if x_span <= 0.0 || !x_span.is_finite() {
        cursor.next_index = end_limit;
        cursor.in_segment = false;
        cursor.segment_points_seen = 0;
        return None;
    }

    let mut processed = 0usize;
    while cursor.next_index < end_limit && processed < max_points_to_process {
        let view_index = cursor.next_index;
        cursor.next_index += 1;
        processed += 1;

        let Some(i) = selection.get_raw_index(len, view_index) else {
            continue;
        };

        let xi = x.get(i).copied().unwrap_or(f64::NAN);
        let yi = y_at(i);
        let gap = !xi.is_finite() || !yi.is_finite() || is_gap(i, xi, yi);

        if gap {
            if cursor.in_segment && cursor.segment_points_seen > 0 {
                let seg_points_seen = cursor.segment_points_seen;
                cursor.in_segment = false;
                cursor.segment_points_seen = 0;

                let segment = minmax_per_pixel_finalize(
                    scratch,
                    x,
                    bounds,
                    viewport,
                    out_points,
                    out_indices,
                );
                scratch.reset_buckets();

                return Some(SegmentedMinMaxPerPixelStepResult {
                    done: false,
                    segment,
                    segment_points_seen: seg_points_seen,
                });
            }
            continue;
        }

        // Points outside the current mapping window are excluded from emission, but do not
        // necessarily create gaps (ECharts `filter` / `none` vs `empty` semantics are handled by
        // `is_gap`).
        if xi < bounds.x_min || xi > bounds.x_max {
            continue;
        }

        let t = (xi - bounds.x_min) / x_span;
        if !t.is_finite() {
            continue;
        }
        let bucket =
            ((t.clamp(0.0, 1.0) * (width_px - 1) as f64).round() as usize).min(width_px - 1);

        let b = &mut scratch.buckets[bucket];
        let yi_clamped = yi.clamp(bounds.y_min, bounds.y_max);
        let seq = view_index.min(u32::MAX as usize) as u32;
        let c = Candidate {
            index: i,
            y: yi,
            y_clamped: yi_clamped,
            seq,
        };

        if b.first.is_none() {
            b.first = Some(c);
        }
        b.last = Some(c);

        let min_y = b.min.map(|m| m.y_clamped).unwrap_or(yi_clamped);
        if yi_clamped < min_y || b.min.is_none() {
            b.min = Some(c);
        }

        let max_y = b.max.map(|m| m.y_clamped).unwrap_or(yi_clamped);
        if yi_clamped > max_y || b.max.is_none() {
            b.max = Some(c);
        }

        cursor.in_segment = true;
        cursor.segment_points_seen = cursor.segment_points_seen.saturating_add(1);
    }

    if cursor.next_index >= end_limit && cursor.in_segment && cursor.segment_points_seen > 0 {
        let seg_points_seen = cursor.segment_points_seen;
        cursor.in_segment = false;
        cursor.segment_points_seen = 0;

        let segment =
            minmax_per_pixel_finalize(scratch, x, bounds, viewport, out_points, out_indices);
        scratch.reset_buckets();

        return Some(SegmentedMinMaxPerPixelStepResult {
            done: true,
            segment,
            segment_points_seen: seg_points_seen,
        });
    }

    None
}

pub fn minmax_per_pixel_step_segmented_selection_with(
    cursor: &mut SegmentedMinMaxPerPixelSelectionCursor,
    scratch: &mut LodScratch,
    x: &[f64],
    bounds: &DataBounds,
    viewport: Rect,
    selection: &RowSelection,
    max_points_to_process: usize,
    out_points: &mut Vec<Point>,
    out_indices: &mut Vec<u32>,
    mut y_at: impl FnMut(usize) -> f64,
    mut is_valid: impl FnMut(usize, f64, f64) -> bool,
) -> Option<SegmentedMinMaxPerPixelStepResult> {
    let width_px = viewport.size.width.0.max(1.0).ceil() as usize;
    scratch.ensure_bucket_count(width_px);

    let len = x.len();
    if cursor.next_view_index == 0 {
        cursor.in_segment = false;
        cursor.segment_points_seen = 0;
    }

    let end_limit = selection.view_len(len);
    if cursor.next_view_index >= end_limit {
        cursor.in_segment = false;
        cursor.segment_points_seen = 0;
        return None;
    }

    let x_span = bounds.x_max - bounds.x_min;
    if x_span <= 0.0 || !x_span.is_finite() {
        cursor.next_view_index = end_limit;
        cursor.in_segment = false;
        cursor.segment_points_seen = 0;
        return None;
    }

    let mut processed = 0usize;
    while cursor.next_view_index < end_limit && processed < max_points_to_process {
        let view_index = cursor.next_view_index;
        cursor.next_view_index += 1;
        processed += 1;

        let Some(i) = selection.get_raw_index(len, view_index) else {
            continue;
        };

        let xi = x.get(i).copied().unwrap_or(f64::NAN);
        let yi = y_at(i);
        let valid = xi.is_finite()
            && yi.is_finite()
            && xi >= bounds.x_min
            && xi <= bounds.x_max
            && is_valid(i, xi, yi);

        if !valid {
            if cursor.in_segment && cursor.segment_points_seen > 0 {
                let seg_points_seen = cursor.segment_points_seen;
                cursor.in_segment = false;
                cursor.segment_points_seen = 0;

                let segment = minmax_per_pixel_finalize(
                    scratch,
                    x,
                    bounds,
                    viewport,
                    out_points,
                    out_indices,
                );
                scratch.reset_buckets();

                return Some(SegmentedMinMaxPerPixelStepResult {
                    done: false,
                    segment,
                    segment_points_seen: seg_points_seen,
                });
            }
            continue;
        }

        let t = (xi - bounds.x_min) / x_span;
        if !t.is_finite() {
            continue;
        }
        let bucket =
            ((t.clamp(0.0, 1.0) * (width_px - 1) as f64).round() as usize).min(width_px - 1);

        let b = &mut scratch.buckets[bucket];
        let yi_clamped = yi.clamp(bounds.y_min, bounds.y_max);
        let seq = view_index.min(u32::MAX as usize) as u32;
        let c = Candidate {
            index: i,
            y: yi,
            y_clamped: yi_clamped,
            seq,
        };

        if b.first.is_none() {
            b.first = Some(c);
        }
        b.last = Some(c);

        let min_y = b.min.map(|m| m.y_clamped).unwrap_or(yi_clamped);
        if yi_clamped < min_y || b.min.is_none() {
            b.min = Some(c);
        }

        let max_y = b.max.map(|m| m.y_clamped).unwrap_or(yi_clamped);
        if yi_clamped > max_y || b.max.is_none() {
            b.max = Some(c);
        }

        cursor.in_segment = true;
        cursor.segment_points_seen = cursor.segment_points_seen.saturating_add(1);
    }

    if cursor.next_view_index >= end_limit && cursor.in_segment && cursor.segment_points_seen > 0 {
        let seg_points_seen = cursor.segment_points_seen;
        cursor.in_segment = false;
        cursor.segment_points_seen = 0;

        let segment =
            minmax_per_pixel_finalize(scratch, x, bounds, viewport, out_points, out_indices);
        scratch.reset_buckets();

        return Some(SegmentedMinMaxPerPixelStepResult {
            done: true,
            segment,
            segment_points_seen: seg_points_seen,
        });
    }

    None
}

pub fn minmax_per_pixel_finalize(
    scratch: &mut LodScratch,
    x: &[f64],
    bounds: &DataBounds,
    viewport: Rect,
    out_points: &mut Vec<Point>,
    out_indices: &mut Vec<u32>,
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
        let yi = yi.clamp(bounds.y_min, bounds.y_max);
        let tx = (xi - bounds.x_min) / x_span;
        let ty = (yi - bounds.y_min) / y_span;

        let px_x = viewport.origin.x.0 + (tx as f32) * viewport.size.width.0;
        let px_y = viewport.origin.y.0 + (1.0 - (ty as f32)) * viewport.size.height.0;

        Point::new(Px(px_x), Px(px_y))
    };

    let buckets = &scratch.buckets;
    let candidates = &mut scratch.tmp_candidates;
    candidates.clear();

    for bucket in buckets {
        if let Some(c) = bucket.first {
            candidates.push(c);
        }
        if let Some(c) = bucket.min {
            candidates.push(c);
        }
        if let Some(c) = bucket.max {
            candidates.push(c);
        }
        if let Some(c) = bucket.last {
            candidates.push(c);
        }
    }

    candidates.sort_by(|a, b| {
        let ord = a.index.cmp(&b.index);
        if ord == core::cmp::Ordering::Equal {
            a.seq.cmp(&b.seq)
        } else {
            ord
        }
    });
    candidates.dedup_by(|a, b| a.index == b.index);

    candidates.sort_by(|a, b| {
        let ord = a.seq.cmp(&b.seq);
        if ord == core::cmp::Ordering::Equal {
            a.index.cmp(&b.index)
        } else {
            ord
        }
    });

    for c in candidates.iter() {
        let xi = x.get(c.index).copied().unwrap_or(f64::NAN);
        if !xi.is_finite() || !c.y.is_finite() {
            continue;
        }
        out_points.push(to_px(xi, c.y));
        out_indices.push(c.index.min(u32::MAX as usize) as u32);
    }

    start..out_points.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Px, Size};

    #[test]
    fn minmax_finalize_is_pixel_bounded_and_index_aligned_for_monotonic_x() {
        let n = 100_000usize;
        let width_px = 120usize;
        let height_px = 80usize;

        let mut x = Vec::with_capacity(n);
        let mut y = Vec::with_capacity(n);
        for i in 0..n {
            x.push(i as f64 / (n as f64 - 1.0));
            y.push(((i as f64) * 0.01).sin());
        }

        let bounds = compute_bounds(&x, &y).expect("expected finite bounds");
        let viewport = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(width_px as f32), Px(height_px as f32)),
        );

        let mut cursor = MinMaxPerPixelCursor::default();
        let mut scratch = LodScratch::default();
        while !minmax_per_pixel_step(
            &mut cursor,
            &mut scratch,
            &x,
            &y,
            &bounds,
            viewport,
            0..n,
            8192,
        ) {}

        let mut out_points = Vec::new();
        let mut out_indices = Vec::new();
        minmax_per_pixel_finalize(
            &mut scratch,
            &x,
            &bounds,
            viewport,
            &mut out_points,
            &mut out_indices,
        );

        assert!(!out_points.is_empty());
        assert_eq!(out_points.len(), out_indices.len());
        assert!(out_points.len() <= width_px * 4);

        assert!(
            out_indices.windows(2).all(|w| w[0] < w[1]),
            "expected strictly increasing raw indices for monotonic X"
        );
        assert!(
            out_indices.iter().all(|&i| (i as usize) < n),
            "expected all indices within 0..n"
        );

        let left = viewport.origin.x.0;
        let right = viewport.origin.x.0 + viewport.size.width.0;
        let top = viewport.origin.y.0;
        let bottom = viewport.origin.y.0 + viewport.size.height.0;

        for p in out_points {
            assert!(
                p.x.0 >= left && p.x.0 <= right,
                "x out of viewport: {}",
                p.x.0
            );
            assert!(
                p.y.0 >= top && p.y.0 <= bottom,
                "y out of viewport: {}",
                p.y.0
            );
        }
    }
}
