use crate::data::{DataTable, DatasetStore};
use crate::engine::ChartState;
use crate::engine::model::ChartModel;
use crate::ids::{DatasetId, Revision};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy)]
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
pub struct ViewState {
    pub revision: Revision,
    pub datasets: Vec<DatasetView>,
    last_model_rev: Revision,
    last_data_rev: Revision,
    last_state_rev: Revision,
}

impl ViewState {
    pub fn sync_inputs(&mut self, model: &ChartModel, datasets: &DatasetStore, state: &ChartState) {
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
        }
    }

    pub fn rebuild(&mut self, model: &ChartModel, datasets: &DatasetStore, state: &ChartState) {
        self.datasets.clear();
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
    }

    pub fn dataset_view(&self, dataset: DatasetId) -> Option<&DatasetView> {
        self.datasets.iter().find(|v| v.dataset == dataset)
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
