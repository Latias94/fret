use crate::data::{DataTable, DatasetStore};
use crate::engine::ChartState;
use crate::engine::model::ChartModel;
use crate::ids::{AxisId, DatasetId, Revision, SeriesId};
use crate::transform::{RowRange, RowSelection, SeriesXPolicy, data_zoom_x_node};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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
    last_data_sig: u64,
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
        let data_sig = dataset_store_signature(model, datasets);

        let state_rev = state.revision;

        if model_rev != self.last_model_rev
            || data_sig != self.last_data_sig
            || state_rev != self.last_state_rev
        {
            self.revision.bump();
            self.last_model_rev = model_rev;
            self.last_data_sig = data_sig;
            self.last_state_rev = state_rev;
            return true;
        }

        false
    }

    pub fn rebuild(&mut self, model: &ChartModel, datasets: &DatasetStore, state: &ChartState) {
        self.datasets.clear();
        self.series.clear();
        for (id, _dataset_model) in &model.datasets {
            let table = datasets.dataset(*id);
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
            let table = datasets.dataset(series.dataset);
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
            let x_window =
                data_zoom_x_node(model, state, series.x_axis, x_axis_range).apply(x, base_range);

            self.series.push(SeriesView {
                series: *series_id,
                dataset: series.dataset,
                x_axis: series.x_axis,
                revision: self.revision,
                data_revision: table.revision,
                selection: x_window.selection,
                x_policy: x_window.x_policy,
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

fn dataset_store_signature(model: &ChartModel, datasets: &DatasetStore) -> u64 {
    let mut hash = 1469598103934665603u64;
    hash = fnv1a_step(hash, model.datasets.len() as u64);
    for dataset_id in model.datasets.keys() {
        hash = fnv1a_step(hash, dataset_id.0);
        if let Some(table) = datasets.dataset(*dataset_id) {
            hash = fnv1a_step(hash, table.revision.0);
            hash = fnv1a_step(hash, table.row_count as u64);
            hash = fnv1a_step(hash, table.columns.len() as u64);
        }
    }
    hash
}

fn fnv1a_step(hash: u64, value: u64) -> u64 {
    (hash ^ value).wrapping_mul(1099511628211u64)
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
    range.as_std_range(table.row_count)
}
