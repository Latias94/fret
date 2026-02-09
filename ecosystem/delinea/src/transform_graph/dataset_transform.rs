use crate::data::DatasetStore;
use crate::engine::model::{ChartModel, DatasetModel};
use crate::ids::{DatasetId, Revision};
use crate::scheduler::WorkBudget;
use crate::spec::{
    DatasetFilterSpecV1, DatasetSortOrder, DatasetSortSpecV1, DatasetTransformSpecV1,
};
use crate::transform::{RowRange, RowSelection};
use crate::view::ViewState;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

const FNV1A_OFFSET: u64 = 0xcbf29ce484222325;
const FNV1A_PRIME: u64 = 0x00000100000001B3;

fn fnv1a_step(hash: u64, value: u64) -> u64 {
    (hash ^ value).wrapping_mul(FNV1A_PRIME)
}

#[derive(Debug, Default, Clone)]
pub(super) struct DatasetTransformStage {
    requested: Vec<DatasetId>,
    requested_set: BTreeSet<DatasetId>,
    cursor: usize,
    cache: BTreeMap<DatasetId, DatasetTransformEntry>,
}

#[derive(Debug, Clone)]
enum DatasetTransformEntry {
    Ready {
        signature: u64,
        data_rev: Revision,
        row_count: usize,
        indices: Arc<[u32]>,
    },
    Building {
        signature: u64,
        data_rev: Revision,
        row_count: usize,
        next: usize,
        end: usize,
        indices: Vec<u32>,
    },
}

impl DatasetTransformStage {
    pub fn clear(&mut self) {
        self.requested.clear();
        self.requested_set.clear();
        self.cursor = 0;
        self.cache.clear();
    }

    pub fn begin_frame(&mut self) {
        self.requested.clear();
        self.requested_set.clear();
        self.cursor = 0;
    }

    pub fn prepare_requests(&mut self) {
        let keep = self.requested_set.clone();
        self.cache.retain(|k, _| keep.contains(k));
    }

    pub fn request_dataset(&mut self, dataset: DatasetId) -> bool {
        if !self.requested_set.insert(dataset) {
            return false;
        }
        self.requested.push(dataset);
        true
    }

    pub fn selection_for_dataset(
        &self,
        model: &ChartModel,
        datasets: &DatasetStore,
        view: &ViewState,
        dataset: DatasetId,
    ) -> Option<RowSelection> {
        let signature = dataset_transform_signature(model, datasets, view, dataset)?;
        let root = model.root_dataset_id(dataset);
        let table = datasets.dataset(root)?;
        match self.cache.get(&dataset) {
            Some(DatasetTransformEntry::Ready {
                signature: sig,
                data_rev,
                indices,
                ..
            }) if *sig == signature && *data_rev == table.revision => {
                Some(RowSelection::Indices(indices.clone()))
            }
            _ => None,
        }
    }

    pub fn step(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        view: &ViewState,
        budget: &mut WorkBudget,
    ) -> bool {
        if self.requested.is_empty() {
            return true;
        }

        let mut any_unfinished = false;

        while budget.points > 0 && self.cursor < self.requested.len() {
            let dataset = self.requested[self.cursor];
            self.cursor = self.cursor.saturating_add(1);

            let Some(dataset_model) = model.datasets.get(&dataset) else {
                continue;
            };
            if dataset_model.from.is_none() || dataset_model.transforms.is_empty() {
                continue;
            }

            let Some(signature) = dataset_transform_signature(model, datasets, view, dataset)
            else {
                continue;
            };

            // Ensure the lineage root exists.
            let root = model.root_dataset_id(dataset);
            let Some(table) = datasets.dataset(root) else {
                continue;
            };

            // If the parent dataset is derived, we must wait until its mapping is ready before
            // building the child mapping (lineage composition).
            if let Some(parent) = dataset_model.from
                && model
                    .datasets
                    .get(&parent)
                    .is_some_and(|d| d.from.is_some() && !d.transforms.is_empty())
                && self
                    .selection_for_dataset(model, datasets, view, parent)
                    .is_none()
            {
                any_unfinished = true;
                continue;
            }

            let row_count = table.row_count;
            let data_rev = table.revision;

            let entry_needs_rebuild = match self.cache.get(&dataset) {
                Some(DatasetTransformEntry::Ready {
                    signature: sig,
                    data_rev: rev,
                    row_count: rows,
                    ..
                }) => *sig != signature || *rev != data_rev || *rows != row_count,
                Some(DatasetTransformEntry::Building {
                    signature: sig,
                    data_rev: rev,
                    row_count: rows,
                    ..
                }) => *sig != signature || *rev != data_rev || *rows != row_count,
                None => true,
            };

            if entry_needs_rebuild {
                let base_range =
                    view.dataset_view(dataset)
                        .map(|v| v.row_range)
                        .unwrap_or(RowRange {
                            start: 0,
                            end: row_count,
                        });

                let Some(parent) = dataset_model.from else {
                    continue;
                };
                let parent_base_range =
                    view.dataset_view(parent)
                        .map(|v| v.row_range)
                        .unwrap_or(RowRange {
                            start: 0,
                            end: row_count,
                        });

                let upstream_selection = if model
                    .datasets
                    .get(&parent)
                    .is_some_and(|d| d.from.is_some() && !d.transforms.is_empty())
                {
                    self.selection_for_dataset(model, datasets, view, parent)
                        .unwrap_or(RowSelection::Range(parent_base_range))
                } else {
                    RowSelection::Range(parent_base_range)
                };

                let _ = base_range;
                let end = upstream_selection.view_len(row_count);
                self.cache.insert(
                    dataset,
                    DatasetTransformEntry::Building {
                        signature,
                        data_rev,
                        row_count,
                        next: 0,
                        end,
                        indices: Vec::new(),
                    },
                );
            }

            let base_range = view
                .dataset_view(dataset)
                .map(|v| v.row_range)
                .unwrap_or(RowRange {
                    start: 0,
                    end: row_count,
                });

            let Some(parent) = dataset_model.from else {
                continue;
            };

            let parent_base_range =
                view.dataset_view(parent)
                    .map(|v| v.row_range)
                    .unwrap_or(RowRange {
                        start: 0,
                        end: row_count,
                    });

            let upstream_selection = if model
                .datasets
                .get(&parent)
                .is_some_and(|d| d.from.is_some() && !d.transforms.is_empty())
            {
                self.selection_for_dataset(model, datasets, view, parent)
                    .unwrap_or(RowSelection::Range(parent_base_range))
            } else {
                RowSelection::Range(parent_base_range)
            };

            let Some(entry) = self.cache.get_mut(&dataset) else {
                continue;
            };

            let DatasetTransformEntry::Building {
                signature: entry_signature,
                data_rev: entry_rev,
                row_count: entry_rows,
                next,
                end,
                indices,
            } = entry
            else {
                continue;
            };

            // Scan as many upstream view rows as the budget allows.
            let remaining = end.saturating_sub(*next);
            let take = budget.take_points(remaining.min(u32::MAX as usize) as u32) as usize;
            for view_index in *next..(*next + take) {
                let Some(raw_index) = upstream_selection.get_raw_index(row_count, view_index)
                else {
                    continue;
                };
                let raw_index_u32 = raw_index.min(u32::MAX as usize) as u32;
                if raw_index < base_range.start || raw_index >= base_range.end {
                    continue;
                }
                if !passes_all_filters(dataset_model, table, raw_index) {
                    continue;
                }
                indices.push(raw_index_u32);
            }
            *next = (*next + take).min(*end);

            if *next < *end {
                any_unfinished = true;
                continue;
            }

            // Finalize by applying sorts (if any) to the collected raw indices.
            apply_sorts_in_place(dataset_model, table, indices);
            let ready_indices: Arc<[u32]> = std::mem::take(indices).into();
            let signature = *entry_signature;
            let data_rev = *entry_rev;
            let row_count = *entry_rows;
            *entry = DatasetTransformEntry::Ready {
                signature,
                data_rev,
                row_count,
                indices: ready_indices,
            };
        }

        !any_unfinished
    }
}

impl super::TransformGraph {
    pub fn request_dataset_transforms_for_dataset(
        &mut self,
        model: &ChartModel,
        dataset: DatasetId,
    ) -> bool {
        let Some(ds) = model.datasets.get(&dataset) else {
            return false;
        };
        if ds.from.is_none() || ds.transforms.is_empty() {
            return false;
        }

        // Ensure the parent is requested first so `step` can build deterministically.
        if let Some(parent) = ds.from {
            let _ = self.request_dataset_transforms_for_dataset(model, parent);
        }

        self.dataset_transform_stage.request_dataset(dataset)
    }

    pub fn dataset_transform_selection_for_dataset(
        &self,
        model: &ChartModel,
        datasets: &DatasetStore,
        view: &ViewState,
        dataset: DatasetId,
    ) -> Option<RowSelection> {
        self.dataset_transform_stage
            .selection_for_dataset(model, datasets, view, dataset)
    }
}

fn dataset_transform_signature(
    model: &ChartModel,
    datasets: &DatasetStore,
    view: &ViewState,
    dataset: DatasetId,
) -> Option<u64> {
    let ds = model.datasets.get(&dataset)?;
    if ds.from.is_none() || ds.transforms.is_empty() {
        return None;
    }

    let root = model.root_dataset_id(dataset);
    let table = datasets.dataset(root)?;

    let mut h = FNV1A_OFFSET;
    h = fnv1a_step(h, model.revs.spec.0);
    h = fnv1a_step(h, dataset.0);
    h = fnv1a_step(h, root.0);
    h = fnv1a_step(h, table.revision.0);
    h = fnv1a_step(h, table.row_count as u64);

    let base_range = view
        .dataset_view(dataset)
        .map(|v| v.row_range)
        .unwrap_or(RowRange {
            start: 0,
            end: table.row_count,
        });
    h = fnv1a_step(h, base_range.start as u64);
    h = fnv1a_step(h, base_range.end as u64);

    if let Some(parent) = ds.from {
        h = fnv1a_step(h, parent.0);
        // Include parent signature so dataset_row_ranges changes upstream invalidate this node.
        if let Some(parent_sig) = dataset_transform_signature(model, datasets, view, parent) {
            h = fnv1a_step(h, parent_sig);
        } else {
            // Parent is raw (no transforms); include its base row range.
            let parent_base = view
                .dataset_view(parent)
                .map(|v| v.row_range)
                .unwrap_or(RowRange {
                    start: 0,
                    end: table.row_count,
                });
            h = fnv1a_step(h, parent_base.start as u64);
            h = fnv1a_step(h, parent_base.end as u64);
        }
    }

    for t in &ds.transforms {
        match t {
            DatasetTransformSpecV1::Filter(f) => {
                h = fnv1a_step(h, 1);
                h = fnv1a_step(h, f.field.0);
                h = fnv1a_step(h, f.gte.map(|v| v.to_bits()).unwrap_or(0));
                h = fnv1a_step(h, f.gt.map(|v| v.to_bits()).unwrap_or(0));
                h = fnv1a_step(h, f.lte.map(|v| v.to_bits()).unwrap_or(0));
                h = fnv1a_step(h, f.lt.map(|v| v.to_bits()).unwrap_or(0));
                h = fnv1a_step(h, f.eq.map(|v| v.to_bits()).unwrap_or(0));
                h = fnv1a_step(h, f.ne.map(|v| v.to_bits()).unwrap_or(0));
            }
            DatasetTransformSpecV1::Sort(s) => {
                h = fnv1a_step(h, 2);
                h = fnv1a_step(h, s.field.0);
                h = fnv1a_step(
                    h,
                    match s.order {
                        DatasetSortOrder::Asc => 1,
                        DatasetSortOrder::Desc => 2,
                    },
                );
            }
        }
    }

    Some(h)
}

fn passes_all_filters(ds: &DatasetModel, table: &crate::data::DataTable, raw_index: usize) -> bool {
    for t in &ds.transforms {
        let DatasetTransformSpecV1::Filter(f) = t else {
            continue;
        };
        if !passes_filter(f, ds, table, raw_index) {
            return false;
        }
    }
    true
}

fn passes_filter(
    f: &DatasetFilterSpecV1,
    ds: &DatasetModel,
    table: &crate::data::DataTable,
    raw_index: usize,
) -> bool {
    let Some(col) = ds.fields.get(&f.field).copied() else {
        return false;
    };
    let Some(values) = table.column_f64(col) else {
        return false;
    };
    let x = values.get(raw_index).copied().unwrap_or(f64::NAN);
    if !x.is_finite() {
        return false;
    }
    if let Some(t) = f.gte {
        if x < t {
            return false;
        }
    }
    if let Some(t) = f.gt {
        if x <= t {
            return false;
        }
    }
    if let Some(t) = f.lte {
        if x > t {
            return false;
        }
    }
    if let Some(t) = f.lt {
        if x >= t {
            return false;
        }
    }
    if let Some(t) = f.eq {
        if x != t {
            return false;
        }
    }
    if let Some(t) = f.ne {
        if x == t {
            return false;
        }
    }
    true
}

fn apply_sorts_in_place(ds: &DatasetModel, table: &crate::data::DataTable, indices: &mut Vec<u32>) {
    for t in &ds.transforms {
        let DatasetTransformSpecV1::Sort(s) = t else {
            continue;
        };
        apply_sort_in_place(ds, table, indices, s);
    }
}

fn apply_sort_in_place(
    ds: &DatasetModel,
    table: &crate::data::DataTable,
    indices: &mut Vec<u32>,
    s: &DatasetSortSpecV1,
) {
    let Some(col) = ds.fields.get(&s.field).copied() else {
        return;
    };
    let Some(values) = table.column_f64(col) else {
        return;
    };

    indices.sort_by(|&a, &b| {
        let ia = a as usize;
        let ib = b as usize;
        let ka = values.get(ia).copied().unwrap_or(f64::NAN);
        let kb = values.get(ib).copied().unwrap_or(f64::NAN);

        let ord = match (ka.is_finite(), kb.is_finite()) {
            (true, true) => ka.partial_cmp(&kb).unwrap_or(core::cmp::Ordering::Equal),
            (true, false) => core::cmp::Ordering::Less,
            (false, true) => core::cmp::Ordering::Greater,
            (false, false) => core::cmp::Ordering::Equal,
        };

        let ord = match s.order {
            DatasetSortOrder::Asc => ord,
            DatasetSortOrder::Desc => ord.reverse(),
        };

        if ord == core::cmp::Ordering::Equal {
            ia.cmp(&ib)
        } else {
            ord
        }
    });
}
