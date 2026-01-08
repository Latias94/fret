use fret_core::{Point, Px, Rect};

use crate::data::DatasetStore;
use crate::engine::ChartState;
use crate::engine::lod::{
    BoundsAccum, BoundsCursor, DataBounds, LodScratch, MinMaxPerPixelCursor, compute_bounds_step,
    finalize_bounds, minmax_per_pixel_finalize, minmax_per_pixel_step,
};
use crate::engine::model::ChartModel;
use crate::engine::window::{DataWindow, DataWindowX, DataWindowY};
use crate::ids::MarkId;
use crate::marks::{MarkKind, MarkNode, MarkOrderKey, MarkPayloadRef, MarkPolylineRef, MarkTree};
use crate::paint::StrokeStyleV2;
use crate::scheduler::WorkBudget;
use crate::spec::AxisRange;
use crate::stats::EngineStats;
use crate::view::ViewState;
use core::ops::Range;
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone)]
pub struct MarksStage {
    series_index: usize,
    cursor: MinMaxPerPixelCursor,
    bounds_cursor: BoundsCursor,
    bounds_accum: BoundsAccum,
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

            let table = datasets
                .datasets
                .iter()
                .find_map(|(id, t)| (*id == series.dataset).then_some(t));
            let Some(table) = table else {
                self.series_index += 1;
                continue;
            };
            let (row_range, view_x_filter, view_x_mapping_window) =
                if let Some(v) = view.series_view(series.id) {
                    (
                        v.selection.as_range(table.row_count),
                        v.x_policy.filter,
                        v.x_policy.mapping_window,
                    )
                } else {
                    (
                        0..table.row_count,
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
            {
                scratch.reset_buckets();
                self.finalized = false;
                self.bounds = None;
                self.bounds_cursor = BoundsCursor::default();
                self.bounds_accum.reset();
            }

            if self.bounds.is_none() {
                let points_budget = budget.take_points(4096) as usize;
                if points_budget == 0 {
                    return false;
                }

                let Some(bounds) = compute_series_bounds(
                    model,
                    state,
                    series.kind,
                    series.x_axis,
                    series.y_axis,
                    x,
                    y0,
                    y1,
                    row_range.clone(),
                    view_x_filter,
                    view_x_mapping_window,
                    points_budget,
                    &mut self.bounds_cursor,
                    &mut self.bounds_accum,
                    budget,
                ) else {
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
                self.bounds = None;
                continue;
            };
            bounds.clamp_non_degenerate();

            let mut finished_scan = false;
            while !finished_scan {
                let points_budget = budget.take_points(4096) as usize;
                if points_budget == 0 {
                    return false;
                }

                finished_scan = minmax_per_pixel_step(
                    &mut self.cursor,
                    scratch,
                    x,
                    y0,
                    &bounds,
                    viewport,
                    row_range.clone(),
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
                    y0,
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
        if let Some(table) = datasets
            .datasets
            .iter()
            .find_map(|(id, t)| (*id == dataset_id).then_some(t))
        {
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
    row_range: Range<usize>,
    x_filter: crate::engine::window_policy::AxisFilter1D,
    x_mapping_window: Option<DataWindowX>,
    initial_points_budget: usize,
    bounds_cursor: &mut BoundsCursor,
    bounds_accum: &mut BoundsAccum,
    budget: &mut WorkBudget,
) -> Option<DataBounds> {
    let y_axis_range = model.axes.get(&y_axis).map(|a| a.range).unwrap_or_default();
    let y_window_for_bounds =
        axis_locked_window_y(y_axis_range).or(state.data_window_y.get(&y_axis).copied());

    if let Some(mut y_window) = y_window_for_bounds {
        y_window.clamp_non_degenerate();

        let (x_min, x_max) = if let Some(mut w) = x_mapping_window {
            w.clamp_non_degenerate();
            (w.min, w.max)
        } else {
            let mut bounds =
                compute_bounds_in_range_filtered(x, y0, row_range, x_filter).unwrap_or_default();
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
        let mut bounds0 = compute_bounds_in_range_filtered(x, y0, row_range.clone(), x_filter)
            .unwrap_or_default();
        bounds0.clamp_non_degenerate();
        let mut bounds1 =
            compute_bounds_in_range_filtered(x, y1, row_range, x_filter).unwrap_or_default();
        bounds1.clamp_non_degenerate();

        let mut combined = DataBounds {
            x_min: bounds0.x_min.min(bounds1.x_min),
            x_max: bounds0.x_max.max(bounds1.x_max),
            y_min: bounds0.y_min.min(bounds1.y_min),
            y_max: bounds0.y_max.max(bounds1.y_max),
        };

        if let Some(mut w) = x_mapping_window {
            w.clamp_non_degenerate();
            combined.x_min = w.min;
            combined.x_max = w.max;
        }

        apply_axis_constraints(model, x_axis, y_axis, &mut combined);
        combined.clamp_non_degenerate();
        return Some(combined);
    }

    let mut done = compute_bounds_step(
        bounds_cursor,
        bounds_accum,
        x,
        y0,
        row_range.clone(),
        x_filter,
        initial_points_budget,
    )
    .unwrap_or(true);

    while !done {
        let points_budget = budget.take_points(4096) as usize;
        if points_budget == 0 {
            return None;
        }
        done = compute_bounds_step(
            bounds_cursor,
            bounds_accum,
            x,
            y0,
            row_range.clone(),
            x_filter,
            points_budget,
        )
        .unwrap_or(true);
    }

    let mut bounds = finalize_bounds(bounds_accum).unwrap_or_default();
    if let Some(mut w) = x_mapping_window {
        w.clamp_non_degenerate();
        bounds.x_min = w.min;
        bounds.x_max = w.max;
    }
    apply_axis_constraints(model, x_axis, y_axis, &mut bounds);
    bounds.clamp_non_degenerate();
    Some(bounds)
}

fn compute_bounds_in_range_filtered(
    x: &[f64],
    y: &[f64],
    row_range: core::ops::Range<usize>,
    filter: crate::engine::window_policy::AxisFilter1D,
) -> Option<DataBounds> {
    let len = x.len().min(y.len());
    let start = row_range.start.min(len);
    let end = row_range.end.min(len);
    if start >= end {
        return None;
    }

    let mut bounds = DataBounds {
        x_min: f64::INFINITY,
        x_max: f64::NEG_INFINITY,
        y_min: f64::INFINITY,
        y_max: f64::NEG_INFINITY,
    };

    for i in start..end {
        let xi = x[i];
        let yi = y[i];
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
