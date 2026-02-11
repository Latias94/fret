//! Incremental indices views (X filter / XY weakFilter).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::data::{DataTable, DataTableView, DatasetStore};
use crate::engine::model::ChartModel;
use crate::engine::window_policy::AxisFilter1D;
use crate::ids::{DatasetId, Revision};
use crate::transform::{RowRange, RowSelection};
use crate::view::ViewState;

#[derive(Debug, Default, Clone)]
pub struct DataViewStage {
    requested: Vec<DataViewKey>,
    requested_set: BTreeSet<DataViewKey>,
    cursor: usize,
    cache: BTreeMap<DataViewKey, DataViewEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DataViewKey {
    dataset: DatasetId,
    root_dataset: DatasetId,
    kind: DataViewKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataViewKind {
    XFilter {
        x_col: usize,
        start: u32,
        end: u32,
        min_bits: u64,
        max_bits: u64,
    },
    XYWeakFilter {
        x_col: usize,
        y_col: usize,
        start: u32,
        end: u32,
        x_min_bits: u64,
        x_max_bits: u64,
        y_min_bits: u64,
        y_max_bits: u64,
    },
    XYWeakFilterBand {
        x_col: usize,
        y0_col: usize,
        y1_col: usize,
        start: u32,
        end: u32,
        x_min_bits: u64,
        x_max_bits: u64,
        y_min_bits: u64,
        y_max_bits: u64,
    },
}

impl DataViewKey {
    pub fn x_filter(
        dataset: DatasetId,
        root_dataset: DatasetId,
        x_col: usize,
        range: RowRange,
        filter: AxisFilter1D,
    ) -> Self {
        Self {
            dataset,
            root_dataset,
            kind: DataViewKind::XFilter {
                x_col,
                start: range.start.min(u32::MAX as usize) as u32,
                end: range.end.min(u32::MAX as usize) as u32,
                min_bits: filter.min.map(|v| v.to_bits()).unwrap_or(u64::MAX),
                max_bits: filter.max.map(|v| v.to_bits()).unwrap_or(u64::MAX),
            },
        }
    }

    pub fn xy_weak_filter(
        dataset: DatasetId,
        root_dataset: DatasetId,
        x_col: usize,
        y_col: usize,
        range: RowRange,
        x_filter: AxisFilter1D,
        y_filter: AxisFilter1D,
    ) -> Self {
        Self {
            dataset,
            root_dataset,
            kind: DataViewKind::XYWeakFilter {
                x_col,
                y_col,
                start: range.start.min(u32::MAX as usize) as u32,
                end: range.end.min(u32::MAX as usize) as u32,
                x_min_bits: x_filter.min.map(|v| v.to_bits()).unwrap_or(u64::MAX),
                x_max_bits: x_filter.max.map(|v| v.to_bits()).unwrap_or(u64::MAX),
                y_min_bits: y_filter.min.map(|v| v.to_bits()).unwrap_or(u64::MAX),
                y_max_bits: y_filter.max.map(|v| v.to_bits()).unwrap_or(u64::MAX),
            },
        }
    }

    pub fn xy_weak_filter_band(
        dataset: DatasetId,
        root_dataset: DatasetId,
        x_col: usize,
        y0_col: usize,
        y1_col: usize,
        range: RowRange,
        x_filter: AxisFilter1D,
        y_filter: AxisFilter1D,
    ) -> Self {
        Self {
            dataset,
            root_dataset,
            kind: DataViewKind::XYWeakFilterBand {
                x_col,
                y0_col,
                y1_col,
                start: range.start.min(u32::MAX as usize) as u32,
                end: range.end.min(u32::MAX as usize) as u32,
                x_min_bits: x_filter.min.map(|v| v.to_bits()).unwrap_or(u64::MAX),
                x_max_bits: x_filter.max.map(|v| v.to_bits()).unwrap_or(u64::MAX),
                y_min_bits: y_filter.min.map(|v| v.to_bits()).unwrap_or(u64::MAX),
                y_max_bits: y_filter.max.map(|v| v.to_bits()).unwrap_or(u64::MAX),
            },
        }
    }
}

#[derive(Debug, Clone)]
enum DataViewEntry {
    Ready {
        data_rev: Revision,
        row_count: usize,
        end_limit: usize,
        indices: Arc<[u32]>,
    },
    Building {
        data_rev: Revision,
        row_count: usize,
        next: usize,
        end: usize,
        indices: Vec<u32>,
    },
}

impl DataViewStage {
    pub fn begin_frame(&mut self) {
        self.requested.clear();
        self.requested_set.clear();
        self.cursor = 0;
    }

    pub fn request(&mut self, key: DataViewKey) -> bool {
        if !self.requested_set.insert(key) {
            return false;
        }
        self.requested.push(key);
        true
    }

    pub fn request_x_filter_for_series(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        view: &ViewState,
        series_id: crate::ids::SeriesId,
    ) -> bool {
        // v1 policy: build indices when:
        // - the series is visible,
        // - filter mode is "filter" (so `x_policy.filter` is active),
        // - the current view selection is a broad `Range`,
        // - and the X slice is probably non-monotonic (so we cannot shrink range cheaply).
        //
        // Note: while indices started as an optimization carrier, line-family rendering relies on
        // indices materialization to apply filtering semantics for non-monotonic inputs (range
        // slicing cannot represent the filter, and mark LOD should not scan unfiltered rows).

        let Some(series) = model.series.get(&series_id) else {
            return false;
        };
        if !series.visible {
            return false;
        }

        let Some(series_view) = view.series_view(series_id) else {
            return false;
        };

        let root = model.root_dataset_id(series.dataset);
        let table = datasets.dataset(root);
        let Some(table) = table else {
            return false;
        };

        let selection_range = series_view.selection.as_range(table.row_count());
        let selection_range = RowRange {
            start: selection_range.start,
            end: selection_range.end,
        };
        let visible_len = selection_range.end.saturating_sub(selection_range.start);
        if visible_len == 0 {
            return false;
        }

        let filter = series_view.x_policy.filter;
        if filter.min.is_none() && filter.max.is_none() {
            return false;
        }

        let Some(dataset) = model.datasets.get(&series.dataset) else {
            return false;
        };
        let Some(x_col) = dataset.fields.get(&series.encode.x).copied() else {
            return false;
        };
        let Some(x_values) = table.column_f64(x_col) else {
            return false;
        };

        if crate::transform::is_probably_monotonic_in_range(x_values, selection_range).is_some() {
            return false;
        }

        self.request(DataViewKey::x_filter(
            series.dataset,
            root,
            x_col,
            selection_range,
            filter,
        ))
    }

    pub fn request_xy_weak_filter_for_series(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        view: &ViewState,
        series_id: crate::ids::SeriesId,
        selection_range: RowRange,
        x_filter: AxisFilter1D,
        y_filter: AxisFilter1D,
    ) -> bool {
        let Some(series) = model.series.get(&series_id) else {
            return false;
        };
        if !series.visible {
            return false;
        }

        if series.stack.is_some() {
            return false;
        }
        if !matches!(
            series.kind,
            crate::spec::SeriesKind::Scatter
                | crate::spec::SeriesKind::Line
                | crate::spec::SeriesKind::Area
        ) {
            return false;
        }

        let Some(series_view) = view.series_view(series_id) else {
            return false;
        };
        if !matches!(
            series_view.selection,
            RowSelection::All | RowSelection::Range(_)
        ) {
            return false;
        }

        let root = model.root_dataset_id(series.dataset);
        let table = datasets.dataset(root);
        let Some(table) = table else {
            return false;
        };

        let visible_len = selection_range.end.saturating_sub(selection_range.start);
        if visible_len == 0 {
            return false;
        }

        if (x_filter.min.is_none() && x_filter.max.is_none())
            || (y_filter.min.is_none() && y_filter.max.is_none())
        {
            return false;
        }

        let Some(dataset) = model.datasets.get(&series.dataset) else {
            return false;
        };
        let Some(x_col) = dataset.fields.get(&series.encode.x).copied() else {
            return false;
        };
        let Some(y_col) = dataset.fields.get(&series.encode.y).copied() else {
            return false;
        };
        if table.column_f64(x_col).is_none() || table.column_f64(y_col).is_none() {
            return false;
        }

        self.request(DataViewKey::xy_weak_filter(
            series.dataset,
            root,
            x_col,
            y_col,
            selection_range,
            x_filter,
            y_filter,
        ))
    }

    pub fn request_xy_weak_filter_band_for_series(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        view: &ViewState,
        series_id: crate::ids::SeriesId,
        selection_range: RowRange,
        x_filter: AxisFilter1D,
        y_filter: AxisFilter1D,
    ) -> bool {
        let Some(series) = model.series.get(&series_id) else {
            return false;
        };
        if !series.visible {
            return false;
        }

        if series.stack.is_some() {
            return false;
        }
        if series.kind != crate::spec::SeriesKind::Band {
            return false;
        }

        let Some(series_view) = view.series_view(series_id) else {
            return false;
        };
        if !matches!(
            series_view.selection,
            RowSelection::All | RowSelection::Range(_)
        ) {
            return false;
        }

        let root = model.root_dataset_id(series.dataset);
        let table = datasets.dataset(root);
        let Some(table) = table else {
            return false;
        };

        let visible_len = selection_range.end.saturating_sub(selection_range.start);
        if visible_len == 0 {
            return false;
        }

        if (x_filter.min.is_none() && x_filter.max.is_none())
            || (y_filter.min.is_none() && y_filter.max.is_none())
        {
            return false;
        }

        let Some(dataset) = model.datasets.get(&series.dataset) else {
            return false;
        };
        let Some(x_col) = dataset.fields.get(&series.encode.x).copied() else {
            return false;
        };
        let Some(y0_col) = dataset.fields.get(&series.encode.y).copied() else {
            return false;
        };
        let Some(y1_field) = series.encode.y2 else {
            return false;
        };
        let Some(y1_col) = dataset.fields.get(&y1_field).copied() else {
            return false;
        };
        if table.column_f64(x_col).is_none()
            || table.column_f64(y0_col).is_none()
            || table.column_f64(y1_col).is_none()
        {
            return false;
        }

        self.request(DataViewKey::xy_weak_filter_band(
            series.dataset,
            root,
            x_col,
            y0_col,
            y1_col,
            selection_range,
            x_filter,
            y_filter,
        ))
    }

    pub fn prepare_requests(&mut self, datasets: &DatasetStore) {
        // Prune cache entries that are no longer desired, but keep a single best "prefix" entry
        // per requested key so append-only streams can reuse prefix scans when the request end grows.
        let mut keep = self.requested_set.clone();
        for key in &self.requested {
            if let Some(prefix) = self.best_prefix_x_filter_key(*key) {
                keep.insert(prefix);
            }
        }
        self.cache.retain(|k, _| keep.contains(k));

        // Ensure desired entries exist (as placeholders) so step() can build them deterministically.
        for key in &self.requested {
            if self.cache.contains_key(key) {
                continue;
            }
            let table = datasets.dataset(key.root_dataset);
            let Some(table) = table else {
                continue;
            };

            match key.kind {
                DataViewKind::XFilter { start, end, .. } => {
                    // Append-only optimization: if the request end grew (typical when selection is `All`),
                    // reuse the best available prefix entry and continue scanning from its completion point.
                    let mut seed_indices: Vec<u32> = Vec::new();
                    let mut seed_next = start as usize;
                    if let Some((seed, seed_end_limit)) = self.best_prefix_x_filter_seed(*key) {
                        seed_indices = seed;
                        seed_next = seed_end_limit;
                    }

                    self.cache.insert(
                        *key,
                        DataViewEntry::Building {
                            data_rev: table.revision(),
                            row_count: table.row_count(),
                            next: seed_next,
                            end: end as usize,
                            indices: seed_indices,
                        },
                    );
                }
                DataViewKind::XYWeakFilter { start, end, .. }
                | DataViewKind::XYWeakFilterBand { start, end, .. } => {
                    self.cache.insert(
                        *key,
                        DataViewEntry::Building {
                            data_rev: table.revision(),
                            row_count: table.row_count(),
                            next: start as usize,
                            end: end as usize,
                            indices: Vec::new(),
                        },
                    );
                }
            }
        }
    }

    fn best_prefix_x_filter_key(&self, requested: DataViewKey) -> Option<DataViewKey> {
        let DataViewKind::XFilter {
            x_col,
            start,
            end,
            min_bits,
            max_bits,
        } = requested.kind
        else {
            return None;
        };

        let requested_end = end as usize;
        let mut best: Option<(usize, DataViewKey)> = None;

        for (k, _) in self.cache.iter() {
            if k.dataset != requested.dataset || k.root_dataset != requested.root_dataset {
                continue;
            }
            let DataViewKind::XFilter {
                x_col: c,
                start: s,
                end: e,
                min_bits: min_b,
                max_bits: max_b,
            } = k.kind
            else {
                continue;
            };
            if c != x_col || s != start || min_b != min_bits || max_b != max_bits {
                continue;
            }

            let prefix_end = e as usize;
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

    fn best_prefix_x_filter_seed(&self, requested: DataViewKey) -> Option<(Vec<u32>, usize)> {
        let DataViewKind::XFilter {
            x_col,
            start,
            end,
            min_bits,
            max_bits,
        } = requested.kind
        else {
            return None;
        };

        let requested_end = end as usize;
        let mut best: Option<(usize, Vec<u32>, usize)> = None;

        for (k, entry) in self.cache.iter() {
            if k.dataset != requested.dataset || k.root_dataset != requested.root_dataset {
                continue;
            }
            let DataViewKind::XFilter {
                x_col: c,
                start: s,
                end: e,
                min_bits: min_b,
                max_bits: max_b,
            } = k.kind
            else {
                continue;
            };
            if c != x_col || s != start || min_b != min_bits || max_b != max_bits {
                continue;
            }

            let prefix_end = e as usize;
            if prefix_end >= requested_end {
                continue;
            }

            let (indices, end_limit) = match entry {
                DataViewEntry::Ready {
                    indices, end_limit, ..
                } => (indices.to_vec(), *end_limit),
                DataViewEntry::Building { indices, next, .. } => (indices.clone(), *next),
            };

            match best {
                Some((best_end, ..)) if prefix_end <= best_end => {}
                _ => best = Some((prefix_end, indices, end_limit)),
            }
        }

        best.map(|(_, indices, next)| (indices, next))
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

            let table = datasets.dataset(key.root_dataset);
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

            let data_rev = table.revision();
            match entry {
                DataViewEntry::Ready {
                    data_rev: r,
                    row_count: cached_len,
                    end_limit,
                    indices,
                } => {
                    if *r == data_rev {
                        self.cursor += 1;
                        continue;
                    }

                    let (start, end) = match key.kind {
                        DataViewKind::XFilter { start, end, .. }
                        | DataViewKind::XYWeakFilter { start, end, .. } => (start, end),
                        DataViewKind::XYWeakFilterBand { start, end, .. } => (start, end),
                    };
                    let requested_end = end as usize;
                    let next_end_limit = requested_end.min(table.row_count());

                    // Append-only fast path: if the dataset grew, continue scanning from the
                    // previous completion point and keep accumulated indices.
                    let is_append_only = table.row_count() >= *cached_len;
                    if is_append_only && next_end_limit >= *end_limit {
                        if next_end_limit == *end_limit {
                            // No new rows inside the request range; just bump the revision.
                            *r = data_rev;
                            *cached_len = table.row_count();
                            self.cursor += 1;
                            continue;
                        }

                        *entry = DataViewEntry::Building {
                            data_rev,
                            row_count: table.row_count(),
                            next: *end_limit,
                            end: requested_end,
                            indices: indices.to_vec(),
                        };
                    } else {
                        *entry = DataViewEntry::Building {
                            data_rev,
                            row_count: table.row_count(),
                            next: start as usize,
                            end: requested_end,
                            indices: Vec::new(),
                        };
                    }
                }
                DataViewEntry::Building {
                    data_rev: r,
                    row_count: cached_len,
                    next,
                    indices,
                    ..
                } => {
                    if *r != data_rev {
                        let start = match key.kind {
                            DataViewKind::XFilter { start, .. }
                            | DataViewKind::XYWeakFilter { start, .. } => start,
                            DataViewKind::XYWeakFilterBand { start, .. } => start,
                        };
                        let is_append_only = table.row_count() >= *cached_len;
                        let can_resume = is_append_only && *next <= *cached_len;

                        *r = data_rev;
                        *cached_len = table.row_count();
                        if !can_resume {
                            *next = start as usize;
                            indices.clear();
                        }
                    }
                }
            }

            let DataViewEntry::Building {
                next, end, indices, ..
            } = entry
            else {
                self.cursor += 1;
                continue;
            };

            let points_budget = budget.take_points(4096) as usize;
            if points_budget == 0 {
                return false;
            }

            points_consumed += points_budget;

            match key.kind {
                DataViewKind::XFilter {
                    x_col,
                    min_bits,
                    max_bits,
                    ..
                } => {
                    let Some(x_values) = table.column_f64(x_col) else {
                        self.cache.remove(&key);
                        self.cursor += 1;
                        continue;
                    };

                    let filter = AxisFilter1D {
                        min: (min_bits != u64::MAX).then(|| f64::from_bits(min_bits)),
                        max: (max_bits != u64::MAX).then(|| f64::from_bits(max_bits)),
                    };

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
                        *entry = DataViewEntry::Ready {
                            data_rev,
                            row_count: table.row_count(),
                            end_limit,
                            indices: frozen,
                        };
                        self.cursor += 1;
                    }
                }
                DataViewKind::XYWeakFilter {
                    x_col,
                    y_col,
                    x_min_bits,
                    x_max_bits,
                    y_min_bits,
                    y_max_bits,
                    ..
                } => {
                    let Some(x_values) = table.column_f64(x_col) else {
                        self.cache.remove(&key);
                        self.cursor += 1;
                        continue;
                    };
                    let Some(y_values) = table.column_f64(y_col) else {
                        self.cache.remove(&key);
                        self.cursor += 1;
                        continue;
                    };

                    let x_filter = AxisFilter1D {
                        min: (x_min_bits != u64::MAX).then(|| f64::from_bits(x_min_bits)),
                        max: (x_max_bits != u64::MAX).then(|| f64::from_bits(x_max_bits)),
                    };
                    let y_filter = AxisFilter1D {
                        min: (y_min_bits != u64::MAX).then(|| f64::from_bits(y_min_bits)),
                        max: (y_max_bits != u64::MAX).then(|| f64::from_bits(y_max_bits)),
                    };

                    #[derive(Clone, Copy, PartialEq, Eq)]
                    enum Side {
                        Below,
                        Inside,
                        Above,
                    }

                    fn side(filter: AxisFilter1D, v: f64) -> Side {
                        if let Some(min) = filter.min
                            && v < min
                        {
                            return Side::Below;
                        }
                        if let Some(max) = filter.max
                            && v > max
                        {
                            return Side::Above;
                        }
                        Side::Inside
                    }

                    let len = x_values.len().min(y_values.len());
                    let end_limit = (*end).min(len);
                    if *next > end_limit {
                        *next = end_limit;
                    }
                    let chunk_end = (*next + points_budget).min(end_limit);

                    for i in *next..chunk_end {
                        let xi = x_values.get(i).copied().unwrap_or(f64::NAN);
                        let yi = y_values.get(i).copied().unwrap_or(f64::NAN);
                        if !xi.is_finite() || !yi.is_finite() {
                            continue;
                        }

                        // ECharts `weakFilter` (XY subset): filter only when both dims are
                        // out-of-window on the same side.
                        let sx = side(x_filter, xi);
                        let sy = side(y_filter, yi);
                        if matches!(
                            (sx, sy),
                            (Side::Below, Side::Below) | (Side::Above, Side::Above)
                        ) {
                            continue;
                        }

                        if i <= u32::MAX as usize {
                            indices.push(i as u32);
                        }
                    }

                    *next = chunk_end;

                    if *next >= end_limit {
                        let frozen: Arc<[u32]> = std::mem::take(indices).into();
                        *entry = DataViewEntry::Ready {
                            data_rev,
                            row_count: table.row_count(),
                            end_limit,
                            indices: frozen,
                        };
                        self.cursor += 1;
                    }
                }
                DataViewKind::XYWeakFilterBand {
                    x_col,
                    y0_col,
                    y1_col,
                    x_min_bits,
                    x_max_bits,
                    y_min_bits,
                    y_max_bits,
                    ..
                } => {
                    let Some(x_values) = table.column_f64(x_col) else {
                        self.cache.remove(&key);
                        self.cursor += 1;
                        continue;
                    };
                    let Some(y0_values) = table.column_f64(y0_col) else {
                        self.cache.remove(&key);
                        self.cursor += 1;
                        continue;
                    };
                    let Some(y1_values) = table.column_f64(y1_col) else {
                        self.cache.remove(&key);
                        self.cursor += 1;
                        continue;
                    };

                    let x_filter = AxisFilter1D {
                        min: (x_min_bits != u64::MAX).then(|| f64::from_bits(x_min_bits)),
                        max: (x_max_bits != u64::MAX).then(|| f64::from_bits(x_max_bits)),
                    };
                    let y_filter = AxisFilter1D {
                        min: (y_min_bits != u64::MAX).then(|| f64::from_bits(y_min_bits)),
                        max: (y_max_bits != u64::MAX).then(|| f64::from_bits(y_max_bits)),
                    };

                    #[derive(Clone, Copy, PartialEq, Eq)]
                    enum Side {
                        Below,
                        Inside,
                        Above,
                    }

                    fn side(filter: AxisFilter1D, v: f64) -> Side {
                        if let Some(min) = filter.min
                            && v < min
                        {
                            return Side::Below;
                        }
                        if let Some(max) = filter.max
                            && v > max
                        {
                            return Side::Above;
                        }
                        Side::Inside
                    }

                    fn side_interval(filter: AxisFilter1D, a: f64, b: f64) -> Side {
                        let lo = a.min(b);
                        let hi = a.max(b);
                        if let Some(min) = filter.min
                            && hi < min
                        {
                            return Side::Below;
                        }
                        if let Some(max) = filter.max
                            && lo > max
                        {
                            return Side::Above;
                        }
                        Side::Inside
                    }

                    let len = x_values.len().min(y0_values.len()).min(y1_values.len());
                    let end_limit = (*end).min(len);
                    if *next > end_limit {
                        *next = end_limit;
                    }
                    let chunk_end = (*next + points_budget).min(end_limit);

                    for i in *next..chunk_end {
                        let xi = x_values.get(i).copied().unwrap_or(f64::NAN);
                        let y0 = y0_values.get(i).copied().unwrap_or(f64::NAN);
                        let y1 = y1_values.get(i).copied().unwrap_or(f64::NAN);
                        if !xi.is_finite() || !y0.is_finite() || !y1.is_finite() {
                            continue;
                        }

                        // ECharts `weakFilter` (XY subset, band interval): treat the Y dimension as an
                        // interval and filter only when both dims are out-of-window on the same side.
                        let sx = side(x_filter, xi);
                        let sy = side_interval(y_filter, y0, y1);
                        if matches!(
                            (sx, sy),
                            (Side::Below, Side::Below) | (Side::Above, Side::Above)
                        ) {
                            continue;
                        }

                        if i <= u32::MAX as usize {
                            indices.push(i as u32);
                        }
                    }

                    *next = chunk_end;

                    if *next >= end_limit {
                        let frozen: Arc<[u32]> = std::mem::take(indices).into();
                        *entry = DataViewEntry::Ready {
                            data_rev,
                            row_count: table.row_count(),
                            end_limit,
                            indices: frozen,
                        };
                        self.cursor += 1;
                    }
                }
            }

            if points_consumed >= max_points_per_step {
                return self.cursor >= self.requested.len();
            }
        }

        true
    }

    pub fn selection_for(
        &self,
        dataset: DatasetId,
        root_dataset: DatasetId,
        x_col: usize,
        selection_range: RowRange,
        filter: AxisFilter1D,
        table_rev: Revision,
    ) -> Option<RowSelection> {
        let key = DataViewKey::x_filter(dataset, root_dataset, x_col, selection_range, filter);
        match self.cache.get(&key) {
            Some(DataViewEntry::Ready {
                data_rev, indices, ..
            }) if *data_rev == table_rev => Some(RowSelection::Indices(indices.clone())),
            _ => None,
        }
    }

    pub fn selection_for_xy_weak_filter(
        &self,
        dataset: DatasetId,
        root_dataset: DatasetId,
        x_col: usize,
        y_col: usize,
        selection_range: RowRange,
        x_filter: AxisFilter1D,
        y_filter: AxisFilter1D,
        table_rev: Revision,
    ) -> Option<RowSelection> {
        let key = DataViewKey::xy_weak_filter(
            dataset,
            root_dataset,
            x_col,
            y_col,
            selection_range,
            x_filter,
            y_filter,
        );
        match self.cache.get(&key) {
            Some(DataViewEntry::Ready {
                data_rev, indices, ..
            }) if *data_rev == table_rev => Some(RowSelection::Indices(indices.clone())),
            _ => None,
        }
    }

    pub fn selection_for_xy_weak_filter_band(
        &self,
        dataset: DatasetId,
        root_dataset: DatasetId,
        x_col: usize,
        y0_col: usize,
        y1_col: usize,
        selection_range: RowRange,
        x_filter: AxisFilter1D,
        y_filter: AxisFilter1D,
        table_rev: Revision,
    ) -> Option<RowSelection> {
        let key = DataViewKey::xy_weak_filter_band(
            dataset,
            root_dataset,
            x_col,
            y0_col,
            y1_col,
            selection_range,
            x_filter,
            y_filter,
        );
        match self.cache.get(&key) {
            Some(DataViewEntry::Ready {
                data_rev, indices, ..
            }) if *data_rev == table_rev => Some(RowSelection::Indices(indices.clone())),
            _ => None,
        }
    }

    pub fn table_view_for<'a>(
        &self,
        table: &'a DataTable,
        dataset: DatasetId,
        root_dataset: DatasetId,
        x_col: usize,
        selection_range: RowRange,
        filter: AxisFilter1D,
        base_selection: RowSelection,
    ) -> DataTableView<'a> {
        let selection = self
            .selection_for(
                dataset,
                root_dataset,
                x_col,
                selection_range,
                filter,
                table.revision(),
            )
            .unwrap_or(base_selection);
        DataTableView::new(table, selection)
    }
}

#[cfg(test)]
mod tests {
    use crate::data::{Column, DataTable, DatasetStore};
    use crate::engine::window_policy::AxisFilter1D;
    use crate::ids::DatasetId;
    use crate::scheduler::WorkBudget;
    use crate::transform::RowRange;

    use super::*;

    fn build_stage_with_single_key(
        dataset_id: DatasetId,
        root_dataset_id: DatasetId,
        x_col: usize,
        range: RowRange,
        filter: AxisFilter1D,
        data_rev: Revision,
        row_count: usize,
    ) -> DataViewStage {
        let key = DataViewKey::x_filter(dataset_id, root_dataset_id, x_col, range, filter);
        let mut stage = DataViewStage::default();
        stage.requested.push(key);
        stage.requested_set.insert(key);
        let DataViewKind::XFilter { start, end, .. } = key.kind else {
            panic!("expected x_filter key");
        };
        stage.cache.insert(
            key,
            DataViewEntry::Building {
                data_rev,
                row_count,
                next: start as usize,
                end: end as usize,
                indices: Vec::new(),
            },
        );
        stage
    }

    fn build_stage_with_single_xy_key(
        dataset_id: DatasetId,
        root_dataset_id: DatasetId,
        x_col: usize,
        y_col: usize,
        range: RowRange,
        x_filter: AxisFilter1D,
        y_filter: AxisFilter1D,
        data_rev: Revision,
        row_count: usize,
    ) -> DataViewStage {
        let key = DataViewKey::xy_weak_filter(
            dataset_id,
            root_dataset_id,
            x_col,
            y_col,
            range,
            x_filter,
            y_filter,
        );
        let mut stage = DataViewStage::default();
        stage.requested.push(key);
        stage.requested_set.insert(key);
        let DataViewKind::XYWeakFilter { start, end, .. } = key.kind else {
            panic!("expected xy_weak_filter key");
        };
        stage.cache.insert(
            key,
            DataViewEntry::Building {
                data_rev,
                row_count,
                next: start as usize,
                end: end as usize,
                indices: Vec::new(),
            },
        );
        stage
    }

    fn build_stage_with_single_xy_band_key(
        dataset_id: DatasetId,
        root_dataset_id: DatasetId,
        x_col: usize,
        y0_col: usize,
        y1_col: usize,
        range: RowRange,
        x_filter: AxisFilter1D,
        y_filter: AxisFilter1D,
        data_rev: Revision,
        row_count: usize,
    ) -> DataViewStage {
        let key = DataViewKey::xy_weak_filter_band(
            dataset_id,
            root_dataset_id,
            x_col,
            y0_col,
            y1_col,
            range,
            x_filter,
            y_filter,
        );
        let mut stage = DataViewStage::default();
        stage.requested.push(key);
        stage.requested_set.insert(key);
        let DataViewKind::XYWeakFilterBand { start, end, .. } = key.kind else {
            panic!("expected xy_weak_filter_band key");
        };
        stage.cache.insert(
            key,
            DataViewEntry::Building {
                data_rev,
                row_count,
                next: start as usize,
                end: end as usize,
                indices: Vec::new(),
            },
        );
        stage
    }

    #[test]
    fn data_view_stage_builds_indices_incrementally_and_exposes_row_selection() {
        let dataset_id = DatasetId::new(1);

        let mut store = DatasetStore::default();
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![0.0, 10.0, 5.0, 7.0, 2.0]));
        store.insert(dataset_id, table.clone());

        let filter = AxisFilter1D {
            min: Some(4.0),
            max: Some(8.0),
        };

        let range = RowRange { start: 0, end: 100 };

        let mut stage = build_stage_with_single_key(
            dataset_id,
            dataset_id,
            0,
            range,
            filter,
            table.revision(),
            table.row_count(),
        );

        let mut budget = WorkBudget::new(1, 0, 0);
        assert!(!stage.step(&store, &mut budget));

        let mut budget = WorkBudget::new(4096, 0, 0);
        assert!(stage.step(&store, &mut budget));

        let sel = stage
            .selection_for(dataset_id, dataset_id, 0, range, filter, table.revision())
            .expect("selection should be ready");

        let RowSelection::Indices(indices) = sel else {
            panic!("expected indices selection");
        };

        // Values in [4,8] are 5.0 (idx 2) and 7.0 (idx 3).
        assert_eq!(&indices[..], &[2u32, 3u32]);
    }

    #[test]
    fn data_view_stage_builds_xy_weakfilter_indices_incrementally_and_exposes_row_selection() {
        let dataset_id = DatasetId::new(1);

        let mut store = DatasetStore::default();
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![-1.0, -1.0, 5.0, 12.0, 12.0]));
        table.push_column(Column::F64(vec![5.0, -2.0, -2.0, 12.0, 5.0]));
        store.insert(dataset_id, table.clone());

        let x_filter = AxisFilter1D {
            min: Some(0.0),
            max: Some(10.0),
        };
        let y_filter = AxisFilter1D {
            min: Some(0.0),
            max: Some(10.0),
        };

        let range = RowRange { start: 0, end: 100 };

        let mut stage = build_stage_with_single_xy_key(
            dataset_id,
            dataset_id,
            0,
            1,
            range,
            x_filter,
            y_filter,
            table.revision(),
            table.row_count(),
        );

        let mut budget = WorkBudget::new(1, 0, 0);
        assert!(!stage.step(&store, &mut budget));

        let mut budget = WorkBudget::new(4096, 0, 0);
        assert!(stage.step(&store, &mut budget));

        let sel = stage
            .selection_for_xy_weak_filter(
                dataset_id,
                dataset_id,
                0,
                1,
                range,
                x_filter,
                y_filter,
                table.revision(),
            )
            .expect("selection should be ready");

        let RowSelection::Indices(indices) = sel else {
            panic!("expected indices selection");
        };

        // Filter only when both dims are out-of-window on the same side:
        // - idx0: x below, y inside -> keep
        // - idx1: x below, y below -> drop
        // - idx2: x inside, y below -> keep
        // - idx3: x above, y above -> drop
        // - idx4: x above, y inside -> keep
        assert_eq!(&indices[..], &[0u32, 2u32, 4u32]);
    }

    #[test]
    fn data_view_stage_builds_xy_weakfilter_band_indices_incrementally_and_exposes_row_selection() {
        let dataset_id = DatasetId::new(1);

        let mut store = DatasetStore::default();
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![-1.0, -1.0, 5.0, 12.0, 12.0]));
        table.push_column(Column::F64(vec![5.0, -2.0, -2.0, 12.0, 5.0]));
        table.push_column(Column::F64(vec![6.0, -1.0, -1.0, 13.0, 6.0]));
        store.insert(dataset_id, table.clone());

        let x_filter = AxisFilter1D {
            min: Some(0.0),
            max: Some(10.0),
        };
        let y_filter = AxisFilter1D {
            min: Some(0.0),
            max: Some(10.0),
        };

        let range = RowRange { start: 0, end: 100 };

        let mut stage = build_stage_with_single_xy_band_key(
            dataset_id,
            dataset_id,
            0,
            1,
            2,
            range,
            x_filter,
            y_filter,
            table.revision(),
            table.row_count(),
        );

        let mut budget = WorkBudget::new(1, 0, 0);
        assert!(!stage.step(&store, &mut budget));

        let mut budget = WorkBudget::new(4096, 0, 0);
        assert!(stage.step(&store, &mut budget));

        let sel = stage
            .selection_for_xy_weak_filter_band(
                dataset_id,
                dataset_id,
                0,
                1,
                2,
                range,
                x_filter,
                y_filter,
                table.revision(),
            )
            .expect("selection should be ready");

        let RowSelection::Indices(indices) = sel else {
            panic!("expected indices selection");
        };

        // Same-side drop rule, but treat Y as an interval:
        // - idx0: x below, y interval inside -> keep
        // - idx1: x below, y interval below  -> drop
        // - idx2: x inside, y interval below -> keep
        // - idx3: x above, y interval above  -> drop
        // - idx4: x above, y interval inside -> keep
        assert_eq!(&indices[..], &[0u32, 2u32, 4u32]);
    }

    #[test]
    fn data_view_stage_invalidates_indices_on_data_revision_change() {
        let dataset_id = DatasetId::new(1);

        let mut store = DatasetStore::default();
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![0.0, 10.0, 5.0, 7.0, 2.0]));
        store.insert(dataset_id, table);

        let filter = AxisFilter1D {
            min: Some(4.0),
            max: Some(8.0),
        };
        let range = RowRange { start: 0, end: 100 };

        let data_rev = store.dataset(dataset_id).unwrap().revision();
        let mut stage = build_stage_with_single_key(
            dataset_id,
            dataset_id,
            0,
            range,
            filter,
            data_rev,
            store.dataset(dataset_id).unwrap().row_count(),
        );

        let mut budget = WorkBudget::new(4096, 0, 0);
        assert!(stage.step(&store, &mut budget));

        let sel = stage
            .selection_for(dataset_id, dataset_id, 0, range, filter, data_rev)
            .expect("selection should be ready");
        assert!(matches!(sel, RowSelection::Indices(_)));

        let old_rev = data_rev;
        let table = store.dataset_mut(dataset_id).unwrap();
        table.append_row_f64(&[6.0]).unwrap();
        let new_rev = table.revision();
        assert_ne!(old_rev, new_rev);

        // The cached selection should be considered stale when queried with the new data revision.
        assert!(
            stage
                .selection_for(dataset_id, dataset_id, 0, range, filter, new_rev)
                .is_none()
        );

        // A step should rebuild the selection for the same key, now including the appended row.
        stage.cursor = 0;
        let mut budget = WorkBudget::new(1, 0, 0);
        assert!(stage.step(&store, &mut budget));

        let sel = stage
            .selection_for(dataset_id, dataset_id, 0, range, filter, new_rev)
            .expect("selection should be rebuilt");
        let RowSelection::Indices(indices) = sel else {
            panic!("expected indices selection");
        };

        assert_eq!(&indices[..], &[2u32, 3u32, 5u32]);
    }

    #[test]
    fn data_view_stage_invalidates_xy_weakfilter_indices_on_data_revision_change() {
        let dataset_id = DatasetId::new(1);

        let mut store = DatasetStore::default();
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![-1.0, -1.0, 5.0, 12.0, 12.0]));
        table.push_column(Column::F64(vec![5.0, -2.0, -2.0, 12.0, 5.0]));
        store.insert(dataset_id, table);

        let x_filter = AxisFilter1D {
            min: Some(0.0),
            max: Some(10.0),
        };
        let y_filter = AxisFilter1D {
            min: Some(0.0),
            max: Some(10.0),
        };
        let range = RowRange { start: 0, end: 100 };

        let data_rev = store.dataset(dataset_id).unwrap().revision();
        let mut stage = build_stage_with_single_xy_key(
            dataset_id,
            dataset_id,
            0,
            1,
            range,
            x_filter,
            y_filter,
            data_rev,
            store.dataset(dataset_id).unwrap().row_count(),
        );

        let mut budget = WorkBudget::new(4096, 0, 0);
        assert!(stage.step(&store, &mut budget));

        let sel = stage
            .selection_for_xy_weak_filter(
                dataset_id, dataset_id, 0, 1, range, x_filter, y_filter, data_rev,
            )
            .expect("selection should be ready");
        assert!(matches!(sel, RowSelection::Indices(_)));

        let old_rev = data_rev;
        let table = store.dataset_mut(dataset_id).unwrap();
        table.append_row_f64(&[12.0, 5.0]).unwrap();
        let new_rev = table.revision();
        assert_ne!(old_rev, new_rev);

        // The cached selection should be considered stale when queried with the new data revision.
        assert!(
            stage
                .selection_for_xy_weak_filter(
                    dataset_id, dataset_id, 0, 1, range, x_filter, y_filter, new_rev,
                )
                .is_none()
        );

        // A step should rebuild the selection for the same key, now including the appended row.
        stage.cursor = 0;
        let mut budget = WorkBudget::new(1, 0, 0);
        assert!(stage.step(&store, &mut budget));

        let sel = stage
            .selection_for_xy_weak_filter(
                dataset_id, dataset_id, 0, 1, range, x_filter, y_filter, new_rev,
            )
            .expect("selection should be rebuilt");
        let RowSelection::Indices(indices) = sel else {
            panic!("expected indices selection");
        };

        assert_eq!(&indices[..], &[0u32, 2u32, 4u32, 5u32]);
    }

    #[test]
    fn data_view_stage_resumes_scans_on_append_only_changes() {
        let dataset_id = DatasetId::new(1);

        let mut store = DatasetStore::default();
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![0.0, 10.0, 5.0, 7.0, 2.0]));
        store.insert(dataset_id, table);

        let filter = AxisFilter1D {
            min: Some(4.0),
            max: Some(8.0),
        };
        let range = RowRange { start: 0, end: 100 };

        let rev0 = store.dataset(dataset_id).unwrap().revision();
        let mut stage = build_stage_with_single_key(
            dataset_id,
            dataset_id,
            0,
            range,
            filter,
            rev0,
            store.dataset(dataset_id).unwrap().row_count(),
        );

        let mut budget = WorkBudget::new(4096, 0, 0);
        assert!(stage.step(&store, &mut budget));

        let table = store.dataset_mut(dataset_id).unwrap();
        table.append_row_f64(&[6.0]).unwrap();
        let rev1 = table.revision();

        stage.cursor = 0;
        let mut budget = WorkBudget::new(1, 0, 0);
        assert!(
            stage.step(&store, &mut budget),
            "append-only should require scanning only the appended rows"
        );

        let sel = stage
            .selection_for(dataset_id, dataset_id, 0, range, filter, rev1)
            .expect("selection should be updated for the new revision");
        let RowSelection::Indices(indices) = sel else {
            panic!("expected indices selection");
        };
        assert_eq!(&indices[..], &[2u32, 3u32, 5u32]);
    }

    #[test]
    fn data_view_stage_resumes_xy_weakfilter_scans_on_append_only_changes() {
        let dataset_id = DatasetId::new(1);

        let mut store = DatasetStore::default();
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![-1.0, -1.0, 5.0, 12.0, 12.0]));
        table.push_column(Column::F64(vec![5.0, -2.0, -2.0, 12.0, 5.0]));
        store.insert(dataset_id, table);

        let x_filter = AxisFilter1D {
            min: Some(0.0),
            max: Some(10.0),
        };
        let y_filter = AxisFilter1D {
            min: Some(0.0),
            max: Some(10.0),
        };
        let range = RowRange { start: 0, end: 100 };

        let rev0 = store.dataset(dataset_id).unwrap().revision();
        let mut stage = build_stage_with_single_xy_key(
            dataset_id,
            dataset_id,
            0,
            1,
            range,
            x_filter,
            y_filter,
            rev0,
            store.dataset(dataset_id).unwrap().row_count(),
        );

        let mut budget = WorkBudget::new(4096, 0, 0);
        assert!(stage.step(&store, &mut budget));

        let table = store.dataset_mut(dataset_id).unwrap();
        table.append_row_f64(&[12.0, 5.0]).unwrap();
        let rev1 = table.revision();

        stage.cursor = 0;
        let mut budget = WorkBudget::new(1, 0, 0);
        assert!(
            stage.step(&store, &mut budget),
            "append-only should require scanning only the appended rows"
        );

        let sel = stage
            .selection_for_xy_weak_filter(
                dataset_id, dataset_id, 0, 1, range, x_filter, y_filter, rev1,
            )
            .expect("selection should be updated for the new revision");
        let RowSelection::Indices(indices) = sel else {
            panic!("expected indices selection");
        };
        assert_eq!(&indices[..], &[0u32, 2u32, 4u32, 5u32]);
    }
}
