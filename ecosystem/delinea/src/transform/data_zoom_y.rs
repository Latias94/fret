use std::sync::Arc;

use crate::engine::ChartState;
use crate::engine::model::ChartModel;
use crate::engine::window::DataWindow;
use crate::engine::window_policy::{AxisFilter1D, axis_filter_1d};
use crate::ids::{AxisId, DataZoomId};
use crate::spec::{AxisRange, FilterMode};
use crate::transform::RowSelection;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct DataZoomYNode {
    pub id: Option<DataZoomId>,
    pub axis: AxisId,
    pub axis_range: AxisRange,
    pub state_window: Option<DataWindow>,
    pub filter_mode: FilterMode,
}

impl DataZoomYNode {
    pub fn y_filter(self) -> AxisFilter1D {
        axis_filter_1d(self.axis_range, self.state_window, self.filter_mode)
    }

    pub fn apply_y_filter_indices(
        self,
        y_values: &[f64],
        selection: &RowSelection,
        max_view_len: usize,
    ) -> Option<RowSelection> {
        if !matches!(
            self.filter_mode,
            FilterMode::Filter | FilterMode::WeakFilter
        ) {
            return None;
        }

        let len = y_values.len();
        let view_len = selection.view_len(len);
        if view_len == 0 {
            return Some(RowSelection::Indices(Arc::from([])));
        }
        if view_len > max_view_len {
            return None;
        }

        let filter = self.y_filter();
        if filter.min.is_none() && filter.max.is_none() {
            return None;
        }

        let mut indices: Vec<u32> = Vec::with_capacity(view_len.min(4096));

        let mut kept = 0usize;
        for view_index in 0..view_len {
            let Some(raw) = selection.get_raw_index(len, view_index) else {
                continue;
            };
            let v = y_values.get(raw).copied().unwrap_or(f64::NAN);
            if !v.is_finite() {
                continue;
            }
            if filter.contains(v) {
                indices.push(raw.min(u32::MAX as usize) as u32);
                kept += 1;
            }
        }

        if kept == view_len {
            return None;
        }
        Some(RowSelection::Indices(indices.into()))
    }
}

pub fn data_zoom_y_node(
    model: &ChartModel,
    state: &ChartState,
    axis: AxisId,
    axis_range: AxisRange,
) -> DataZoomYNode {
    let id = model.data_zoom_y_by_axis.get(&axis).copied();
    let filter_mode = id
        .and_then(|id| model.data_zoom_y.get(&id))
        .map(|z| z.filter_mode)
        .unwrap_or(FilterMode::None);
    let state_window = state.data_window_y.get(&axis).copied();

    DataZoomYNode {
        id,
        axis,
        axis_range,
        state_window,
        filter_mode,
    }
}
