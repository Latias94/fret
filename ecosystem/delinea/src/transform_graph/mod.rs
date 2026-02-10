//! Incremental transform graph scaffolding.
//!
//! This is a v1 stepping stone towards an ECharts-class processor pipeline:
//! - cached node outputs keyed by dataset/model/state signatures,
//! - derived columns and domain transforms as first-class nodes,
//! - and a single source of truth for domain/selection outputs.
//!
//! For now, we keep the surface intentionally small and migrate behavior incrementally.

mod data_view;
mod dataset_transform;
mod filter_plan;
mod filter_plan_output;
mod x_range;
mod y_indices;

pub use data_view::*;
pub use filter_plan::*;
pub use y_indices::*;

use crate::data::DatasetStore;
use crate::engine::ChartState;
use crate::engine::model::ChartModel;
use crate::engine::stages::FilterPlanOutput;
use crate::engine::window::DataWindow;
use crate::engine::window_policy::AxisFilter1D;
use crate::ids::{AxisId, Revision, SeriesId};
use crate::scheduler::WorkBudget;
use crate::spec::{AxisKind, AxisRange, FilterMode};
use crate::transform::RowRange;
use crate::transform::RowSelection;
use crate::view::ViewState;
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone)]
pub struct TransformGraph {
    x_extent_cache: BTreeMap<AxisId, CachedExtent>,
    y_percent_extents_cache: BTreeMap<crate::ids::GridId, CachedYExtents>,
    dataset_transform_stage: dataset_transform::DatasetTransformStage,
    data_views: DataViewStage,
    filter_plan_output: FilterPlanOutput,
    filter_plan_cache: Option<CachedFilterPlan>,
    filter_plan_output_cache: Option<filter_plan_output::CachedFilterPlanOutput>,
    x_range_cache: BTreeMap<SeriesId, x_range::CachedXRangeNode>,
    y_indices_cache: BTreeMap<SeriesId, y_indices::CachedYIndicesNode>,
}

#[derive(Debug, Default, Clone)]
struct CachedExtent {
    signature: u64,
    extent: Option<(f64, f64)>,
}

#[derive(Debug, Default, Clone)]
struct CachedYExtents {
    signature: u64,
    extents: BTreeMap<AxisId, (f64, f64)>,
}

#[derive(Debug, Clone)]
struct CachedFilterPlan {
    model_rev: Revision,
    plan: FilterPlan,
}

impl TransformGraph {
    pub fn clear(&mut self) {
        self.x_extent_cache.clear();
        self.y_percent_extents_cache.clear();
        self.dataset_transform_stage.clear();
        self.data_views = DataViewStage::default();
        self.filter_plan_output = FilterPlanOutput::default();
        self.filter_plan_cache = None;
        self.filter_plan_output_cache = None;
        self.x_range_cache.clear();
        self.y_indices_cache.clear();
    }

    pub fn data_views(&self) -> &DataViewStage {
        &self.data_views
    }

    pub fn data_views_mut(&mut self) -> &mut DataViewStage {
        &mut self.data_views
    }

    pub fn begin_frame(&mut self) {
        self.dataset_transform_stage.begin_frame();
        self.data_views.begin_frame();
    }

    pub fn filter_plan_output(&self) -> &FilterPlanOutput {
        &self.filter_plan_output
    }

    pub fn filter_plan(&mut self, model: &ChartModel) -> &FilterPlan {
        let model_rev = model.revs.spec;
        let needs_rebuild = match self.filter_plan_cache.as_ref() {
            Some(cached) => cached.model_rev != model_rev,
            None => true,
        };

        if needs_rebuild {
            let plan = build_filter_plan(model);
            self.filter_plan_cache = Some(CachedFilterPlan { model_rev, plan });
        }

        &self.filter_plan_cache.as_ref().expect("cache set").plan
    }

    pub fn apply_y_percent_for_grid(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        state: &mut ChartState,
        view: &mut ViewState,
        view_series_index: &BTreeMap<SeriesId, usize>,
        series: &[SeriesId],
        grid: crate::ids::GridId,
        y_percent_extents_by_grid: &mut BTreeMap<crate::ids::GridId, BTreeMap<AxisId, (f64, f64)>>,
    ) -> bool {
        let mut view_changed = false;

        let y_axes_in_grid = self.y_percent_extents_scoped_by_x_for_grid(
            model,
            datasets,
            state,
            view,
            view_series_index,
            grid,
            series,
        );

        if !y_axes_in_grid.is_empty() {
            y_percent_extents_by_grid.insert(grid, y_axes_in_grid.clone());
        }

        for (axis, extent) in y_axes_in_grid {
            let Some((start, end)) = state.axis_percent_windows.get(&axis).copied() else {
                continue;
            };

            let axis_range = model.axes.get(&axis).map(|a| a.range).unwrap_or_default();
            let Some(window) =
                TransformGraph::percent_range_to_value_window(extent, axis_range, start, end)
            else {
                continue;
            };

            let prev = state.data_window_y.get(&axis).copied();
            if prev != Some(window) {
                state.data_window_y.insert(axis, window);
            }

            // Ensure the view reflects the derived filter so downstream materialization uses the
            // correct contract within the same step.
            for series_id in series {
                let Some(series_model) = model.series.get(series_id) else {
                    continue;
                };
                if !series_model.visible || series_model.y_axis != axis {
                    continue;
                }

                let Some(series_view_index) = view_series_index.get(series_id).copied() else {
                    continue;
                };
                let series_view = &mut view.series[series_view_index];

                let mode = series_view.y_filter_mode;
                let next_filter =
                    crate::engine::window_policy::axis_filter_1d(axis_range, Some(window), mode);
                if series_view.y_filter != next_filter {
                    series_view.y_filter = next_filter;
                    view_changed = true;
                }
            }
        }

        view_changed
    }

    pub fn prepare_requests(&mut self, datasets: &DatasetStore) {
        self.dataset_transform_stage.prepare_requests();
        self.data_views.prepare_requests(datasets);
    }

    pub fn step(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        view: &ViewState,
        budget: &mut WorkBudget,
    ) -> bool {
        let dataset_done = self
            .dataset_transform_stage
            .step(model, datasets, view, budget);
        let views_done = self.data_views.step(datasets, budget);
        dataset_done && views_done
    }

    pub fn request_x_filter_for_series(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        view: &ViewState,
        series_id: crate::ids::SeriesId,
    ) -> bool {
        self.data_views
            .request_x_filter_for_series(model, datasets, view, series_id)
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
        self.data_views.request_xy_weak_filter_for_series(
            model,
            datasets,
            view,
            series_id,
            selection_range,
            x_filter,
            y_filter,
        )
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
        self.data_views.request_xy_weak_filter_band_for_series(
            model,
            datasets,
            view,
            series_id,
            selection_range,
            x_filter,
            y_filter,
        )
    }

    pub fn selection_for_x_filter(
        &self,
        dataset: crate::ids::DatasetId,
        root_dataset: crate::ids::DatasetId,
        x_col: usize,
        selection_range: RowRange,
        filter: AxisFilter1D,
        table_rev: Revision,
    ) -> Option<RowSelection> {
        self.data_views.selection_for(
            dataset,
            root_dataset,
            x_col,
            selection_range,
            filter,
            table_rev,
        )
    }

    pub fn selection_for_xy_weak_filter(
        &self,
        dataset: crate::ids::DatasetId,
        root_dataset: crate::ids::DatasetId,
        x_col: usize,
        y_col: usize,
        selection_range: RowRange,
        x_filter: AxisFilter1D,
        y_filter: AxisFilter1D,
        table_rev: Revision,
    ) -> Option<RowSelection> {
        self.data_views.selection_for_xy_weak_filter(
            dataset,
            root_dataset,
            x_col,
            y_col,
            selection_range,
            x_filter,
            y_filter,
            table_rev,
        )
    }

    pub fn selection_for_xy_weak_filter_band(
        &self,
        dataset: crate::ids::DatasetId,
        root_dataset: crate::ids::DatasetId,
        x_col: usize,
        y0_col: usize,
        y1_col: usize,
        selection_range: RowRange,
        x_filter: AxisFilter1D,
        y_filter: AxisFilter1D,
        table_rev: Revision,
    ) -> Option<RowSelection> {
        self.data_views.selection_for_xy_weak_filter_band(
            dataset,
            root_dataset,
            x_col,
            y0_col,
            y1_col,
            selection_range,
            x_filter,
            y_filter,
            table_rev,
        )
    }

    /// Returns a finite `(min, max)` data extent for the X axis based on visible series and the
    /// effective dataset row ranges.
    ///
    /// The result is cached using a metadata signature (dataset revisions + row ranges + bindings),
    /// so changing any of those inputs invalidates the cached extent.
    pub fn x_data_extent(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        state: &ChartState,
        axis: AxisId,
    ) -> Option<(f64, f64)> {
        let signature = x_extent_signature(model, datasets, state, axis);
        if let Some(cached) = self.x_extent_cache.get(&axis)
            && cached.signature == signature
        {
            return cached.extent;
        }

        let extent = scan_x_extent(model, datasets, state, axis);
        self.x_extent_cache
            .insert(axis, CachedExtent { signature, extent });
        extent
    }

    /// Maps an ECharts-style percent range (0..=100) into a value-space window, clamping to a valid
    /// non-degenerate window and applying axis range constraints (lock min/max).
    pub fn percent_range_to_value_window(
        extent: (f64, f64),
        axis_range: AxisRange,
        start: f64,
        end: f64,
    ) -> Option<DataWindow> {
        let (mut dmin, mut dmax) = extent;
        if dmin > dmax {
            core::mem::swap(&mut dmin, &mut dmax);
        }

        match axis_range {
            AxisRange::Fixed { min, max } => {
                dmin = min;
                dmax = max;
            }
            AxisRange::Auto | AxisRange::LockMin { .. } | AxisRange::LockMax { .. } => {
                if let Some(min) = axis_range.locked_min() {
                    dmin = min;
                }
                if let Some(max) = axis_range.locked_max() {
                    dmax = max;
                }
            }
        }
        if !dmin.is_finite() || !dmax.is_finite() {
            return None;
        }

        let span = dmax - dmin;
        if !span.is_finite() || span <= 0.0 {
            return None;
        }

        if !start.is_finite() || !end.is_finite() {
            return None;
        }

        let mut a = (start / 100.0).clamp(0.0, 1.0);
        let mut b = (end / 100.0).clamp(0.0, 1.0);
        if a > b {
            core::mem::swap(&mut a, &mut b);
        }

        let mut window = DataWindow {
            min: dmin + span * a,
            max: dmin + span * b,
        };
        window.clamp_non_degenerate();
        Some(window.apply_constraints(axis_range.locked_min(), axis_range.locked_max()))
    }

    /// Returns Y data extents for axes that currently have percent windows, scoped by the current
    /// view selection and (when active) the X filter predicate.
    ///
    /// This is the "order-sensitive percent domain" building block: Y percent extents must be
    /// derived after X has affected the visible selection/domain (ECharts `dataZoomProcessor`
    /// semantics; v1 cartesian subset).
    pub fn y_percent_extents_scoped_by_x_for_grid(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        state: &ChartState,
        view: &ViewState,
        view_series_index: &BTreeMap<SeriesId, usize>,
        grid: crate::ids::GridId,
        series: &[SeriesId],
    ) -> BTreeMap<AxisId, (f64, f64)> {
        let signature =
            y_percent_extents_signature(model, datasets, state, view, view_series_index, series);
        if let Some(cached) = self.y_percent_extents_cache.get(&grid)
            && cached.signature == signature
        {
            return cached.extents.clone();
        }

        let mut out: BTreeMap<AxisId, (f64, f64)> = BTreeMap::new();

        for series_id in series {
            let Some(series_model) = model.series.get(series_id) else {
                continue;
            };
            if !series_model.visible {
                continue;
            }

            let axis = series_model.y_axis;
            if !state.axis_percent_windows.contains_key(&axis) {
                continue;
            }
            if !model.axes.get(&axis).is_some_and(|a| a.kind == AxisKind::Y) {
                continue;
            }

            let Some(series_view_index) = view_series_index.get(series_id).copied() else {
                continue;
            };
            let series_view = &view.series[series_view_index];

            let x_filter_mode = series_view.x_filter_mode;
            let x_filter = series_view.x_policy.filter;
            let x_active = !matches!(x_filter_mode, FilterMode::None)
                && (x_filter.min.is_some() || x_filter.max.is_some());

            let Some(table) = datasets.dataset(model.root_dataset_id(series_model.dataset)) else {
                continue;
            };
            let Some(dataset) = model.datasets.get(&series_model.dataset) else {
                continue;
            };
            let Some(x_col) = dataset.fields.get(&series_model.encode.x).copied() else {
                continue;
            };
            let Some(y0_col) = dataset.fields.get(&series_model.encode.y).copied() else {
                continue;
            };
            let y1_col = series_model
                .encode
                .y2
                .and_then(|f| dataset.fields.get(&f).copied());

            let Some(x_values) = table.column_f64(x_col) else {
                continue;
            };
            let Some(y0_values) = table.column_f64(y0_col) else {
                continue;
            };
            let y1_values = y1_col.and_then(|c| table.column_f64(c));

            let len = table.row_count();
            let view_len = series_view.selection.view_len(len);
            if view_len == 0 {
                continue;
            }

            let mut min = f64::INFINITY;
            let mut max = f64::NEG_INFINITY;
            for view_index in 0..view_len {
                let Some(raw) = series_view.selection.get_raw_index(len, view_index) else {
                    continue;
                };

                if x_active {
                    let xv = x_values.get(raw).copied().unwrap_or(f64::NAN);
                    if !xv.is_finite() || !x_filter.contains(xv) {
                        continue;
                    }
                }

                let y0 = y0_values.get(raw).copied().unwrap_or(f64::NAN);
                if !y0.is_finite() {
                    continue;
                }
                if let Some(y1_values) = y1_values {
                    let y1 = y1_values.get(raw).copied().unwrap_or(f64::NAN);
                    if !y1.is_finite() {
                        continue;
                    }
                    min = min.min(y0.min(y1));
                    max = max.max(y0.max(y1));
                } else {
                    min = min.min(y0);
                    max = max.max(y0);
                }
            }

            if !min.is_finite() || !max.is_finite() {
                continue;
            }

            out.entry(axis)
                .and_modify(|ext| {
                    ext.0 = ext.0.min(min);
                    ext.1 = ext.1.max(max);
                })
                .or_insert((min, max));
        }

        self.y_percent_extents_cache.insert(
            grid,
            CachedYExtents {
                signature,
                extents: out.clone(),
            },
        );
        out
    }
}

const FNV1A_OFFSET: u64 = 0xcbf29ce484222325;
const FNV1A_PRIME: u64 = 0x00000100000001B3;

fn fnv1a_step(hash: u64, value: u64) -> u64 {
    (hash ^ value).wrapping_mul(FNV1A_PRIME)
}

fn rev_u64(rev: Revision) -> u64 {
    rev.0 as u64
}

fn hash_opt_f64(mut h: u64, v: Option<f64>) -> u64 {
    match v {
        Some(v) => {
            h = fnv1a_step(h, 1);
            fnv1a_step(h, v.to_bits())
        }
        None => fnv1a_step(h, 0),
    }
}

fn hash_selection(mut h: u64, sel: &crate::transform::RowSelection) -> u64 {
    match sel {
        crate::transform::RowSelection::All => fnv1a_step(h, 1),
        crate::transform::RowSelection::Range(r) => {
            h = fnv1a_step(h, 2);
            h = fnv1a_step(h, r.start as u64);
            fnv1a_step(h, r.end as u64)
        }
        crate::transform::RowSelection::Indices(indices) => {
            h = fnv1a_step(h, 3);
            h = fnv1a_step(h, indices.len() as u64);
            let ptr = indices.as_ref().as_ptr() as usize as u64;
            fnv1a_step(h, ptr)
        }
    }
}

fn y_percent_extents_signature(
    model: &ChartModel,
    datasets: &DatasetStore,
    state: &ChartState,
    view: &ViewState,
    view_series_index: &BTreeMap<SeriesId, usize>,
    series: &[SeriesId],
) -> u64 {
    let mut h = FNV1A_OFFSET;
    h = fnv1a_step(h, model.revs.spec.0 as u64);
    h = fnv1a_step(h, view.revision.0 as u64);

    for series_id in series {
        let Some(series_model) = model.series.get(series_id) else {
            continue;
        };
        if !series_model.visible {
            continue;
        }

        h = fnv1a_step(h, series_id.0);
        h = fnv1a_step(h, series_model.dataset.0);

        let table_rev = datasets
            .dataset(series_model.dataset)
            .map(|t| rev_u64(t.revision()))
            .unwrap_or(0);
        h = fnv1a_step(h, table_rev);

        let axis = series_model.y_axis;
        h = fnv1a_step(h, axis.0);
        if let Some((a, b)) = state.axis_percent_windows.get(&axis) {
            h = fnv1a_step(h, 1);
            h = fnv1a_step(h, a.to_bits());
            h = fnv1a_step(h, b.to_bits());
        } else {
            h = fnv1a_step(h, 0);
        }

        let Some(series_view_index) = view_series_index.get(series_id).copied() else {
            continue;
        };
        let series_view = &view.series[series_view_index];
        h = hash_selection(h, &series_view.selection);
        h = fnv1a_step(h, series_view.x_filter_mode as u64);
        h = hash_opt_f64(h, series_view.x_policy.filter.min);
        h = hash_opt_f64(h, series_view.x_policy.filter.max);
    }

    h
}

fn x_extent_signature(
    model: &ChartModel,
    datasets: &DatasetStore,
    state: &ChartState,
    axis: AxisId,
) -> u64 {
    let mut h = FNV1A_OFFSET;
    h = fnv1a_step(h, axis.0);

    for series_id in &model.series_order {
        let Some(series) = model.series.get(series_id) else {
            continue;
        };
        if !series.visible || series.x_axis != axis {
            continue;
        }

        h = fnv1a_step(h, series_id.0);
        h = fnv1a_step(h, series.dataset.0);

        let table_rev = datasets
            .dataset(series.dataset)
            .map(|t| rev_u64(t.revision()))
            .unwrap_or(0);
        h = fnv1a_step(h, table_rev);

        let x_col = model
            .datasets
            .get(&series.dataset)
            .and_then(|ds| ds.fields.get(&series.encode.x).copied())
            .unwrap_or(usize::MAX);
        h = fnv1a_step(h, x_col as u64);

        let range = state.dataset_row_ranges.get(&series.dataset).copied();
        match range {
            Some(r) => {
                h = fnv1a_step(h, r.start as u64);
                h = fnv1a_step(h, r.end as u64);
            }
            None => {
                h = fnv1a_step(h, 0);
                h = fnv1a_step(h, u64::MAX);
            }
        }
    }

    h
}

fn scan_x_extent(
    model: &ChartModel,
    datasets: &DatasetStore,
    state: &ChartState,
    axis: AxisId,
) -> Option<(f64, f64)> {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut found = false;

    for series_id in &model.series_order {
        let Some(series) = model.series.get(series_id) else {
            continue;
        };
        if !series.visible || series.x_axis != axis {
            continue;
        }
        let Some(table) = datasets.dataset(model.root_dataset_id(series.dataset)) else {
            continue;
        };
        let Some(dataset) = model.datasets.get(&series.dataset) else {
            continue;
        };
        let Some(x_col) = dataset.fields.get(&series.encode.x).copied() else {
            continue;
        };
        let Some(x_values) = table.column_f64(x_col) else {
            continue;
        };

        let mut range = state
            .dataset_row_ranges
            .get(&series.dataset)
            .copied()
            .unwrap_or(crate::transform::RowRange {
                start: 0,
                end: table.row_count(),
            });
        range.clamp_to_len(table.row_count());

        for i in range.start..range.end {
            let v = x_values.get(i).copied().unwrap_or(f64::NAN);
            if !v.is_finite() {
                continue;
            }
            min = min.min(v);
            max = max.max(v);
            found = true;
        }
    }

    if found && min.is_finite() && max.is_finite() {
        Some((min, max))
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
