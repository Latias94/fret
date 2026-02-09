use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::data::DatasetStore;
use crate::engine::window_policy::AxisFilter1D;
use crate::ids::{DatasetId, Revision};
use crate::transform::RowRange;

#[derive(Debug, Default, Clone)]
pub struct NearestXIndexStage {
    requested: Vec<NearestXIndexKey>,
    requested_set: BTreeSet<NearestXIndexKey>,
    cursor: usize,
    cache: BTreeMap<NearestXIndexKey, NearestXIndexEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NearestXIndexKey {
    dataset: DatasetId,
    root_dataset: DatasetId,
    x_col: usize,
    start: u32,
    end: u32,
    min_bits: u64,
    max_bits: u64,
}

impl NearestXIndexKey {
    pub fn new(
        dataset: DatasetId,
        root_dataset: DatasetId,
        x_col: usize,
        range: RowRange,
        filter: AxisFilter1D,
    ) -> Self {
        Self {
            dataset,
            root_dataset,
            x_col,
            start: range.start.min(u32::MAX as usize) as u32,
            end: range.end.min(u32::MAX as usize) as u32,
            min_bits: filter.min.map(|v| v.to_bits()).unwrap_or(u64::MAX),
            max_bits: filter.max.map(|v| v.to_bits()).unwrap_or(u64::MAX),
        }
    }

    fn filter(self) -> AxisFilter1D {
        AxisFilter1D {
            min: (self.min_bits != u64::MAX).then(|| f64::from_bits(self.min_bits)),
            max: (self.max_bits != u64::MAX).then(|| f64::from_bits(self.max_bits)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NearestXIndexItem {
    pub x: f64,
    pub raw: u32,
}

#[derive(Debug, Clone)]
enum NearestXIndexEntry {
    Ready {
        data_rev: Revision,
        row_count: usize,
        end_limit: usize,
        items: Arc<[NearestXIndexItem]>,
    },
    Building {
        data_rev: Revision,
        row_count: usize,
        next: usize,
        end: usize,
        items: Vec<NearestXIndexItem>,
    },
}

impl NearestXIndexStage {
    pub fn begin_frame(&mut self) {
        self.requested.clear();
        self.requested_set.clear();
        self.cursor = 0;
    }

    pub fn request(&mut self, key: NearestXIndexKey) -> bool {
        if !self.requested_set.insert(key) {
            return false;
        }
        self.requested.push(key);
        true
    }

    pub fn prepare_requests(&mut self, datasets: &DatasetStore) {
        // Keep existing entries even when there are no active requests so a user can re-hover
        // without forcing an immediate rebuild. When requests are present, prune unrelated entries,
        // but keep a single best "prefix" entry per requested key so growing requests can reuse
        // prefix scans.
        if !self.requested_set.is_empty() {
            let mut keep = self.requested_set.clone();
            for key in &self.requested {
                if let Some(prefix) = self.best_prefix_key(*key) {
                    keep.insert(prefix);
                }
            }
            self.cache.retain(|k, _| keep.contains(k));
        }

        for key in &self.requested {
            if self.cache.contains_key(key) {
                continue;
            }

            let Some(table) = datasets.dataset(key.root_dataset) else {
                continue;
            };

            let start = key.start as usize;
            let end = key.end as usize;

            let mut seed_items: Vec<NearestXIndexItem> = Vec::new();
            let mut seed_next = start;
            if let Some((seed, seed_next_limit)) = self.best_prefix_seed(*key) {
                seed_items = seed;
                seed_next = seed_next_limit;
            }

            self.cache.insert(
                *key,
                NearestXIndexEntry::Building {
                    data_rev: table.revision,
                    row_count: table.row_count,
                    next: seed_next,
                    end,
                    items: seed_items,
                },
            );
        }
    }

    fn best_prefix_key(&self, requested: NearestXIndexKey) -> Option<NearestXIndexKey> {
        let requested_end = requested.end as usize;
        let mut best: Option<(usize, NearestXIndexKey)> = None;

        for (k, _) in self.cache.iter() {
            if k.dataset != requested.dataset || k.root_dataset != requested.root_dataset {
                continue;
            }
            if k.x_col != requested.x_col {
                continue;
            }
            if k.start != requested.start {
                continue;
            }
            if k.min_bits != requested.min_bits || k.max_bits != requested.max_bits {
                continue;
            }

            let prefix_end = k.end as usize;
            if prefix_end >= requested_end {
                continue;
            }

            match best {
                Some((best_end, _)) if prefix_end <= best_end => {}
                _ => best = Some((prefix_end, *k)),
            }
        }

        best.map(|(_, k)| k)
    }

    fn best_prefix_seed(
        &self,
        requested: NearestXIndexKey,
    ) -> Option<(Vec<NearestXIndexItem>, usize)> {
        let requested_end = requested.end as usize;
        let mut best: Option<(usize, Vec<NearestXIndexItem>, usize)> = None;

        for (k, entry) in self.cache.iter() {
            if k.dataset != requested.dataset || k.root_dataset != requested.root_dataset {
                continue;
            }
            if k.x_col != requested.x_col {
                continue;
            }
            if k.start != requested.start {
                continue;
            }
            if k.min_bits != requested.min_bits || k.max_bits != requested.max_bits {
                continue;
            }

            let prefix_end = k.end as usize;
            if prefix_end >= requested_end {
                continue;
            }

            let (items, next) = match entry {
                NearestXIndexEntry::Ready {
                    items, end_limit, ..
                } => (items.to_vec(), *end_limit),
                NearestXIndexEntry::Building { items, next, .. } => (items.clone(), *next),
            };

            match best {
                Some((best_end, ..)) if prefix_end <= best_end => {}
                _ => best = Some((prefix_end, items, next)),
            }
        }

        best.map(|(_, items, next)| (items, next))
    }

    pub fn step(
        &mut self,
        datasets: &DatasetStore,
        budget: &mut crate::scheduler::WorkBudget,
    ) -> bool {
        if self.requested.is_empty() {
            return true;
        }

        let mut points_consumed = 0usize;
        let max_points_per_step = 16_384usize;

        while self.cursor < self.requested.len() {
            let key = self.requested[self.cursor];

            let Some(table) = datasets.dataset(key.root_dataset) else {
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
                NearestXIndexEntry::Ready {
                    data_rev: r,
                    row_count: cached_len,
                    end_limit,
                    items,
                } => {
                    if *r == data_rev {
                        self.cursor += 1;
                        continue;
                    }

                    let requested_end = key.end as usize;
                    let next_end_limit = requested_end.min(table.row_count);

                    let is_append_only = table.row_count >= *cached_len;
                    if is_append_only && next_end_limit >= *end_limit {
                        if next_end_limit == *end_limit {
                            *r = data_rev;
                            *cached_len = table.row_count;
                            self.cursor += 1;
                            continue;
                        }

                        *entry = NearestXIndexEntry::Building {
                            data_rev,
                            row_count: table.row_count,
                            next: *end_limit,
                            end: requested_end,
                            items: items.to_vec(),
                        };
                    } else {
                        *entry = NearestXIndexEntry::Building {
                            data_rev,
                            row_count: table.row_count,
                            next: key.start as usize,
                            end: requested_end,
                            items: Vec::new(),
                        };
                    }
                }
                NearestXIndexEntry::Building {
                    data_rev: r,
                    row_count: cached_len,
                    next,
                    items,
                    ..
                } => {
                    if *r != data_rev {
                        let is_append_only = table.row_count >= *cached_len;
                        let can_resume = is_append_only && *next <= *cached_len;
                        *r = data_rev;
                        *cached_len = table.row_count;
                        if !can_resume {
                            *next = key.start as usize;
                            items.clear();
                        }
                    }
                }
            }

            let NearestXIndexEntry::Building {
                next, end, items, ..
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

            let filter = key.filter();

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

            for raw_index in *next..chunk_end {
                let x = x_values.get(raw_index).copied().unwrap_or(f64::NAN);
                if !x.is_finite() {
                    continue;
                }
                if !filter.contains(x) {
                    continue;
                }
                if raw_index <= u32::MAX as usize {
                    items.push(NearestXIndexItem {
                        x,
                        raw: raw_index as u32,
                    });
                }
            }

            *next = chunk_end;

            if *next >= end_limit {
                items.sort_unstable_by(|a, b| a.x.total_cmp(&b.x).then(a.raw.cmp(&b.raw)));
                let frozen: Arc<[NearestXIndexItem]> = std::mem::take(items).into();
                *entry = NearestXIndexEntry::Ready {
                    data_rev,
                    row_count: table.row_count,
                    end_limit,
                    items: frozen,
                };
                self.cursor += 1;
            }

            if points_consumed >= max_points_per_step {
                return self.cursor >= self.requested.len();
            }
        }

        true
    }

    pub fn items_for(
        &self,
        key: NearestXIndexKey,
        table_rev: Revision,
    ) -> Option<&[NearestXIndexItem]> {
        match self.cache.get(&key) {
            Some(NearestXIndexEntry::Ready {
                data_rev, items, ..
            }) if *data_rev == table_rev => Some(items),
            _ => None,
        }
    }

    pub fn nearest_raw_index(
        &self,
        key: NearestXIndexKey,
        table_rev: Revision,
        x_value: f64,
    ) -> Option<(usize, f64)> {
        let items = self.items_for(key, table_rev)?;
        nearest_raw_index_in_sorted_x_index(items, x_value)
    }
}

pub fn nearest_raw_index_in_sorted_x_index(
    items: &[NearestXIndexItem],
    x_value: f64,
) -> Option<(usize, f64)> {
    if !x_value.is_finite() {
        return None;
    }
    if items.is_empty() {
        return None;
    }

    let idx = items.partition_point(|p| p.x < x_value);
    let mut best = if idx >= items.len() {
        items.len() - 1
    } else {
        idx
    };
    if idx > 0 {
        let left = idx - 1;
        let dx_best = (items[best].x - x_value).abs();
        let dx_left = (items[left].x - x_value).abs();
        if dx_left < dx_best || (dx_left == dx_best && items[left].raw < items[best].raw) {
            best = left;
        }
    }

    let raw = items[best].raw as usize;
    Some((raw, items[best].x))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Column, DataTable, DatasetStore};
    use crate::ids::DatasetId;
    use crate::scheduler::WorkBudget;

    #[test]
    fn nearest_raw_index_prefers_closest_x_then_smallest_raw_index() {
        let items = vec![
            NearestXIndexItem { x: 0.0, raw: 5 },
            NearestXIndexItem { x: 1.0, raw: 1 },
            NearestXIndexItem { x: 1.0, raw: 2 },
            NearestXIndexItem { x: 3.0, raw: 9 },
        ];

        assert_eq!(
            nearest_raw_index_in_sorted_x_index(&items, 2.9),
            Some((9, 3.0))
        );
        assert_eq!(
            nearest_raw_index_in_sorted_x_index(&items, 0.6),
            Some((1, 1.0))
        );
        assert_eq!(
            nearest_raw_index_in_sorted_x_index(&items, 1.0),
            Some((1, 1.0))
        );
    }

    #[test]
    fn prepare_requests_seeds_from_best_prefix_entry() {
        let dataset_id = DatasetId::new(1);

        let mut table = DataTable::default();
        table.push_column(Column::F64((0..10).map(|i| i as f64).collect()));

        let mut datasets = DatasetStore::default();
        datasets.insert(dataset_id, table);

        let mut stage = NearestXIndexStage::default();
        let filter = AxisFilter1D::default();

        let key_prefix = NearestXIndexKey::new(
            dataset_id,
            dataset_id,
            0,
            RowRange { start: 0, end: 3 },
            filter,
        );
        let key_full = NearestXIndexKey::new(
            dataset_id,
            dataset_id,
            0,
            RowRange { start: 0, end: 10 },
            filter,
        );

        stage.begin_frame();
        stage.request(key_prefix);
        stage.prepare_requests(&datasets);
        let mut budget = WorkBudget::new(1_000_000, 0, 0);
        assert!(stage.step(&datasets, &mut budget));

        stage.begin_frame();
        stage.request(key_full);
        stage.prepare_requests(&datasets);

        let entry = stage
            .cache
            .get(&key_full)
            .expect("full request should exist");
        let NearestXIndexEntry::Building { next, items, .. } = entry else {
            panic!("expected seeded Building entry");
        };
        assert_eq!(*next, 3);
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn step_resumes_append_only_growth_for_same_key() {
        let dataset_id = DatasetId::new(1);

        let mut table = DataTable::default();
        table.push_column(Column::F64((0..5).map(|i| i as f64).collect()));

        let mut datasets = DatasetStore::default();
        datasets.insert(dataset_id, table);

        let mut stage = NearestXIndexStage::default();
        let filter = AxisFilter1D::default();

        let key = NearestXIndexKey::new(
            dataset_id,
            dataset_id,
            0,
            RowRange { start: 0, end: 10 },
            filter,
        );

        stage.begin_frame();
        stage.request(key);
        stage.prepare_requests(&datasets);
        let mut budget = WorkBudget::new(1_000_000, 0, 0);
        assert!(stage.step(&datasets, &mut budget));

        let table = datasets.dataset(dataset_id).unwrap();
        let table_rev_before = table.revision;
        assert_eq!(table.row_count, 5);
        assert_eq!(stage.items_for(key, table_rev_before).unwrap().len(), 5);

        {
            let table = datasets.dataset_mut(dataset_id).unwrap();
            for i in 5..10 {
                table.append_row_f64(&[i as f64]).unwrap();
            }
        }

        stage.begin_frame();
        stage.request(key);
        stage.prepare_requests(&datasets);

        // Force the entry transition without consuming any points so we can inspect the resumed
        // building cursor.
        let mut zero_budget = WorkBudget::new(0, 0, 0);
        assert!(!stage.step(&datasets, &mut zero_budget));

        let entry = stage.cache.get(&key).expect("cache should contain the key");
        let NearestXIndexEntry::Building { next, items, .. } = entry else {
            panic!("expected Building entry after revision change");
        };
        assert_eq!(*next, 5);
        assert_eq!(items.len(), 5);

        let mut finish_budget = WorkBudget::new(1_000_000, 0, 0);
        assert!(stage.step(&datasets, &mut finish_budget));

        let table = datasets.dataset(dataset_id).unwrap();
        let table_rev_after = table.revision;
        assert_eq!(table.row_count, 10);
        assert_eq!(stage.items_for(key, table_rev_after).unwrap().len(), 10);
    }
}
