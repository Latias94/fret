use crate::data::DatasetStore;
use crate::engine::ChartState;
use crate::engine::model::ChartModel;
use crate::ids::{AxisId, DatasetId, GridId, Revision, SeriesId};
use crate::spec::FilterMode;
use crate::transform::{RowRange, RowSelection, SeriesXPolicy};
use crate::transform_graph::{FilterPlanStepKind, TransformGraph};
use crate::view::SeriesEmptyMask;
use crate::view::ViewState;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Debug, Default, Clone)]
pub struct FilterProcessorStage {}

#[derive(Debug, Default, Clone)]
pub struct FilterPlanOutput {
    pub revision: Revision,
    pub grids: Vec<GridFilterOutput>,
    pub series: Vec<SeriesFilterOutput>,
}

#[derive(Debug, Default, Clone)]
pub struct GridFilterOutput {
    pub grid: GridId,
    pub series: Vec<SeriesId>,
    pub y_percent_extents: BTreeMap<AxisId, (f64, f64)>,
}

#[derive(Debug, Default, Clone)]
pub struct SeriesFilterOutput {
    pub series: SeriesId,
    pub dataset: DatasetId,
    pub grid: GridId,
    pub data_revision: Revision,
    pub selection: RowSelection,
    pub x_policy: SeriesXPolicy,
    pub x_filter_mode: FilterMode,
    pub y_filter_mode: FilterMode,
    pub y_filter: crate::engine::window_policy::AxisFilter1D,
    pub empty_mask: SeriesEmptyMask,
}

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

#[derive(Debug, Clone)]
pub struct SeriesParticipationContract {
    pub selection_range: RowRange,
    pub selection: RowSelection,
    pub x_policy: SeriesXPolicy,
    pub x_filter_mode: FilterMode,
    pub y_filter_mode: FilterMode,
    pub y_filter: crate::engine::window_policy::AxisFilter1D,
    pub empty_mask: SeriesEmptyMask,
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

    pub fn series_contract(
        &self,
        series: SeriesId,
        row_count: usize,
    ) -> SeriesParticipationContract {
        let Some(p) = self.series_participation(series) else {
            return SeriesParticipationContract {
                selection_range: RowRange {
                    start: 0,
                    end: row_count,
                },
                selection: RowSelection::All,
                x_policy: SeriesXPolicy::default(),
                x_filter_mode: FilterMode::None,
                y_filter_mode: FilterMode::None,
                y_filter: crate::engine::window_policy::AxisFilter1D::default(),
                empty_mask: SeriesEmptyMask::default(),
            };
        };

        let selection_range = p.selection.as_range(row_count);
        SeriesParticipationContract {
            selection_range: RowRange {
                start: selection_range.start,
                end: selection_range.end,
            },
            selection: p.selection.clone(),
            x_policy: p.x_policy,
            x_filter_mode: p.x_filter_mode,
            y_filter_mode: p.y_filter_mode,
            y_filter: p.y_filter,
            empty_mask: p.empty_mask,
        }
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

    pub fn rebuild_from_plan_output(&mut self, model: &ChartModel, plan: &FilterPlanOutput) {
        self.series.clear();
        self.series_index.clear();
        self.series.reserve(model.series_order.len());
        self.revision = plan.revision;

        let mut by_id: BTreeMap<SeriesId, &SeriesFilterOutput> = BTreeMap::new();
        by_id.extend(plan.series.iter().map(|s| (s.series, s)));

        for (i, series_id) in model.series_order.iter().copied().enumerate() {
            self.series_index.insert(series_id, i);
            let Some(series_model) = model.series.get(&series_id) else {
                self.series.push(SeriesParticipation {
                    series: series_id,
                    ..Default::default()
                });
                continue;
            };

            if let Some(s) = by_id.get(&series_id) {
                self.series.push(SeriesParticipation {
                    series: series_id,
                    dataset: series_model.dataset,
                    revision: plan.revision,
                    data_revision: s.data_revision,
                    selection: s.selection.clone(),
                    x_policy: s.x_policy,
                    x_filter_mode: s.x_filter_mode,
                    y_filter_mode: s.y_filter_mode,
                    y_filter: s.y_filter,
                    empty_mask: s.empty_mask,
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
    pub plan_grids: u32,
    pub plan_steps_run: u32,
    pub xy_weak_filter_applied_series: u32,
    pub xy_weak_filter_pending_series: u32,
    pub xy_weak_filter_skipped_view_len_cap_series: u32,
    pub x_indices_applied_series: u32,
    pub y_indices_applied_series: u32,
    pub y_indices_skipped_view_len_cap_series: u32,
    pub y_indices_skipped_indices_scan_avoid_series: u32,
}

impl FilterProcessorStage {
    pub fn request_data_views(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        state: &ChartState,
        view: &ViewState,
        transform_graph: &mut TransformGraph,
    ) {
        for series_id in &model.series_order {
            let Some(series) = model.series.get(series_id) else {
                continue;
            };

            // Dataset transforms are engine-owned and must be materialized before downstream
            // dataZoom/filter steps can produce correct selections.
            transform_graph.request_dataset_transforms_for_dataset(model, series.dataset);

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
                let mut requested_xy_weak_filter = false;
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

                        requested_xy_weak_filter = match series.kind {
                            crate::spec::SeriesKind::Band => transform_graph
                                .request_xy_weak_filter_band_for_series(
                                    model, datasets, view, *series_id, base_range, x_filter,
                                    y_filter,
                                ),
                            _ => transform_graph.request_xy_weak_filter_for_series(
                                model, datasets, view, *series_id, base_range, x_filter, y_filter,
                            ),
                        };
                    }
                }

                // When the XY weakFilter subset is active, do not request an X-only indices view.
                // The X-only filter predicate cannot represent weakFilter semantics and can also
                // interfere with the XY materialization ordering.
                if requested_xy_weak_filter {
                    continue;
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

            transform_graph.request_x_filter_for_series(model, datasets, view, *series_id);
        }
    }

    pub fn apply(
        &mut self,
        transform_graph: &mut TransformGraph,
        model: &ChartModel,
        datasets: &DatasetStore,
        state: &mut ChartState,
        view: &mut ViewState,
    ) -> FilterProcessorResult {
        let mut xy_weak_filter_pending = false;
        let mut view_changed = false;
        let mut x_indices_applied: BTreeSet<SeriesId> = BTreeSet::new();
        let mut y_percent_extents_by_grid: BTreeMap<GridId, BTreeMap<AxisId, (f64, f64)>> =
            BTreeMap::new();

        let mut xy_weak_filter_applied_series = 0u32;
        let mut xy_weak_filter_pending_series = 0u32;
        let mut xy_weak_filter_skipped_view_len_cap_series = 0u32;
        let mut x_indices_applied_series = 0u32;
        let mut y_indices_applied_series = 0u32;
        let mut y_indices_skipped_view_len_cap_series = 0u32;
        let mut y_indices_skipped_indices_scan_avoid_series = 0u32;

        // ECharts `dataZoomProcessor` applies transforms in an order-sensitive way per grid (e.g.
        // X before Y). `TransformGraph` owns the current v1 plan scaffold and caches it by model
        // revision.
        let plan = transform_graph.filter_plan(model).clone();

        let view_series_index: BTreeMap<SeriesId, usize> = view
            .series
            .iter()
            .enumerate()
            .map(|(i, v)| (v.series, i))
            .collect();

        apply_dataset_transforms_for_view(
            transform_graph,
            model,
            datasets,
            view,
            &view_series_index,
            &mut view_changed,
        );

        // Step ordering is intentionally explicit and per-grid (ECharts-style ordering scaffold).
        const MAX_MULTI_DIM_WEAKFILTER_VIEW_LEN: usize = 200_000;
        const MAX_Y_FILTER_VIEW_LEN: usize = 200_000;

        let plan_grids = plan.grids.len().min(u32::MAX as usize) as u32;
        let plan_steps_run = plan.steps.len().min(u32::MAX as usize) as u32;

        for step in &plan.steps {
            let Some(series) = plan.grids.get(&step.grid) else {
                continue;
            };

            match step.kind {
                FilterPlanStepKind::XYWeakFilter => apply_xy_weak_filter_for_grid(
                    model,
                    datasets,
                    state,
                    view,
                    transform_graph,
                    &view_series_index,
                    series,
                    MAX_MULTI_DIM_WEAKFILTER_VIEW_LEN,
                    &mut xy_weak_filter_pending,
                    &mut view_changed,
                    &mut xy_weak_filter_applied_series,
                    &mut xy_weak_filter_pending_series,
                    &mut xy_weak_filter_skipped_view_len_cap_series,
                ),
                FilterPlanStepKind::XRange => apply_x_range_for_grid(
                    transform_graph,
                    model,
                    datasets,
                    view,
                    &view_series_index,
                    series,
                    &mut view_changed,
                ),
                FilterPlanStepKind::XIndices => apply_x_indices_for_grid(
                    model,
                    datasets,
                    view,
                    transform_graph,
                    &view_series_index,
                    series,
                    &mut view_changed,
                    &mut x_indices_applied,
                    &mut x_indices_applied_series,
                ),
                FilterPlanStepKind::YPercent => apply_y_percent_for_grid(
                    transform_graph,
                    model,
                    datasets,
                    state,
                    view,
                    &view_series_index,
                    series,
                    step.grid,
                    &mut y_percent_extents_by_grid,
                    &mut view_changed,
                ),
                FilterPlanStepKind::YIndices => apply_y_indices_for_grid(
                    transform_graph,
                    model,
                    datasets,
                    state,
                    view,
                    &view_series_index,
                    series,
                    MAX_Y_FILTER_VIEW_LEN,
                    &x_indices_applied,
                    &mut view_changed,
                    &mut y_indices_applied_series,
                    &mut y_indices_skipped_view_len_cap_series,
                    &mut y_indices_skipped_indices_scan_avoid_series,
                ),
            }
        }

        if view_changed {
            view.revision.bump();
            for series_view in &mut view.series {
                series_view.revision = view.revision;
            }
        }

        transform_graph.refresh_filter_plan_output(model, view, &y_percent_extents_by_grid);

        FilterProcessorResult {
            xy_weak_filter_pending,
            plan_grids,
            plan_steps_run,
            xy_weak_filter_applied_series,
            xy_weak_filter_pending_series,
            xy_weak_filter_skipped_view_len_cap_series,
            x_indices_applied_series,
            y_indices_applied_series,
            y_indices_skipped_view_len_cap_series,
            y_indices_skipped_indices_scan_avoid_series,
        }
    }
}

fn apply_dataset_transforms_for_view(
    transform_graph: &TransformGraph,
    model: &ChartModel,
    datasets: &DatasetStore,
    view: &mut ViewState,
    view_series_index: &BTreeMap<SeriesId, usize>,
    view_changed: &mut bool,
) {
    for series_id in &model.series_order {
        let Some(series_model) = model.series.get(series_id) else {
            continue;
        };
        let Some(dataset_model) = model.datasets.get(&series_model.dataset) else {
            continue;
        };
        if dataset_model.from.is_none() || dataset_model.transforms.is_empty() {
            continue;
        }

        let Some(series_view_index) = view_series_index.get(series_id).copied() else {
            continue;
        };

        let root = model.root_dataset_id(series_model.dataset);
        let Some(table) = datasets.dataset(root) else {
            continue;
        };

        let base_range = view
            .dataset_view(series_model.dataset)
            .map(|v| v.row_range)
            .unwrap_or(RowRange {
                start: 0,
                end: table.row_count,
            });

        let sel = transform_graph.dataset_transform_selection_for_dataset(
            model,
            datasets,
            view,
            series_model.dataset,
        );
        let sel = sel.unwrap_or(RowSelection::Range(base_range));

        let series_view = &mut view.series[series_view_index];
        if series_view.selection != sel {
            series_view.selection = sel;
            *view_changed = true;
        }
    }
}

fn apply_xy_weak_filter_for_grid(
    model: &ChartModel,
    datasets: &DatasetStore,
    state: &ChartState,
    view: &mut ViewState,
    transform_graph: &TransformGraph,
    view_series_index: &BTreeMap<SeriesId, usize>,
    series: &[SeriesId],
    max_view_len: usize,
    xy_weak_filter_pending: &mut bool,
    view_changed: &mut bool,
    xy_weak_filter_applied_series: &mut u32,
    xy_weak_filter_pending_series: &mut u32,
    xy_weak_filter_skipped_view_len_cap_series: &mut u32,
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

        let root = model.root_dataset_id(series_model.dataset);
        let Some(table) = datasets.dataset(root) else {
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
            *xy_weak_filter_skipped_view_len_cap_series =
                xy_weak_filter_skipped_view_len_cap_series.saturating_add(1);
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
            Some(y1_col) => transform_graph.selection_for_xy_weak_filter_band(
                series_model.dataset,
                root,
                x_col,
                y0_col,
                y1_col,
                base_range,
                x_filter,
                y_filter,
                table.revision,
            ),
            None => transform_graph.selection_for_xy_weak_filter(
                series_model.dataset,
                root,
                x_col,
                y0_col,
                base_range,
                x_filter,
                y_filter,
                table.revision,
            ),
        };

        let series_view = &mut view.series[series_view_index];

        if series_view.x_policy.filter != Default::default() {
            series_view.x_policy.filter = Default::default();
            *view_changed = true;
        }

        if let Some(sel) = sel {
            *xy_weak_filter_applied_series = xy_weak_filter_applied_series.saturating_add(1);
            if series_view.selection != sel {
                series_view.selection = sel;
                *view_changed = true;
            }
        } else {
            *xy_weak_filter_pending = true;
            *xy_weak_filter_pending_series = xy_weak_filter_pending_series.saturating_add(1);

            // While the indices view is materializing, preserve a stable base row space. X-only
            // slicing cannot represent weakFilter semantics.
            if series_view.selection != RowSelection::Range(base_range) {
                series_view.selection = RowSelection::Range(base_range);
                *view_changed = true;
            }
        }
    }
}

fn apply_x_range_for_grid(
    transform_graph: &mut TransformGraph,
    model: &ChartModel,
    datasets: &DatasetStore,
    view: &mut ViewState,
    view_series_index: &BTreeMap<SeriesId, usize>,
    series: &[SeriesId],
    view_changed: &mut bool,
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
        let Some(sel) = transform_graph.x_range_selection_for_series(
            model,
            datasets,
            view,
            *series_id,
            series_view_index,
        ) else {
            continue;
        };
        let series_view = &mut view.series[series_view_index];
        if series_view.selection != sel {
            series_view.selection = sel;
            *view_changed = true;
        }
    }
}

fn apply_x_indices_for_grid(
    model: &ChartModel,
    datasets: &DatasetStore,
    view: &mut ViewState,
    transform_graph: &TransformGraph,
    view_series_index: &BTreeMap<SeriesId, usize>,
    series: &[SeriesId],
    view_changed: &mut bool,
    x_indices_applied: &mut BTreeSet<SeriesId>,
    x_indices_applied_series: &mut u32,
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

        let root = model.root_dataset_id(series_model.dataset);
        let Some(table) = datasets.dataset(root) else {
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

        let Some(sel) = transform_graph.selection_for_x_filter(
            series_model.dataset,
            root,
            x_col,
            selection_range,
            x_filter,
            table.revision,
        ) else {
            continue;
        };

        *x_indices_applied_series = x_indices_applied_series.saturating_add(1);
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
    transform_graph: &mut TransformGraph,
    model: &ChartModel,
    datasets: &DatasetStore,
    state: &ChartState,
    view: &mut ViewState,
    view_series_index: &BTreeMap<SeriesId, usize>,
    series: &[SeriesId],
    max_view_len: usize,
    x_indices_applied: &BTreeSet<SeriesId>,
    view_changed: &mut bool,
    y_indices_applied_series: &mut u32,
    y_indices_skipped_view_len_cap_series: &mut u32,
    y_indices_skipped_indices_scan_avoid_series: &mut u32,
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
        let x_indices_applied = x_indices_applied.contains(series_id);
        let out = transform_graph.y_indices_node_for_series(
            model,
            datasets,
            view,
            *series_id,
            series_view_index,
            max_view_len,
            x_indices_applied,
        );

        match out.result {
            crate::transform_graph::YIndicesNodeResult::NoChange => {}
            crate::transform_graph::YIndicesNodeResult::SkippedViewLenCap => {
                *y_indices_skipped_view_len_cap_series =
                    y_indices_skipped_view_len_cap_series.saturating_add(1);
            }
            crate::transform_graph::YIndicesNodeResult::SkippedIndicesScanAvoid => {
                *y_indices_skipped_indices_scan_avoid_series =
                    y_indices_skipped_indices_scan_avoid_series.saturating_add(1);
            }
            crate::transform_graph::YIndicesNodeResult::Indices(indices) => {
                let series_view = &mut view.series[series_view_index];
                *y_indices_applied_series = y_indices_applied_series.saturating_add(1);
                if series_view.selection != RowSelection::Indices(indices.clone()) {
                    series_view.selection = RowSelection::Indices(indices);
                }
                if out.x_filter_should_cull_selection
                    && series_view.x_policy.filter != Default::default()
                {
                    series_view.x_policy.filter = Default::default();
                }
                *view_changed = true;
            }
        }
    }
}

fn apply_y_percent_for_grid(
    transform_graph: &mut TransformGraph,
    model: &ChartModel,
    datasets: &DatasetStore,
    state: &mut ChartState,
    view: &mut ViewState,
    view_series_index: &BTreeMap<SeriesId, usize>,
    series: &[SeriesId],
    grid: GridId,
    y_percent_extents_by_grid: &mut BTreeMap<GridId, BTreeMap<AxisId, (f64, f64)>>,
    view_changed: &mut bool,
) {
    if transform_graph.apply_y_percent_for_grid(
        model,
        datasets,
        state,
        view,
        view_series_index,
        series,
        grid,
        y_percent_extents_by_grid,
    ) {
        *view_changed = true;
    }
}
