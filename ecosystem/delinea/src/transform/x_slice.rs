use crate::engine::window::DataWindowX;
use crate::engine::window_policy::AxisFilter1D;
use std::sync::Arc;

use crate::transform::{RowRange, RowSelection};

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

pub fn row_selection_for_x_filter(
    values: &[f64],
    base: RowRange,
    filter: AxisFilter1D,
) -> RowSelection {
    if let Some(window) = filter.as_window() {
        return row_selection_for_x_window(values, base, window);
    }
    if let Some(min) = filter.min {
        return row_selection_for_x_min(values, base, min);
    }
    if let Some(max) = filter.max {
        return row_selection_for_x_max(values, base, max);
    }
    RowSelection::Range(base)
}

pub fn row_range_for_x_window(values: &[f64], base: RowRange, window: DataWindowX) -> RowRange {
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

pub fn row_selection_for_x_window(
    values: &[f64],
    base: RowRange,
    window: DataWindowX,
) -> RowSelection {
    let mut base = base;
    base.clamp_to_len(values.len());
    if base.is_empty() {
        return RowSelection::Range(base);
    }

    let slice = &values[base.start..base.end];
    let Some(first) = slice.first().copied() else {
        return RowSelection::Range(base);
    };
    let Some(last) = slice.last().copied() else {
        return RowSelection::Range(base);
    };
    if first.is_finite() && last.is_finite() {
        let ascending = first <= last;
        if is_probably_monotonic(slice, ascending) {
            return RowSelection::Range(row_range_for_x_window(values, base, window));
        }
    }

    let indices = indices_for_x_window(values, base, window);
    RowSelection::Indices(indices)
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

pub fn row_selection_for_x_min(values: &[f64], base: RowRange, min: f64) -> RowSelection {
    let mut base = base;
    base.clamp_to_len(values.len());
    if base.is_empty() || !min.is_finite() {
        return RowSelection::Range(base);
    }

    let slice = &values[base.start..base.end];
    let Some(first) = slice.first().copied() else {
        return RowSelection::Range(base);
    };
    let Some(last) = slice.last().copied() else {
        return RowSelection::Range(base);
    };
    if first.is_finite() && last.is_finite() {
        let ascending = first <= last;
        if is_probably_monotonic(slice, ascending) {
            return RowSelection::Range(row_range_for_x_min(values, base, min));
        }
    }

    let indices = indices_for_x_min(values, base, min);
    RowSelection::Indices(indices)
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

pub fn row_selection_for_x_max(values: &[f64], base: RowRange, max: f64) -> RowSelection {
    let mut base = base;
    base.clamp_to_len(values.len());
    if base.is_empty() || !max.is_finite() {
        return RowSelection::Range(base);
    }

    let slice = &values[base.start..base.end];
    let Some(first) = slice.first().copied() else {
        return RowSelection::Range(base);
    };
    let Some(last) = slice.last().copied() else {
        return RowSelection::Range(base);
    };
    if first.is_finite() && last.is_finite() {
        let ascending = first <= last;
        if is_probably_monotonic(slice, ascending) {
            return RowSelection::Range(row_range_for_x_max(values, base, max));
        }
    }

    let indices = indices_for_x_max(values, base, max);
    RowSelection::Indices(indices)
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

fn indices_for_x_min(values: &[f64], base: RowRange, min: f64) -> Arc<[u32]> {
    let mut base = base;
    base.clamp_to_len(values.len());

    let mut out: Vec<u32> = Vec::new();
    for i in base.start..base.end {
        let v = values.get(i).copied().unwrap_or(f64::NAN);
        if !v.is_finite() {
            continue;
        }
        if v >= min && i <= u32::MAX as usize {
            out.push(i as u32);
        }
    }

    out.into()
}

fn indices_for_x_max(values: &[f64], base: RowRange, max: f64) -> Arc<[u32]> {
    let mut base = base;
    base.clamp_to_len(values.len());

    let mut out: Vec<u32> = Vec::new();
    for i in base.start..base.end {
        let v = values.get(i).copied().unwrap_or(f64::NAN);
        if !v.is_finite() {
            continue;
        }
        if v <= max && i <= u32::MAX as usize {
            out.push(i as u32);
        }
    }

    out.into()
}

fn indices_for_x_window(values: &[f64], base: RowRange, window: DataWindowX) -> Arc<[u32]> {
    let (min, max) = if window.min <= window.max {
        (window.min, window.max)
    } else {
        (window.max, window.min)
    };

    let mut out: Vec<u32> = Vec::new();
    for i in base.start..base.end {
        let v = values.get(i).copied().unwrap_or(f64::NAN);
        if !v.is_finite() {
            continue;
        }
        if v >= min && v <= max && i <= u32::MAX as usize {
            out.push(i as u32);
        }
    }

    out.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_selection_for_x_window_uses_indices_for_non_monotonic_data() {
        let xs = vec![0.0, 10.0, 0.0, 10.0, 0.0];
        let base = RowRange {
            start: 0,
            end: xs.len(),
        };
        let window = DataWindowX {
            min: 9.0,
            max: 10.0,
        };

        let sel = row_selection_for_x_window(&xs, base, window);
        match &sel {
            RowSelection::Indices(indices) => {
                assert_eq!(&**indices, &[1u32, 3u32]);
            }
            other => panic!("expected Indices, got {other:?}"),
        }
        assert_eq!(sel.len(xs.len()), 2);
        assert_eq!(sel.as_range(xs.len()), 1..4);
        assert_eq!(sel.get_raw_index(xs.len(), 0), Some(1));
        assert_eq!(sel.get_raw_index(xs.len(), 1), Some(3));
    }

    #[test]
    fn row_selection_for_x_window_uses_range_for_monotonic_data() {
        let xs = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let base = RowRange {
            start: 0,
            end: xs.len(),
        };
        let window = DataWindowX { min: 1.0, max: 2.0 };

        let sel = row_selection_for_x_window(&xs, base, window);
        assert_eq!(sel, RowSelection::Range(RowRange { start: 1, end: 3 }));
        assert_eq!(sel.len(xs.len()), 2);
        assert_eq!(sel.as_range(xs.len()), 1..3);
        assert_eq!(sel.get_raw_index(xs.len(), 0), Some(1));
        assert_eq!(sel.get_raw_index(xs.len(), 1), Some(2));
    }
}
