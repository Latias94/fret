use crate::engine::window::DataWindowX;
use crate::engine::window_policy::AxisFilter1D;
use crate::view::RowRange;

pub fn row_range_for_x_filter(values: &[f64], base: RowRange, filter: AxisFilter1D) -> RowRange {
    if let Some(window) = filter.as_window() {
        return row_range_for_x_window(values, base, window);
    }
    if let Some(min) = filter.min {
        return row_range_for_x_min(values, base, min);
    }
    if let Some(max) = filter.max {
        return row_range_for_x_max(values, base, max);
    }
    base
}

fn row_range_for_x_window(values: &[f64], base: RowRange, window: DataWindowX) -> RowRange {
    let mut base = base;
    base.clamp_to_len(values.len());
    if base.is_empty() {
        return base;
    }

    let slice = &values[base.start..base.end];
    let Some(first) = slice.first().copied() else {
        return base;
    };
    let Some(last) = slice.last().copied() else {
        return base;
    };
    if !first.is_finite() || !last.is_finite() {
        return row_range_for_x_window_linear(values, base, window);
    }

    let ascending = first <= last;
    if !is_probably_monotonic(slice, ascending) {
        return row_range_for_x_window_linear(values, base, window);
    }

    let (min, max) = if window.min <= window.max {
        (window.min, window.max)
    } else {
        (window.max, window.min)
    };

    let (start, end) = if ascending {
        let lo = slice.partition_point(|&v| v < min);
        let hi = slice.partition_point(|&v| v <= max);
        (base.start + lo, base.start + hi)
    } else {
        let lo = slice.partition_point(|&v| v > max);
        let hi = slice.partition_point(|&v| v >= min);
        (base.start + lo, base.start + hi)
    };

    RowRange { start, end }
}

fn is_probably_monotonic(values: &[f64], ascending: bool) -> bool {
    let len = values.len();
    if len <= 2 {
        return true;
    }

    let samples = 8usize.min(len);
    let mut prev = values[0];
    if !prev.is_finite() {
        return false;
    }

    for s in 1..samples {
        let i = s * (len - 1) / (samples - 1);
        let cur = values[i];
        if !cur.is_finite() {
            return false;
        }
        if ascending {
            if cur < prev {
                return false;
            }
        } else if cur > prev {
            return false;
        }
        prev = cur;
    }
    true
}

fn row_range_for_x_min(values: &[f64], base: RowRange, min: f64) -> RowRange {
    let mut base = base;
    base.clamp_to_len(values.len());
    if base.is_empty() || !min.is_finite() {
        return base;
    }

    let slice = &values[base.start..base.end];
    let Some(first) = slice.first().copied() else {
        return base;
    };
    let Some(last) = slice.last().copied() else {
        return base;
    };
    if !first.is_finite() || !last.is_finite() {
        return row_range_for_x_min_linear(values, base, min);
    }

    let ascending = first <= last;
    if !is_probably_monotonic(slice, ascending) {
        return row_range_for_x_min_linear(values, base, min);
    }

    if ascending {
        let lo = slice.partition_point(|&v| v < min);
        RowRange {
            start: base.start + lo,
            end: base.end,
        }
    } else {
        let hi = slice.partition_point(|&v| v >= min);
        RowRange {
            start: base.start,
            end: base.start + hi,
        }
    }
}

fn row_range_for_x_max(values: &[f64], base: RowRange, max: f64) -> RowRange {
    let mut base = base;
    base.clamp_to_len(values.len());
    if base.is_empty() || !max.is_finite() {
        return base;
    }

    let slice = &values[base.start..base.end];
    let Some(first) = slice.first().copied() else {
        return base;
    };
    let Some(last) = slice.last().copied() else {
        return base;
    };
    if !first.is_finite() || !last.is_finite() {
        return row_range_for_x_max_linear(values, base, max);
    }

    let ascending = first <= last;
    if !is_probably_monotonic(slice, ascending) {
        return row_range_for_x_max_linear(values, base, max);
    }

    if ascending {
        let hi = slice.partition_point(|&v| v <= max);
        RowRange {
            start: base.start,
            end: base.start + hi,
        }
    } else {
        let lo = slice.partition_point(|&v| v > max);
        RowRange {
            start: base.start + lo,
            end: base.end,
        }
    }
}

fn row_range_for_x_min_linear(values: &[f64], base: RowRange, min: f64) -> RowRange {
    let mut base = base;
    base.clamp_to_len(values.len());
    if base.is_empty() || !min.is_finite() {
        return base;
    }

    let mut first: Option<usize> = None;
    let mut last: Option<usize> = None;
    for i in base.start..base.end {
        let v = values.get(i).copied().unwrap_or(f64::NAN);
        if !v.is_finite() {
            continue;
        }
        if v >= min {
            first.get_or_insert(i);
            last = Some(i);
        }
    }

    match (first, last) {
        (Some(a), Some(b)) if b >= a => RowRange {
            start: a,
            end: b + 1,
        },
        _ => RowRange {
            start: base.start,
            end: base.start,
        },
    }
}

fn row_range_for_x_max_linear(values: &[f64], base: RowRange, max: f64) -> RowRange {
    let mut base = base;
    base.clamp_to_len(values.len());
    if base.is_empty() || !max.is_finite() {
        return base;
    }

    let mut first: Option<usize> = None;
    let mut last: Option<usize> = None;
    for i in base.start..base.end {
        let v = values.get(i).copied().unwrap_or(f64::NAN);
        if !v.is_finite() {
            continue;
        }
        if v <= max {
            first.get_or_insert(i);
            last = Some(i);
        }
    }

    match (first, last) {
        (Some(a), Some(b)) if b >= a => RowRange {
            start: a,
            end: b + 1,
        },
        _ => RowRange {
            start: base.start,
            end: base.start,
        },
    }
}

fn row_range_for_x_window_linear(values: &[f64], base: RowRange, window: DataWindowX) -> RowRange {
    let (min, max) = if window.min <= window.max {
        (window.min, window.max)
    } else {
        (window.max, window.min)
    };

    let mut first: Option<usize> = None;
    let mut last: Option<usize> = None;
    for i in base.start..base.end {
        let v = values.get(i).copied().unwrap_or(f64::NAN);
        if !v.is_finite() {
            continue;
        }
        if v >= min && v <= max {
            first.get_or_insert(i);
            last = Some(i);
        }
    }

    match (first, last) {
        (Some(a), Some(b)) if b >= a => RowRange {
            start: a,
            end: b + 1,
        },
        _ => RowRange {
            start: base.start,
            end: base.start,
        },
    }
}
