use crate::data::{DataTable, DatasetStore};
use crate::engine::ChartState;
use crate::engine::model::ChartModel;
use crate::engine::window_policy::AxisFilter1D;
use crate::ids::{AxisId, DatasetId, Revision, SeriesId};
use crate::spec::{FilterMode, SeriesKind};
use crate::transform::{RowRange, RowSelection, SeriesXPolicy, data_zoom_x_node, data_zoom_y_node};

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
    pub x_filter_mode: FilterMode,
    pub y_filter_mode: FilterMode,
    pub y_filter: AxisFilter1D,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SeriesEmptyMask {
    pub x_active: bool,
    pub x_filter: AxisFilter1D,
    pub y_active: bool,
    pub y_filter: AxisFilter1D,
    pub y_is_interval: bool,
}

impl SeriesEmptyMask {
    pub fn allows_axis_x_value(self, axis_x: f64) -> bool {
        if !self.x_active {
            return true;
        }
        axis_x.is_finite() && self.x_filter.contains(axis_x)
    }

    pub fn allows_raw_index(
        self,
        raw_index: usize,
        x: &[f64],
        y0: &[f64],
        y1: Option<&[f64]>,
    ) -> bool {
        if self.x_active {
            let x_raw = x.get(raw_index).copied().unwrap_or(f64::NAN);
            if !x_raw.is_finite() || !self.x_filter.contains(x_raw) {
                return false;
            }
        }

        if self.y_active {
            if self.y_is_interval {
                let Some(y1) = y1 else {
                    return false;
                };
                let y0_raw = y0.get(raw_index).copied().unwrap_or(f64::NAN);
                let y1_raw = y1.get(raw_index).copied().unwrap_or(f64::NAN);
                if !y0_raw.is_finite()
                    || !y1_raw.is_finite()
                    || !self.y_filter.intersects_interval(y0_raw, y1_raw)
                {
                    return false;
                }
            } else {
                let y_raw = y0.get(raw_index).copied().unwrap_or(f64::NAN);
                if !y_raw.is_finite() || !self.y_filter.contains(y_raw) {
                    return false;
                }
            }
        }

        true
    }
}

impl SeriesView {
    pub fn empty_mask(&self, kind: SeriesKind, stacked: bool) -> SeriesEmptyMask {
        let x_active = self.x_filter_mode == FilterMode::Empty
            && (self.x_policy.filter.min.is_some() || self.x_policy.filter.max.is_some());
        let y_active = self.y_filter_mode == FilterMode::Empty
            && !stacked
            && (self.y_filter.min.is_some() || self.y_filter.max.is_some())
            && matches!(
                kind,
                SeriesKind::Line
                    | SeriesKind::Area
                    | SeriesKind::Band
                    | SeriesKind::Scatter
                    | SeriesKind::Bar
            );

        SeriesEmptyMask {
            x_active,
            x_filter: self.x_policy.filter,
            y_active,
            y_filter: self.y_filter,
            y_is_interval: kind == SeriesKind::Band,
        }
    }
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
            let x_node = data_zoom_x_node(model, state, series.x_axis, x_axis_range);
            let x_filter_mode = x_node.filter_mode();
            let mut x_window = x_node.apply(x, base_range);

            let mut selection = x_window.selection;

            let y_filter_mode = model
                .data_zoom_y_by_axis
                .get(&series.y_axis)
                .and_then(|id| model.data_zoom_y.get(id))
                .map(|z| z.filter_mode)
                .unwrap_or(FilterMode::None);

            let y_axis_range = model
                .axes
                .get(&series.y_axis)
                .map(|a| a.range)
                .unwrap_or_default();
            let y_node = data_zoom_y_node(model, state, series.y_axis, y_axis_range);
            let y_filter = y_node.y_filter();

            const MAX_MULTI_DIM_WEAKFILTER_VIEW_LEN: usize = 200_000;
            let band_y1_ready = if series.kind == crate::spec::SeriesKind::Band {
                if let Some(y1_field) = series.encode.y2
                    && let Some(y1_col) = dataset.fields.get(&y1_field).copied()
                {
                    table.column_f64(y1_col).is_some()
                } else {
                    false
                }
            } else {
                true
            };

            let xy_weak_filter_active = series.stack.is_none()
                && matches!(
                    series.kind,
                    crate::spec::SeriesKind::Scatter
                        | crate::spec::SeriesKind::Line
                        | crate::spec::SeriesKind::Area
                        | crate::spec::SeriesKind::Band
                )
                && x_filter_mode == FilterMode::WeakFilter
                && y_filter_mode == FilterMode::WeakFilter
                && x_node.window().is_some()
                && state.data_window_y.get(&series.y_axis).is_some()
                && band_y1_ready
                && base_range.end.saturating_sub(base_range.start)
                    <= MAX_MULTI_DIM_WEAKFILTER_VIEW_LEN;

            if xy_weak_filter_active {
                selection = RowSelection::Range(base_range);
                x_window.x_policy.filter = Default::default();
            }

            self.series.push(SeriesView {
                series: *series_id,
                dataset: series.dataset,
                x_axis: series.x_axis,
                revision: self.revision,
                data_revision: table.revision,
                selection,
                x_policy: x_window.x_policy,
                x_filter_mode,
                y_filter_mode,
                y_filter,
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
