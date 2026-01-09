use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::data::DatasetStore;
use crate::engine::model::ChartModel;
use crate::engine::window_policy::AxisFilter1D;
use crate::ids::{DatasetId, Revision};
use crate::transform::{RowRange, RowSelection};
use crate::view::ViewState;

#[derive(Debug, Default, Clone)]
pub struct SelectionStage {
    desired: Vec<SelectionKey>,
    cursor: usize,
    cache: BTreeMap<SelectionKey, SelectionEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SelectionKey {
    dataset: DatasetId,
    x_col: usize,
    start: u32,
    end: u32,
    min_bits: u64,
    max_bits: u64,
}

impl SelectionKey {
    fn new(dataset: DatasetId, x_col: usize, range: RowRange, filter: AxisFilter1D) -> Self {
        Self {
            dataset,
            x_col,
            start: range.start.min(u32::MAX as usize) as u32,
            end: range.end.min(u32::MAX as usize) as u32,
            min_bits: filter.min.map(|v| v.to_bits()).unwrap_or(u64::MAX),
            max_bits: filter.max.map(|v| v.to_bits()).unwrap_or(u64::MAX),
        }
    }
}

#[derive(Debug, Clone)]
enum SelectionEntry {
    Ready {
        data_rev: Revision,
        indices: Arc<[u32]>,
    },
    Building {
        data_rev: Revision,
        next: usize,
        end: usize,
        indices: Vec<u32>,
    },
}

impl SelectionStage {
    pub fn sync_inputs(&mut self, model: &ChartModel, datasets: &DatasetStore, view: &ViewState) {
        self.desired.clear();
        self.cursor = 0;

        // Keep only selections that are still relevant for the current inputs.
        //
        // v1 policy: build indices only when:
        // - the series is visible,
        // - filter mode is "filter" (so `x_policy.filter` is active),
        // - the current view selection is a broad `Range`,
        // - the X slice is probably non-monotonic (so we cannot shrink range cheaply),
        // - and the selection is "large enough" to benefit from an index view.
        //
        // This is intentionally conservative: indices are an optimization carrier, not required for correctness.
        let mut desired_set: BTreeSet<SelectionKey> = BTreeSet::new();

        for series_id in &model.series_order {
            let Some(series) = model.series.get(series_id) else {
                continue;
            };
            if !series.visible {
                continue;
            }

            let Some(series_view) = view.series_view(*series_id) else {
                continue;
            };

            let table = datasets.dataset(series.dataset);
            let Some(table) = table else {
                continue;
            };

            let selection_range = series_view.selection.as_range(table.row_count);
            let selection_range = RowRange {
                start: selection_range.start,
                end: selection_range.end,
            };
            let visible_len = selection_range.end.saturating_sub(selection_range.start);
            if visible_len < 50_000 {
                continue;
            }

            let filter = series_view.x_policy.filter;
            if filter.min.is_none() && filter.max.is_none() {
                continue;
            }

            let Some(dataset) = model.datasets.get(&series.dataset) else {
                continue;
            };
            let Some(x_col) = dataset.fields.get(&series.encode.x).copied() else {
                continue;
            };
            let Some(x_values) = table.column_f64(x_col) else {
                continue;
            };

            if crate::transform::is_probably_monotonic_in_range(x_values, selection_range).is_some()
            {
                continue;
            }

            let key = SelectionKey::new(series.dataset, x_col, selection_range, filter);
            if desired_set.insert(key) {
                self.desired.push(key);
            }
        }

        // Prune cache entries that are no longer desired.
        self.cache.retain(|k, _| desired_set.contains(k));

        // Ensure desired entries exist (as placeholders) so step() can build them deterministically.
        for key in &self.desired {
            if self.cache.contains_key(key) {
                continue;
            }
            let table = datasets.dataset(key.dataset);
            let Some(table) = table else {
                continue;
            };
            self.cache.insert(
                *key,
                SelectionEntry::Building {
                    data_rev: table.revision,
                    next: key.start as usize,
                    end: key.end as usize,
                    indices: Vec::new(),
                },
            );
        }
    }

    pub fn step(
        &mut self,
        datasets: &DatasetStore,
        budget: &mut crate::scheduler::WorkBudget,
    ) -> bool {
        if self.desired.is_empty() {
            return true;
        }

        let mut points_consumed = 0usize;
        let max_points_per_step = 16_384usize;

        while self.cursor < self.desired.len() {
            let key = self.desired[self.cursor];

            let table = datasets.dataset(key.dataset);
            let Some(table) = table else {
                self.cache.remove(&key);
                self.cursor += 1;
                continue;
            };

            let entry = self.cache.get_mut(&key);
            let Some(entry) = entry else {
                self.cursor += 1;
                continue;
            };

            let data_rev = table.revision;
            match entry {
                SelectionEntry::Ready { data_rev: r, .. } => {
                    if *r != data_rev {
                        *entry = SelectionEntry::Building {
                            data_rev,
                            next: key.start as usize,
                            end: key.end as usize,
                            indices: Vec::new(),
                        };
                    } else {
                        self.cursor += 1;
                        continue;
                    }
                }
                SelectionEntry::Building {
                    data_rev: r,
                    next,
                    indices,
                    ..
                } => {
                    if *r != data_rev {
                        *r = data_rev;
                        *next = key.start as usize;
                        indices.clear();
                    }
                }
            }

            let SelectionEntry::Building {
                next, end, indices, ..
            } = entry
            else {
                self.cursor += 1;
                continue;
            };

            let Some(x_values) = table.column_f64(key.x_col) else {
                self.cache.remove(&key);
                self.cursor += 1;
                continue;
            };

            let filter = AxisFilter1D {
                min: (key.min_bits != u64::MAX).then(|| f64::from_bits(key.min_bits)),
                max: (key.max_bits != u64::MAX).then(|| f64::from_bits(key.max_bits)),
            };

            let points_budget = budget.take_points(4096) as usize;
            if points_budget == 0 {
                return false;
            }

            points_consumed += points_budget;

            let len = x_values.len();
            let end_limit = (*end).min(len);
            if *next > end_limit {
                *next = end_limit;
            }
            let chunk_end = (*next + points_budget).min(end_limit);

            for i in *next..chunk_end {
                let xi = x_values.get(i).copied().unwrap_or(f64::NAN);
                if !xi.is_finite() {
                    continue;
                }
                if !filter.contains(xi) {
                    continue;
                }
                if i <= u32::MAX as usize {
                    indices.push(i as u32);
                }
            }

            *next = chunk_end;

            if *next >= end_limit {
                let frozen: Arc<[u32]> = std::mem::take(indices).into();
                *entry = SelectionEntry::Ready {
                    data_rev,
                    indices: frozen,
                };
                self.cursor += 1;
            }

            if points_consumed >= max_points_per_step {
                return self.cursor >= self.desired.len();
            }
        }

        true
    }

    pub fn selection_for(
        &self,
        dataset: DatasetId,
        x_col: usize,
        selection_range: RowRange,
        filter: AxisFilter1D,
        table_rev: Revision,
    ) -> Option<RowSelection> {
        let key = SelectionKey::new(dataset, x_col, selection_range, filter);
        match self.cache.get(&key) {
            Some(SelectionEntry::Ready { data_rev, indices }) if *data_rev == table_rev => {
                Some(RowSelection::Indices(indices.clone()))
            }
            _ => None,
        }
    }
}
