use fret_core::{Point, Px, Rect};

use crate::data::DatasetStore;
use crate::engine::ChartState;
use crate::engine::lod::{
    BoundsAccum, BoundsCursor, DataBounds, LodScratch, MinMaxPerPixelCursor,
    compute_bounds_step_selection, compute_bounds_step_selection_with, finalize_bounds,
    minmax_per_pixel_finalize, minmax_per_pixel_step_segmented_with,
    minmax_per_pixel_step_selection, minmax_per_pixel_step_selection_with,
};
use crate::engine::model::ChartModel;
use crate::engine::window::{DataWindow, DataWindowX, DataWindowY};
use crate::ids::GridId;
use crate::ids::SeriesId;
use crate::ids::StackId;
use crate::ids::series_mark_id;
use crate::marks::{
    MarkKind, MarkNode, MarkOrderKey, MarkPayloadRef, MarkPointsRef, MarkPolylineRef, MarkRectRef,
    MarkTree,
};
use crate::paint::StrokeStyleV2;
use crate::scheduler::WorkBudget;
use crate::spec::AxisRange;
use crate::spec::BarWidthSpec;
use crate::stats::EngineStats;
use crate::transform::RowSelection;
use crate::transform_graph::DataViewStage;
use std::collections::BTreeMap;

use super::{BarLayoutStage, ParticipationState, StackDimsStage};

const SCATTER_LARGE_MODE_VISIBLE_LEN_THRESHOLD: usize = 20_000;

#[derive(Debug, Default, Clone)]
struct MinMaxAppendCache {
    data_rev: crate::ids::Revision,
    row_count: usize,
    bounds: DataBounds,
    viewport_width_px: usize,
    cursor: MinMaxPerPixelCursor,
    scratch: LodScratch,
}

#[derive(Debug, Clone)]
struct StackBoundsBuild {
    stack: StackId,
    x_axis: crate::ids::AxisId,
    y_axis: crate::ids::AxisId,
    series: Vec<SeriesId>,
    series_index: usize,
    cursor: BoundsCursor,
    accum: BoundsAccum,
    selection: RowSelection,
    x_filter: crate::engine::window_policy::AxisFilter1D,
    x_mapping_window: Option<DataWindowX>,
}

#[derive(Debug, Clone)]
struct ScatterBucketBuild {
    series: SeriesId,
    visual_map: crate::ids::VisualMapId,
    bucket_count: u16,
    next_view_index: usize,
    points_by_bucket: Vec<Vec<Point>>,
    indices_by_bucket: Vec<Vec<u32>>,
}

#[derive(Debug, Clone)]
struct BarBucketBuild {
    series: SeriesId,
    visual_map: crate::ids::VisualMapId,
    bucket_count: u16,
    next_view_index: usize,
    rects_by_bucket: Vec<Vec<Rect>>,
    indices_by_bucket: Vec<Vec<u32>>,
}

#[derive(Debug, Default, Clone)]
pub struct MarksStage {
    series_index: usize,
    cursor: MinMaxPerPixelCursor,
    segmented_cursor: crate::engine::lod::SegmentedMinMaxPerPixelCursor,
    segmented_series: Option<SeriesId>,
    segmented_segment_index: u64,
    bounds_cursor: BoundsCursor,
    bounds_accum: BoundsAccum,
    active_series: Option<SeriesId>,
    active_selection: RowSelection,
    stack_bounds: BTreeMap<StackId, DataBounds>,
    stack_bounds_build: Option<StackBoundsBuild>,
    scatter_next_index: usize,
    scatter_points_start: usize,
    scatter_node_index: Option<usize>,
    scatter_bucket_build: Option<ScatterBucketBuild>,
    bar_next_index: usize,
    bar_rects_start: usize,
    bar_node_index: Option<usize>,
    bar_bucket_build: Option<BarBucketBuild>,
    finalized: bool,
    dirty: bool,
    last_series_count: usize,
    last_model_marks_rev: crate::ids::Revision,
    last_data_sig: u64,
    last_participation_rev: crate::ids::Revision,
    pending_append_rebuild: bool,
    last_dataset_meta: BTreeMap<crate::ids::DatasetId, (crate::ids::Revision, usize, usize)>,
    append_rebuild_mode: bool,
    minmax_append_cache: BTreeMap<SeriesId, MinMaxAppendCache>,
    bounds: Option<DataBounds>,
    axis_windows: BTreeMap<crate::ids::AxisId, DataWindow>,
}

impl MarksStage {
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn sync_inputs(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        participation: &ParticipationState,
    ) {
        let series_count = model.series_order.len();
        if series_count != self.last_series_count {
            self.dirty = true;
        }
        self.last_series_count = series_count;

        if model.revs.marks != self.last_model_marks_rev {
            self.dirty = true;
        }
        self.last_model_marks_rev = model.revs.marks;

        let mut append_only_change = false;
        let mut full_reset = false;
        let mut current_meta: BTreeMap<
            crate::ids::DatasetId,
            (crate::ids::Revision, usize, usize),
        > = BTreeMap::new();

        for series_id in &model.series_order {
            let Some(series) = model.series.get(series_id) else {
                continue;
            };
            let dataset_id = series.dataset;
            let Some(table) = datasets.dataset(model.root_dataset_id(dataset_id)) else {
                full_reset = true;
                continue;
            };
            current_meta.insert(
                dataset_id,
                (table.revision(), table.row_count(), table.column_count()),
            );
        }

        if !self.last_dataset_meta.is_empty() {
            for (dataset_id, (rev, row_count, col_count)) in current_meta.iter() {
                let Some((prev_rev, prev_row_count, prev_col_count)) =
                    self.last_dataset_meta.get(dataset_id)
                else {
                    full_reset = true;
                    continue;
                };
                if *col_count != *prev_col_count {
                    full_reset = true;
                    continue;
                }
                if *row_count < *prev_row_count {
                    full_reset = true;
                    continue;
                }
                if *row_count > *prev_row_count {
                    append_only_change = true;
                    continue;
                }
                if *rev != *prev_rev {
                    // Revision changed without row_count growth; treat as a replace/mutate-in-place.
                    full_reset = true;
                }
            }

            for dataset_id in self.last_dataset_meta.keys() {
                if !current_meta.contains_key(dataset_id) {
                    full_reset = true;
                }
            }
        }

        self.last_dataset_meta = current_meta;

        if full_reset {
            self.dirty = true;
        } else if append_only_change {
            // Append-only rebuild keeps the previous `MarkTree` around and rebuilds nodes in-place.
            // VisualMap produces a variable set of bucket batches; hiding/removing now-stale buckets
            // requires additional book-keeping, so we conservatively fall back to a full reset.
            if !model.visual_map_by_series.is_empty() {
                self.dirty = true;
            } else {
                self.pending_append_rebuild = true;
            }
        }

        let data_sig = dataset_store_signature(model, datasets);
        self.last_data_sig = data_sig;

        if participation.revision != self.last_participation_rev {
            // The participation revision bumps on any view-affecting input change, including the
            // dataset signature. For append-only dataset growth we keep the previous `MarkTree`
            // around and rebuild nodes in-place under budget, so a revision bump should not force
            // a full reset.
            if !self.pending_append_rebuild {
                self.dirty = true;
            }
        }
        self.last_participation_rev = participation.revision;
    }

    pub fn take_append_rebuild(&mut self) -> bool {
        if self.dirty {
            self.pending_append_rebuild = false;
            return false;
        }
        std::mem::take(&mut self.pending_append_rebuild)
    }

    pub fn reset(&mut self) {
        self.series_index = 0;
        self.cursor = MinMaxPerPixelCursor::default();
        self.segmented_cursor = crate::engine::lod::SegmentedMinMaxPerPixelCursor::default();
        self.segmented_series = None;
        self.segmented_segment_index = 0;
        self.bounds_cursor = BoundsCursor::default();
        self.bounds_accum.reset();
        self.active_series = None;
        self.active_selection = RowSelection::default();
        self.stack_bounds.clear();
        self.stack_bounds_build = None;
        self.scatter_next_index = 0;
        self.scatter_points_start = 0;
        self.scatter_node_index = None;
        self.scatter_bucket_build = None;
        self.bar_next_index = 0;
        self.bar_rects_start = 0;
        self.bar_node_index = None;
        self.bar_bucket_build = None;
        self.finalized = false;
        self.dirty = false;
        self.bounds = None;
        self.axis_windows.clear();
        self.pending_append_rebuild = false;
        self.append_rebuild_mode = false;
        self.minmax_append_cache.clear();
    }

    pub fn begin_append_rebuild(&mut self) {
        self.series_index = 0;
        self.cursor = MinMaxPerPixelCursor::default();
        self.segmented_cursor = crate::engine::lod::SegmentedMinMaxPerPixelCursor::default();
        self.segmented_series = None;
        self.segmented_segment_index = 0;
        self.bounds_cursor = BoundsCursor::default();
        self.bounds_accum.reset();
        self.active_series = None;
        self.active_selection = RowSelection::default();
        self.stack_bounds.clear();
        self.stack_bounds_build = None;
        self.scatter_next_index = 0;
        self.scatter_points_start = 0;
        self.scatter_node_index = None;
        self.scatter_bucket_build = None;
        self.bar_next_index = 0;
        self.bar_rects_start = 0;
        self.bar_node_index = None;
        self.bar_bucket_build = None;
        self.finalized = false;
        self.dirty = false;
        self.bounds = None;
        self.axis_windows.clear();
        self.append_rebuild_mode = true;
    }

    pub fn axis_windows(&self) -> &BTreeMap<crate::ids::AxisId, DataWindow> {
        &self.axis_windows
    }

    pub fn step(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        state: &ChartState,
        selection_stage: &DataViewStage,
        stack_dims: &StackDimsStage,
        bar_layout: &BarLayoutStage,
        participation: &ParticipationState,
        plot_viewports_by_grid: &BTreeMap<GridId, Rect>,
        budget: &mut WorkBudget,
        scratch: &mut LodScratch,
        marks: &mut MarkTree,
        stats: &mut EngineStats,
    ) -> bool {
        while self.series_index < model.series_order.len() {
            let series_id = model.series_order[self.series_index];
            let Some(series) = model.series.get(&series_id) else {
                self.series_index += 1;
                continue;
            };
            if !series.visible {
                self.series_index += 1;
                continue;
            }

            let viewport = model
                .axes
                .get(&series.x_axis)
                .and_then(|axis| plot_viewports_by_grid.get(&axis.grid))
                .copied()
                .or_else(|| plot_viewports_by_grid.values().next().copied())
                .unwrap_or_default();

            let root = model.root_dataset_id(series.dataset);
            let table = datasets.dataset(root);
            let Some(table) = table else {
                self.series_index += 1;
                continue;
            };
            let contract = participation.series_contract(series.id, table.row_count());
            let base_selection = contract.selection;
            let selection_range = contract.selection_range;
            let view_x_filter = contract.x_policy.filter;
            let view_x_mapping_window = contract.x_policy.mapping_window;
            let view_y_filter = contract.y_filter;
            let empty_mask = contract.empty_mask;

            let Some(dataset) = model.datasets.get(&series.dataset) else {
                self.series_index += 1;
                continue;
            };
            let Some(x_col) = dataset.fields.get(&series.encode.x).copied() else {
                self.series_index += 1;
                continue;
            };
            let Some(y_col) = dataset.fields.get(&series.encode.y).copied() else {
                self.series_index += 1;
                continue;
            };

            if self.active_series != Some(series.id) {
                let view = selection_stage.table_view_for(
                    table,
                    series.dataset,
                    root,
                    x_col,
                    selection_range,
                    view_x_filter,
                    base_selection,
                );
                self.active_series = Some(series.id);
                self.active_selection = view.selection().clone();
            }

            let selection = self.active_selection.clone();

            let Some(x) = table.column_f64(x_col) else {
                self.series_index += 1;
                continue;
            };
            let Some(y0) = table.column_f64(y_col) else {
                self.series_index += 1;
                continue;
            };
            let y1 = if series.kind == crate::spec::SeriesKind::Band {
                let Some(y2_field) = series.encode.y2 else {
                    self.series_index += 1;
                    continue;
                };
                let Some(y2_col) = dataset.fields.get(&y2_field).copied() else {
                    self.series_index += 1;
                    continue;
                };
                let Some(y1) = table.column_f64(y2_col) else {
                    self.series_index += 1;
                    continue;
                };
                Some(y1)
            } else {
                None
            };

            if self.cursor.next_index == 0
                && self.bounds.is_none()
                && self.bounds_cursor.next_index == 0
                && self.scatter_next_index == 0
                && self.bar_next_index == 0
            {
                scratch.reset_buckets();
                self.finalized = false;
                self.bounds = None;
                self.bounds_cursor = BoundsCursor::default();
                self.bounds_accum.reset();
                self.scatter_points_start = 0;
                self.scatter_node_index = None;
                self.scatter_bucket_build = None;
                self.bar_rects_start = 0;
                self.bar_node_index = None;
                self.bar_bucket_build = None;
            }

            if self.bounds.is_none() {
                let points_budget = budget.take_points(4096) as usize;
                if points_budget == 0 {
                    return false;
                }

                if let Some(stack) = series.stack {
                    if let Some(bounds) = self.stack_bounds.get(&stack).copied() {
                        self.bounds = Some(bounds);
                    } else {
                        if self.stack_bounds_build.is_none()
                            || self
                                .stack_bounds_build
                                .as_ref()
                                .is_some_and(|b| b.stack != stack)
                        {
                            let stack_series: Vec<SeriesId> = model
                                .series_order
                                .iter()
                                .copied()
                                .filter(|id| {
                                    model.series.get(id).is_some_and(|s| {
                                        s.visible
                                            && s.stack == Some(stack)
                                            && s.x_axis == series.x_axis
                                            && s.y_axis == series.y_axis
                                            && s.dataset == series.dataset
                                            && s.encode.x == series.encode.x
                                    })
                                })
                                .collect();

                            let mut accum = BoundsAccum::default();
                            accum.reset();

                            self.stack_bounds_build = Some(StackBoundsBuild {
                                stack,
                                x_axis: series.x_axis,
                                y_axis: series.y_axis,
                                series: stack_series,
                                series_index: 0,
                                cursor: BoundsCursor::default(),
                                accum,
                                selection: selection.clone(),
                                x_filter: view_x_filter,
                                x_mapping_window: view_x_mapping_window,
                            });
                        }

                        let Some(build) = self.stack_bounds_build.as_mut() else {
                            return false;
                        };

                        while build.series_index < build.series.len() {
                            let series_id = build.series[build.series_index];
                            let Some(s) = model.series.get(&series_id) else {
                                build.series_index += 1;
                                build.cursor = BoundsCursor::default();
                                continue;
                            };

                            let Some(dataset) = model.datasets.get(&s.dataset) else {
                                build.series_index += 1;
                                build.cursor = BoundsCursor::default();
                                continue;
                            };
                            let Some(x_col) = dataset.fields.get(&s.encode.x).copied() else {
                                build.series_index += 1;
                                build.cursor = BoundsCursor::default();
                                continue;
                            };

                            let root = model.root_dataset_id(s.dataset);
                            let table = datasets.dataset(root);
                            let Some(table) = table else {
                                build.series_index += 1;
                                build.cursor = BoundsCursor::default();
                                continue;
                            };

                            let Some(x) = table.column_f64(x_col) else {
                                build.series_index += 1;
                                build.cursor = BoundsCursor::default();
                                continue;
                            };
                            let Some(arrays) = stack_dims.stack_arrays(
                                build.stack,
                                series_id,
                                model.revs.marks,
                                table.revision(),
                            ) else {
                                return false;
                            };

                            let points_budget = budget.take_points(4096) as usize;
                            if points_budget == 0 {
                                return false;
                            }

                            let done = compute_bounds_step_selection_with(
                                &mut build.cursor,
                                &mut build.accum,
                                x,
                                &build.selection,
                                build.x_filter,
                                points_budget,
                                |i| arrays.stacked.get(i).copied().unwrap_or(f64::NAN),
                            )
                            .unwrap_or(true);

                            if done {
                                build.series_index += 1;
                                build.cursor = BoundsCursor::default();
                            }
                        }

                        let mut bounds = finalize_bounds(&build.accum).unwrap_or_default();

                        if build.series.iter().any(|id| {
                            model
                                .series
                                .get(id)
                                .is_some_and(|s| s.kind == crate::spec::SeriesKind::Bar)
                        }) {
                            bounds.y_min = bounds.y_min.min(0.0);
                            bounds.y_max = bounds.y_max.max(0.0);
                        }

                        if let Some(mut w) = build.x_mapping_window.or(model
                            .axes
                            .get(&build.x_axis)
                            .and_then(crate::engine::axis::category_domain_window))
                        {
                            w.clamp_non_degenerate();
                            bounds.x_min = w.min;
                            bounds.x_max = w.max;
                        }

                        let y_axis_range = model
                            .axes
                            .get(&build.y_axis)
                            .map(|a| a.range)
                            .unwrap_or_default();
                        let y_window_for_bounds = axis_locked_window_y(y_axis_range)
                            .or(state.data_window_y.get(&build.y_axis).copied());
                        if let Some(mut y_window) = y_window_for_bounds {
                            y_window.clamp_non_degenerate();
                            bounds.y_min = y_window.min;
                            bounds.y_max = y_window.max;
                        }

                        apply_axis_constraints(model, build.x_axis, build.y_axis, &mut bounds);
                        bounds.clamp_non_degenerate();

                        self.stack_bounds.insert(stack, bounds);
                        self.stack_bounds_build = None;
                        continue;
                    }
                }

                if self.bounds.is_none() {
                    let Some(bounds) = compute_series_bounds(
                        model,
                        state,
                        series.kind,
                        series.x_axis,
                        series.y_axis,
                        x,
                        y0,
                        y1,
                        &selection,
                        view_x_filter,
                        view_x_mapping_window,
                        points_budget,
                        &mut self.bounds_cursor,
                        &mut self.bounds_accum,
                        budget,
                    ) else {
                        return false;
                    };
                    self.bounds = Some(bounds);
                }

                let Some(bounds) = self.bounds else {
                    return false;
                };
                merge_axis_window(
                    &mut self.axis_windows,
                    series.x_axis,
                    DataWindow {
                        min: bounds.x_min,
                        max: bounds.x_max,
                    },
                );
                merge_axis_window(
                    &mut self.axis_windows,
                    series.y_axis,
                    DataWindow {
                        min: bounds.y_min,
                        max: bounds.y_max,
                    },
                );
                self.bounds = Some(bounds);
            }
            let Some(mut bounds) = self.bounds else {
                self.series_index += 1;
                self.cursor.next_index = 0;
                self.bar_next_index = 0;
                self.bar_rects_start = 0;
                self.bar_node_index = None;
                self.bar_bucket_build = None;
                self.bounds = None;
                continue;
            };
            bounds.clamp_non_degenerate();

            if series.kind == crate::spec::SeriesKind::Scatter {
                let x_window = view_x_mapping_window.unwrap_or(DataWindow {
                    min: bounds.x_min,
                    max: bounds.x_max,
                });
                let mut x_window = x_window;
                x_window.clamp_non_degenerate();

                let y_window = DataWindow {
                    min: bounds.y_min,
                    max: bounds.y_max,
                };
                let mut y_window = y_window;
                y_window.clamp_non_degenerate();

                let visible_len = selection.view_len(table.row_count());
                let large_threshold = series
                    .lod
                    .large_threshold
                    .map(|v| v as usize)
                    .unwrap_or(SCATTER_LARGE_MODE_VISIBLE_LEN_THRESHOLD);
                let large_enabled = series.lod.large != Some(false);
                let use_lod = large_enabled && visible_len > large_threshold;

                let progressive_cap = series.lod.progressive.and_then(|cap| {
                    let cap = (cap.max(1)) as usize;
                    let threshold = series
                        .lod
                        .progressive_threshold
                        .map(|v| v as usize)
                        .unwrap_or(large_threshold);
                    (visible_len >= threshold).then_some(cap)
                });
                let mut progressive_spent = 0usize;

                if use_lod {
                    let width_px = viewport.size.width.0.max(1.0).ceil() as usize;
                    if self.append_rebuild_mode {
                        if let Some(cache) = self.minmax_append_cache.get_mut(&series.id)
                            && table.row_count() >= cache.row_count
                            && cache.viewport_width_px == width_px
                            && cache.bounds == bounds
                            && self.cursor.next_index == 0
                        {
                            self.cursor = cache.cursor.clone();
                            std::mem::swap(scratch, &mut cache.scratch);
                        }
                    }

                    let mut finished_scan = false;
                    while !finished_scan {
                        let mut request = 4096usize;
                        if let Some(cap) = progressive_cap {
                            let remaining = cap.saturating_sub(progressive_spent);
                            if remaining == 0 {
                                return false;
                            }
                            request = request.min(remaining);
                        }

                        let points_budget = budget.take_points(request as u32) as usize;
                        if points_budget == 0 {
                            return false;
                        }
                        progressive_spent = progressive_spent.saturating_add(points_budget);

                        finished_scan = minmax_per_pixel_step_selection_with(
                            &mut self.cursor,
                            scratch,
                            x,
                            &bounds,
                            viewport,
                            &selection,
                            points_budget,
                            |i| {
                                if (empty_mask.x_active || empty_mask.y_active)
                                    && !empty_mask.allows_raw_index(i, x, y0, None)
                                {
                                    return f64::NAN;
                                }
                                y0.get(i).copied().unwrap_or(f64::NAN)
                            },
                        );

                        if let Some(cap) = progressive_cap
                            && progressive_spent >= cap
                            && !finished_scan
                        {
                            return false;
                        }
                    }

                    if !self.finalized {
                        let range = minmax_per_pixel_finalize(
                            scratch,
                            x,
                            &bounds,
                            viewport,
                            &mut marks.arena.points,
                            &mut marks.arena.data_indices,
                        );
                        let id = series_mark_id(series.id, 0);
                        let node_index = marks.nodes.iter().position(|n| n.id == id);
                        if range.is_empty() {
                            if let Some(i) = node_index {
                                marks.nodes.remove(i);
                                marks.revision.bump();
                            }
                            self.finalized = true;
                        } else {
                            let base_order = self.series_index as u32;

                            let mut created = false;
                            let node_index = if let Some(i) = node_index {
                                i
                            } else {
                                if budget.take_marks(1) == 0 {
                                    return false;
                                }
                                created = true;
                                marks.nodes.push(MarkNode {
                                    id,
                                    parent: None,
                                    layer: crate::ids::LayerId(1),
                                    order: MarkOrderKey(base_order.saturating_mul(2)),
                                    kind: MarkKind::Points,
                                    source_series: Some(series.id),
                                    payload: MarkPayloadRef::Points(MarkPointsRef {
                                        points: range.clone(),
                                        fill: None,
                                        opacity_mul: None,
                                        radius_mul: None,
                                        stroke: None,
                                    }),
                                });
                                marks.nodes.len() - 1
                            };

                            if let Some(node) = marks.nodes.get_mut(node_index) {
                                let MarkPayloadRef::Points(p) = &mut node.payload else {
                                    return false;
                                };
                                p.points = range.clone();
                                p.fill = None;
                                p.opacity_mul = None;
                                p.radius_mul = None;
                                p.stroke = None;
                            }

                            stats.points_emitted += (range.end - range.start) as u64;
                            if created {
                                stats.marks_emitted += 1;
                            }
                            marks.revision.bump();
                            self.finalized = true;
                        }
                    }

                    {
                        let entry = self.minmax_append_cache.entry(series.id).or_default();
                        entry.data_rev = table.revision();
                        entry.row_count = table.row_count();
                        entry.bounds = bounds;
                        entry.viewport_width_px = width_px;
                        entry.cursor = self.cursor.clone();
                        std::mem::swap(scratch, &mut entry.scratch);
                    }

                    self.series_index += 1;
                    self.cursor.next_index = 0;
                    self.scatter_next_index = 0;
                    self.scatter_points_start = 0;
                    self.scatter_node_index = None;
                    self.scatter_bucket_build = None;
                    self.bar_next_index = 0;
                    self.bar_rects_start = 0;
                    self.bar_node_index = None;
                    self.bar_bucket_build = None;
                    self.bounds = None;
                    scratch.clear();
                    continue;
                }

                if let Some(visual_map_id) = model.visual_map_by_series.get(&series.id).copied() {
                    let Some(vm) = model.visual_maps.get(&visual_map_id).copied() else {
                        return false;
                    };
                    let selected_range = state
                        .visual_map_range
                        .get(&visual_map_id)
                        .copied()
                        .unwrap_or(vm.initial_range);
                    let selected_piece_mask = state
                        .visual_map_piece_mask
                        .get(&visual_map_id)
                        .copied()
                        .unwrap_or(vm.initial_piece_mask);

                    let Some(dataset) = model.datasets.get(&series.dataset) else {
                        return false;
                    };
                    let Some(vm_col) = dataset.fields.get(&vm.field).copied() else {
                        return false;
                    };
                    let Some(vm_values) = table.column_f64(vm_col) else {
                        return false;
                    };

                    let total_buckets = vm.buckets as usize * 2;
                    let rebuild = match self.scatter_bucket_build.as_ref() {
                        Some(b) => {
                            b.series != series.id
                                || b.visual_map != visual_map_id
                                || b.bucket_count != vm.buckets
                        }
                        None => true,
                    };
                    if rebuild {
                        self.scatter_bucket_build = Some(ScatterBucketBuild {
                            series: series.id,
                            visual_map: visual_map_id,
                            bucket_count: vm.buckets,
                            next_view_index: 0,
                            points_by_bucket: vec![Vec::default(); total_buckets],
                            indices_by_bucket: vec![Vec::default(); total_buckets],
                        });
                    }
                    let Some(build) = self.scatter_bucket_build.as_mut() else {
                        return false;
                    };

                    let row_end = selection.view_len(table.row_count());
                    while build.next_view_index < row_end {
                        let points_budget = budget.take_points(4096) as usize;
                        if points_budget == 0 {
                            return false;
                        }

                        let chunk_end = (build.next_view_index + points_budget).min(row_end);

                        let x_span = x_window.span();
                        let y_span = y_window.span();
                        let x_span = if x_span.is_finite() && x_span > 0.0 {
                            x_span
                        } else {
                            1.0
                        };
                        let y_span = if y_span.is_finite() && y_span > 0.0 {
                            y_span
                        } else {
                            1.0
                        };

                        let data_len = x.len().min(y0.len()).min(vm_values.len());
                        for view_index in build.next_view_index..chunk_end {
                            let Some(i) = selection.get_raw_index(data_len, view_index) else {
                                continue;
                            };

                            let xi = x.get(i).copied().unwrap_or(f64::NAN);
                            let yi = y0.get(i).copied().unwrap_or(f64::NAN);
                            if !xi.is_finite() || !yi.is_finite() {
                                continue;
                            }
                            if xi < x_window.min || xi > x_window.max {
                                continue;
                            }
                            if yi < y_window.min || yi > y_window.max {
                                continue;
                            }

                            let tx = ((xi - x_window.min) / x_span).clamp(0.0, 1.0);
                            let ty = ((yi - y_window.min) / y_span).clamp(0.0, 1.0);

                            let px_x = viewport.origin.x.0 + (tx as f32) * viewport.size.width.0;
                            let px_y =
                                viewport.origin.y.0 + (1.0 - (ty as f32)) * viewport.size.height.0;

                            let bucket = crate::visual_map::eval_bucket_for_value(
                                &vm,
                                selected_range,
                                selected_piece_mask,
                                vm_values.get(i).copied().unwrap_or(f64::NAN),
                            );
                            let bucket_key = bucket.bucket as usize
                                + if bucket.in_range {
                                    0
                                } else {
                                    vm.buckets as usize
                                };

                            build.points_by_bucket[bucket_key].push(Point::new(Px(px_x), Px(px_y)));
                            build.indices_by_bucket[bucket_key].push(i as u32);
                        }

                        build.next_view_index = chunk_end;
                    }

                    let required_marks = build
                        .points_by_bucket
                        .iter()
                        .filter(|p| !p.is_empty())
                        .count();
                    if required_marks > 0 {
                        if budget.take_marks(required_marks as u32) != required_marks as u32 {
                            return false;
                        }

                        let base_order = self.series_index as u32;
                        for (bucket_key, points) in build.points_by_bucket.iter_mut().enumerate() {
                            if points.is_empty() {
                                continue;
                            }
                            let indices = std::mem::take(&mut build.indices_by_bucket[bucket_key]);
                            let points = std::mem::take(points);
                            let range = marks
                                .arena
                                .extend_points_with_indices(points.into_iter(), indices);

                            let paint_bucket = (bucket_key % (vm.buckets as usize)) as u16;
                            let in_range = bucket_key < (vm.buckets as usize);
                            let radius_mul = vm.point_radius_mul_range.and_then(|(a, b)| {
                                let denom = (vm.buckets.saturating_sub(1) as f32).max(1.0);
                                let t = (paint_bucket as f32 / denom).clamp(0.0, 1.0);
                                let v = a + t * (b - a);
                                (v.is_finite() && v > 0.0).then_some(v)
                            });
                            let opacity_mul = crate::visual_map::opacity_mul_for_bucket(
                                &vm,
                                paint_bucket,
                                in_range,
                            );
                            let stroke =
                                crate::visual_map::stroke_width_for_bucket(&vm, paint_bucket).map(
                                    |width| {
                                        (
                                            crate::ids::PaintId::new(paint_bucket as u64),
                                            StrokeStyleV2 {
                                                width,
                                                ..StrokeStyleV2::default()
                                            },
                                        )
                                    },
                                );

                            marks.nodes.push(MarkNode {
                                id: series_mark_id(series.id, 16 + bucket_key as u64),
                                parent: None,
                                layer: crate::ids::LayerId(1),
                                order: MarkOrderKey(base_order.saturating_mul(2)),
                                kind: MarkKind::Points,
                                source_series: Some(series.id),
                                payload: MarkPayloadRef::Points(MarkPointsRef {
                                    points: range.clone(),
                                    fill: Some(crate::ids::PaintId::new(paint_bucket as u64)),
                                    opacity_mul,
                                    radius_mul,
                                    stroke,
                                }),
                            });

                            stats.points_emitted += (range.end - range.start) as u64;
                            stats.marks_emitted += 1;
                        }

                        marks.revision.bump();
                    }

                    self.series_index += 1;
                    self.cursor.next_index = 0;
                    self.scatter_next_index = 0;
                    self.scatter_points_start = 0;
                    self.scatter_node_index = None;
                    self.scatter_bucket_build = None;
                    self.bar_next_index = 0;
                    self.bar_rects_start = 0;
                    self.bar_node_index = None;
                    self.bar_bucket_build = None;
                    self.bounds = None;
                    scratch.clear();
                    continue;
                }

                if self.scatter_node_index.is_none() {
                    self.scatter_next_index = 0;
                    self.scatter_points_start = marks.arena.points.len();
                    let range = self.scatter_points_start..self.scatter_points_start;
                    let base_order = self.series_index as u32;
                    let id = series_mark_id(series.id, 0);

                    let mut created = false;
                    let node_index = if let Some(i) = marks.nodes.iter().position(|n| n.id == id) {
                        i
                    } else {
                        if budget.take_marks(1) == 0 {
                            return false;
                        }
                        created = true;
                        marks.nodes.push(MarkNode {
                            id,
                            parent: None,
                            layer: crate::ids::LayerId(1),
                            order: MarkOrderKey(base_order.saturating_mul(2)),
                            kind: MarkKind::Points,
                            source_series: Some(series.id),
                            payload: MarkPayloadRef::Points(MarkPointsRef {
                                points: range.clone(),
                                fill: None,
                                opacity_mul: None,
                                radius_mul: None,
                                stroke: None,
                            }),
                        });
                        marks.nodes.len() - 1
                    };

                    if let Some(node) = marks.nodes.get_mut(node_index) {
                        let MarkPayloadRef::Points(p) = &mut node.payload else {
                            return false;
                        };
                        p.points = range;
                        p.fill = None;
                        p.opacity_mul = None;
                        p.radius_mul = None;
                        p.stroke = None;
                    }

                    self.scatter_node_index = Some(node_index);
                    marks.revision.bump();
                    if created {
                        stats.marks_emitted += 1;
                    }
                }

                let row_end = selection.view_len(table.row_count());
                while self.scatter_next_index < row_end {
                    let points_budget = budget.take_points(4096) as usize;
                    if points_budget == 0 {
                        return false;
                    }

                    let chunk_end = (self.scatter_next_index + points_budget).min(row_end);
                    let start_len = marks.arena.points.len();

                    let x_span = x_window.span();
                    let y_span = y_window.span();
                    let x_span = if x_span.is_finite() && x_span > 0.0 {
                        x_span
                    } else {
                        1.0
                    };
                    let y_span = if y_span.is_finite() && y_span > 0.0 {
                        y_span
                    } else {
                        1.0
                    };

                    let data_len = x.len().min(y0.len());
                    for view_index in self.scatter_next_index..chunk_end {
                        let Some(i) = selection.get_raw_index(data_len, view_index) else {
                            continue;
                        };

                        let xi = x.get(i).copied().unwrap_or(f64::NAN);
                        let yi = y0.get(i).copied().unwrap_or(f64::NAN);
                        if !xi.is_finite() || !yi.is_finite() {
                            continue;
                        }
                        if (empty_mask.x_active || empty_mask.y_active)
                            && !empty_mask.allows_raw_index(i, x, y0, None)
                        {
                            continue;
                        }
                        if xi < x_window.min || xi > x_window.max {
                            continue;
                        }
                        if yi < y_window.min || yi > y_window.max {
                            continue;
                        }

                        let tx = ((xi - x_window.min) / x_span).clamp(0.0, 1.0);
                        let ty = ((yi - y_window.min) / y_span).clamp(0.0, 1.0);

                        let px_x = viewport.origin.x.0 + (tx as f32) * viewport.size.width.0;
                        let px_y =
                            viewport.origin.y.0 + (1.0 - (ty as f32)) * viewport.size.height.0;

                        marks.arena.points.push(Point::new(Px(px_x), Px(px_y)));
                        marks.arena.data_indices.push(i as u32);
                    }

                    if let Some(node_index) = self.scatter_node_index
                        && let Some(node) = marks.nodes.get_mut(node_index)
                    {
                        let MarkPayloadRef::Points(p) = &mut node.payload else {
                            return false;
                        };
                        p.points.end = marks.arena.points.len();
                    }

                    marks.revision.bump();
                    stats.points_emitted +=
                        marks.arena.points.len().saturating_sub(start_len) as u64;
                    self.scatter_next_index = chunk_end;
                }

                self.series_index += 1;
                self.cursor.next_index = 0;
                self.scatter_next_index = 0;
                self.scatter_points_start = 0;
                self.scatter_node_index = None;
                self.scatter_bucket_build = None;
                self.bar_next_index = 0;
                self.bar_rects_start = 0;
                self.bar_node_index = None;
                self.bar_bucket_build = None;
                self.bounds = None;
                scratch.clear();
                continue;
            }

            if series.kind == crate::spec::SeriesKind::Bar {
                let Some(mapping) = crate::engine::bar::bar_mapping_for_series(model, series.id)
                else {
                    return false;
                };
                let is_vertical =
                    mapping.orientation == crate::engine::bar::BarOrientation::Vertical;

                let stack_arrays = series.stack.and_then(|stack| {
                    stack_dims.stack_arrays(stack, series.id, model.revs.marks, table.revision())
                });
                if series.stack.is_some() && stack_arrays.is_none() {
                    return false;
                }

                let Some(layout) = bar_layout.layout_for_series(model, series.id) else {
                    return false;
                };

                let (category_window, value_window) = if is_vertical {
                    let x_axis = model.axes.get(&series.x_axis);
                    let x_domain = x_axis.and_then(crate::engine::axis::category_domain_window);
                    let x_window = view_x_mapping_window.or(x_domain).unwrap_or(DataWindow {
                        min: bounds.x_min,
                        max: bounds.x_max,
                    });
                    let mut category_window = x_window;
                    category_window.clamp_non_degenerate();

                    let mut value_window = DataWindow {
                        min: bounds.y_min,
                        max: bounds.y_max,
                    };
                    value_window.clamp_non_degenerate();

                    (category_window, value_window)
                } else {
                    let y_axis = model.axes.get(&series.y_axis);
                    let y_domain = y_axis.and_then(crate::engine::axis::category_domain_window);
                    let y_window = y_domain.unwrap_or(DataWindow {
                        min: bounds.y_min,
                        max: bounds.y_max,
                    });
                    let mut category_window = y_window;
                    category_window.clamp_non_degenerate();

                    let x_window = view_x_mapping_window.unwrap_or(DataWindow {
                        min: bounds.x_min,
                        max: bounds.x_max,
                    });
                    let mut value_window = x_window;
                    value_window.clamp_non_degenerate();

                    (category_window, value_window)
                };

                if let Some(visual_map_id) = model.visual_map_by_series.get(&series.id).copied() {
                    let Some(vm) = model.visual_maps.get(&visual_map_id).copied() else {
                        return false;
                    };
                    let selected_range = state
                        .visual_map_range
                        .get(&visual_map_id)
                        .copied()
                        .unwrap_or(vm.initial_range);
                    let selected_piece_mask = state
                        .visual_map_piece_mask
                        .get(&visual_map_id)
                        .copied()
                        .unwrap_or(vm.initial_piece_mask);

                    let Some(dataset) = model.datasets.get(&series.dataset) else {
                        return false;
                    };
                    let Some(vm_col) = dataset.fields.get(&vm.field).copied() else {
                        return false;
                    };
                    let Some(vm_values) = table.column_f64(vm_col) else {
                        return false;
                    };

                    let total_buckets = vm.buckets as usize * 2;
                    let rebuild = match self.bar_bucket_build.as_ref() {
                        Some(b) => {
                            b.series != series.id
                                || b.visual_map != visual_map_id
                                || b.bucket_count != vm.buckets
                        }
                        None => true,
                    };
                    if rebuild {
                        self.bar_bucket_build = Some(BarBucketBuild {
                            series: series.id,
                            visual_map: visual_map_id,
                            bucket_count: vm.buckets,
                            next_view_index: 0,
                            rects_by_bucket: vec![Vec::default(); total_buckets],
                            indices_by_bucket: vec![Vec::default(); total_buckets],
                        });
                    }
                    let Some(build) = self.bar_bucket_build.as_mut() else {
                        return false;
                    };

                    let row_end = selection.view_len(table.row_count());
                    while build.next_view_index < row_end {
                        let points_budget = budget.take_points(4096) as usize;
                        if points_budget == 0 {
                            return false;
                        }

                        let chunk_end = (build.next_view_index + points_budget).min(row_end);

                        let cat_span = category_window.span();
                        let band_px = if is_vertical {
                            (viewport.size.width.0 as f64 / cat_span.max(1.0)).max(1.0)
                        } else {
                            (viewport.size.height.0 as f64 / cat_span.max(1.0)).max(1.0)
                        };
                        let mut bar_thickness_px = (layout.width_cat * band_px).max(1.0) as f32;
                        let mut slot_offset_px: Option<f64> = None;

                        if let Some(BarWidthSpec::Px(px)) = layout.bar_width {
                            let slot_count = layout.slot_count.max(1) as usize;
                            let slot_index = (layout.slot_index as usize).min(slot_count - 1);
                            let group_width_px =
                                (1.0 - layout.bar_category_gap).clamp(0.0, 1.0) * band_px;
                            let mut bar_w_px = px.max(1.0);
                            let mut gap_px = bar_w_px * layout.bar_gap;
                            let mut total_px = (slot_count as f64) * bar_w_px
                                + (slot_count.saturating_sub(1) as f64) * gap_px;

                            if group_width_px.is_finite()
                                && group_width_px > 0.0
                                && total_px > group_width_px
                            {
                                let scale = group_width_px / total_px;
                                bar_w_px *= scale;
                                gap_px *= scale;
                                total_px = group_width_px;
                            }

                            let group_left_px = -0.5 * total_px;
                            let slot_left_px =
                                group_left_px + (slot_index as f64) * (bar_w_px + gap_px);
                            slot_offset_px = Some(slot_left_px + 0.5 * bar_w_px);
                            bar_thickness_px = bar_w_px.max(1.0) as f32;
                        }

                        let data_len = x.len().min(y0.len()).min(vm_values.len());
                        for view_index in build.next_view_index..chunk_end {
                            let Some(i) = selection.get_raw_index(data_len, view_index) else {
                                continue;
                            };

                            let xi = x.get(i).copied().unwrap_or(f64::NAN);
                            let yi = y0.get(i).copied().unwrap_or(f64::NAN);
                            if !xi.is_finite() || !yi.is_finite() {
                                continue;
                            }
                            if !view_x_filter.contains(xi) {
                                continue;
                            }

                            let (cat_v, value_v0) = if is_vertical { (xi, yi) } else { (yi, xi) };
                            if cat_v < category_window.min || cat_v > category_window.max {
                                continue;
                            }

                            let (value_top, value_base) = if let Some(arrays) = &stack_arrays {
                                let base = arrays.base.get(i).copied().unwrap_or(f64::NAN);
                                let top = arrays.stacked.get(i).copied().unwrap_or(f64::NAN);
                                (top, base)
                            } else {
                                (value_v0, 0.0)
                            };
                            if !value_top.is_finite() || !value_base.is_finite() {
                                continue;
                            }

                            let cat_span = category_window.span();
                            let val_span = value_window.span();
                            let t_cat = ((cat_v - category_window.min) / cat_span).clamp(0.0, 1.0);
                            let t_val = ((value_top - value_window.min) / val_span).clamp(0.0, 1.0);
                            let t_val0 =
                                ((value_base - value_window.min) / val_span).clamp(0.0, 1.0);

                            let rect = if is_vertical {
                                let mut px_x =
                                    viewport.origin.x.0 + (t_cat as f32) * viewport.size.width.0;
                                if let Some(offset_px) = slot_offset_px {
                                    px_x += offset_px as f32;
                                } else {
                                    let x_center = cat_v + layout.offset_cat;
                                    let t_center = ((x_center - category_window.min) / cat_span)
                                        .clamp(0.0, 1.0);
                                    px_x = viewport.origin.x.0
                                        + (t_center as f32) * viewport.size.width.0;
                                }

                                let px_y = viewport.origin.y.0
                                    + (1.0 - (t_val as f32)) * viewport.size.height.0;
                                let px_y0 = viewport.origin.y.0
                                    + (1.0 - (t_val0 as f32)) * viewport.size.height.0;

                                let top = px_y.min(px_y0);
                                let bottom = px_y.max(px_y0);
                                let h = (bottom - top).max(1.0);

                                Rect::new(
                                    Point::new(Px(px_x - 0.5 * bar_thickness_px), Px(top)),
                                    fret_core::Size::new(Px(bar_thickness_px), Px(h)),
                                )
                            } else {
                                let mut px_y = viewport.origin.y.0
                                    + (1.0 - (t_cat as f32)) * viewport.size.height.0;
                                if let Some(offset_px) = slot_offset_px {
                                    px_y -= offset_px as f32;
                                } else {
                                    let y_center = cat_v + layout.offset_cat;
                                    let t_center = ((y_center - category_window.min) / cat_span)
                                        .clamp(0.0, 1.0);
                                    px_y = viewport.origin.y.0
                                        + (1.0 - (t_center as f32)) * viewport.size.height.0;
                                }

                                let px_x =
                                    viewport.origin.x.0 + (t_val as f32) * viewport.size.width.0;
                                let px_x0 =
                                    viewport.origin.x.0 + (t_val0 as f32) * viewport.size.width.0;
                                let left = px_x.min(px_x0);
                                let right = px_x.max(px_x0);
                                let w = (right - left).max(1.0);

                                Rect::new(
                                    Point::new(Px(left), Px(px_y - 0.5 * bar_thickness_px)),
                                    fret_core::Size::new(Px(w), Px(bar_thickness_px)),
                                )
                            };

                            let bucket = crate::visual_map::eval_bucket_for_value(
                                &vm,
                                selected_range,
                                selected_piece_mask,
                                vm_values.get(i).copied().unwrap_or(f64::NAN),
                            );
                            let bucket_key = bucket.bucket as usize
                                + if bucket.in_range {
                                    0
                                } else {
                                    vm.buckets as usize
                                };

                            build.rects_by_bucket[bucket_key].push(rect);
                            build.indices_by_bucket[bucket_key].push(i as u32);
                        }

                        build.next_view_index = chunk_end;
                    }

                    let required_marks = build
                        .rects_by_bucket
                        .iter()
                        .filter(|r| !r.is_empty())
                        .count();
                    if required_marks > 0 {
                        if budget.take_marks(required_marks as u32) != required_marks as u32 {
                            return false;
                        }

                        let base_order = self.series_index as u32;
                        for (bucket_key, rects) in build.rects_by_bucket.iter_mut().enumerate() {
                            if rects.is_empty() {
                                continue;
                            }

                            let indices = std::mem::take(&mut build.indices_by_bucket[bucket_key]);
                            let rects = std::mem::take(rects);
                            let range = marks
                                .arena
                                .extend_rects_with_indices(rects.into_iter(), indices);

                            let paint_bucket = (bucket_key % (vm.buckets as usize)) as u16;
                            let in_range = bucket_key < (vm.buckets as usize);
                            let opacity_mul = crate::visual_map::opacity_mul_for_bucket(
                                &vm,
                                paint_bucket,
                                in_range,
                            );
                            let stroke =
                                crate::visual_map::stroke_width_for_bucket(&vm, paint_bucket).map(
                                    |width| {
                                        (
                                            crate::ids::PaintId::new(paint_bucket as u64),
                                            StrokeStyleV2 {
                                                width,
                                                ..StrokeStyleV2::default()
                                            },
                                        )
                                    },
                                );

                            marks.nodes.push(MarkNode {
                                id: series_mark_id(series.id, 16 + bucket_key as u64),
                                parent: None,
                                layer: crate::ids::LayerId(1),
                                order: MarkOrderKey(base_order.saturating_mul(2)),
                                kind: MarkKind::Rect,
                                source_series: Some(series.id),
                                payload: MarkPayloadRef::Rect(MarkRectRef {
                                    rects: range.clone(),
                                    fill: Some(crate::ids::PaintId::new(paint_bucket as u64)),
                                    opacity_mul,
                                    stroke,
                                }),
                            });

                            stats.points_emitted += (range.end - range.start) as u64;
                            stats.marks_emitted += 1;
                        }

                        marks.revision.bump();
                    }

                    self.series_index += 1;
                    self.cursor.next_index = 0;
                    self.scatter_next_index = 0;
                    self.scatter_points_start = 0;
                    self.scatter_node_index = None;
                    self.scatter_bucket_build = None;
                    self.bar_next_index = 0;
                    self.bar_rects_start = 0;
                    self.bar_node_index = None;
                    self.bar_bucket_build = None;
                    self.bounds = None;
                    scratch.clear();
                    continue;
                }

                if self.bar_node_index.is_none() {
                    self.bar_next_index = 0;
                    self.bar_rects_start = marks.arena.rects.len();
                    let range = self.bar_rects_start..self.bar_rects_start;
                    let base_order = self.series_index as u32;
                    let id = series_mark_id(series.id, 0);

                    let mut created = false;
                    let node_index = if let Some(i) = marks.nodes.iter().position(|n| n.id == id) {
                        i
                    } else {
                        if budget.take_marks(1) == 0 {
                            return false;
                        }
                        created = true;
                        marks.nodes.push(MarkNode {
                            id,
                            parent: None,
                            layer: crate::ids::LayerId(1),
                            order: MarkOrderKey(base_order.saturating_mul(2)),
                            kind: MarkKind::Rect,
                            source_series: Some(series.id),
                            payload: MarkPayloadRef::Rect(MarkRectRef {
                                rects: range.clone(),
                                fill: None,
                                opacity_mul: None,
                                stroke: None,
                            }),
                        });
                        marks.nodes.len() - 1
                    };

                    if let Some(node) = marks.nodes.get_mut(node_index) {
                        let MarkPayloadRef::Rect(r) = &mut node.payload else {
                            return false;
                        };
                        r.rects = range;
                        r.fill = None;
                        r.opacity_mul = None;
                        r.stroke = None;
                    }

                    self.bar_node_index = Some(node_index);
                    marks.revision.bump();
                    if created {
                        stats.marks_emitted += 1;
                    }
                }

                let row_end = selection.view_len(table.row_count());
                while self.bar_next_index < row_end {
                    let points_budget = budget.take_points(4096) as usize;
                    if points_budget == 0 {
                        return false;
                    }

                    let chunk_end = (self.bar_next_index + points_budget).min(row_end);

                    let cat_span = category_window.span();
                    let band_px = if is_vertical {
                        (viewport.size.width.0 as f64 / cat_span.max(1.0)).max(1.0)
                    } else {
                        (viewport.size.height.0 as f64 / cat_span.max(1.0)).max(1.0)
                    };
                    let mut bar_thickness_px = (layout.width_cat * band_px).max(1.0) as f32;
                    let mut slot_offset_px: Option<f64> = None;

                    if let Some(BarWidthSpec::Px(px)) = layout.bar_width {
                        let slot_count = layout.slot_count.max(1) as usize;
                        let slot_index = (layout.slot_index as usize).min(slot_count - 1);
                        let group_width_px =
                            (1.0 - layout.bar_category_gap).clamp(0.0, 1.0) * band_px;
                        let mut bar_w_px = px.max(1.0);
                        let mut gap_px = bar_w_px * layout.bar_gap;
                        let mut total_px = (slot_count as f64) * bar_w_px
                            + (slot_count.saturating_sub(1) as f64) * gap_px;

                        if group_width_px.is_finite()
                            && group_width_px > 0.0
                            && total_px > group_width_px
                        {
                            let scale = group_width_px / total_px;
                            bar_w_px *= scale;
                            gap_px *= scale;
                            total_px = group_width_px;
                        }

                        let group_left_px = -0.5 * total_px;
                        let slot_left_px =
                            group_left_px + (slot_index as f64) * (bar_w_px + gap_px);
                        slot_offset_px = Some(slot_left_px + 0.5 * bar_w_px);
                        bar_thickness_px = bar_w_px.max(1.0) as f32;
                    }

                    let mut rects = Vec::new();
                    let mut indices = Vec::new();
                    rects.reserve(chunk_end - self.bar_next_index);
                    indices.reserve(chunk_end - self.bar_next_index);

                    let data_len = x.len().min(y0.len());
                    for view_index in self.bar_next_index..chunk_end {
                        let Some(i) = selection.get_raw_index(data_len, view_index) else {
                            continue;
                        };

                        let xi = x.get(i).copied().unwrap_or(f64::NAN);
                        let yi = y0.get(i).copied().unwrap_or(f64::NAN);
                        if !xi.is_finite() || !yi.is_finite() {
                            continue;
                        }
                        if (empty_mask.x_active || empty_mask.y_active)
                            && !empty_mask.allows_raw_index(i, x, y0, None)
                        {
                            continue;
                        }
                        if !view_x_filter.contains(xi) {
                            continue;
                        }

                        let (cat_v, value_v0) = if is_vertical { (xi, yi) } else { (yi, xi) };
                        if cat_v < category_window.min || cat_v > category_window.max {
                            continue;
                        }

                        let (value_top, value_base) = if let Some(arrays) = &stack_arrays {
                            let base = arrays.base.get(i).copied().unwrap_or(f64::NAN);
                            let top = arrays.stacked.get(i).copied().unwrap_or(f64::NAN);
                            (top, base)
                        } else {
                            (value_v0, 0.0)
                        };
                        if !value_top.is_finite() || !value_base.is_finite() {
                            continue;
                        }

                        let cat_span = category_window.span();
                        let val_span = value_window.span();
                        let t_cat = ((cat_v - category_window.min) / cat_span).clamp(0.0, 1.0);
                        let t_val = ((value_top - value_window.min) / val_span).clamp(0.0, 1.0);
                        let t_val0 = ((value_base - value_window.min) / val_span).clamp(0.0, 1.0);

                        if is_vertical {
                            let mut px_x =
                                viewport.origin.x.0 + (t_cat as f32) * viewport.size.width.0;
                            if let Some(offset_px) = slot_offset_px {
                                px_x += offset_px as f32;
                            } else {
                                let x_center = cat_v + layout.offset_cat;
                                let t_center =
                                    ((x_center - category_window.min) / cat_span).clamp(0.0, 1.0);
                                px_x =
                                    viewport.origin.x.0 + (t_center as f32) * viewport.size.width.0;
                            }

                            let px_y = viewport.origin.y.0
                                + (1.0 - (t_val as f32)) * viewport.size.height.0;
                            let px_y0 = viewport.origin.y.0
                                + (1.0 - (t_val0 as f32)) * viewport.size.height.0;

                            let top = px_y.min(px_y0);
                            let bottom = px_y.max(px_y0);
                            let h = (bottom - top).max(1.0);

                            rects.push(Rect::new(
                                Point::new(Px(px_x - 0.5 * bar_thickness_px), Px(top)),
                                fret_core::Size::new(Px(bar_thickness_px), Px(h)),
                            ));
                        } else {
                            let mut px_y = viewport.origin.y.0
                                + (1.0 - (t_cat as f32)) * viewport.size.height.0;
                            if let Some(offset_px) = slot_offset_px {
                                px_y -= offset_px as f32;
                            } else {
                                let y_center = cat_v + layout.offset_cat;
                                let t_center =
                                    ((y_center - category_window.min) / cat_span).clamp(0.0, 1.0);
                                px_y = viewport.origin.y.0
                                    + (1.0 - (t_center as f32)) * viewport.size.height.0;
                            }

                            let px_x = viewport.origin.x.0 + (t_val as f32) * viewport.size.width.0;
                            let px_x0 =
                                viewport.origin.x.0 + (t_val0 as f32) * viewport.size.width.0;
                            let left = px_x.min(px_x0);
                            let right = px_x.max(px_x0);
                            let w = (right - left).max(1.0);

                            rects.push(Rect::new(
                                Point::new(Px(left), Px(px_y - 0.5 * bar_thickness_px)),
                                fret_core::Size::new(Px(w), Px(bar_thickness_px)),
                            ));
                        }
                        indices.push(i as u32);
                    }

                    if !rects.is_empty() {
                        let range = marks.arena.extend_rects_with_indices(rects, indices);
                        if let Some(node_index) = self.bar_node_index
                            && let Some(node) = marks.nodes.get_mut(node_index)
                        {
                            let MarkPayloadRef::Rect(r) = &mut node.payload else {
                                return false;
                            };
                            r.rects.end = range.end;
                        }
                        marks.revision.bump();
                        stats.points_emitted += (range.end - range.start) as u64;
                    }

                    self.bar_next_index = chunk_end;
                }

                self.series_index += 1;
                self.cursor.next_index = 0;
                self.scatter_next_index = 0;
                self.scatter_points_start = 0;
                self.scatter_node_index = None;
                self.scatter_bucket_build = None;
                self.bar_next_index = 0;
                self.bar_rects_start = 0;
                self.bar_node_index = None;
                self.bar_bucket_build = None;
                self.bounds = None;
                scratch.clear();
                continue;
            }

            let stack_arrays = series.stack.and_then(|stack| {
                stack_dims.stack_arrays(stack, series.id, model.revs.marks, table.revision())
            });
            if series.stack.is_some() && stack_arrays.is_none() {
                return false;
            }

            if matches!(
                series.kind,
                crate::spec::SeriesKind::Line
                    | crate::spec::SeriesKind::Area
                    | crate::spec::SeriesKind::Band
            ) {
                // v1 policy: `Empty` masking is implemented via mark-level segmentation for line-family series.
                // For now, apply Y-empty only for non-stacked series (stacked empty semantics are TBD).
                let y_empty_active = empty_mask.y_active;
                let break_on_out_of_window = empty_mask.x_active;
                if self.segmented_series != Some(series.id) {
                    self.segmented_series = Some(series.id);
                    self.segmented_cursor =
                        crate::engine::lod::SegmentedMinMaxPerPixelCursor::default();
                    self.segmented_segment_index = 0;
                    scratch.reset_buckets();
                }

                let end_limit = selection.view_len(x.len());

                let stroke = Some((crate::ids::PaintId(0), StrokeStyleV2::default()));
                let base_order = self.series_index as u32;

                loop {
                    if self.segmented_cursor.next_index >= end_limit {
                        break;
                    }

                    let points_budget = budget.take_points(4096) as usize;
                    if points_budget == 0 {
                        return false;
                    }

                    let step = if series.kind == crate::spec::SeriesKind::Band {
                        let Some(y1) = y1 else {
                            return false;
                        };
                        minmax_per_pixel_step_segmented_with(
                            &mut self.segmented_cursor,
                            scratch,
                            x,
                            &bounds,
                            viewport,
                            &selection,
                            points_budget,
                            &mut marks.arena.points,
                            &mut marks.arena.data_indices,
                            |i| y0.get(i).copied().unwrap_or(f64::NAN),
                            |i, xi, yi| {
                                if break_on_out_of_window && !view_x_filter.contains(xi) {
                                    return true;
                                }
                                let y_upper = y1.get(i).copied().unwrap_or(f64::NAN);
                                if !y_upper.is_finite() {
                                    return true;
                                }
                                y_empty_active
                                    && !empty_mask.y_filter.intersects_interval(yi, y_upper)
                            },
                        )
                    } else if series.kind == crate::spec::SeriesKind::Area && series.stack.is_some()
                    {
                        let Some(arrays) = stack_arrays.as_ref() else {
                            return false;
                        };
                        minmax_per_pixel_step_segmented_with(
                            &mut self.segmented_cursor,
                            scratch,
                            x,
                            &bounds,
                            viewport,
                            &selection,
                            points_budget,
                            &mut marks.arena.points,
                            &mut marks.arena.data_indices,
                            |i| arrays.stacked.get(i).copied().unwrap_or(f64::NAN),
                            |i, xi, _yi| {
                                (break_on_out_of_window && !view_x_filter.contains(xi))
                                    || !arrays.base.get(i).copied().unwrap_or(f64::NAN).is_finite()
                            },
                        )
                    } else if let Some(arrays) = stack_arrays.as_ref() {
                        minmax_per_pixel_step_segmented_with(
                            &mut self.segmented_cursor,
                            scratch,
                            x,
                            &bounds,
                            viewport,
                            &selection,
                            points_budget,
                            &mut marks.arena.points,
                            &mut marks.arena.data_indices,
                            |i| arrays.stacked.get(i).copied().unwrap_or(f64::NAN),
                            |_, xi, _yi| break_on_out_of_window && !view_x_filter.contains(xi),
                        )
                    } else {
                        minmax_per_pixel_step_segmented_with(
                            &mut self.segmented_cursor,
                            scratch,
                            x,
                            &bounds,
                            viewport,
                            &selection,
                            points_budget,
                            &mut marks.arena.points,
                            &mut marks.arena.data_indices,
                            |i| y0.get(i).copied().unwrap_or(f64::NAN),
                            |_, xi, yi| {
                                (break_on_out_of_window && !view_x_filter.contains(xi))
                                    || (y_empty_active && !view_y_filter.contains(yi))
                            },
                        )
                    };

                    let Some(step) = step else {
                        continue;
                    };

                    let range_len = step.segment.end.saturating_sub(step.segment.start);
                    if range_len < 2 {
                        marks.arena.points.truncate(step.segment.start);
                        marks.arena.data_indices.truncate(step.segment.start);
                        if step.done {
                            break;
                        }
                        continue;
                    }

                    let seg = self.segmented_segment_index;
                    self.segmented_segment_index = self.segmented_segment_index.saturating_add(1);

                    let mut created = 0u64;

                    if series.kind == crate::spec::SeriesKind::Band {
                        let Some(y1) = y1 else {
                            return false;
                        };
                        let lower_range = step.segment.clone();

                        let start_upper = marks.arena.points.len();
                        let indices = scratch.tmp_indices_mut();
                        indices.clear();
                        indices.extend(
                            marks.arena.data_indices[lower_range.clone()]
                                .iter()
                                .copied()
                                .map(|i| i as usize),
                        );

                        marks.arena.points.reserve(indices.len());
                        marks.arena.data_indices.reserve(indices.len());

                        let x_span = bounds.x_max - bounds.x_min;
                        let y_span = bounds.y_max - bounds.y_min;
                        let x_span = if x_span.is_finite() && x_span > 0.0 {
                            x_span
                        } else {
                            1.0
                        };
                        let y_span = if y_span.is_finite() && y_span > 0.0 {
                            y_span
                        } else {
                            1.0
                        };

                        for &i in indices.iter() {
                            let xi = x.get(i).copied().unwrap_or(f64::NAN);
                            let yi = y1.get(i).copied().unwrap_or(f64::NAN);
                            if !xi.is_finite() || !yi.is_finite() {
                                continue;
                            }
                            let yi = yi.clamp(bounds.y_min, bounds.y_max);
                            let tx = ((xi - bounds.x_min) / x_span).clamp(0.0, 1.0);
                            let ty = ((yi - bounds.y_min) / y_span).clamp(0.0, 1.0);
                            let px_x = viewport.origin.x.0 + (tx as f32) * viewport.size.width.0;
                            let px_y =
                                viewport.origin.y.0 + (1.0 - (ty as f32)) * viewport.size.height.0;
                            marks.arena.points.push(Point::new(Px(px_x), Px(px_y)));
                            marks.arena.data_indices.push(i as u32);
                        }

                        let upper_range = start_upper..marks.arena.points.len();

                        let lower_id = series_mark_id(series.id, 1 + seg.saturating_mul(2));
                        let lower_index =
                            if let Some(i) = marks.nodes.iter().position(|n| n.id == lower_id) {
                                i
                            } else {
                                if budget.take_marks(1) == 0 {
                                    return false;
                                }
                                created += 1;
                                marks.nodes.push(MarkNode {
                                    id: lower_id,
                                    parent: None,
                                    layer: crate::ids::LayerId(1),
                                    order: MarkOrderKey(
                                        base_order
                                            .saturating_mul(2)
                                            .saturating_add((seg.saturating_mul(2)) as u32),
                                    ),
                                    kind: MarkKind::Polyline,
                                    source_series: Some(series.id),
                                    payload: MarkPayloadRef::Polyline(MarkPolylineRef {
                                        points: lower_range.clone(),
                                        stroke: stroke.clone(),
                                    }),
                                });
                                marks.nodes.len() - 1
                            };

                        let upper_id = series_mark_id(series.id, 2 + seg.saturating_mul(2));
                        let upper_index =
                            if let Some(i) = marks.nodes.iter().position(|n| n.id == upper_id) {
                                i
                            } else {
                                if budget.take_marks(1) == 0 {
                                    return false;
                                }
                                created += 1;
                                marks.nodes.push(MarkNode {
                                    id: upper_id,
                                    parent: None,
                                    layer: crate::ids::LayerId(1),
                                    order: MarkOrderKey(
                                        base_order
                                            .saturating_mul(2)
                                            .saturating_add((seg.saturating_mul(2)) as u32)
                                            .saturating_add(1),
                                    ),
                                    kind: MarkKind::Polyline,
                                    source_series: Some(series.id),
                                    payload: MarkPayloadRef::Polyline(MarkPolylineRef {
                                        points: upper_range.clone(),
                                        stroke: stroke.clone(),
                                    }),
                                });
                                marks.nodes.len() - 1
                            };

                        if let Some(node) = marks.nodes.get_mut(lower_index) {
                            let MarkPayloadRef::Polyline(p) = &mut node.payload else {
                                return false;
                            };
                            p.points = lower_range.clone();
                            p.stroke.clone_from(&stroke);
                        }
                        if let Some(node) = marks.nodes.get_mut(upper_index) {
                            let MarkPayloadRef::Polyline(p) = &mut node.payload else {
                                return false;
                            };
                            p.points = upper_range.clone();
                            p.stroke.clone_from(&stroke);
                        }

                        stats.points_emitted += (lower_range.end - lower_range.start) as u64;
                        stats.points_emitted += (upper_range.end - upper_range.start) as u64;
                    } else if series.kind == crate::spec::SeriesKind::Area && series.stack.is_some()
                    {
                        let Some(arrays) = stack_arrays.as_ref() else {
                            return false;
                        };

                        let upper_range = step.segment.clone();
                        let start_lower = marks.arena.points.len();

                        let indices = scratch.tmp_indices_mut();
                        indices.clear();
                        indices.extend(
                            marks.arena.data_indices[upper_range.clone()]
                                .iter()
                                .copied()
                                .map(|i| i as usize),
                        );

                        marks.arena.points.reserve(indices.len());
                        marks.arena.data_indices.reserve(indices.len());

                        let x_span = bounds.x_max - bounds.x_min;
                        let y_span = bounds.y_max - bounds.y_min;
                        let x_span = if x_span.is_finite() && x_span > 0.0 {
                            x_span
                        } else {
                            1.0
                        };
                        let y_span = if y_span.is_finite() && y_span > 0.0 {
                            y_span
                        } else {
                            1.0
                        };

                        for &i in indices.iter() {
                            let xi = x.get(i).copied().unwrap_or(f64::NAN);
                            let yi = arrays.base.get(i).copied().unwrap_or(f64::NAN);
                            if !xi.is_finite() || !yi.is_finite() {
                                continue;
                            }
                            let yi = yi.clamp(bounds.y_min, bounds.y_max);
                            let tx = ((xi - bounds.x_min) / x_span).clamp(0.0, 1.0);
                            let ty = ((yi - bounds.y_min) / y_span).clamp(0.0, 1.0);
                            let px_x = viewport.origin.x.0 + (tx as f32) * viewport.size.width.0;
                            let px_y =
                                viewport.origin.y.0 + (1.0 - (ty as f32)) * viewport.size.height.0;
                            marks.arena.points.push(Point::new(Px(px_x), Px(px_y)));
                            marks.arena.data_indices.push(i as u32);
                        }

                        let lower_range = start_lower..marks.arena.points.len();

                        let lower_id = series_mark_id(series.id, 1 + seg.saturating_mul(2));
                        let lower_index =
                            if let Some(i) = marks.nodes.iter().position(|n| n.id == lower_id) {
                                i
                            } else {
                                if budget.take_marks(1) == 0 {
                                    return false;
                                }
                                created += 1;
                                marks.nodes.push(MarkNode {
                                    id: lower_id,
                                    parent: None,
                                    layer: crate::ids::LayerId(1),
                                    order: MarkOrderKey(
                                        base_order
                                            .saturating_mul(2)
                                            .saturating_add((seg.saturating_mul(2)) as u32),
                                    ),
                                    kind: MarkKind::Polyline,
                                    source_series: Some(series.id),
                                    payload: MarkPayloadRef::Polyline(MarkPolylineRef {
                                        points: lower_range.clone(),
                                        stroke: stroke.clone(),
                                    }),
                                });
                                marks.nodes.len() - 1
                            };

                        let upper_id = series_mark_id(series.id, 2 + seg.saturating_mul(2));
                        let upper_index =
                            if let Some(i) = marks.nodes.iter().position(|n| n.id == upper_id) {
                                i
                            } else {
                                if budget.take_marks(1) == 0 {
                                    return false;
                                }
                                created += 1;
                                marks.nodes.push(MarkNode {
                                    id: upper_id,
                                    parent: None,
                                    layer: crate::ids::LayerId(1),
                                    order: MarkOrderKey(
                                        base_order
                                            .saturating_mul(2)
                                            .saturating_add((seg.saturating_mul(2)) as u32)
                                            .saturating_add(1),
                                    ),
                                    kind: MarkKind::Polyline,
                                    source_series: Some(series.id),
                                    payload: MarkPayloadRef::Polyline(MarkPolylineRef {
                                        points: upper_range.clone(),
                                        stroke: stroke.clone(),
                                    }),
                                });
                                marks.nodes.len() - 1
                            };

                        if let Some(node) = marks.nodes.get_mut(lower_index) {
                            let MarkPayloadRef::Polyline(p) = &mut node.payload else {
                                return false;
                            };
                            p.points = lower_range.clone();
                            p.stroke.clone_from(&stroke);
                        }
                        if let Some(node) = marks.nodes.get_mut(upper_index) {
                            let MarkPayloadRef::Polyline(p) = &mut node.payload else {
                                return false;
                            };
                            p.points = upper_range.clone();
                            p.stroke.clone_from(&stroke);
                        }

                        stats.points_emitted += (lower_range.end - lower_range.start) as u64;
                        stats.points_emitted += (upper_range.end - upper_range.start) as u64;
                    } else {
                        let id = series_mark_id(series.id, seg);
                        let node_index = marks.nodes.iter().position(|n| n.id == id);
                        let node_index = if let Some(i) = node_index {
                            i
                        } else {
                            if budget.take_marks(1) == 0 {
                                return false;
                            }
                            created += 1;
                            marks.nodes.push(MarkNode {
                                id,
                                parent: None,
                                layer: crate::ids::LayerId(1),
                                order: MarkOrderKey(
                                    base_order.saturating_mul(2).saturating_add(seg as u32),
                                ),
                                kind: MarkKind::Polyline,
                                source_series: Some(series.id),
                                payload: MarkPayloadRef::Polyline(MarkPolylineRef {
                                    points: step.segment.clone(),
                                    stroke: stroke.clone(),
                                }),
                            });
                            marks.nodes.len() - 1
                        };

                        if let Some(node) = marks.nodes.get_mut(node_index) {
                            let MarkPayloadRef::Polyline(p) = &mut node.payload else {
                                return false;
                            };
                            p.points = step.segment.clone();
                            p.stroke.clone_from(&stroke);
                        }

                        stats.points_emitted += range_len as u64;
                    }

                    stats.marks_emitted += created;
                    marks.revision.bump();

                    if step.done {
                        break;
                    }
                }

                self.series_index += 1;
                self.cursor.next_index = 0;
                self.segmented_cursor.next_index = 0;
                self.scatter_next_index = 0;
                self.scatter_points_start = 0;
                self.scatter_node_index = None;
                self.scatter_bucket_build = None;
                self.bar_next_index = 0;
                self.bar_rects_start = 0;
                self.bar_node_index = None;
                self.bar_bucket_build = None;
                self.bounds = None;
                scratch.clear();
                continue;
            }

            let mut finished_scan = false;

            let width_px = viewport.size.width.0.max(1.0).ceil() as usize;
            if self.append_rebuild_mode {
                if let Some(cache) = self.minmax_append_cache.get_mut(&series.id)
                    && table.row_count() >= cache.row_count
                    && cache.viewport_width_px == width_px
                    && cache.bounds == bounds
                    && self.cursor.next_index == 0
                {
                    self.cursor = cache.cursor.clone();
                    std::mem::swap(scratch, &mut cache.scratch);
                }
            }
            while !finished_scan {
                let points_budget = budget.take_points(4096) as usize;
                if points_budget == 0 {
                    return false;
                }

                if let Some(arrays) = &stack_arrays {
                    finished_scan = minmax_per_pixel_step_selection_with(
                        &mut self.cursor,
                        scratch,
                        x,
                        &bounds,
                        viewport,
                        &selection,
                        points_budget,
                        |i| arrays.stacked.get(i).copied().unwrap_or(f64::NAN),
                    );
                } else {
                    finished_scan = minmax_per_pixel_step_selection(
                        &mut self.cursor,
                        scratch,
                        x,
                        y0,
                        &bounds,
                        viewport,
                        &selection,
                        points_budget,
                    );
                }
            }

            if !self.finalized {
                let range = minmax_per_pixel_finalize(
                    scratch,
                    x,
                    &bounds,
                    viewport,
                    &mut marks.arena.points,
                    &mut marks.arena.data_indices,
                );
                let range_len = (range.end - range.start) as u64;
                let stroke = Some((crate::ids::PaintId(0), StrokeStyleV2::default()));
                let base_order = self.series_index as u32;

                if series.kind == crate::spec::SeriesKind::Band
                    && let Some(y1) = y1
                {
                    let lower_range = range.clone();
                    let start_upper = marks.arena.points.len();

                    let x_span = bounds.x_max - bounds.x_min;
                    let y_span = bounds.y_max - bounds.y_min;
                    let x_span = if x_span.is_finite() && x_span > 0.0 {
                        x_span
                    } else {
                        1.0
                    };
                    let y_span = if y_span.is_finite() && y_span > 0.0 {
                        y_span
                    } else {
                        1.0
                    };

                    let indices = scratch.tmp_indices_mut();
                    indices.clear();
                    indices.extend(
                        marks.arena.data_indices[lower_range.clone()]
                            .iter()
                            .copied()
                            .map(|i| i as usize),
                    );

                    marks.arena.points.reserve(indices.len());
                    marks.arena.data_indices.reserve(indices.len());

                    for &i in indices.iter() {
                        let xi = x.get(i).copied().unwrap_or(f64::NAN);
                        let yi = y1.get(i).copied().unwrap_or(f64::NAN);
                        if !xi.is_finite() || !yi.is_finite() {
                            continue;
                        }

                        let yi = yi.clamp(bounds.y_min, bounds.y_max);
                        let tx = ((xi - bounds.x_min) / x_span).clamp(0.0, 1.0);
                        let ty = ((yi - bounds.y_min) / y_span).clamp(0.0, 1.0);

                        let px_x = viewport.origin.x.0 + (tx as f32) * viewport.size.width.0;
                        let px_y =
                            viewport.origin.y.0 + (1.0 - (ty as f32)) * viewport.size.height.0;

                        marks.arena.points.push(Point::new(Px(px_x), Px(px_y)));
                        marks.arena.data_indices.push(i as u32);
                    }

                    let upper_range = start_upper..marks.arena.points.len();

                    let mut created = 0u64;

                    let lower_id = series_mark_id(series.id, 1);
                    let lower_index =
                        if let Some(i) = marks.nodes.iter().position(|n| n.id == lower_id) {
                            i
                        } else {
                            if budget.take_marks(1) == 0 {
                                return false;
                            }
                            created += 1;
                            marks.nodes.push(MarkNode {
                                id: lower_id,
                                parent: None,
                                layer: crate::ids::LayerId(1),
                                order: MarkOrderKey(base_order.saturating_mul(2)),
                                kind: MarkKind::Polyline,
                                source_series: Some(series.id),
                                payload: MarkPayloadRef::Polyline(MarkPolylineRef {
                                    points: lower_range.clone(),
                                    stroke: stroke.clone(),
                                }),
                            });
                            marks.nodes.len() - 1
                        };

                    let upper_id = series_mark_id(series.id, 2);
                    let upper_index =
                        if let Some(i) = marks.nodes.iter().position(|n| n.id == upper_id) {
                            i
                        } else {
                            if budget.take_marks(1) == 0 {
                                return false;
                            }
                            created += 1;
                            marks.nodes.push(MarkNode {
                                id: upper_id,
                                parent: None,
                                layer: crate::ids::LayerId(1),
                                order: MarkOrderKey(base_order.saturating_mul(2).saturating_add(1)),
                                kind: MarkKind::Polyline,
                                source_series: Some(series.id),
                                payload: MarkPayloadRef::Polyline(MarkPolylineRef {
                                    points: upper_range.clone(),
                                    stroke: stroke.clone(),
                                }),
                            });
                            marks.nodes.len() - 1
                        };

                    if let Some(node) = marks.nodes.get_mut(lower_index) {
                        let MarkPayloadRef::Polyline(p) = &mut node.payload else {
                            return false;
                        };
                        p.points = lower_range.clone();
                        p.stroke.clone_from(&stroke);
                    }
                    if let Some(node) = marks.nodes.get_mut(upper_index) {
                        let MarkPayloadRef::Polyline(p) = &mut node.payload else {
                            return false;
                        };
                        p.points = upper_range.clone();
                        p.stroke.clone_from(&stroke);
                    }

                    stats.points_emitted += (lower_range.end - lower_range.start) as u64;
                    stats.points_emitted += (upper_range.end - upper_range.start) as u64;
                    stats.marks_emitted += created;
                    marks.revision.bump();
                } else if series.kind == crate::spec::SeriesKind::Area && series.stack.is_some() {
                    let upper_range = range.clone();
                    let start_lower = marks.arena.points.len();

                    let x_span = bounds.x_max - bounds.x_min;
                    let y_span = bounds.y_max - bounds.y_min;
                    let x_span = if x_span.is_finite() && x_span > 0.0 {
                        x_span
                    } else {
                        1.0
                    };
                    let y_span = if y_span.is_finite() && y_span > 0.0 {
                        y_span
                    } else {
                        1.0
                    };

                    let indices = scratch.tmp_indices_mut();
                    indices.clear();
                    indices.extend(
                        marks.arena.data_indices[upper_range.clone()]
                            .iter()
                            .copied()
                            .map(|i| i as usize),
                    );

                    marks.arena.points.reserve(indices.len());
                    marks.arena.data_indices.reserve(indices.len());

                    for &i in indices.iter() {
                        let xi = x.get(i).copied().unwrap_or(f64::NAN);
                        let yi = y0.get(i).copied().unwrap_or(f64::NAN);
                        if !xi.is_finite() || !yi.is_finite() {
                            continue;
                        }

                        let Some(arrays) = stack_arrays.as_ref() else {
                            return false;
                        };
                        let y_base = arrays.base.get(i).copied().unwrap_or(f64::NAN);
                        if !y_base.is_finite() {
                            continue;
                        }

                        let y_base = y_base.clamp(bounds.y_min, bounds.y_max);
                        let tx = ((xi - bounds.x_min) / x_span).clamp(0.0, 1.0);
                        let ty = ((y_base - bounds.y_min) / y_span).clamp(0.0, 1.0);

                        let px_x = viewport.origin.x.0 + (tx as f32) * viewport.size.width.0;
                        let px_y =
                            viewport.origin.y.0 + (1.0 - (ty as f32)) * viewport.size.height.0;

                        marks.arena.points.push(Point::new(Px(px_x), Px(px_y)));
                        marks.arena.data_indices.push(i as u32);
                    }

                    let lower_range = start_lower..marks.arena.points.len();

                    let mut created = 0u64;

                    let lower_id = series_mark_id(series.id, 1);
                    let lower_index =
                        if let Some(i) = marks.nodes.iter().position(|n| n.id == lower_id) {
                            i
                        } else {
                            if budget.take_marks(1) == 0 {
                                return false;
                            }
                            created += 1;
                            marks.nodes.push(MarkNode {
                                id: lower_id,
                                parent: None,
                                layer: crate::ids::LayerId(1),
                                order: MarkOrderKey(base_order.saturating_mul(2)),
                                kind: MarkKind::Polyline,
                                source_series: Some(series.id),
                                payload: MarkPayloadRef::Polyline(MarkPolylineRef {
                                    points: lower_range.clone(),
                                    stroke: stroke.clone(),
                                }),
                            });
                            marks.nodes.len() - 1
                        };

                    let upper_id = series_mark_id(series.id, 2);
                    let upper_index =
                        if let Some(i) = marks.nodes.iter().position(|n| n.id == upper_id) {
                            i
                        } else {
                            if budget.take_marks(1) == 0 {
                                return false;
                            }
                            created += 1;
                            marks.nodes.push(MarkNode {
                                id: upper_id,
                                parent: None,
                                layer: crate::ids::LayerId(1),
                                order: MarkOrderKey(base_order.saturating_mul(2).saturating_add(1)),
                                kind: MarkKind::Polyline,
                                source_series: Some(series.id),
                                payload: MarkPayloadRef::Polyline(MarkPolylineRef {
                                    points: upper_range.clone(),
                                    stroke: stroke.clone(),
                                }),
                            });
                            marks.nodes.len() - 1
                        };

                    if let Some(node) = marks.nodes.get_mut(lower_index) {
                        let MarkPayloadRef::Polyline(p) = &mut node.payload else {
                            return false;
                        };
                        p.points = lower_range.clone();
                        p.stroke.clone_from(&stroke);
                    }
                    if let Some(node) = marks.nodes.get_mut(upper_index) {
                        let MarkPayloadRef::Polyline(p) = &mut node.payload else {
                            return false;
                        };
                        p.points = upper_range.clone();
                        p.stroke.clone_from(&stroke);
                    }

                    stats.points_emitted += (lower_range.end - lower_range.start) as u64;
                    stats.points_emitted += range_len;
                    stats.marks_emitted += created;
                    marks.revision.bump();
                } else {
                    let id = series_mark_id(series.id, 0);
                    let node_index = marks.nodes.iter().position(|n| n.id == id);
                    let mut created = false;
                    let node_index = if let Some(i) = node_index {
                        i
                    } else {
                        if budget.take_marks(1) == 0 {
                            return false;
                        }
                        created = true;
                        marks.nodes.push(MarkNode {
                            id,
                            parent: None,
                            layer: crate::ids::LayerId(1),
                            order: MarkOrderKey(base_order.saturating_mul(2)),
                            kind: MarkKind::Polyline,
                            source_series: Some(series.id),
                            payload: MarkPayloadRef::Polyline(MarkPolylineRef {
                                points: range.clone(),
                                stroke: stroke.clone(),
                            }),
                        });
                        marks.nodes.len() - 1
                    };

                    if let Some(node) = marks.nodes.get_mut(node_index) {
                        let MarkPayloadRef::Polyline(p) = &mut node.payload else {
                            return false;
                        };
                        p.points = range.clone();
                        p.stroke.clone_from(&stroke);
                    }

                    stats.points_emitted += range_len;
                    if created {
                        stats.marks_emitted += 1;
                    }
                    marks.revision.bump();
                }
                self.finalized = true;
            }

            {
                let entry = self.minmax_append_cache.entry(series.id).or_default();
                entry.data_rev = table.revision();
                entry.row_count = table.row_count();
                entry.bounds = bounds;
                entry.viewport_width_px = width_px;
                entry.cursor = self.cursor.clone();
                std::mem::swap(scratch, &mut entry.scratch);
            }

            self.series_index += 1;
            self.cursor.next_index = 0;
            self.scatter_next_index = 0;
            self.scatter_points_start = 0;
            self.scatter_node_index = None;
            self.scatter_bucket_build = None;
            self.bar_next_index = 0;
            self.bar_rects_start = 0;
            self.bar_node_index = None;
            self.bar_bucket_build = None;
            self.bounds = None;
            scratch.clear();
        }

        self.append_rebuild_mode = false;
        true
    }
}

fn merge_axis_window(
    axis_windows: &mut BTreeMap<crate::ids::AxisId, DataWindow>,
    axis: crate::ids::AxisId,
    mut window: DataWindow,
) {
    window.clamp_non_degenerate();
    axis_windows
        .entry(axis)
        .and_modify(|w| {
            w.clamp_non_degenerate();
            w.min = w.min.min(window.min);
            w.max = w.max.max(window.max);
            w.clamp_non_degenerate();
        })
        .or_insert(window);
}

fn dataset_store_signature(model: &ChartModel, datasets: &DatasetStore) -> u64 {
    let mut hash = 1469598103934665603u64;
    hash = fnv1a_step(hash, model.series_order.len() as u64);
    for series_id in &model.series_order {
        let Some(series) = model.series.get(series_id) else {
            continue;
        };
        let dataset_id = series.dataset;
        hash = fnv1a_step(hash, dataset_id.0);
        if let Some(table) = datasets.dataset(model.root_dataset_id(dataset_id)) {
            hash = fnv1a_step(hash, table.revision().0);
            hash = fnv1a_step(hash, table.row_count() as u64);
            hash = fnv1a_step(hash, table.column_count() as u64);
        }
    }
    hash
}

fn fnv1a_step(hash: u64, value: u64) -> u64 {
    (hash ^ value).wrapping_mul(1099511628211u64)
}

fn compute_series_bounds(
    model: &ChartModel,
    state: &ChartState,
    kind: crate::spec::SeriesKind,
    x_axis: crate::ids::AxisId,
    y_axis: crate::ids::AxisId,
    x: &[f64],
    y0: &[f64],
    y1: Option<&[f64]>,
    selection: &RowSelection,
    x_filter: crate::engine::window_policy::AxisFilter1D,
    x_mapping_window: Option<DataWindowX>,
    initial_points_budget: usize,
    bounds_cursor: &mut BoundsCursor,
    bounds_accum: &mut BoundsAccum,
    budget: &mut WorkBudget,
) -> Option<DataBounds> {
    let x_domain_window = model
        .axes
        .get(&x_axis)
        .and_then(crate::engine::axis::category_domain_window);
    let y_domain_window = model
        .axes
        .get(&y_axis)
        .and_then(crate::engine::axis::category_domain_window);

    let y_axis_range = model.axes.get(&y_axis).map(|a| a.range).unwrap_or_default();
    let y_window_for_bounds =
        axis_locked_window_y(y_axis_range).or(state.data_window_y.get(&y_axis).copied());

    if let Some(mut y_window) = y_window_for_bounds {
        y_window.clamp_non_degenerate();

        let x_axis_range = model.axes.get(&x_axis).map(|a| a.range).unwrap_or_default();
        let x_window_for_bounds = axis_locked_window_1d(x_axis_range);

        let (x_min, x_max) =
            if let Some(mut w) = x_mapping_window.or(x_domain_window).or(x_window_for_bounds) {
                w.clamp_non_degenerate();
                (w.min, w.max)
            } else {
                let mut bounds = compute_bounds_in_selection_filtered(x, y0, selection, x_filter)
                    .unwrap_or_default();
                bounds.clamp_non_degenerate();
                (bounds.x_min, bounds.x_max)
            };

        let mut bounds = DataBounds {
            x_min,
            x_max,
            y_min: y_window.min,
            y_max: y_window.max,
        };
        apply_axis_constraints(model, x_axis, y_axis, &mut bounds);
        bounds.clamp_non_degenerate();
        return Some(bounds);
    }

    if kind == crate::spec::SeriesKind::Band
        && let Some(y1) = y1
    {
        let mut bounds0 =
            compute_bounds_in_selection_filtered(x, y0, selection, x_filter).unwrap_or_default();
        bounds0.clamp_non_degenerate();
        let mut bounds1 =
            compute_bounds_in_selection_filtered(x, y1, selection, x_filter).unwrap_or_default();
        bounds1.clamp_non_degenerate();

        let mut combined = DataBounds {
            x_min: bounds0.x_min.min(bounds1.x_min),
            x_max: bounds0.x_max.max(bounds1.x_max),
            y_min: bounds0.y_min.min(bounds1.y_min),
            y_max: bounds0.y_max.max(bounds1.y_max),
        };

        if let Some(mut w) = x_mapping_window.or(x_domain_window) {
            w.clamp_non_degenerate();
            combined.x_min = w.min;
            combined.x_max = w.max;
        }
        if let Some(mut w) = y_domain_window {
            w.clamp_non_degenerate();
            combined.y_min = w.min;
            combined.y_max = w.max;
        }

        apply_axis_constraints(model, x_axis, y_axis, &mut combined);
        combined.clamp_non_degenerate();
        return Some(combined);
    }

    let mut done = compute_bounds_step_selection(
        bounds_cursor,
        bounds_accum,
        x,
        y0,
        selection,
        x_filter,
        initial_points_budget,
    )
    .unwrap_or(true);

    while !done {
        let points_budget = budget.take_points(4096) as usize;
        if points_budget == 0 {
            return None;
        }
        done = compute_bounds_step_selection(
            bounds_cursor,
            bounds_accum,
            x,
            y0,
            selection,
            x_filter,
            points_budget,
        )
        .unwrap_or(true);
    }

    let mut bounds = finalize_bounds(bounds_accum).unwrap_or_default();
    if let Some(mut w) = x_mapping_window.or(x_domain_window) {
        w.clamp_non_degenerate();
        bounds.x_min = w.min;
        bounds.x_max = w.max;
    }
    if let Some(mut w) = y_domain_window {
        w.clamp_non_degenerate();
        bounds.y_min = w.min;
        bounds.y_max = w.max;
    }
    if kind == crate::spec::SeriesKind::Bar {
        let x_is_category = model
            .axes
            .get(&x_axis)
            .is_some_and(|a| matches!(a.scale, crate::scale::AxisScale::Category(_)));
        let y_is_category = model
            .axes
            .get(&y_axis)
            .is_some_and(|a| matches!(a.scale, crate::scale::AxisScale::Category(_)));

        // ECharts-like bar baseline:
        // - Vertical bars: include 0 on Y (value axis).
        // - Horizontal bars: include 0 on X (value axis).
        if x_is_category && !y_is_category {
            bounds.y_min = bounds.y_min.min(0.0);
            bounds.y_max = bounds.y_max.max(0.0);
        } else if !x_is_category && y_is_category {
            bounds.x_min = bounds.x_min.min(0.0);
            bounds.x_max = bounds.x_max.max(0.0);
        } else {
            bounds.y_min = bounds.y_min.min(0.0);
            bounds.y_max = bounds.y_max.max(0.0);
        }
    }
    apply_axis_constraints(model, x_axis, y_axis, &mut bounds);
    bounds.clamp_non_degenerate();
    Some(bounds)
}

fn compute_bounds_in_selection_filtered(
    x: &[f64],
    y: &[f64],
    selection: &RowSelection,
    filter: crate::engine::window_policy::AxisFilter1D,
) -> Option<DataBounds> {
    let len = x.len().min(y.len());
    let end_limit = selection.view_len(len);
    if end_limit == 0 {
        return None;
    }

    let mut bounds = DataBounds {
        x_min: f64::INFINITY,
        x_max: f64::NEG_INFINITY,
        y_min: f64::INFINITY,
        y_max: f64::NEG_INFINITY,
    };

    for view_index in 0..end_limit {
        let Some(i) = selection.get_raw_index(len, view_index) else {
            continue;
        };

        let xi = x.get(i).copied().unwrap_or(f64::NAN);
        let yi = y.get(i).copied().unwrap_or(f64::NAN);
        if !xi.is_finite() || !yi.is_finite() {
            continue;
        }
        if !filter.contains(xi) {
            continue;
        }

        bounds.x_min = bounds.x_min.min(xi);
        bounds.x_max = bounds.x_max.max(xi);
        bounds.y_min = bounds.y_min.min(yi);
        bounds.y_max = bounds.y_max.max(yi);
    }

    if bounds.is_valid() {
        Some(bounds)
    } else {
        None
    }
}

fn axis_locked_window_y(range: AxisRange) -> Option<DataWindowY> {
    axis_locked_window_1d(range)
}

fn axis_locked_window_1d(range: AxisRange) -> Option<DataWindowX> {
    match range {
        AxisRange::Auto => None,
        AxisRange::LockMin { .. } | AxisRange::LockMax { .. } => None,
        AxisRange::Fixed { min, max } => {
            let mut w = DataWindowX { min, max };
            w.clamp_non_degenerate();
            Some(w)
        }
    }
}

fn apply_axis_constraints(
    model: &ChartModel,
    x_axis: crate::ids::AxisId,
    y_axis: crate::ids::AxisId,
    bounds: &mut DataBounds,
) {
    if let Some(axis) = model.axes.get(&x_axis) {
        apply_axis_constraint_1d(axis.range, &mut bounds.x_min, &mut bounds.x_max);
    }
    if let Some(axis) = model.axes.get(&y_axis) {
        apply_axis_constraint_1d(axis.range, &mut bounds.y_min, &mut bounds.y_max);
    }
}

fn apply_axis_constraint_1d(range: AxisRange, min: &mut f64, max: &mut f64) {
    match range {
        AxisRange::Auto => {}
        AxisRange::Fixed {
            min: fixed_min,
            max: fixed_max,
        } => {
            *min = fixed_min;
            *max = fixed_max;
        }
        AxisRange::LockMin { min: fixed_min } => {
            *min = fixed_min;
            if !max.is_finite() || *max <= *min {
                *max = fixed_min + 1.0;
            }
        }
        AxisRange::LockMax { max: fixed_max } => {
            *max = fixed_max;
            if !min.is_finite() || *min >= *max {
                *min = fixed_max - 1.0;
            }
        }
    }
}
