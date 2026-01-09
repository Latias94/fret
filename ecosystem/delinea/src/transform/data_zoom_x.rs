use crate::engine::model::ChartModel;
use crate::engine::window::DataWindowX;
use crate::engine::window_policy::AxisFilter1D;
use crate::engine::{ChartState, DataZoomXState};
use crate::ids::{AxisId, DataZoomId};
use crate::spec::{AxisRange, FilterMode};
use crate::transform::{
    RowRange, RowSelection, SeriesXPolicy, row_selection_for_x_filter, series_x_policy,
};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct XWindowResult {
    pub selection: RowSelection,
    pub x_policy: SeriesXPolicy,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct DataZoomXNode {
    pub id: Option<DataZoomId>,
    pub axis: AxisId,
    pub axis_range: AxisRange,
    pub state_present: bool,
    pub state: DataZoomXState,
    pub default_filter_mode: FilterMode,
}

impl DataZoomXNode {
    pub fn filter_mode(self) -> FilterMode {
        if self.state_present {
            self.state.filter_mode
        } else {
            self.default_filter_mode
        }
    }

    pub fn window(self) -> Option<DataWindowX> {
        if self.state_present {
            self.state.window
        } else {
            None
        }
    }

    pub fn x_policy(self) -> SeriesXPolicy {
        series_x_policy(self.axis_range, self.window(), self.filter_mode())
    }

    pub fn x_filter(self) -> AxisFilter1D {
        self.x_policy().filter
    }

    pub fn mapping_window(self) -> Option<DataWindowX> {
        self.x_policy().mapping_window
    }

    pub fn apply(self, x_values: &[f64], base_range: RowRange) -> XWindowResult {
        let x_policy = self.x_policy();
        let selection = if self.filter_mode() == FilterMode::Filter {
            row_selection_for_x_filter(x_values, base_range, x_policy.filter)
        } else {
            RowSelection::Range(base_range)
        };

        XWindowResult {
            selection,
            x_policy,
        }
    }
}

pub fn data_zoom_x_node(
    model: &ChartModel,
    state: &ChartState,
    axis: AxisId,
    axis_range: AxisRange,
) -> DataZoomXNode {
    let id = model.data_zoom_x_by_axis.get(&axis).copied();
    let default_filter_mode = id
        .and_then(|id| model.data_zoom_x.get(&id))
        .map(|z| z.filter_mode)
        .unwrap_or_default();

    let (state_present, state_entry) = state
        .data_zoom_x
        .get(&axis)
        .copied()
        .map(|s| (true, s))
        .unwrap_or_default();

    DataZoomXNode {
        id,
        axis,
        axis_range,
        state_present,
        state: state_entry,
        default_filter_mode,
    }
}
