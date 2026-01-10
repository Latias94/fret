use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::data::DatasetStore;
use crate::engine::window_policy::AxisFilter1D;
use crate::ids::{DatasetId, Revision};
use crate::scheduler::WorkBudget;
use crate::transform::RowRange;

const INDEX_NOT_FOUND: i32 = -1;

#[derive(Debug, Default, Clone)]
pub struct OrdinalIndexStage {
    requested: Vec<OrdinalIndexKey>,
    requested_set: BTreeSet<OrdinalIndexKey>,
    cursor: usize,
    cache: BTreeMap<OrdinalIndexKey, OrdinalIndexEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrdinalIndexKey {
    dataset: DatasetId,
    x_col: usize,
    ordinal_len: u32,
    start: u32,
    end: u32,
    min_bits: u64,
    max_bits: u64,
}

#[derive(Debug, Clone)]
enum OrdinalIndexEntry {
    Ready {
        data_rev: Revision,
        map: Arc<[i32]>,
    },
    Building {
        data_rev: Revision,
        next: usize,
        end: usize,
        map: Vec<i32>,
    },
}

impl OrdinalIndexStage {
    pub fn begin_frame(&mut self) {
        self.requested.clear();
        self.requested_set.clear();
        self.cursor = 0;
    }

    pub fn request(&mut self, key: OrdinalIndexKey) -> bool {
        if !self.requested_set.insert(key) {
            return false;
        }
        self.requested.push(key);
        true
    }

    pub fn prepare_requests(&mut self, datasets: &DatasetStore) {
        self.cache.retain(|k, _| self.requested_set.contains(k));

        for key in &self.requested {
            if self.cache.contains_key(key) {
                continue;
            }

            let Some(table) = datasets.dataset(key.dataset) else {
                continue;
            };

            let ordinal_len = key.ordinal_len as usize;
            self.cache.insert(
                *key,
                OrdinalIndexEntry::Building {
                    data_rev: table.revision,
                    next: key.start as usize,
                    end: key.end as usize,
                    map: vec![INDEX_NOT_FOUND; ordinal_len],
                },
            );
        }
    }

    pub fn step(&mut self, datasets: &DatasetStore, budget: &mut WorkBudget) -> bool {
        if self.requested.is_empty() {
            return true;
        }

        while self.cursor < self.requested.len() {
            let key = self.requested[self.cursor];

            let Some(table) = datasets.dataset(key.dataset) else {
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
                OrdinalIndexEntry::Ready { data_rev: r, .. } => {
                    if *r == data_rev {
                        self.cursor += 1;
                        continue;
                    }
                    *entry = OrdinalIndexEntry::Building {
                        data_rev,
                        next: key.start as usize,
                        end: key.end as usize,
                        map: vec![INDEX_NOT_FOUND; key.ordinal_len as usize],
                    };
                }
                OrdinalIndexEntry::Building {
                    data_rev: r,
                    next,
                    end,
                    map,
                } => {
                    if *r != data_rev {
                        *r = data_rev;
                        *next = key.start as usize;
                        *end = key.end as usize;
                        map.clear();
                        map.resize(key.ordinal_len as usize, INDEX_NOT_FOUND);
                    }
                }
            }

            let OrdinalIndexEntry::Building { next, end, map, .. } = entry else {
                self.cursor += 1;
                continue;
            };

            let Some(x_values) = table.column_f64(key.x_col) else {
                self.cache.remove(&key);
                self.cursor += 1;
                continue;
            };

            let len = x_values.len();
            let start = (*next).min(len);
            let end_limit = (*end).min(len);
            if end_limit <= start {
                let ready = OrdinalIndexEntry::Ready {
                    data_rev,
                    map: Arc::from(map.clone().into_boxed_slice()),
                };
                self.cache.insert(key, ready);
                self.cursor += 1;
                continue;
            }

            let max_points = budget.take_points(16_384) as usize;
            if max_points == 0 {
                return false;
            }

            let filter = AxisFilter1D {
                min: from_bits(key.min_bits),
                max: from_bits(key.max_bits),
            };

            let end_now = (start + max_points).min(end_limit);
            for i in start..end_now {
                let x = x_values.get(i).copied().unwrap_or(f64::NAN);
                if !x.is_finite() {
                    continue;
                }
                if !filter.contains(x) {
                    continue;
                }

                // ECharts: inverted indices only for ordinal dimensions, only distinct values.
                // We follow the same assumption: if duplicates exist, we keep the first mapping.
                let ord = (x.round() as i64) as i32;
                if ord < 0 {
                    continue;
                }
                let ord = ord as usize;
                if ord >= map.len() {
                    continue;
                }
                if map[ord] == INDEX_NOT_FOUND {
                    map[ord] = i as i32;
                }
            }

            *next = end_now;
            if *next >= end_limit {
                let ready = OrdinalIndexEntry::Ready {
                    data_rev,
                    map: Arc::from(map.clone().into_boxed_slice()),
                };
                self.cache.insert(key, ready);
                self.cursor += 1;
            } else {
                return false;
            }
        }

        true
    }

    pub fn raw_index_of_ordinal(
        &self,
        key: OrdinalIndexKey,
        ordinal: u32,
        data_rev: Revision,
    ) -> Option<usize> {
        match self.cache.get(&key) {
            Some(OrdinalIndexEntry::Ready { data_rev: r, map }) if *r == data_rev => map
                .get(ordinal as usize)
                .copied()
                .filter(|&i| i != INDEX_NOT_FOUND)
                .map(|i| i as usize),
            _ => None,
        }
    }
}

impl OrdinalIndexKey {
    pub fn new(
        dataset: DatasetId,
        x_col: usize,
        ordinal_len: usize,
        range: RowRange,
        filter: AxisFilter1D,
    ) -> Self {
        Self {
            dataset,
            x_col,
            ordinal_len: ordinal_len.min(u32::MAX as usize) as u32,
            start: range.start.min(u32::MAX as usize) as u32,
            end: range.end.min(u32::MAX as usize) as u32,
            min_bits: filter.min.map(|v| v.to_bits()).unwrap_or(u64::MAX),
            max_bits: filter.max.map(|v| v.to_bits()).unwrap_or(u64::MAX),
        }
    }
}

fn from_bits(bits: u64) -> Option<f64> {
    if bits == u64::MAX {
        None
    } else {
        Some(f64::from_bits(bits))
    }
}

#[cfg(test)]
mod tests {
    use crate::data::{Column, DataTable};
    use crate::scheduler::WorkBudget;

    use super::*;

    #[test]
    fn ordinal_index_builds_incrementally_and_maps_ordinals_to_raw_indices() {
        let dataset = DatasetId::new(1);
        let mut datasets = DatasetStore::default();

        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![0.0, 2.0, 1.0]));
        let rev = table.revision;
        datasets.insert(dataset, table);

        let key = OrdinalIndexKey::new(
            dataset,
            0,
            3,
            RowRange { start: 0, end: 100 },
            AxisFilter1D::default(),
        );

        let mut stage = OrdinalIndexStage::default();
        stage.begin_frame();
        stage.request(key);
        stage.prepare_requests(&datasets);

        let mut budget = WorkBudget::new(1, 0, 0);
        assert!(!stage.step(&datasets, &mut budget));

        let mut budget = WorkBudget::new(1024, 0, 0);
        assert!(stage.step(&datasets, &mut budget));

        assert_eq!(stage.raw_index_of_ordinal(key, 0, rev), Some(0));
        assert_eq!(stage.raw_index_of_ordinal(key, 1, rev), Some(2));
        assert_eq!(stage.raw_index_of_ordinal(key, 2, rev), Some(1));
        assert_eq!(stage.raw_index_of_ordinal(key, 3, rev), None);
    }
}
