use super::DataViewStage;
use crate::data::DatasetStore;
use crate::engine::ChartState;
use crate::engine::model::ChartModel;
use crate::ids::{DatasetId, GridId, Revision, SeriesId};
use crate::spec::FilterMode;
use crate::transform::{RowSelection, SeriesXPolicy};
use crate::view::SeriesEmptyMask;
use crate::view::ViewState;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Debug, Default, Clone)]
pub struct FilterProcessorStage;

#[derive(Debug, Default, Clone)]
pub struct SeriesParticipation {
    pub series: SeriesId,
    pub dataset: DatasetId,
    pub revision: Revision,
    pub data_revision: Revision,
    pub selection: RowSelection,
    pub x_policy: SeriesXPolicy,
    pub x_filter_mode: FilterMode,
    pub y_filter_mode: FilterMode,
    pub y_filter: crate::engine::window_policy::AxisFilter1D,
    pub empty_mask: SeriesEmptyMask,
}

#[derive(Debug, Default, Clone)]
pub struct ParticipationState {
    pub revision: Revision,
    pub series: Vec<SeriesParticipation>,
    series_index: BTreeMap<SeriesId, usize>,
}

impl ParticipationState {
    pub fn clear(&mut self) {
        self.revision = Revision::default();
        self.series.clear();
        self.series_index.clear();
    }

    pub fn series_participation(&self, series: SeriesId) -> Option<&SeriesParticipation> {
        self.series_index
            .get(&series)
            .copied()
            .and_then(|i| self.series.get(i))
    }

    pub fn rebuild_from_view(&mut self, model: &ChartModel, view: &ViewState) {
        self.series.clear();
        self.series_index.clear();
        self.series.reserve(model.series_order.len());
        self.revision = view.revision;

        for (i, series_id) in model.series_order.iter().copied().enumerate() {
            self.series_index.insert(series_id, i);
            let Some(series_model) = model.series.get(&series_id) else {
                self.series.push(SeriesParticipation {
                    series: series_id,
                    ..Default::default()
                });
                continue;
            };

            if let Some(v) = view.series_view(series_id) {
                let empty_mask = v.empty_mask(series_model.kind, series_model.stack.is_some());
                self.series.push(SeriesParticipation {
                    series: series_id,
                    dataset: series_model.dataset,
                    revision: v.revision,
                    data_revision: v.data_revision,
                    selection: v.selection.clone(),
                    x_policy: v.x_policy,
                    x_filter_mode: v.x_filter_mode,
                    y_filter_mode: v.y_filter_mode,
                    y_filter: v.y_filter,
                    empty_mask,
                });
            } else {
                self.series.push(SeriesParticipation {
                    series: series_id,
                    dataset: series_model.dataset,
                    ..Default::default()
                });
            }
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FilterProcessorResult {
    pub xy_weak_filter_pending: bool,
}

#[derive(Debug, Default, Clone)]
struct GridFilterPlan {
    series: Vec<SeriesId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilterPlanStepKind {
    XYWeakFilter,
    XIndices,
    YIndices,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FilterPlanStep {
    grid: GridId,
    kind: FilterPlanStepKind,
}

impl FilterProcessorStage {
    pub fn request_data_views(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        state: &ChartState,
        view: &ViewState,
        data_view: &mut DataViewStage,
    ) {
        for series_id in &model.series_order {
            let Some(series) = model.series.get(series_id) else {
                continue;
            };

            let Some(zoom_state) = state.data_zoom_x.get(&series.x_axis) else {
                continue;
            };
            if zoom_state.window.is_none() {
                continue;
            }
            let filter_mode = zoom_state.filter_mode;

            // Multi-dimensional `weakFilter` (v1 subset): request an indices view when both X and Y
            // dataZoom are `WeakFilter` and the Y window is active. This is correctness-critical
            // for the subset because X-only range slicing cannot represent weakFilter semantics.
            if filter_mode == crate::spec::FilterMode::WeakFilter {
                let y_filter_mode = model
                    .data_zoom_y_by_axis
                    .get(&series.y_axis)
                    .and_then(|id| model.data_zoom_y.get(id))
                    .map(|z| z.filter_mode)
                    .unwrap_or(crate::spec::FilterMode::None);

                const MAX_MULTI_DIM_WEAKFILTER_VIEW_LEN: usize = 200_000;
                if y_filter_mode == crate::spec::FilterMode::WeakFilter
                    && state.data_window_y.get(&series.y_axis).is_some()
                    && series.stack.is_none()
                    && matches!(
                        series.kind,
                        crate::spec::SeriesKind::Scatter
                            | crate::spec::SeriesKind::Line
                            | crate::spec::SeriesKind::Area
                            | crate::spec::SeriesKind::Band
                    )
                {
                    let Some(series_view) = view.series_view(*series_id) else {
                        continue;
                    };
                    let Some(dataset_view) = view.dataset_view(series.dataset) else {
                        continue;
                    };

                    let base_range = dataset_view.row_range;
                    let base_len = base_range.end.saturating_sub(base_range.start);
                    if base_len <= MAX_MULTI_DIM_WEAKFILTER_VIEW_LEN
                        && matches!(
                            series_view.selection,
                            RowSelection::All | RowSelection::Range(_)
                        )
                    {
                        let x_axis_range = model
                            .axes
                            .get(&series.x_axis)
                            .map(|a| a.range)
                            .unwrap_or_default();
                        let y_axis_range = model
                            .axes
                            .get(&series.y_axis)
                            .map(|a| a.range)
                            .unwrap_or_default();

                        let x_filter = crate::engine::window_policy::axis_filter_1d(
                            x_axis_range,
                            zoom_state.window,
                            crate::spec::FilterMode::WeakFilter,
                        );
                        let y_filter = crate::engine::window_policy::axis_filter_1d(
                            y_axis_range,
                            state.data_window_y.get(&series.y_axis).copied(),
                            crate::spec::FilterMode::WeakFilter,
                        );

                        match series.kind {
                            crate::spec::SeriesKind::Band => {
                                let _ = data_view.request_xy_weak_filter_band_for_series(
                                    model, datasets, view, *series_id, base_range, x_filter,
                                    y_filter,
                                );
                            }
                            _ => {
                                let _ = data_view.request_xy_weak_filter_for_series(
                                    model, datasets, view, *series_id, base_range, x_filter,
                                    y_filter,
                                );
                            }
                        }
                    }
                }
            }

            // ADR 1150:
            // - `Filter` / `WeakFilter` may use indices views as an optimization carrier.
            // - `Empty` must preserve a stable row/index space (avoid indices-backed selections).
            if !matches!(
                filter_mode,
                crate::spec::FilterMode::Filter | crate::spec::FilterMode::WeakFilter
            ) {
                continue;
            }

            data_view.request_x_filter_for_series(model, datasets, view, *series_id);
        }
    }

    pub fn apply(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        state: &ChartState,
        view: &mut ViewState,
        data_views: &DataViewStage,
    ) -> FilterProcessorResult {
        let mut xy_weak_filter_pending = false;
        let mut view_changed = false;
        let mut x_indices_applied: BTreeSet<SeriesId> = BTreeSet::new();

        // ECharts `dataZoomProcessor` applies transforms in an order-sensitive way per grid (e.g. X
        // before Y). We currently only allow one dataZoom per axis, but structuring this stage by
        // grid provides a stable footing for a future general transform plan.
        let mut grid_plans: BTreeMap<GridId, GridFilterPlan> = BTreeMap::new();
        for series_id in &model.series_order {
            let Some(series_model) = model.series.get(series_id) else {
                continue;
            };
            let Some(axis) = model.axes.get(&series_model.x_axis) else {
                continue;
            };
            let grid = axis.grid;
            let plan = grid_plans
                .entry(grid)
                .or_insert_with(|| GridFilterPlan { series: Vec::new() });
            plan.series.push(*series_id);
        }

        let view_series_index: BTreeMap<SeriesId, usize> = view
            .series
            .iter()
            .enumerate()
            .map(|(i, v)| (v.series, i))
            .collect();

        let mut plan_steps: Vec<FilterPlanStep> = Vec::new();
        plan_steps.reserve(grid_plans.len() * 3);
        for grid in grid_plans.keys().copied() {
            plan_steps.push(FilterPlanStep {
                grid,
                kind: FilterPlanStepKind::XYWeakFilter,
            });
            plan_steps.push(FilterPlanStep {
                grid,
                kind: FilterPlanStepKind::XIndices,
            });
            plan_steps.push(FilterPlanStep {
                grid,
                kind: FilterPlanStepKind::YIndices,
            });
        }

        // Step ordering is intentionally explicit and per-grid (ECharts-style ordering scaffold).
        const MAX_MULTI_DIM_WEAKFILTER_VIEW_LEN: usize = 200_000;
        const MAX_Y_FILTER_VIEW_LEN: usize = 200_000;

        for step in &plan_steps {
            let Some(plan) = grid_plans.get(&step.grid) else {
                continue;
            };

            match step.kind {
                FilterPlanStepKind::XYWeakFilter => apply_xy_weak_filter_for_grid(
                    model,
                    datasets,
                    state,
                    view,
                    data_views,
                    &view_series_index,
                    &plan.series,
                    MAX_MULTI_DIM_WEAKFILTER_VIEW_LEN,
                    &mut xy_weak_filter_pending,
                    &mut view_changed,
                ),
                FilterPlanStepKind::XIndices => apply_x_indices_for_grid(
                    model,
                    datasets,
                    view,
                    data_views,
                    &view_series_index,
                    &plan.series,
                    &mut view_changed,
                    &mut x_indices_applied,
                ),
                FilterPlanStepKind::YIndices => apply_y_indices_for_grid(
                    model,
                    datasets,
                    state,
                    view,
                    &view_series_index,
                    &plan.series,
                    MAX_Y_FILTER_VIEW_LEN,
                    &x_indices_applied,
                    &mut view_changed,
                ),
            }
        }

        if view_changed {
            view.revision.bump();
            for series_view in &mut view.series {
                series_view.revision = view.revision;
            }
        }

        FilterProcessorResult {
            xy_weak_filter_pending,
        }
    }
}

fn apply_xy_weak_filter_for_grid(
    model: &ChartModel,
    datasets: &DatasetStore,
    state: &ChartState,
    view: &mut ViewState,
    data_views: &DataViewStage,
    view_series_index: &BTreeMap<SeriesId, usize>,
    series: &[SeriesId],
    max_view_len: usize,
    xy_weak_filter_pending: &mut bool,
    view_changed: &mut bool,
) {
    for series_id in series {
        let Some(series_model) = model.series.get(series_id) else {
            continue;
        };
        if !series_model.visible || series_model.stack.is_some() {
            continue;
        }
        if !matches!(
            series_model.kind,
            crate::spec::SeriesKind::Scatter
                | crate::spec::SeriesKind::Line
                | crate::spec::SeriesKind::Area
                | crate::spec::SeriesKind::Band
        ) {
            continue;
        }

        let Some(zoom_state) = state.data_zoom_x.get(&series_model.x_axis) else {
            continue;
        };
        if zoom_state.filter_mode != crate::spec::FilterMode::WeakFilter
            || zoom_state.window.is_none()
        {
            continue;
        }

        let y_filter_mode = model
            .data_zoom_y_by_axis
            .get(&series_model.y_axis)
            .and_then(|id| model.data_zoom_y.get(id))
            .map(|z| z.filter_mode)
            .unwrap_or(crate::spec::FilterMode::None);
        if y_filter_mode != crate::spec::FilterMode::WeakFilter {
            continue;
        }

        let Some(y_window) = state.data_window_y.get(&series_model.y_axis).copied() else {
            continue;
        };

        let Some(series_view_index) = view_series_index.get(series_id).copied() else {
            continue;
        };
        if !matches!(
            view.series[series_view_index].selection,
            RowSelection::All | RowSelection::Range(_) | RowSelection::Indices(_)
        ) {
            continue;
        }

        let Some(table) = datasets.dataset(series_model.dataset) else {
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
        let y1_col = if series_model.kind == crate::spec::SeriesKind::Band {
            let Some(y1_field) = series_model.encode.y2 else {
                continue;
            };
            let Some(y1_col) = dataset.fields.get(&y1_field).copied() else {
                continue;
            };
            Some(y1_col)
        } else {
            None
        };

        let base_range = view
            .dataset_view(series_model.dataset)
            .map(|v| v.row_range)
            .unwrap_or(crate::transform::RowRange {
                start: 0,
                end: table.row_count,
            });
        let base_len = base_range.end.saturating_sub(base_range.start);
        if base_len > max_view_len {
            continue;
        }

        let x_axis_range = model
            .axes
            .get(&series_model.x_axis)
            .map(|a| a.range)
            .unwrap_or_default();
        let y_axis_range = model
            .axes
            .get(&series_model.y_axis)
            .map(|a| a.range)
            .unwrap_or_default();

        let x_filter = crate::engine::window_policy::axis_filter_1d(
            x_axis_range,
            zoom_state.window,
            crate::spec::FilterMode::WeakFilter,
        );
        let y_filter = crate::engine::window_policy::axis_filter_1d(
            y_axis_range,
            Some(y_window),
            crate::spec::FilterMode::WeakFilter,
        );
        let multi_active = (x_filter.min.is_some() || x_filter.max.is_some())
            && (y_filter.min.is_some() || y_filter.max.is_some());
        if !multi_active {
            continue;
        }

        let sel = match y1_col {
            Some(y1_col) => data_views.selection_for_xy_weak_filter_band(
                series_model.dataset,
                x_col,
                y0_col,
                y1_col,
                base_range,
                x_filter,
                y_filter,
                table.revision,
            ),
            None => data_views.selection_for_xy_weak_filter(
                series_model.dataset,
                x_col,
                y0_col,
                base_range,
                x_filter,
                y_filter,
                table.revision,
            ),
        };

        if let Some(sel) = sel {
            let series_view = &mut view.series[series_view_index];
            if series_view.selection != sel {
                series_view.selection = sel;
                *view_changed = true;
            }
            if series_view.x_policy.filter != Default::default() {
                series_view.x_policy.filter = Default::default();
                *view_changed = true;
            }
        } else {
            *xy_weak_filter_pending = true;
        }
    }
}

fn apply_x_indices_for_grid(
    model: &ChartModel,
    datasets: &DatasetStore,
    view: &mut ViewState,
    data_views: &DataViewStage,
    view_series_index: &BTreeMap<SeriesId, usize>,
    series: &[SeriesId],
    view_changed: &mut bool,
    x_indices_applied: &mut BTreeSet<SeriesId>,
) {
    for series_id in series {
        let Some(series_model) = model.series.get(series_id) else {
            continue;
        };
        if !series_model.visible {
            continue;
        }

        let Some(series_view_index) = view_series_index.get(series_id).copied() else {
            continue;
        };
        let series_view = &mut view.series[series_view_index];

        if !matches!(
            series_view.x_filter_mode,
            crate::spec::FilterMode::Filter | crate::spec::FilterMode::WeakFilter
        ) {
            continue;
        }

        if matches!(series_view.selection, RowSelection::Indices(_)) {
            continue;
        }

        let x_filter = series_view.x_policy.filter;
        if x_filter.min.is_none() && x_filter.max.is_none() {
            continue;
        }

        let Some(table) = datasets.dataset(series_model.dataset) else {
            continue;
        };
        let Some(dataset) = model.datasets.get(&series_model.dataset) else {
            continue;
        };
        let Some(x_col) = dataset.fields.get(&series_model.encode.x).copied() else {
            continue;
        };

        let selection_range = series_view.selection.as_range(table.row_count);
        let selection_range = crate::transform::RowRange {
            start: selection_range.start,
            end: selection_range.end,
        };

        let Some(sel) = data_views.selection_for(
            series_model.dataset,
            x_col,
            selection_range,
            x_filter,
            table.revision,
        ) else {
            continue;
        };

        if series_view.selection != sel {
            series_view.selection = sel;
            *view_changed = true;
        }
        if series_view.x_policy.filter != Default::default() {
            series_view.x_policy.filter = Default::default();
            *view_changed = true;
        }
        x_indices_applied.insert(*series_id);
    }
}

fn apply_y_indices_for_grid(
    model: &ChartModel,
    datasets: &DatasetStore,
    state: &ChartState,
    view: &mut ViewState,
    view_series_index: &BTreeMap<SeriesId, usize>,
    series: &[SeriesId],
    max_view_len: usize,
    x_indices_applied: &BTreeSet<SeriesId>,
    view_changed: &mut bool,
) {
    for series_id in series {
        let Some(series_model) = model.series.get(series_id) else {
            continue;
        };
        if !series_model.visible || series_model.stack.is_some() {
            continue;
        }
        if !matches!(
            series_model.kind,
            crate::spec::SeriesKind::Scatter
                | crate::spec::SeriesKind::Line
                | crate::spec::SeriesKind::Area
        ) {
            continue;
        }

        let y_filter_mode = model
            .data_zoom_y_by_axis
            .get(&series_model.y_axis)
            .and_then(|id| model.data_zoom_y.get(id))
            .map(|z| z.filter_mode)
            .unwrap_or(crate::spec::FilterMode::None);
        if !matches!(
            y_filter_mode,
            crate::spec::FilterMode::Filter | crate::spec::FilterMode::WeakFilter
        ) {
            continue;
        }

        // Multi-dimensional weakFilter (v1 subset) is handled separately (indices view) and must
        // not be replaced by a simple intersection filter.
        if y_filter_mode == crate::spec::FilterMode::WeakFilter
            && state
                .data_zoom_x
                .get(&series_model.x_axis)
                .is_some_and(|s| {
                    s.filter_mode == crate::spec::FilterMode::WeakFilter && s.window.is_some()
                })
            && state.data_window_y.get(&series_model.y_axis).is_some()
        {
            continue;
        }

        let Some(series_view_index) = view_series_index.get(series_id).copied() else {
            continue;
        };
        let series_view = &mut view.series[series_view_index];

        let y_filter = series_view.y_filter;
        if y_filter.min.is_none() && y_filter.max.is_none() {
            continue;
        }

        let base_selection = series_view.selection.clone();
        if matches!(base_selection, RowSelection::Indices(_))
            && !x_indices_applied.contains(series_id)
        {
            // Avoid repeatedly scanning indices selections every frame. The primary order-sensitive
            // behavior we need is X-before-Y in the same frame when X indices were just applied.
            continue;
        }

        let Some(table) = datasets.dataset(series_model.dataset) else {
            continue;
        };
        let Some(dataset) = model.datasets.get(&series_model.dataset) else {
            continue;
        };
        let Some(x_col) = dataset.fields.get(&series_model.encode.x).copied() else {
            continue;
        };
        let Some(y_col) = dataset.fields.get(&series_model.encode.y).copied() else {
            continue;
        };
        let Some(x) = table.column_f64(x_col) else {
            continue;
        };
        let Some(y) = table.column_f64(y_col) else {
            continue;
        };

        let len = x.len().min(y.len());
        let view_len = base_selection.view_len(len);
        if view_len == 0 {
            continue;
        }
        if view_len > max_view_len {
            continue;
        }

        // Apply the X filter predicate only when X is in a filtering mode (Filter/WeakFilter).
        // For `Empty`, the X window is represented as a masking predicate and must not cull the
        // row participation space.
        let x_filter_mode = series_view.x_filter_mode;
        let x_filter = series_view.x_policy.filter;
        let x_filter_active = x_filter.min.is_some() || x_filter.max.is_some();
        let x_filter_should_cull_selection = matches!(
            x_filter_mode,
            crate::spec::FilterMode::Filter | crate::spec::FilterMode::WeakFilter
        );

        let mut indices: Vec<u32> = Vec::new();
        indices.reserve(view_len.min(4096));

        let mut kept = 0usize;
        for view_index in 0..view_len {
            let Some(raw_index) = base_selection.get_raw_index(len, view_index) else {
                continue;
            };
            let xi = x.get(raw_index).copied().unwrap_or(f64::NAN);
            let yi = y.get(raw_index).copied().unwrap_or(f64::NAN);
            if !xi.is_finite() || !yi.is_finite() {
                continue;
            }
            if x_filter_should_cull_selection && x_filter_active && !x_filter.contains(xi) {
                continue;
            }
            if !y_filter.contains(yi) {
                continue;
            }
            indices.push(raw_index.min(u32::MAX as usize) as u32);
            kept += 1;
        }

        if kept == view_len {
            continue;
        }

        series_view.selection = RowSelection::Indices(indices.into());
        if x_filter_should_cull_selection && series_view.x_policy.filter != Default::default() {
            series_view.x_policy.filter = Default::default();
        }
        *view_changed = true;
    }
}
