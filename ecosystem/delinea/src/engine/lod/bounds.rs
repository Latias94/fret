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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BoundsCursor {
    pub next_index: usize,
    pub saw_any: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct BoundsAccum {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl BoundsAccum {
    pub fn reset(&mut self) {
        self.x_min = f64::INFINITY;
        self.x_max = f64::NEG_INFINITY;
        self.y_min = f64::INFINITY;
        self.y_max = f64::NEG_INFINITY;
    }

    pub fn is_valid(&self) -> bool {
        self.x_min.is_finite()
            && self.x_max.is_finite()
            && self.y_min.is_finite()
            && self.y_max.is_finite()
            && self.x_max > self.x_min
            && self.y_max > self.y_min
    }
}

pub fn compute_bounds_step(
    cursor: &mut BoundsCursor,
    accum: &mut BoundsAccum,
    x: &[f64],
    y: &[f64],
    row_range: core::ops::Range<usize>,
    x_filter: crate::engine::window_policy::AxisFilter1D,
    max_points_to_process: usize,
) -> Option<bool> {
    compute_bounds_step_with(
        cursor,
        accum,
        x,
        row_range,
        x_filter,
        max_points_to_process,
        |i| y.get(i).copied().unwrap_or(f64::NAN),
    )
}

pub fn compute_bounds_step_selection(
    cursor: &mut BoundsCursor,
    accum: &mut BoundsAccum,
    x: &[f64],
    y: &[f64],
    selection: &RowSelection,
    x_filter: crate::engine::window_policy::AxisFilter1D,
    max_points_to_process: usize,
) -> Option<bool> {
    compute_bounds_step_selection_with(
        cursor,
        accum,
        x,
        selection,
        x_filter,
        max_points_to_process,
        |i| y.get(i).copied().unwrap_or(f64::NAN),
    )
}

pub fn compute_bounds_step_selection_with(
    cursor: &mut BoundsCursor,
    accum: &mut BoundsAccum,
    x: &[f64],
    selection: &RowSelection,
    x_filter: crate::engine::window_policy::AxisFilter1D,
    max_points_to_process: usize,
    mut y_at: impl FnMut(usize) -> f64,
) -> Option<bool> {
    let len = x.len();
    let end_limit = selection.view_len(len);
    if cursor.next_index >= end_limit {
        return Some(true);
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

        if !x_filter.contains(xi) {
            continue;
        }

        cursor.saw_any = true;
        accum.x_min = accum.x_min.min(xi);
        accum.x_max = accum.x_max.max(xi);
        accum.y_min = accum.y_min.min(yi);
        accum.y_max = accum.y_max.max(yi);
    }

    Some(cursor.next_index >= end_limit)
}

pub fn compute_bounds_step_with(
    cursor: &mut BoundsCursor,
    accum: &mut BoundsAccum,
    x: &[f64],
    row_range: core::ops::Range<usize>,
    x_filter: crate::engine::window_policy::AxisFilter1D,
    max_points_to_process: usize,
    mut y_at: impl FnMut(usize) -> f64,
) -> Option<bool> {
    let len = x.len();
    if cursor.next_index == 0 {
        cursor.next_index = row_range.start.min(len);
    }

    let end_limit = row_range.end.min(len);
    if cursor.next_index >= end_limit {
        return Some(true);
    }

    let end = (cursor.next_index + max_points_to_process).min(end_limit);

    for (offset, &xi) in x[cursor.next_index..end].iter().enumerate() {
        let i = cursor.next_index + offset;
        let yi = y_at(i);
        if !xi.is_finite() || !yi.is_finite() {
            continue;
        }

        if !x_filter.contains(xi) {
            continue;
        }

        cursor.saw_any = true;
        accum.x_min = accum.x_min.min(xi);
        accum.x_max = accum.x_max.max(xi);
        accum.y_min = accum.y_min.min(yi);
        accum.y_max = accum.y_max.max(yi);
    }

    cursor.next_index = end;
    Some(cursor.next_index >= end_limit)
}

pub fn finalize_bounds(accum: &BoundsAccum) -> Option<DataBounds> {
    if !accum.is_valid() {
        return None;
    }
    Some(DataBounds {
        x_min: accum.x_min,
        x_max: accum.x_max,
        y_min: accum.y_min,
        y_max: accum.y_max,
    })
}
use crate::transform::RowSelection;
