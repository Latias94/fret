use crate::engine::window::DataWindowX;
use crate::engine::window_policy::{AxisFilter1D, axis_filter_1d, axis_mapping_window_1d};
use crate::spec::AxisRange;
use crate::spec::FilterMode;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
