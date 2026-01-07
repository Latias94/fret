use fret_core::Rect;

use crate::data::DatasetStore;
use crate::engine::ChartState;
use crate::engine::lod::{
    BoundsAccum, BoundsCursor, DataBounds, LodScratch, MinMaxPerPixelCursor, compute_bounds,
    compute_bounds_step, finalize_bounds, minmax_per_pixel_finalize, minmax_per_pixel_step,
};
use crate::engine::model::ChartModel;
use crate::engine::window::DataWindowX;
use crate::marks::{MarkKind, MarkNode, MarkOrderKey, MarkPayloadRef, MarkPolylineRef, MarkTree};
use crate::paint::StrokeStyleV2;
use crate::scheduler::WorkBudget;
use crate::spec::AxisRange;
use crate::stats::EngineStats;

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
    last_data_rev: crate::ids::Revision,
    bounds: Option<DataBounds>,
}

impl MarksStage {
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn sync_inputs(&mut self, model: &ChartModel, datasets: &DatasetStore) {
        let series_count = model.series_order.len();
        if series_count != self.last_series_count {
            self.dirty = true;
        }
        self.last_series_count = series_count;

        if model.revs.marks != self.last_model_marks_rev {
            self.dirty = true;
        }
        self.last_model_marks_rev = model.revs.marks;

        let data_rev = datasets
            .datasets
            .iter()
            .next()
            .map(|(_, t)| t.revision)
            .unwrap_or_default();
        if data_rev != self.last_data_rev {
            self.dirty = true;
        }
        self.last_data_rev = data_rev;
    }

    pub fn reset(&mut self) {
        self.series_index = 0;
        self.cursor = MinMaxPerPixelCursor::default();
        self.bounds_cursor = BoundsCursor::default();
        self.bounds_accum.reset();
        self.finalized = false;
        self.dirty = false;
        self.bounds = None;
    }

    pub fn step(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        state: &ChartState,
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
            if series.kind != crate::spec::SeriesKind::Line {
                self.series_index += 1;
                continue;
            }
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

            let Some(x) = table.column_f64(series.x_col) else {
                self.series_index += 1;
                continue;
            };
            let Some(y) = table.column_f64(series.y_col) else {
                self.series_index += 1;
                continue;
            };

            if self.cursor.next_index == 0 {
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

                let x_axis_range = model
                    .axes
                    .get(&series.x_axis)
                    .map(|a| a.range)
                    .unwrap_or_default();
                let window_for_bounds = axis_locked_window_x(x_axis_range).or(state.data_window_x);

                let done = compute_bounds_step(
                    &mut self.bounds_cursor,
                    &mut self.bounds_accum,
                    x,
                    y,
                    window_for_bounds,
                    points_budget,
                )
                .unwrap_or(true);

                if !done {
                    return false;
                }

                let mut bounds = compute_bounds(x, y).unwrap_or_default();
                bounds.clamp_non_degenerate();

                let mut windowed = finalize_bounds(&self.bounds_accum, window_for_bounds)
                    .unwrap_or(DataBounds {
                        x_min: bounds.x_min,
                        x_max: bounds.x_max,
                        y_min: bounds.y_min,
                        y_max: bounds.y_max,
                    });

                if window_for_bounds.is_none() {
                    windowed.x_min = bounds.x_min;
                    windowed.x_max = bounds.x_max;
                }
                apply_axis_ranges(model, series.x_axis, series.y_axis, &mut windowed);
                windowed.clamp_non_degenerate();
                self.bounds = Some(windowed);
            }
            let Some(mut bounds) = self.bounds else {
                self.series_index += 1;
                self.cursor.next_index = 0;
                self.bounds = None;
                continue;
            };
            bounds.clamp_non_degenerate();

            let x_axis_range = model
                .axes
                .get(&series.x_axis)
                .map(|a| a.range)
                .unwrap_or_default();
            if axis_locked_window_x(x_axis_range).is_none()
                && let Some(window) = state.data_window_x
            {
                let mut window = window;
                window.clamp_non_degenerate();

                bounds.x_min = bounds.x_min.max(window.x_min);
                bounds.x_max = bounds.x_max.min(window.x_max);
                bounds.clamp_non_degenerate();
            }

            let points_budget = budget.take_points(4096) as usize;
            if points_budget == 0 {
                return false;
            }

            let finished_scan = minmax_per_pixel_step(
                &mut self.cursor,
                scratch,
                x,
                y,
                &bounds,
                viewport,
                points_budget,
            );

            if !finished_scan {
                return false;
            }

            if !self.finalized {
                if budget.take_marks(1) == 0 {
                    return false;
                }

                let range = minmax_per_pixel_finalize(
                    scratch,
                    x,
                    y,
                    &bounds,
                    viewport,
                    &mut marks.arena.points,
                    &mut marks.arena.data_indices,
                );
                let point_count = range.len() as u64;

                let stroke = Some((crate::ids::PaintId(0), StrokeStyleV2::default()));
                marks.nodes.push(MarkNode {
                    id: crate::ids::MarkId(series.id.0),
                    parent: None,
                    layer: crate::ids::LayerId(1),
                    order: MarkOrderKey(self.series_index as u32),
                    kind: MarkKind::Polyline,
                    source_series: Some(series.id),
                    payload: MarkPayloadRef::Polyline(MarkPolylineRef {
                        points: range,
                        stroke,
                    }),
                });

                stats.points_emitted += point_count;
                stats.marks_emitted += 1;
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

fn axis_locked_window_x(range: AxisRange) -> Option<DataWindowX> {
    match range {
        AxisRange::Auto => None,
        AxisRange::Fixed { min, max } => {
            let mut w = DataWindowX {
                x_min: min,
                x_max: max,
            };
            w.clamp_non_degenerate();
            Some(w)
        }
    }
}

fn apply_axis_ranges(
    model: &ChartModel,
    x_axis: crate::ids::AxisId,
    y_axis: crate::ids::AxisId,
    bounds: &mut DataBounds,
) {
    if let Some(axis) = model.axes.get(&x_axis) {
        if let AxisRange::Fixed { min, max } = axis.range {
            bounds.x_min = bounds.x_min.max(min);
            bounds.x_max = bounds.x_max.min(max);
        }
    }
    if let Some(axis) = model.axes.get(&y_axis) {
        if let AxisRange::Fixed { min, max } = axis.range {
            bounds.y_min = bounds.y_min.max(min);
            bounds.y_max = bounds.y_max.min(max);
        }
    }
}
