use fret_core::{Point, Px, Rect};

use crate::data::DatasetStore;
use crate::engine::ChartState;
use crate::engine::lod::{
    BoundsAccum, BoundsCursor, DataBounds, LodScratch, MinMaxPerPixelCursor,
    compute_bounds_step_selection, compute_bounds_step_selection_with, finalize_bounds,
    minmax_per_pixel_finalize, minmax_per_pixel_step_selection,
    minmax_per_pixel_step_selection_with,
};
use crate::engine::model::ChartModel;
use crate::engine::window::{DataWindow, DataWindowX, DataWindowY};
use crate::ids::MarkId;
use crate::ids::SeriesId;
use crate::ids::StackId;
use crate::marks::{
    MarkKind, MarkNode, MarkOrderKey, MarkPayloadRef, MarkPointsRef, MarkPolylineRef, MarkRectRef,
    MarkTree,
};
use crate::paint::StrokeStyleV2;
use crate::scheduler::WorkBudget;
use crate::spec::AxisRange;
use crate::spec::StackStrategy;
use crate::stats::EngineStats;
use crate::transform::RowSelection;
use crate::transform::stack_base_at_index;
use crate::view::ViewState;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
struct StackBoundsBuild {
    stack: StackId,
    x_axis: crate::ids::AxisId,
    y_axis: crate::ids::AxisId,
    series: Vec<SeriesId>,
    series_index: usize,
    cursor: BoundsCursor,
    accum: BoundsAccum,
    base: StackAccum,
    selection: RowSelection,
    x_filter: crate::engine::window_policy::AxisFilter1D,
    x_mapping_window: Option<DataWindowX>,
}

#[derive(Debug, Default, Clone)]
struct StackAccum {
    strategy: StackStrategy,
    pos: Vec<f64>,
    neg: Vec<f64>,
}

impl StackAccum {
    fn ensure_len(&mut self, len: usize, strategy: StackStrategy) {
        self.strategy = strategy;
        if self.pos.len() != len {
            self.pos.clear();
            self.pos.resize(len, 0.0);
        }
        if strategy == StackStrategy::SameSign && self.neg.len() != len {
            self.neg.clear();
            self.neg.resize(len, 0.0);
        } else if strategy != StackStrategy::SameSign {
            self.neg.clear();
        }
    }

    fn base_for(&self, y: f64, index: usize) -> f64 {
        match self.strategy {
            StackStrategy::All => self.pos.get(index).copied().unwrap_or(0.0),
            StackStrategy::SameSign => {
                if y >= 0.0 {
                    self.pos.get(index).copied().unwrap_or(0.0)
                } else {
                    self.neg.get(index).copied().unwrap_or(0.0)
                }
            }
        }
    }

    fn apply(&mut self, y: f64, index: usize) {
        match self.strategy {
            StackStrategy::All => {
                if let Some(b) = self.pos.get_mut(index) {
                    *b += y;
                }
            }
            StackStrategy::SameSign => {
                if y >= 0.0 {
                    if let Some(b) = self.pos.get_mut(index) {
                        *b += y;
                    }
                } else if let Some(b) = self.neg.get_mut(index) {
                    *b += y;
                }
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MarksStage {
    series_index: usize,
    cursor: MinMaxPerPixelCursor,
    bounds_cursor: BoundsCursor,
    bounds_accum: BoundsAccum,
    active_series: Option<SeriesId>,
    active_selection: RowSelection,
    stack_accum: BTreeMap<StackId, StackAccum>,
    stack_bounds: BTreeMap<StackId, DataBounds>,
    stack_bounds_build: Option<StackBoundsBuild>,
    scatter_next_index: usize,
    scatter_points_start: usize,
    scatter_node_index: Option<usize>,
    bar_next_index: usize,
    bar_rects_start: usize,
    bar_node_index: Option<usize>,
    finalized: bool,
    dirty: bool,
    last_series_count: usize,
    last_model_marks_rev: crate::ids::Revision,
    last_data_sig: u64,
    last_view_rev: crate::ids::Revision,
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

    pub fn sync_inputs(&mut self, model: &ChartModel, datasets: &DatasetStore, view: &ViewState) {
        let series_count = model.series_order.len();
        if series_count != self.last_series_count {
            self.dirty = true;
        }
        self.last_series_count = series_count;

        if model.revs.marks != self.last_model_marks_rev {
            self.dirty = true;
        }
        self.last_model_marks_rev = model.revs.marks;

        let data_sig = dataset_store_signature(model, datasets);
        if data_sig != self.last_data_sig {
            self.dirty = true;
        }
        self.last_data_sig = data_sig;

        if view.revision != self.last_view_rev {
            self.dirty = true;
        }
        self.last_view_rev = view.revision;
    }

    pub fn reset(&mut self) {
        self.series_index = 0;
        self.cursor = MinMaxPerPixelCursor::default();
        self.bounds_cursor = BoundsCursor::default();
        self.bounds_accum.reset();
        self.active_series = None;
        self.active_selection = RowSelection::default();
        self.stack_accum.clear();
        self.stack_bounds.clear();
        self.stack_bounds_build = None;
        self.scatter_next_index = 0;
        self.scatter_points_start = 0;
        self.scatter_node_index = None;
        self.bar_next_index = 0;
        self.bar_rects_start = 0;
        self.bar_node_index = None;
        self.finalized = false;
        self.dirty = false;
        self.bounds = None;
        self.axis_windows.clear();
    }

    pub fn axis_windows(&self) -> &BTreeMap<crate::ids::AxisId, DataWindow> {
        &self.axis_windows
    }

    pub fn step(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        state: &ChartState,
        view: &ViewState,
        selection_stage: &crate::engine::stages::SelectionStage,
        viewport: Rect,
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

            let table = datasets.dataset(series.dataset);
            let Some(table) = table else {
                self.series_index += 1;
                continue;
            };
            let (base_selection, view_x_filter, view_x_mapping_window) =
                if let Some(v) = view.series_view(series.id) {
                    (
                        v.selection.clone(),
                        v.x_policy.filter,
                        v.x_policy.mapping_window,
                    )
                } else {
                    (
                        RowSelection::All,
                        crate::engine::window_policy::AxisFilter1D::default(),
                        None,
                    )
                };

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

            let selection_range = base_selection.as_range(table.row_count);
            let selection_range = crate::transform::RowRange {
                start: selection_range.start,
                end: selection_range.end,
            };

            if self.active_series != Some(series.id) {
                let view = selection_stage.table_view_for(
                    table,
                    series.dataset,
                    x_col,
                    selection_range,
                    view_x_filter,
                    base_selection.clone(),
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
                self.bar_rects_start = 0;
                self.bar_node_index = None;
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

                            let mut base = StackAccum::default();
                            base.ensure_len(x.len().min(y0.len()), series.stack_strategy);
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
                                base,
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
                            let Some(y_col) = dataset.fields.get(&s.encode.y).copied() else {
                                build.series_index += 1;
                                build.cursor = BoundsCursor::default();
                                continue;
                            };

                            let table = datasets.dataset(s.dataset);
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
                            let Some(y) = table.column_f64(y_col) else {
                                build.series_index += 1;
                                build.cursor = BoundsCursor::default();
                                continue;
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
                                |i| {
                                    let yi = y.get(i).copied().unwrap_or(f64::NAN);
                                    if !yi.is_finite() {
                                        return yi;
                                    }
                                    let base = build.base.base_for(yi, i);
                                    let y_eff = yi + base;
                                    build.base.apply(yi, i);
                                    y_eff
                                },
                            )
                            .unwrap_or(true);

                            if done {
                                build.series_index += 1;
                                build.cursor = BoundsCursor::default();
                            }
                        }

                        let mut bounds = finalize_bounds(&build.accum).unwrap_or_default();

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

                let visible_len = selection.view_len(table.row_count);
                let use_lod = visible_len > 20_000;

                if use_lod {
                    let mut finished_scan = false;
                    while !finished_scan {
                        let points_budget = budget.take_points(4096) as usize;
                        if points_budget == 0 {
                            return false;
                        }

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

                    if !self.finalized {
                        if budget.take_marks(1) == 0 {
                            return false;
                        }

                        let range = minmax_per_pixel_finalize(
                            scratch,
                            x,
                            &bounds,
                            viewport,
                            &mut marks.arena.points,
                            &mut marks.arena.data_indices,
                        );
                        let base_order = self.series_index as u32;

                        marks.nodes.push(MarkNode {
                            id: series_mark_id(series.id, 0),
                            parent: None,
                            layer: crate::ids::LayerId(1),
                            order: MarkOrderKey(base_order.saturating_mul(2)),
                            kind: MarkKind::Points,
                            source_series: Some(series.id),
                            payload: MarkPayloadRef::Points(MarkPointsRef {
                                points: range.clone(),
                                fill: Some(crate::ids::PaintId(0)),
                                stroke: None,
                            }),
                        });

                        stats.points_emitted += (range.end - range.start) as u64;
                        stats.marks_emitted += 1;
                        marks.revision.bump();
                        self.finalized = true;
                    }

                    self.series_index += 1;
                    self.cursor.next_index = 0;
                    self.scatter_next_index = 0;
                    self.scatter_points_start = 0;
                    self.scatter_node_index = None;
                    self.bar_next_index = 0;
                    self.bar_rects_start = 0;
                    self.bar_node_index = None;
                    self.bounds = None;
                    scratch.clear();
                    continue;
                }

                if self.scatter_node_index.is_none() {
                    if budget.take_marks(1) == 0 {
                        return false;
                    }

                    self.scatter_next_index = 0;
                    self.scatter_points_start = marks.arena.points.len();
                    let range = self.scatter_points_start..self.scatter_points_start;
                    let base_order = self.series_index as u32;

                    marks.nodes.push(MarkNode {
                        id: series_mark_id(series.id, 0),
                        parent: None,
                        layer: crate::ids::LayerId(1),
                        order: MarkOrderKey(base_order.saturating_mul(2)),
                        kind: MarkKind::Points,
                        source_series: Some(series.id),
                        payload: MarkPayloadRef::Points(MarkPointsRef {
                            points: range,
                            fill: Some(crate::ids::PaintId(0)),
                            stroke: None,
                        }),
                    });
                    self.scatter_node_index = Some(marks.nodes.len() - 1);
                    marks.revision.bump();
                    stats.marks_emitted += 1;
                }

                let row_end = selection.view_len(table.row_count);
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
                        if !view_x_filter.contains(xi) {
                            continue;
                        }

                        let yi = yi.clamp(y_window.min, y_window.max);
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
                self.bar_next_index = 0;
                self.bar_rects_start = 0;
                self.bar_node_index = None;
                self.bounds = None;
                scratch.clear();
                continue;
            }

            if series.kind == crate::spec::SeriesKind::Bar {
                let x_axis = model.axes.get(&series.x_axis);
                let x_window = x_axis.and_then(crate::engine::axis::category_domain_window);
                let x_window = view_x_mapping_window.or(x_window).unwrap_or(DataWindow {
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

                if self.bar_node_index.is_none() {
                    if budget.take_marks(1) == 0 {
                        return false;
                    }

                    self.bar_next_index = 0;
                    self.bar_rects_start = marks.arena.rects.len();
                    let range = self.bar_rects_start..self.bar_rects_start;
                    let base_order = self.series_index as u32;
                    marks.nodes.push(MarkNode {
                        id: series_mark_id(series.id, 0),
                        parent: None,
                        layer: crate::ids::LayerId(1),
                        order: MarkOrderKey(base_order.saturating_mul(2)),
                        kind: MarkKind::Rect,
                        source_series: Some(series.id),
                        payload: MarkPayloadRef::Rect(MarkRectRef {
                            rects: range,
                            fill: Some(crate::ids::PaintId(0)),
                            stroke: None,
                        }),
                    });
                    self.bar_node_index = Some(marks.nodes.len() - 1);
                    marks.revision.bump();
                    stats.marks_emitted += 1;
                }

                let row_end = selection.view_len(table.row_count);
                while self.bar_next_index < row_end {
                    let points_budget = budget.take_points(4096) as usize;
                    if points_budget == 0 {
                        return false;
                    }

                    let chunk_end = (self.bar_next_index + points_budget).min(row_end);
                    let baseline_data = 0.0f64;

                    let x_span = x_window.span();
                    let band_px = (viewport.size.width.0 as f64 / x_span.max(1.0)).max(1.0) as f32;
                    let bar_w = band_px * 0.8;

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
                        if !view_x_filter.contains(xi) {
                            continue;
                        }

                        let tx = ((xi - x_window.min) / x_window.span()).clamp(0.0, 1.0);
                        let ty = ((yi - y_window.min) / y_window.span()).clamp(0.0, 1.0);
                        let ty0 =
                            ((baseline_data - y_window.min) / y_window.span()).clamp(0.0, 1.0);

                        let px_x = viewport.origin.x.0 + (tx as f32) * viewport.size.width.0;
                        let px_y =
                            viewport.origin.y.0 + (1.0 - (ty as f32)) * viewport.size.height.0;
                        let px_y0 =
                            viewport.origin.y.0 + (1.0 - (ty0 as f32)) * viewport.size.height.0;

                        let top = px_y.min(px_y0);
                        let bottom = px_y.max(px_y0);
                        let h = (bottom - top).max(1.0);

                        rects.push(Rect::new(
                            Point::new(Px(px_x - 0.5 * bar_w), Px(top)),
                            fret_core::Size::new(Px(bar_w), Px(h)),
                        ));
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
                self.bar_next_index = 0;
                self.bar_rects_start = 0;
                self.bar_node_index = None;
                self.bounds = None;
                scratch.clear();
                continue;
            }

            let mut finished_scan = false;
            while !finished_scan {
                let points_budget = budget.take_points(4096) as usize;
                if points_budget == 0 {
                    return false;
                }

                if let Some(stack) = series.stack {
                    let len = x.len().min(y0.len());
                    let accum = self.stack_accum.entry(stack).or_default();
                    accum.ensure_len(len, series.stack_strategy);
                    finished_scan = minmax_per_pixel_step_selection_with(
                        &mut self.cursor,
                        scratch,
                        x,
                        &bounds,
                        viewport,
                        &selection,
                        points_budget,
                        |i| {
                            let yi = y0.get(i).copied().unwrap_or(f64::NAN);
                            if !yi.is_finite() {
                                return yi;
                            }
                            let base = accum.base_for(yi, i);
                            let y_eff = yi + base;
                            accum.apply(yi, i);
                            y_eff
                        },
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
                if budget.take_marks(1) == 0 {
                    return false;
                }

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

                    marks.nodes.push(MarkNode {
                        id: series_mark_id(series.id, 1),
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
                    marks.nodes.push(MarkNode {
                        id: series_mark_id(series.id, 2),
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

                    stats.points_emitted += (lower_range.end - lower_range.start) as u64;
                    stats.points_emitted += (upper_range.end - upper_range.start) as u64;
                    stats.marks_emitted += 2;
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
                        let Some(base) = stack_base_at_index(model, datasets, series.id, i, yi)
                        else {
                            continue;
                        };
                        let y_base = base.base;

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

                    marks.nodes.push(MarkNode {
                        id: series_mark_id(series.id, 1),
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
                    marks.nodes.push(MarkNode {
                        id: series_mark_id(series.id, 2),
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

                    stats.points_emitted += (lower_range.end - lower_range.start) as u64;
                    stats.points_emitted += range_len;
                    stats.marks_emitted += 2;
                    marks.revision.bump();
                } else {
                    marks.nodes.push(MarkNode {
                        id: series_mark_id(series.id, 0),
                        parent: None,
                        layer: crate::ids::LayerId(1),
                        order: MarkOrderKey(base_order.saturating_mul(2)),
                        kind: MarkKind::Polyline,
                        source_series: Some(series.id),
                        payload: MarkPayloadRef::Polyline(MarkPolylineRef {
                            points: range,
                            stroke: stroke.clone(),
                        }),
                    });

                    stats.points_emitted += range_len;
                    stats.marks_emitted += 1;
                    marks.revision.bump();
                }
                self.finalized = true;
            }

            self.series_index += 1;
            self.cursor.next_index = 0;
            self.scatter_next_index = 0;
            self.scatter_points_start = 0;
            self.scatter_node_index = None;
            self.bar_next_index = 0;
            self.bar_rects_start = 0;
            self.bar_node_index = None;
            self.bounds = None;
            scratch.clear();
        }

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

fn series_mark_id(series: crate::ids::SeriesId, variant: u64) -> MarkId {
    MarkId((series.0 << 3) | (variant & 0x7))
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
        if let Some(table) = datasets.dataset(dataset_id) {
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

        let (x_min, x_max) = if let Some(mut w) = x_mapping_window.or(x_domain_window) {
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
