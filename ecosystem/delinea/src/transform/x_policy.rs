use crate::engine::window::DataWindowX;
use crate::engine::window_policy::{AxisFilter1D, axis_filter_1d, axis_mapping_window_1d};
use crate::spec::AxisRange;
use crate::spec::FilterMode;
use crate::transform::{RowRange, RowSelection, row_range_for_x_filter};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct SeriesXPolicy {
    pub filter: AxisFilter1D,
    pub mapping_window: Option<DataWindowX>,
}

pub fn series_x_policy(
    axis_range: AxisRange,
    state_window: Option<DataWindowX>,
    filter_mode: FilterMode,
) -> SeriesXPolicy {
    SeriesXPolicy {
        filter: axis_filter_1d(axis_range, state_window, filter_mode),
        mapping_window: axis_mapping_window_1d(axis_range, state_window),
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct XWindowResult {
    pub selection: RowSelection,
    pub x_policy: SeriesXPolicy,
}

pub fn apply_x_window_transform(
    x_values: &[f64],
    base_range: RowRange,
    axis_range: AxisRange,
    state_window: Option<DataWindowX>,
    filter_mode: FilterMode,
) -> XWindowResult {
    let x_policy = series_x_policy(axis_range, state_window, filter_mode);

    let row_range = if filter_mode == FilterMode::Filter {
        row_range_for_x_filter(x_values, base_range, x_policy.filter)
    } else {
        base_range
    };

    XWindowResult {
        selection: RowSelection::Range(row_range),
        x_policy,
    }
}
