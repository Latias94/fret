use crate::data::{DataTable, DatasetStore};
use crate::engine::ChartState;
use crate::engine::model::ChartModel;
use crate::ids::{AxisId, DatasetId, Revision, SeriesId};
use crate::spec::FilterMode;
use crate::transform::{RowRange, RowSelection, SeriesXPolicy, data_zoom_x_node, data_zoom_y_node};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

fn row_selection_for_xy_weak_filter(
    x_values: &[f64],
    y_values: &[f64],
    base_range: RowRange,
    x_filter: crate::engine::window_policy::AxisFilter1D,
    y_filter: crate::engine::window_policy::AxisFilter1D,
    max_view_len: usize,
) -> Option<RowSelection> {
    let mut base_range = base_range;
    let len = x_values.len().min(y_values.len());
    base_range.clamp_to_len(len);
    if base_range.is_empty() {
        return Some(RowSelection::Indices(std::sync::Arc::from([])));
    }

    let view_len = base_range.end.saturating_sub(base_range.start);
    if view_len > max_view_len {
        return None;
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Side {
        Below,
        Inside,
        Above,
    }

    fn side(filter: crate::engine::window_policy::AxisFilter1D, v: f64) -> Side {
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

    let mut kept: Vec<u32> = Vec::new();
    kept.reserve(view_len.min(4096));

    let mut kept_count = 0usize;
    for raw in base_range.start..base_range.end {
        let xi = x_values.get(raw).copied().unwrap_or(f64::NAN);
        let yi = y_values.get(raw).copied().unwrap_or(f64::NAN);
        if !xi.is_finite() || !yi.is_finite() {
            continue;
        }

        // ECharts `weakFilter`: filter only if all relevant dimensions are out of the window
        // on the same side (all below or all above). For XY, that means dropping only:
        // - (x below, y below)
        // - (x above, y above)
        let sx = side(x_filter, xi);
        let sy = side(y_filter, yi);
        let keep = !matches!((sx, sy), (Side::Below, Side::Below) | (Side::Above, Side::Above));
        if keep {
            kept.push(raw.min(u32::MAX as usize) as u32);
            kept_count += 1;
        }
    }

    if kept_count == view_len {
        return None;
    }
    Some(RowSelection::Indices(kept.into()))
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
            if series.stack.is_none()
                && matches!(
                    series.kind,
                    crate::spec::SeriesKind::Scatter
                        | crate::spec::SeriesKind::Line
                        | crate::spec::SeriesKind::Area
                )
                && matches!(
                    selection,
                    RowSelection::All | RowSelection::Range(_) | RowSelection::Indices(_)
                )
            {
                let y_filter_mode = model
                    .data_zoom_y_by_axis
                    .get(&series.y_axis)
                    .and_then(|id| model.data_zoom_y.get(id))
                    .map(|z| z.filter_mode)
                    .unwrap_or(FilterMode::None);
                if y_filter_mode != FilterMode::None {
                    if let Some(y_col) = dataset.fields.get(&series.encode.y).copied()
                        && let Some(y) = table.column_f64(y_col)
                    {
                        let y_axis_range = model
                            .axes
                            .get(&series.y_axis)
                            .map(|a| a.range)
                            .unwrap_or_default();
                        let node = data_zoom_y_node(model, state, series.y_axis, y_axis_range);
                        const MAX_VIEW_LEN: usize = 200_000;

                        if x_filter_mode == FilterMode::WeakFilter
                            && y_filter_mode == FilterMode::WeakFilter
                        {
                            let x_filter = x_node.x_filter();
                            let y_filter = node.y_filter();
                            let multi_active = (x_filter.min.is_some() || x_filter.max.is_some())
                                && (y_filter.min.is_some() || y_filter.max.is_some());
                            if multi_active {
                                if let Some(next) = row_selection_for_xy_weak_filter(
                                    x,
                                    y,
                                    base_range,
                                    x_filter,
                                    y_filter,
                                    MAX_VIEW_LEN,
                                ) {
                                    selection = next;
                                    x_window.x_policy.filter = Default::default();
                                }
                            } else if let Some(next) =
                                node.apply_y_filter_indices(y, &selection, MAX_VIEW_LEN)
                            {
                                selection = next;
                            }
                        } else if let Some(next) =
                            node.apply_y_filter_indices(y, &selection, MAX_VIEW_LEN)
                        {
                            selection = next;
                        }
                    }
                }
            }

            self.series.push(SeriesView {
                series: *series_id,
                dataset: series.dataset,
                x_axis: series.x_axis,
                revision: self.revision,
                data_revision: table.revision,
                selection,
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
