use crate::engine::window::DataWindowX;
use crate::engine::window_policy::AxisFilter1D;

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
        return base;
    }

    let ascending = first <= last;
    if !is_probably_monotonic_sampled(slice, ascending) {
        return base;
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
    RowSelection::Range(row_range_for_x_window(values, base, window))
}

pub fn is_probably_monotonic_in_range(values: &[f64], range: RowRange) -> Option<bool> {
    let mut range = range;
    range.clamp_to_len(values.len());
    if range.is_empty() {
        return None;
    }

    let slice = &values[range.start..range.end];
    let first = slice.first().copied()?;
    let last = slice.last().copied()?;
    if !first.is_finite() || !last.is_finite() {
        return None;
    }

    let ascending = first <= last;
    is_probably_monotonic_sampled(slice, ascending).then_some(ascending)
}

fn is_probably_monotonic_sampled(values: &[f64], ascending: bool) -> bool {
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
        return base;
    }

    let ascending = first <= last;
    if !is_probably_monotonic_sampled(slice, ascending) {
        return base;
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
    RowSelection::Range(row_range_for_x_min(values, base, min))
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
        return base;
    }

    let ascending = first <= last;
    if !is_probably_monotonic_sampled(slice, ascending) {
        return base;
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
    RowSelection::Range(row_range_for_x_max(values, base, max))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_selection_for_x_window_falls_back_to_base_range_for_non_monotonic_data() {
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
        assert_eq!(sel, RowSelection::Range(base));
        assert_eq!(sel.len(xs.len()), xs.len());
        assert_eq!(sel.as_range(xs.len()), 0..xs.len());
        assert_eq!(sel.get_raw_index(xs.len(), 0), Some(0));
        assert_eq!(
            sel.get_raw_index(xs.len(), xs.len() - 1),
            Some(xs.len() - 1)
        );
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
