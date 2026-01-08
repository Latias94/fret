use crate::data::{DataTable, DatasetStore};
use crate::engine::ChartState;
use crate::engine::model::ChartModel;
use crate::engine::window_policy::FilterMode;
use crate::ids::{AxisId, DatasetId, Revision, SeriesId};
use crate::transform::{RowSelection, SeriesXPolicy, series_x_policy};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RowRange {
    pub start: usize,
    pub end: usize,
}

impl RowRange {
    pub fn clamp_to_len(&mut self, len: usize) {
        self.start = self.start.min(len);
        self.end = self.end.min(len);
        if self.end < self.start {
            self.end = self.start;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DatasetView {
    pub dataset: DatasetId,
    pub revision: Revision,
    pub data_revision: Revision,
    pub row_range: RowRange,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SeriesView {
    pub series: SeriesId,
    pub dataset: DatasetId,
    pub x_axis: AxisId,
    pub revision: Revision,
    pub data_revision: Revision,
    pub selection: RowSelection,
    pub x_policy: SeriesXPolicy,
}

#[derive(Debug, Default, Clone)]
pub struct ViewState {
    pub revision: Revision,
    pub datasets: Vec<DatasetView>,
    pub series: Vec<SeriesView>,
    last_model_rev: Revision,
    last_data_rev: Revision,
    last_state_rev: Revision,
}

impl ViewState {
    pub fn sync_inputs(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        state: &ChartState,
    ) -> bool {
        let model_rev = model.revs.spec;
        let data_rev = datasets
            .datasets
            .iter()
            .next()
            .map(|(_, t)| t.revision)
            .unwrap_or_default();

        let state_rev = state.revision;

        if model_rev != self.last_model_rev
            || data_rev != self.last_data_rev
            || state_rev != self.last_state_rev
        {
            self.revision.bump();
            self.last_model_rev = model_rev;
            self.last_data_rev = data_rev;
            self.last_state_rev = state_rev;
            return true;
        }

        false
    }

    pub fn rebuild(&mut self, model: &ChartModel, datasets: &DatasetStore, state: &ChartState) {
        self.datasets.clear();
        self.series.clear();
        for (id, _dataset_model) in &model.datasets {
            let table = datasets
                .datasets
                .iter()
                .find_map(|(did, t)| (*did == *id).then_some(t));
            let Some(table) = table else { continue };
            let mut row_range = state
                .dataset_row_ranges
                .get(id)
                .copied()
                .unwrap_or(RowRange {
                    start: 0,
                    end: table.row_count,
                });
            row_range.clamp_to_len(table.row_count);
            self.datasets.push(DatasetView {
                dataset: *id,
                revision: self.revision,
                data_revision: table.revision,
                row_range,
            });
        }

        for series_id in &model.series_order {
            let Some(series) = model.series.get(series_id) else {
                continue;
            };
            let table = datasets
                .datasets
                .iter()
                .find_map(|(did, t)| (*did == series.dataset).then_some(t));
            let Some(table) = table else { continue };
            let Some(dataset) = model.datasets.get(&series.dataset) else {
                continue;
            };
            let Some(x_col) = dataset.fields.get(&series.encode.x).copied() else {
                continue;
            };
            let Some(x) = table.column_f64(x_col) else {
                continue;
            };

            let mut base_range = state
                .dataset_row_ranges
                .get(&series.dataset)
                .copied()
                .unwrap_or(RowRange {
                    start: 0,
                    end: table.row_count,
                });
            base_range.clamp_to_len(table.row_count);

            let x_axis_range = model
                .axes
                .get(&series.x_axis)
                .map(|a| a.range)
                .unwrap_or_default();
            let zoom = state
                .data_zoom_x
                .get(&series.x_axis)
                .copied()
                .unwrap_or_default();
            let state_window = zoom.window;
            let filter_mode = zoom.filter_mode;
            let x_policy = series_x_policy(x_axis_range, state_window, filter_mode);

            let row_range = if filter_mode == FilterMode::Filter
                && let Some(window) = x_policy.mapping_window
            {
                crate::transform::row_range_for_x_window(x, base_range, window)
            } else {
                base_range
            };

            self.series.push(SeriesView {
                series: *series_id,
                dataset: series.dataset,
                x_axis: series.x_axis,
                revision: self.revision,
                data_revision: table.revision,
                selection: RowSelection::Range(row_range),
                x_policy,
            });
        }
    }

    pub fn dataset_view(&self, dataset: DatasetId) -> Option<&DatasetView> {
        self.datasets.iter().find(|v| v.dataset == dataset)
    }

    pub fn series_view(&self, series: SeriesId) -> Option<&SeriesView> {
        self.series.iter().find(|v| v.series == series)
    }
}

pub fn table_row_range<'a>(
    table: &'a DataTable,
    view: Option<&DatasetView>,
) -> core::ops::Range<usize> {
    let mut range = RowRange {
        start: 0,
        end: table.row_count,
    };
    if let Some(view) = view {
        range = view.row_range;
    }
    range.clamp_to_len(table.row_count);
    range.start..range.end
}
